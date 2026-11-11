#!/usr/bin/env python3
"""Validate local time -> Geocalc integration against independent CSPICE.

Requires spiceypy and a user-supplied official SPK covering the requested UTCs.
The developing Geocalc checkout is read only: Cargo's manifest, lock and target
are isolated. This checks geometric J2000 states (NONE), not all geometry APIs.
Three paths are checked separately: common CSPICE ET, context UTC-to-ET, and
context Epoch-to-ET with Geocalc's state_at_epoch. The default samples cover
J2000, the 2016 leap second and 2024. User UTCs replace that finite sample set.
UTC's reference is CSPICE STR2ET on civil components. Epoch's reference is
CSPICE UNITIM on the independently rounded numeric TAI seconds past J2000;
each state reference uses the ET for that specific input contract.
ET and state tolerances are independent: even an ET difference below its limit
can move a body farther than the position tolerance, correctly failing a run.
Example: python tools/validate_geocalc_time.py --spk /path/to/de440s.bsp
"""
import argparse
from decimal import Decimal
import hashlib
import json
import math
from pathlib import Path
import re
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_UTC = ["2000-01-01T12:00:00", "2016-12-31T23:59:59.5",
               "2016-12-31T23:59:60.5", "2017-01-01T00:00:00",
               "2024-04-08T18:00:00.125", "2024-04-08T18:00:00.123456789",
               "2024-04-08T18:00:00.144272509"]
PAIRS = [(399, 0), (301, 399)]
MODES = ("same_et", "utc", "epoch")


def positive_finite(value):
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise ValueError("tolerances must be finite and positive")
    return number


def file_record(path):
    path = Path(path).resolve(strict=True)
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return {"path": str(path), "bytes": path.stat().st_size, "sha256": digest.hexdigest()}


def source_record(path):
    path = Path(path).resolve(strict=True)
    files = [path / "Cargo.toml"]
    if (path / "Cargo.lock").is_file():
        files.append(path / "Cargo.lock")
    files.extend(sorted((path / "src").rglob("*.rs")))
    return {"path": str(path), "files": [file_record(file) for file in files]}


def parse_rows(output):
    rows = {}
    for line in output.splitlines():
        fields = line.split("\t")
        if len(fields) != 13:
            raise ValueError(f"expected 13 protocol columns: {line!r}")
        key = (int(fields[0]), int(fields[1]), int(fields[2]), fields[3])
        if key in rows:
            raise ValueError(f"duplicate row: {key}")
        rows[key] = dict(et=float(fields[4]), sec=int(fields[5]), nanos=int(fields[6]),
                         state=[float(value) for value in fields[7:]])
    return rows


def checked_row(row):
    if len(row["state"]) != 6 or not all(math.isfinite(x) for x in [row["et"], *row["state"]]):
        raise ValueError("non-finite value or wrong state width")
    if type(row["sec"]) is not int or type(row["nanos"]) is not int or not 0 <= row["nanos"] < 10**9:
        raise ValueError("invalid exact epoch parts")


def compare_rows(actual, expected, position_tolerance, velocity_tolerance, et_tolerance):
    for tolerance in (position_tolerance, velocity_tolerance, et_tolerance):
        positive_finite(tolerance)
    if not expected or actual.keys() != expected.keys():
        raise ValueError(f"row set mismatch: missing={expected.keys()-actual.keys()}, extra={actual.keys()-expected.keys()}")
    results = []
    for key, reference in expected.items():
        row = actual[key]
        checked_row(row)
        checked_row(reference)
        if (row["sec"], row["nanos"]) != (reference["sec"], reference["nanos"]):
            raise ValueError(f"exact TAI epoch mismatch: {key}")
        errors = [abs(a-b) for a, b in zip(row["state"], reference["state"])]
        et_error = abs(row["et"] - reference["et"])
        if et_error > et_tolerance or max(errors[:3]) > position_tolerance or max(errors[3:]) > velocity_tolerance:
            raise ValueError(f"tolerance exceeded {key}: ET={et_error:.17g}s, position={max(errors[:3]):.17g}km, velocity={max(errors[3:]):.17g}km/s")
        results.append(dict(sample=key[0], target=key[1], observer=key[2], path=key[3],
                            et_error_seconds=et_error, max_position_error_km=max(errors[:3]),
                            max_velocity_error_km_s=max(errors[3:]), actual=row, reference=reference))
    return results


