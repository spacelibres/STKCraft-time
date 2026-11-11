#!/usr/bin/env python3
"""Generate UTC-contract fixtures exclusively with CSPICE N0067 STR2ET.

Requires spiceypy==8.3.0 and a local official N0067 src/cspice directory.
Pinned source hashes verify TTRANS/UNITIM provenance; no Rust code is called.
"""
import argparse
import csv
import hashlib
import io
from pathlib import Path
import struct

import spiceypy as spice

ROOT = Path(__file__).resolve().parents[1]
SOURCE_HASHES = {
    "ttrans.c": "9644a008c0dd12349b873eefb46821d444b224c24f9e26cb6e617bb95ec1d57d",
    "unitim.c": "cdc57024ff54190648b8e858d7489d70bdb856c39832628c8ba8159c2c0325fd",
    "tkvrsn.c": "933e466425d1bc248f18a120c0cd7006f0ce28ead3994be23b34e69a5a1873c2",
}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-directory", type=Path, required=True)
    parser.add_argument("--lsk", type=Path, default=ROOT/"src/naif0012.tls")
    parser.add_argument("--out", type=Path, default=ROOT/"tests/golden/contract_utc.csv")
    args = parser.parse_args()
    if spice.__version__ != "8.3.0" or spice.tkvrsn("TOOLKIT") != "CSPICE_N0067":
        raise SystemExit("Reproducible oracle requires spiceypy 8.3.0 and CSPICE N0067")
    for name, expected in SOURCE_HASHES.items():
        actual = hashlib.sha256((args.source_directory/name).read_bytes()).hexdigest()
        if actual != expected:
            raise SystemExit(f"Pinned N0067 source hash mismatch for {name}: {actual}")
    # Validated dates spanning the loaded leap table, transition days, J2000,
    # modern dates and future table-driven UTC. Each has fractional edge cases.
    dates = [
        (1972, 1, 1, 0, 0, 0), (1972, 6, 30, 23, 59, 60),
        (1981, 6, 30, 23, 59, 59), (1999, 1, 1, 0, 0, 0),
        (2000, 1, 1, 0, 0, 0), (2000, 1, 1, 12, 0, 0),
        (2015, 6, 30, 23, 59, 60), (2016, 12, 31, 23, 59, 59),
        (2016, 12, 31, 23, 59, 60), (2017, 1, 1, 0, 0, 0),
        (2024, 4, 8, 18, 0, 0), (2026, 11, 11, 0, 0, 0),
        (2049, 12, 31, 23, 59, 59),
    ]
    fractions = sorted({0, 1, 125000000, 123456789, 144272509, 999999999,
                        *((i * 2654435761) % 10**9 for i in range(1, 65))})
    output = io.StringIO(newline="")
    output.write(f"# Oracle: spiceypy {spice.__version__}; {spice.tkvrsn('TOOLKIT')} STR2ET\n")
    output.write("# Definition: https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/str2et_c.html\n")
    output.write("# Source: official N0067 src/cspice; source bytes verified before generation.\n")
    for name, digest in SOURCE_HASHES.items():
        output.write(f"# {name} SHA256: {digest}\n")
    output.write(f"# LSK: {args.lsk.name}; SHA256: {hashlib.sha256(args.lsk.read_bytes()).hexdigest()}\n")
    output.write("# UTC components use the Gregorian calendar and the supplied LSK.\n")
    output.write("# ET values are independent oracle outputs, not physical accuracy claims.\n")
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["year", "month", "day", "hour", "minute", "second", "et_seconds", "et_bits"])
    spice.kclear()
    try:
        spice.furnsh(str(args.lsk.resolve()))
        for year, month, day, hour, minute, second in dates:
            for fraction in fractions:
                seconds = f"{second:02}.{fraction:09}"
                utc = f"{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{seconds} UTC"
                et = float(spice.str2et(utc))
                writer.writerow([year, month, day, hour, minute, seconds, repr(et),
                                 struct.pack(">d", et).hex()])
    finally:
        spice.kclear()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(output.getvalue(), encoding="utf-8", newline="\n")
    print(f"Wrote {args.out} ({len(dates) * len(fractions)} independent vectors)")


if __name__ == "__main__":
    main()
