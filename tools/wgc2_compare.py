#!/usr/bin/env python3
"""Unified comparison of this crate against NASA's time engines.

The report has two sections:

  1. TDB conversion against the live WGC2 TIME_CONVERSION API, plus the
     offline CSPICE golden vectors.
  2. UT1, which WGC2/CSPICE do not provide. The script probes WGC2 to show
     that UT1 is unsupported, then validates UT1 against the real IERS
     `finals2000A.all` data and leap-second physics with
     `tools/validate_ut1.py`.

Only the Python standard library is used.

Example:
    python tools/wgc2_compare.py
    python tools/wgc2_compare.py --insecure
    python tools/wgc2_compare.py --utc 2016-12-31T23:59:60.5 --skip-ut1
"""

from __future__ import annotations

import argparse
import json
import ssl
import subprocess
import sys
import time
import urllib.request
from pathlib import Path

API_BASE = "https://wgc2.jpl.nasa.gov:8443/webgeocalc/api"

DEFAULT_TIMES = [
    "2016-12-31T23:59:60.5",
    "2000-01-01T12:00:00",
    "2017-01-01T00:00:00",
    "2024-01-01T00:00:00",
]


def request_json(url: str, payload: dict | None = None, context: ssl.SSLContext | None = None) -> dict:
    data = None if payload is None else json.dumps(payload).encode()
    request = urllib.request.Request(
        url,
        data=data,
        headers={"Content-Type": "application/json", "Accept": "application/json"},
        method="POST" if payload is not None else "GET",
    )
    with urllib.request.urlopen(request, context=context, timeout=30) as response:
        return json.loads(response.read().decode())


def run_wgc2(times: list[str], kernel_set: int, insecure: bool, timeout: float) -> list[float]:
    context = ssl._create_unverified_context() if insecure else ssl.create_default_context()

    payload = {
        "calculationType": "TIME_CONVERSION",
        "kernels": [{"type": "KERNEL_SET", "id": kernel_set}],
        "timeSystem": "UTC",
        "timeFormat": "CALENDAR",
        "times": times,
        "timeStep": 1,
        "timeStepUnits": "SECONDS",
        "outputTimeSystem": "TDB",
        "outputTimeFormat": "SECONDS_PAST_J2000",
    }

    created = request_json(f"{API_BASE}/calculation/new", payload, context)
    if created.get("status") != "OK":
        raise RuntimeError(f"WGC2 create failed: {created}")

    calculation_id = created["calculationId"]
    deadline = time.time() + timeout

    while time.time() < deadline:
        status = request_json(f"{API_BASE}/calculation/{calculation_id}", None, context)
        phase = status.get("result", {}).get("phase")

        if phase == "COMPLETE":
            results = request_json(
                f"{API_BASE}/calculation/{calculation_id}/results", None, context
            )
            return [float(row[1]) for row in results.get("rows", [])]

        if phase in ("FAILED", "CANCELLED", "EXPIRED"):
            raise RuntimeError(f"WGC2 calculation {phase}: {status}")

        time.sleep(0.5)

    raise TimeoutError(f"WGC2 calculation {calculation_id} did not complete in {timeout}s")


def probe_wgc2_ut1(insecure: bool) -> str:
    """Return "SUPPORTED" or the WGC2 error text for a UT1 request."""
    context = ssl._create_unverified_context() if insecure else ssl.create_default_context()

    payload = {
        "calculationType": "TIME_CONVERSION",
        "kernels": [{"type": "KERNEL_SET", "id": 2}],
        "timeSystem": "UTC",
        "timeFormat": "CALENDAR",
        "times": ["2000-01-01T12:00:00"],
        "timeStep": 1,
        "timeStepUnits": "SECONDS",
        "outputTimeSystem": "UT1",
        "outputTimeFormat": "SECONDS_PAST_J2000",
    }

    try:
        created = request_json(f"{API_BASE}/calculation/new", payload, context)
    except Exception as exc:  # noqa: BLE001 - report the probe failure verbatim
        return f"probe failed: {exc}"

    if created.get("status") == "OK":
        return "SUPPORTED"

    message = created.get("error", {}).get("shortDescription")
    return message or json.dumps(created)[:200]


def run_crate(times: list[str], repo_root: Path) -> list[float]:
    command = ["cargo", "run", "--quiet", "--bin", "wgc-compare", "--"]
    process = subprocess.run(
        command,
        cwd=repo_root,
        input="\n".join(times),
        text=True,
        capture_output=True,
    )

    if process.returncode != 0:
        raise RuntimeError(
            "cargo run failed\n"
            f"stdout:\n{process.stdout}\n"
            f"stderr:\n{process.stderr}"
        )

    values: dict[str, float] = {}
    for line in process.stdout.splitlines():
        if not line.strip():
            continue
        utc, et = line.split(",", 1)
        values[utc] = float(et)

    return [values[time] for time in times]


