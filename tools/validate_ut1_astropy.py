#!/usr/bin/env python3
"""Compare this crate's UT1-UTC with astropy's IERS tables.

astropy is a widely used independent implementation of the IERS
interpolation. This script compares the crate's `FinalPreferred` series
(Bulletin B where present, otherwise Bulletin A) with astropy's combined
`UT1_UTC` column over a dense grid.

Two regimes are reported separately:

  * ordinary days — should agree to about a nanosecond;
  * leap-second days — astropy and this crate use slightly different
    leap-aware interpolation, so agreement is at the microsecond level
    rather than the nanosecond level.

Offline tooling only; needs `astropy`.

Usage:
    python tools/validate_ut1_astropy.py
    python tools/validate_ut1_astropy.py --eop target/eop/finals2000A.all
"""

from __future__ import annotations

import argparse
import math
import subprocess
import sys
from pathlib import Path

import numpy as np
from astropy.time import Time
from astropy.utils.iers import IERS_A


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--eop", default="target/eop/finals2000A.all")
    parser.add_argument("--step", type=int, default=23)
    parser.add_argument("--tolerance-normal", type=float, default=1e-6)
    parser.add_argument("--tolerance-leap", type=float, default=1e-4)
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    eop_path = (repo_root / args.eop).resolve()

    table = IERS_A.open(str(eop_path))
    mjd = np.asarray(table["MJD"].value)
    combined = np.asarray(table["UT1_UTC"].value)

    first = int(math.ceil(mjd[0]))
    last = int(math.floor(mjd[-1])) - 1

    # Integer MJD -> combined UT1-UTC, for leap-boundary detection.
    lut = {int(round(value)): combined[i] for i, value in enumerate(mjd)}

    def is_leap_day(day: int) -> bool:
        here = lut.get(day)
        nxt = lut.get(day + 1)
        prev = lut.get(day - 1)
        after = lut.get(day)
        big_forward = here is not None and nxt is not None and abs(nxt - here) > 0.5
        big_before = prev is not None and after is not None and abs(after - prev) > 0.5
        return big_forward or big_before

    grid = list(range(first, last, args.step))
    samples = [(day, 0.0) for day in grid] + [(day, 0.5) for day in grid]

    iso = []
    references = []
    leap_flags = []
    for day, fraction in samples:
        jd = day + 2400000.5 + fraction
        jd1 = math.floor(jd)
        jd2 = jd - jd1
        iso.append(Time(jd1, jd2, format="jd", scale="utc").isot)
        references.append(float(table.ut1_utc(jd1, jd2).value))
        leap_flags.append(is_leap_day(day))

    command = [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        "ut1-check",
        "--",
        str(eop_path),
        "final",
    ]
    process = subprocess.run(
        command,
        cwd=repo_root,
        input="\n".join(iso),
        text=True,
        capture_output=True,
    )
    if process.returncode != 0:
        print(process.stdout, end="")
        print(process.stderr, end="", file=sys.stderr)
        return 1

    ours = {}
    for line in process.stdout.splitlines():
        if not line.strip():
            continue
        utc, value = line.rsplit(",", 1)
        ours[utc] = float(value)

    worst_normal = 0.0
    worst_leap = 0.0
    worst_label = ""
    for utc, reference, leap in zip(iso, references, leap_flags):
        actual = ours[utc]
        difference = abs(actual - reference)
        if leap:
            worst_leap = max(worst_leap, difference)
        else:
            if difference > worst_normal:
                worst_normal = difference
                worst_label = utc

    print(f"samples: {len(samples)} (independent implementation: astropy IERS)")
    print(f"max |crate - astropy| on ordinary days = {worst_normal:.3e} s")
    if worst_label:
        print(f"  worst ordinary day: {worst_label}")
    print(f"max |crate - astropy| on leap days     = {worst_leap:.3e} s")

    if worst_normal > args.tolerance_normal:
        print("FAIL: ordinary days differ too much", file=sys.stderr)
        return 1
    if worst_leap > args.tolerance_leap:
        print("FAIL: leap days differ too much", file=sys.stderr)
        return 1

    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