# No serialization library is needed by the disposable client. Input consists
# of an index, six UTC fields, and CSPICE ET; output contains exact epoch parts.
CLIENT = r'''
use std::io::{self, BufRead};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};
use stkcraft_time::contract::TimeContext;
use stkcraft_geocalc::SpkKernel;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let lsk = LeapSecondTable::from_lsk(&std::fs::read_to_string(&args[1])?)?;
    let context = TimeContext::new(&lsk);
    let spk = SpkKernel::open(&args[2])?;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let f: Vec<&str> = line.split('\t').collect();
        if f.len() != 8 { return Err("invalid client input".into()); }
        let utc = UtcDateTime::new(f[1].parse()?, f[2].parse()?, f[3].parse()?,
            f[4].parse()?, f[5].parse()?, f[6].parse()?);
        let epoch = stkcraft_time::convert::utc_to_tai(&utc, Calendar::Gregorian, &lsk)?;
        let utc_et = context.utc_to_et(&utc, Calendar::Gregorian)?.seconds();
        let epoch_et = context.to_et(epoch)?.seconds();
        let oracle_et: f64 = f[7].parse()?;
        for (target, observer) in [(399, 0), (301, 399)] {
            for (mode, et) in [("same_et", oracle_et), ("utc", utc_et), ("epoch", epoch_et)] {
                let state = if mode == "epoch" {
                    spk.state_at_epoch(target, observer, epoch, &lsk)?
                } else { spk.state(target, observer, et)? };
                print!("{}\t{}\t{}\t{}\t{:.17e}\t{}\t{}", f[0], target, observer,
                    mode, et, epoch.sec(), epoch.nanos());
                for value in state.position.into_iter().chain(state.velocity) { print!("\t{:.17e}", value); }
                println!();
            }
        }
    }
    Ok(())
}
'''


def prepare_samples(utcs, spice):
    samples = []
    for index, utc in enumerate(utcs):
        match = re.fullmatch(r"(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2}(?:\.\d{1,9})?)Z?", utc)
        if not match:
            raise ValueError(f"expected YYYY-MM-DDTHH:MM:SS[.nanoseconds][Z]: {utc!r}")
        fields = match.groups()
        et = float(spice.str2et(utc.removesuffix("Z").replace("T", " ") + " UTC"))
        # Ask CSPICE about the whole UTC second, then retain decimal fractional
        # nanoseconds as integers. ET's f64 precision must not corrupt this
        # exact check. CSPICE TAI's noon origin is 43200s after Epoch's origin.
        whole_utc = utc.removesuffix("Z").split(".")[0].replace("T", " ") + " UTC"
        whole_tai = float(spice.unitim(spice.str2et(whole_utc), "ET", "TAI")) + 43200.0
        sec = round(whole_tai)
        fraction = fields[5].partition(".")[2]
        nanos = int(fraction.ljust(9, "0")) if fraction else 0
        tai = sec + nanos * 1e-9
        if not math.isfinite(et) or abs(whole_tai - sec) > 1e-5:
            raise ValueError("CSPICE could not resolve the integer TAI second")
        samples.append(dict(index=index, utc=utc, fields=fields, et=et,
                            oracle_tai_seconds=tai, sec=sec, nanos=nanos))
    return samples


def reference_rows(samples, spice):
    """Use distinct CSPICE contracts for UTC strings and numeric TAI epochs.

    A numeric TAI f64 already rounds the continuous epoch. It is a different
    supplied input from the civil UTC components processed by STR2ET/TTRANS.
    Decimal constructs the numeric input independently, rounding only once.
    """
    expected = {}
    for sample in samples:
        tai = float(Decimal(sample["sec"] - 43200) + Decimal(sample["nanos"]) / Decimal(10**9))
        epoch_et = float(spice.unitim(tai, "TAI", "ET"))
        for target, observer in PAIRS:
            for mode in MODES:
                et = epoch_et if mode == "epoch" else sample["et"]
                state, _ = spice.spkez(target, et, "J2000", "NONE", observer)
                expected[(sample["index"], target, observer, mode)] = dict(
                    et=et, sec=sample["sec"], nanos=sample["nanos"], state=list(state),
                    oracle_input=(dict(numeric_tai_seconds_past_j2000=tai) if mode == "epoch"
                                  else dict(utc=sample.get("utc"), et_seconds=sample["et"])))
    return expected


