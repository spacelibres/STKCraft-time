#!/usr/bin/env python3
"""Generate nominal BDT vectors using independent ERFA UTC-to-TAI conversion.

Requires pyerfa (offline tooling only). The BDS ICD time-system definition
sets the origin to 2006-01-01 00:00:00 UTC and nominal TAI-BDT to 33 seconds.
ERFA computes the TAI calendar fields independently, including leap seconds;
Python Gregorian date arithmetic then expresses these as exact integer parts.
No STKCraft implementation or output is used to derive expected values.

Run: python tools/gen_bdt_golden.py --out tests/golden/bdt.csv
"""

import argparse
import csv
from datetime import date, timedelta
from pathlib import Path

import erfa


def vectors():
    """Origin, pre-origin, post-origin leap boundaries and fractional epochs."""
    yield (1980, 1, 6, 0, 0, 0.0)
    yield (2000, 1, 1, 12, 0, 0.25)
    for boundary in [date(2006, 1, 1), date(2009, 1, 1), date(2012, 7, 1),
                     date(2015, 7, 1), date(2017, 1, 1)]:
        previous = boundary - timedelta(days=1)
        for second in [59.0, 60.0, 60.5]:
            yield (*previous.timetuple()[:3], 23, 59, second)
        yield (*boundary.timetuple()[:3], 0, 0, 0.0)
        yield (*boundary.timetuple()[:3], 0, 0, 0.25)
    yield (2026, 11, 12, 0, 0, 0.5)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()
    # Derive the origin via ERFA rather than a copied crate constant.
    origin_utc = erfa.dtf2d("UTC", 2006, 1, 1, 0, 0, 0.0)
    origin_tai = erfa.utctai(*origin_utc)
    oy, om, od, ohms = erfa.d2dtf("TAI", 9, *origin_tai)
    origin_day = date(int(oy), int(om), int(od))
    origin_ns = ((int(ohms[0]) * 60 + int(ohms[1])) * 60 + int(ohms[2])) * 10**9 + int(ohms[3])
    args.out.parent.mkdir(parents=True, exist_ok=True)
    with args.out.open("w", newline="", encoding="utf-8") as handle:
        handle.write(f"# Generated with pyerfa {erfa.__version__}; ERFA utctai/dtf2d/d2dtf; nominal BDS ICD origin\n")
        writer = csv.writer(handle)
        writer.writerow(["year", "month", "day", "hour", "minute", "second", "bdt_sec", "bdt_nanos"])
        for fields in vectors():
            tai = erfa.utctai(*erfa.dtf2d("UTC", *fields))
            y, m, d, hms = erfa.d2dtf("TAI", 9, *tai)
            ns = ((date(int(y), int(m), int(d)) - origin_day).days * 86400
                  + int(hms[0]) * 3600 + int(hms[1]) * 60 + int(hms[2])) * 10**9 + int(hms[3]) - origin_ns
            sec, nanos = divmod(ns, 10**9)
            writer.writerow([*fields, sec, nanos])


if __name__ == "__main__":
    main()
