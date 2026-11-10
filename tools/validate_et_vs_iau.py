#!/usr/bin/env python3
"""Measure the NAIF ET vs IAU TDB difference with independent tools.

Both sides come from outside this crate:

  * NAIF ET: CSPICE `str2et`, via spiceypy.
  * IAU TDB: ERFA `dtdb` (Fairhead & Bretagnon), via astropy.

This explains why a single number is not a fixed "26 us": the difference
is an annual-ish curve of a few tens of microseconds. At
2000-01-01T12:00:00 UTC it is +26.57 us; across a year it ranges roughly
-29 to +29 us, matching NAIF's stated ~30 us model bound.

Offline tooling only; needs `spiceypy` and `astropy`.

Usage:
    python tools/validate_et_vs_iau.py --year 2000
    python tools/validate_et_vs_iau.py --lsk src/naif0012.tls --year 2024
"""

from __future__ import annotations

import argparse
import calendar
import datetime

import spiceypy as spice
from astropy.time import Time

JD_J2000 = 2451545.0


def iau_tdb_seconds(iso: str) -> float:
    time = Time(iso, scale="utc")
    tr = time.tdb
    return (tr.jd1 - JD_J2000) * 86400.0 + tr.jd2 * 86400.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lsk", default="src/naif0012.tls")
    parser.add_argument("--year", type=int, default=2000)
    args = parser.parse_args()

    spice.furnsh(args.lsk)

    # Fixed reference epoch, independent of --year.
    iso = "2000-01-01T12:00:00"
    et = spice.str2et(iso)
    tdb = iau_tdb_seconds(iso)
    print(f"reference {iso}: ET - IAU TDB = {(et - tdb) * 1e6:+.6f} us")

    days = 366 if calendar.isleap(args.year) else 365
    base = datetime.datetime(args.year, 1, 1)
    values = []
    for offset in range(days):
        day = (base + datetime.timedelta(days=offset)).strftime("%Y-%m-%dT%H:%M:%S")
        difference = (spice.str2et(day) - iau_tdb_seconds(day)) * 1e6
        values.append((difference, day))

    spice.kclear()

    low = min(values)
    high = max(values)
    worst = max(values, key=lambda item: abs(item[0]))
    print(f"{args.year}: min {low[0]:+.3f} us ({low[1][:10]}), "
          f"max {high[0]:+.3f} us ({high[1][:10]})")
    print(f"{args.year}: worst |ET - IAU TDB| = {abs(worst[0]):.3f} us "
          f"({worst[1][:10]})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