def run_tdb_compare(times: list[str], repo_root: Path) -> list[tuple[str, float, float, float]]:
    """Return (utc, et, tdb_iau, et_minus_tdb_microseconds)."""
    command = ["cargo", "run", "--quiet", "--bin", "tdb-compare", "--"]
    process = subprocess.run(
        command,
        cwd=repo_root,
        input="\n".join(times),
        text=True,
        capture_output=True,
    )

    if process.returncode != 0:
        raise RuntimeError(
            "cargo run --bin tdb-compare failed\n"
            f"stdout:\n{process.stdout}\n"
            f"stderr:\n{process.stderr}"
        )

    rows = []
    for line in process.stdout.splitlines():
        if not line.strip():
            continue
        utc, et, tdb, difference = line.split(",")
        rows.append((utc, float(et), float(tdb), float(difference)))
    return rows


def load_golden(repo_root: Path) -> dict[str, float]:
    path = repo_root / "tests" / "golden" / "utc_to_et.csv"
    if not path.exists():
        return {}

    golden: dict[str, float] = {}
    with path.open(encoding="utf-8") as handle:
        next(handle, None)
        for line in handle:
            if not line.strip():
                continue
            utc, et = line.strip().split(",", 1)
            golden[utc] = float(et)
    return golden


def run_ut1_validation(repo_root: Path, insecure: bool, eop: str | None) -> bool:
    command = [sys.executable, str(repo_root / "tools" / "validate_ut1.py")]
    if insecure:
        command.append("--insecure")
    if eop:
        command.extend(["--eop", eop])

    process = subprocess.run(command, cwd=repo_root, text=True, capture_output=True)
    if process.stdout:
        print(process.stdout, end="")
    if process.returncode != 0 and process.stderr:
        print(process.stderr, end="", file=sys.stderr)
    return process.returncode == 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--kernel-set", type=int, default=1)
    parser.add_argument("--tolerance", type=float, default=1e-5)
    parser.add_argument("--timeout", type=float, default=60.0)
    parser.add_argument("--insecure", action="store_true")
    parser.add_argument("--utc", action="append", help="UTC epoch, repeatable")
    parser.add_argument("--eop", help="path to finals2000A.all for the UT1 section")
    parser.add_argument("--skip-ut1", action="store_true", help="omit the UT1 section")
    parser.add_argument(
        "--skip-tdb-model", action="store_true", help="omit the ET vs IAU TDB section"
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    times = args.utc or DEFAULT_TIMES
    passed = True

    print("=== TDB conversion ===")
    print(f"WGC2 kernel set: {args.kernel_set}")
    print(f"UTC inputs: {len(times)}")

    wgc_values = run_wgc2(times, args.kernel_set, args.insecure, args.timeout)
    crate_values = run_crate(times, repo_root)
    golden = load_golden(repo_root)

    if len(wgc_values) != len(times):
        raise RuntimeError(
            f"WGC2 returned {len(wgc_values)} rows for {len(times)} inputs"
        )

    print()
    print(f"{'UTC':30} {'WGC2 TDB':>18} {'crate TDB':>18} {'diff':>10}")
    print("-" * 80)

    worst = 0.0
    for utc, wgc_et, crate_et in zip(times, wgc_values, crate_values):
        difference = abs(wgc_et - crate_et)
        worst = max(worst, difference)
        print(f"{utc:30} {wgc_et:18.6f} {crate_et:18.6f} {difference:10.3e}")

    print("-" * 80)
    print(
        f"max WGC2 vs crate difference: {worst:.3e} s "
        f"(tolerance {args.tolerance:.1e})"
    )

    if golden:
        worst_golden = 0.0
        for utc, crate_et in zip(times, crate_values):
            if utc in golden:
                worst_golden = max(worst_golden, abs(crate_et - golden[utc]))
        print(f"max crate vs CSPICE golden difference: {worst_golden:.3e} s")

    if worst > args.tolerance:
        print("TDB: FAIL", file=sys.stderr)
        passed = False
    else:
        print("TDB: PASS")

    if not args.skip_tdb_model:
        print()
        print("=== TDB model: NAIF ET vs IAU ===")
        print("NAIF ET is what SPICE/WGC2 use; IAU TDB is the formal definition.")
        model_rows = run_tdb_compare(times, repo_root)
        print()
        print(f"{'UTC':30} {'ET - IAU TDB (us)':>20}")
        print("-" * 52)
        worst_model = 0.0
        for utc, _et, _tdb, difference in model_rows:
            worst_model = max(worst_model, abs(difference))
            print(f"{utc:30} {difference:20.6f}")
        print("-" * 52)
        print(f"largest over these epochs: {worst_model:.3f} us")
        print("this is a single-epoch value; across a year the difference")
        print("ranges about -29 to +29 us, consistent with NAIF's ~30 us bound.")
        print("the microsecond decimals are arithmetic on two f64 formulas,")
        print("not a claim of microsecond-level model accuracy.")

    if not args.skip_ut1:
        print()
        print("=== UT1 ===")
        ut1_status = probe_wgc2_ut1(args.insecure)
        if ut1_status == "SUPPORTED":
            print("WGC2 UT1: SUPPORTED (unexpected; update this report)")
        else:
            print(f"WGC2 UT1: not supported ({ut1_status})")
        print("UT1 is validated against IERS data and leap physics:")
        if not run_ut1_validation(repo_root, args.insecure, args.eop):
            print("UT1: FAIL", file=sys.stderr)
            passed = False

    print()
    print("OVERALL: PASS" if passed else "OVERALL: FAIL")
    return 0 if passed else 1


if __name__ == "__main__":
    sys.exit(main())
