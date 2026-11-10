#!/usr/bin/env python3
"""Generate golden time vectors with CSPICE.

This script is *offline tooling*. It is not part of the crate, is not
built by Cargo, and is never a runtime dependency. Its output is a CSV
file used by `tests/` to assert alignment with CSPICE.

Usage:
    pip install spiceypy
    python tools/gen_golden.py --lsk path/to/naif0012.tls \
        --out tests/golden/utc_to_et.csv

The generated file contains:
    utc, et
for a grid of epochs that includes every leap-second boundary.
"""

from __future__ import annotations

import argparse
import csv
from pathlib import Path

import spiceypy as spice


def leap_boundaries() -> list[str]:
    """Return UTC strings immediately around known positive leap seconds."""
    dates = [
        "1972-07-01",
        "1973-01-01",
        "1974-01-01",
        "1975-01-01",
        "1976-01-01",
        "1977-01-01",
        "1978-01-01",
        "1979-01-01",
        "1980-01-01",
        "1981-07-01",
        "1982-07-01",
        "1983-07-01",
        "1985-07-01",
        "1988-01-01",
        "1990-01-01",
        "1991-01-01",
        "1992-07-01",
        "1993-07-01",
        "1994-07-01",
        "1996-01-01",
        "1997-07-01",
        "1999-01-01",
        "2006-01-01",
        "2009-01-01",
        "2012-07-01",
        "2015-07-01",
        "2017-01-01",
    ]

    values = ["2000-01-01T12:00:00"]
    for date in dates:
        values.append(f"{date}T00:00:00")
        values.append(f"{date}T00:00:00.000000001")
        # The leap second itself is named on the previous day.
        year, month, day = (int(part) for part in date.split("-"))
        if (month, day) == (1, 1):
            prev = f"{year - 1}-12-31T23:59:60.5"
        else:
            prev = f"{year}-06-30T23:59:60.5"
        values.append(prev)

    return values


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--lsk", required=True, help="path to a NAIF LSK")
    parser.add_argument("--out", required=True, help="output CSV path")
    args = parser.parse_args()

    spice.furnsh(args.lsk)

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)

    with out.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.writer(handle)
        writer.writerow(["utc", "et"])
        for utc in leap_boundaries():
            et = spice.str2et(utc)
            writer.writerow([utc, repr(et)])

    spice.kclear()


if __name__ == "__main__":
    main()
