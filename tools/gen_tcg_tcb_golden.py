#!/usr/bin/env python3
"""Generate TCG/TCB golden vectors with ERFA (via astropy).

TCG and TCB are IAU coordinate scales. Neither the JPL WebGeocalc API nor
CSPICE exposes them (the WGC2 API rejects `"outputTimeSystem": "TCG"` and
`"TCB"` with `No enum constant ...TimeStandard.TCG/TCB`). The independent
reference is therefore the IAU implementation in ERFA/SOFA, reached here
through astropy's `tcg` and `tcb` time scales.

This script is offline tooling. It is not built by Cargo and is never a
runtime dependency.

Usage:
    pip install astropy
    python tools/gen_tcg_tcb_golden.py --out tests/golden/tcg_tcb.csv

Columns are `tai,tt,tdb,tcg,tcb`, all in seconds past Julian Date
2451545.0 on the respective scale, for TAI input epochs.
"""

from __future__ import annotations

import argparse
from pathlib import Path

try:
    from astropy.time import Time
    import astropy.units as u
except ImportError as exc:  # pragma: no cover - tooling only
    raise SystemExit("astropy is required: pip install astropy") from exc

JD_J2000 = 2451545.0
SEC_PER_DAY = 86400.0

# TAI epochs, including the IAU reference epoch T0 = 1977-01-01T00:00:00.
EPOCHS = [
    "1977-01-01T00:00:00",
    "1980-01-06T00:00:00",
    "2000-01-01T12:00:00",
    "2016-12-31T12:00:00",
    "2024-01-01T00:00:00",
    "2050-01-01T00:00:00",
]


def seconds_past_j2000(time, scale: str) -> float:
    tr = getattr(time, scale)
    return (tr.jd1 - JD_J2000) * SEC_PER_DAY + tr.jd2 * SEC_PER_DAY


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="tests/golden/tcg_tcb.csv")
    parser.add_argument(
        "--dense",
        action="store_true",
        help="sample a regular grid from --start to --end",
    )
    parser.add_argument("--start", default="1972-01-01T00:00:00")
    parser.add_argument("--end", default="2050-01-01T00:00:00")
    parser.add_argument("--step-days", type=int, default=30)
    args = parser.parse_args()

    if args.dense:
        start = Time(args.start, scale="tai", format="isot")
        end = Time(args.end, scale="tai", format="isot")
        count = int((end.jd - start.jd) / args.step_days)
        epochs = [
            (start + index * args.step_days * u.day).isot
            for index in range(count + 1)
        ]
    else:
        epochs = EPOCHS

    rows = []
    for iso in epochs:
        time = Time(iso, scale="tai", format="isot")
        rows.append(
            (
                iso,
                seconds_past_j2000(time, "tt"),
                seconds_past_j2000(time, "tdb"),
                seconds_past_j2000(time, "tcg"),
                seconds_past_j2000(time, "tcb"),
            )
        )

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    with out.open("w", encoding="utf-8", newline="\n") as handle:
        handle.write(
            "# Measured from ERFA (IAU SOFA) via astropy Time scales "
            "tcg/tcb.\n"
        )
        handle.write(
            "# Values are seconds past JD 2451545.0 on each scale, for TAI "
            "input epochs.\n"
        )
        handle.write("tai,tt,tdb,tcg,tcb\n")
        for row in rows:
            handle.write(
                f"{row[0]},{row[1]!r},{row[2]!r},{row[3]!r},{row[4]!r}\n"
            )

    print(f"wrote {out} ({len(epochs)} rows)")
    print(f"  {epochs[0]} .. {epochs[-1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
