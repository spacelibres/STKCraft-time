#!/usr/bin/env python3
"""Generate TDB-TT golden vectors from ERFA over 1950-2050.

The reference is ERFA `dtdb` (IAU SOFA `iauDtdb`) called with `u = v = 0`,
i.e. the geocentric Fairhead & Bretagnon series. Used by
`tests/dtdb_iau.rs` to check the Rust port of the same series.

Offline tooling only.

Usage:
    pip install pyerfa
    python tools/gen_dtdb_golden.py --out tests/golden/dtdb.csv
"""

from __future__ import annotations

import argparse
import math
from pathlib import Path

import erfa

START_JD = 2433282.5  # 1950-01-01
END_JD = 2469807.5  # 2050-01-01
STEP_DAYS = 30.0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="tests/golden/dtdb.csv")
    args = parser.parse_args()

    rows = []
    jd = START_JD
    while jd <= END_JD:
        jd1 = math.floor(jd)
        jd2 = jd - jd1
        value = float(erfa.dtdb(jd1, jd2, 0.0, 0.0, 0.0, 0.0))
        rows.append((jd, value))
        jd += STEP_DAYS

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write(
            "# TDB-TT in seconds from ERFA dtdb(jd1,jd2,0,0,0,0), "
            "geocentric Fairhead & Bretagnon series.\n"
        )
        handle.write("# 1950-01-01 .. 2050-01-01, step 30 days.\n")
        handle.write("jd_tt,dtdb\n")
        for jd_value, dtdb in rows:
            handle.write(f"{jd_value!r},{dtdb!r}\n")

    print(f"wrote {out} ({len(rows)} rows)")
    worst = max(abs(value) for _, value in rows)
    print(f"max |dtdb| in range: {worst:.6e} s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