def run_client(geocalc, lsk, spk, samples):
    import os
    with tempfile.TemporaryDirectory(prefix="stkcraft-geocalc-time-") as directory:
        project = Path(directory)
        manifest = ('[package]\nname="geocalc-time-validation"\nversion="0.0.0"\nedition="2021"\n'
                    '[workspace]\n[dependencies]\n'
                    f'stkcraft-time = {{ path = {json.dumps(ROOT.as_posix())} }}\n'
                    f'stkcraft-geocalc = {{ path = {json.dumps(geocalc.as_posix())} }}\n')
        (project / "Cargo.toml").write_text(manifest, encoding="utf-8")
        (project / "src").mkdir()
        (project / "src/main.rs").write_text(CLIENT, encoding="utf-8")
        env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / "target/geocalc-time-validation"))
        protocol = "".join("\t".join([str(s["index"]), *s["fields"], repr(s["et"])])+"\n" for s in samples)
        command = ["cargo", "run", "--release", "--quiet", "--manifest-path", str(project/"Cargo.toml"), "--", str(lsk), str(spk)]
        process = subprocess.run(command, input=protocol, text=True, capture_output=True, env=env, check=True)
        if process.stderr:
            print(process.stderr, file=sys.stderr, end="")
        build = {"manifest": manifest, "client_sha256": hashlib.sha256(CLIENT.encode()).hexdigest(),
                 "lock": (project/"Cargo.lock").read_text(encoding="utf-8"),
                 "rustc": subprocess.check_output(["rustc", "--version"], text=True).strip(),
                 "cargo": subprocess.check_output(["cargo", "--version"], text=True).strip()}
        return parse_rows(process.stdout), build


def parser():
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--spk", type=Path, required=True, help="official local SPK containing Earth and Moon")
    result.add_argument("--lsk", type=Path, default=ROOT/"src/naif0012.tls")
    result.add_argument("--geocalc", type=Path, default=ROOT.parent/"STKCraft-Geocalc")
    result.add_argument("--utc", action="append", help="repeatable; replaces built-in J2000/leap-second/2024 samples")
    result.add_argument("--report", type=Path, help="write JSON provenance and measured results")
    result.add_argument("--position-tolerance", type=positive_finite, default=1e-6, help="absolute per-component km (default 1e-6)")
    result.add_argument("--velocity-tolerance", type=positive_finite, default=1e-10, help="absolute per-component km/s (default 1e-10)")
    result.add_argument("--et-tolerance", type=positive_finite, default=1e-6, help="absolute seconds (default 1e-6)")
    return result


def main(argv=None):
    args = parser().parse_args(argv)
    report = {"status": "failed", "frame": "J2000", "aberration": "NONE",
              "tolerances": {"position_km": args.position_tolerance,
                             "velocity_km_s": args.velocity_tolerance, "et_seconds": args.et_tolerance}}
    try:
        import spiceypy as spice
        lsk, spk, geocalc = [p.resolve(strict=True) for p in (args.lsk, args.spk, args.geocalc)]
        report.update(kernels={"lsk": file_record(lsk), "spk": file_record(spk)},
                      sources={"time": source_record(ROOT), "geocalc": source_record(geocalc),
                               "validator": file_record(__file__)},
                      oracle={"spiceypy": spice.__version__, "cspice": spice.tkvrsn("TOOLKIT"),
                              "python": sys.version})
        spice.kclear()
        try:
            spice.furnsh(str(lsk))
            spice.furnsh(str(spk))
            samples = prepare_samples(args.utc or DEFAULT_UTC, spice)
            actual, build = run_client(geocalc, lsk, spk, samples)
            report.update(samples=samples, build=build)
            expected = reference_rows(samples, spice)
            report["results"] = compare_rows(actual, expected, args.position_tolerance,
                                               args.velocity_tolerance, args.et_tolerance)
            report["status"] = "passed"
        finally:
            spice.kclear()
    except Exception as error:
        report["error"] = str(error)
        if isinstance(error, subprocess.CalledProcessError):
            report["stderr"] = error.stderr
    if args.report:
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(json.dumps(report, indent=2, allow_nan=False)+"\n", encoding="utf-8")
    if report["status"] != "passed":
        print(f"FAILED: {report['error']}", file=sys.stderr)
        if report.get("stderr"):
            print(report["stderr"], file=sys.stderr)
        return 1
    print(f"PASS: {len(report['results'])} rows; CSPICE {report['oracle']['cspice']}; kernels SHA-256 recorded")
    for mode in MODES:
        rows = [row for row in report["results"] if row["path"] == mode]
        print(f"  {mode}: ET {max(r['et_error_seconds'] for r in rows):.3g} s; "
              f"position {max(r['max_position_error_km'] for r in rows):.3g} km; "
              f"velocity {max(r['max_velocity_error_km_s'] for r in rows):.3g} km/s")
    return 0


if __name__ == "__main__":
    sys.exit(main())
