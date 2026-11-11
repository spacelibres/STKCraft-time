#!/usr/bin/env python3
"""Generate independent observer TDB-TT vectors with PyERFA 2.0.1.5.

Requires numpy and pyerfa==2.0.1.5. This tool does not call the Rust port.
The pinned upstream source hash is checked before generating the oracle.
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import io
import math
from pathlib import Path
from urllib.request import urlopen

import erfa

ERFA_COMMIT = "1a8044cde5b7763295d472a6443387239127c6c8"
SOURCE_URL = f"https://raw.githubusercontent.com/liberfa/erfa/{ERFA_COMMIT}/src/dtdb.c"
SOURCE_SHA256 = "228bd7c7c1c88dda44c29a076f289c968f64d77a2dc389138b03ab5d1d442105"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="tests/golden/dtdb_observer.csv")
    args = parser.parse_args()
    if erfa.__version__ != "2.0.1.5":
        raise SystemExit("Reproducible oracle requires pyerfa==2.0.1.5")
    with urlopen(SOURCE_URL, timeout=60) as response:
        digest = hashlib.sha256(response.read()).hexdigest()
    if digest != SOURCE_SHA256:
        raise SystemExit(f"Pinned ERFA source hash mismatch: {digest}")

    # Includes the canonical ERFA t_erfa_c.c t_dtdb test and both domain ends.
    cases = [(2448939.5, 0.123, 0.76543, 5.0123, 5525.242, 3190.0)]
    observers = [
        (0.0, 0.0, 0.0, 0.0),  # geocentre
        (0.0, 0.0, 6378.137, 0.0),  # equator at UT1 midnight
        (0.25, math.pi / 2, 4517.6, 4487.3),
        (0.5, -2.1, 4517.6, -4487.3),
        (0.76543, 5.0123, 5525.242, 3190.0),
        (0.999999999, -math.pi, 0.0, 6356.752),  # north pole
        (0.125, 2 * math.pi, 0.0, -6356.752),  # south pole
    ]
    dates = [2433282.5 + i * 365.25 for i in range(101)]
    dates += [2451545.0 + 123.0, 2469807.5]
    for jd in dates:
        # J2000 split is best suited to the model's time argument.
        for ut1, longitude, u, v in observers:
            cases.append((2451545.0, jd - 2451545.0, ut1, longitude, u, v))
    cases += [
        (2451545.0, 1e-10, 0.25, -1.0, 6378.137, 0.0),
        (2400000.5, 48939.123, 0.76543, 5.0123, 5525.242, 3190.0),
    ]
    output = io.StringIO(newline="")
    output.write(f"# Oracle: PyERFA {erfa.__version__}; bundled ERFA {erfa.version.erfa_version}\n")
    output.write(f"# ERFA source: {SOURCE_URL}\n")
    output.write(f"# Source SHA256: {SOURCE_SHA256}\n")
    output.write("# SOFA-derived ERFA model; see ATTRIBUTION.md for license.\n")
    output.write("# Numerical agreement with an oracle is not an absolute physical accuracy bound.\n")
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["jd1", "jd2", "ut1_fraction", "east_longitude_radians", "u_km", "v_km", "tdb_minus_tt_seconds"])
    for case in cases:
        writer.writerow([*(repr(value) for value in case), repr(float(erfa.dtdb(*case)))])
    path = Path(args.out)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(output.getvalue(), encoding="utf-8", newline="\n")
    print(f"Wrote {path} ({len(cases)} vectors)")


if __name__ == "__main__":
    main()
