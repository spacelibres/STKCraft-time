#!/usr/bin/env python3
"""Validate this crate's UT1 against the real IERS data and leap physics.

The JPL WebGeocalc / CSPICE have no UT1 time system (the WGC2 API rejects
``"timeSystem": "UT1"`` with ``No enum constant ...TimeStandard.UT1``),
so UT1 cannot be compared against WGC2. It is validated here against:

  1. Data fidelity: at ``00:00:00 UTC`` every day, this crate's
     ``UT1 - UTC`` must equal the ``finals2000A.all`` entry for that MJD.
  2. Leap physics: on a positive-leap-second day, ``UT1 - UTC`` is
     essentially constant across the day (it drifts by LOD, a few ms),
     then steps by one second at the day boundary. Naive interpolation
     would smear that step and be ~0.5 s wrong at 12:00.

The script runs the ``ut1-check`` binary on the real file. It uses only
the Python standard library plus the crate's own binary.

Usage:
    python tools/validate_ut1.py
    python tools/validate_ut1.py --eop target/eop/finals2000A.all
    python tools/validate_ut1.py --url https://mirror.example/finals2000A.all
    python tools/validate_ut1.py --insecure   # if TLS verification fails

The download endpoint can also be set once with the ``STKCRAFT_EOP_URL``
environment variable; ``--url`` wins over the environment.
"""

from __future__ import annotations

import argparse
import os
import ssl
import subprocess
import sys
import urllib.request
from pathlib import Path

DEFAULT_URL = "https://datacenter.iers.org/data/9/finals2000A.all"
ENV_URL = "STKCRAFT_EOP_URL"


def ensure_eop(path: Path, insecure: bool, url: str) -> str:
    if path.exists():
        return path.read_text(encoding="utf-8", errors="replace")

    context = ssl._create_unverified_context() if insecure else ssl.create_default_context()
    request = urllib.request.Request(url, headers={"User-Agent": "stkcraft-time"})
    with urllib.request.urlopen(request, context=context, timeout=120) as response:
        payload = response.read()
    path.parent.mkdir(parents=True, exist_ok=True)
    # Write the bytes verbatim; text mode would translate line endings.
    path.write_bytes(payload)
    return payload.decode("utf-8", errors="replace")


def expand_year(two_digit: str) -> int:
    value = int(two_digit)
    return 1900 + value if value >= 73 else 2000 + value


def parse_rows(text: str) -> list[dict]:
    rows = []
    for line in text.splitlines():
        if len(line) < 68:
            continue
        ut1_field = line[58:68].strip()
        mjd_field = line[6:15].strip()
        if not ut1_field or not mjd_field:
            continue
        rows.append(
            {
                "year": expand_year(line[0:2]),
                "month": int(line[2:4]),
                "day": int(line[4:6]),
                "mjd": float(mjd_field),
                "ut1": float(ut1_field),
            }
        )
    return rows


def date_string(row: dict, hour: int) -> str:
    return f"{row['year']:04d}-{row['month']:02d}-{row['day']:02d}T{hour:02d}:00:00"


def run_bin(repo_root: Path, eop_path: Path, lines: list[str]) -> dict[str, str]:
    command = ["cargo", "run", "--quiet", "--bin", "ut1-check", "--", str(eop_path)]
    process = subprocess.run(
        command,
        cwd=repo_root,
        input="\n".join(lines),
        text=True,
        capture_output=True,
    )
    if process.returncode != 0:
        raise RuntimeError(
            "ut1-check failed\n"
            f"stdout:\n{process.stdout}\n"
            f"stderr:\n{process.stderr}"
        )

    values: dict[str, str] = {}
    for line in process.stdout.splitlines():
        if not line.strip():
            continue
        utc, value = line.rsplit(",", 1)
        values[utc] = value
    return values


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--eop",
        default="target/eop/finals2000A.all",
        help="path to finals2000A.all (downloaded if missing)",
    )
    parser.add_argument(
        "--url",
        default=os.environ.get(ENV_URL, DEFAULT_URL),
        help=(
            "source URL used when --eop is missing "
            f"(default: ${ENV_URL} or {DEFAULT_URL})"
        ),
    )
    parser.add_argument("--insecure", action="store_true")
    parser.add_argument(
        "--max-zero-diff",
        type=float,
        default=1e-6,
        help="tolerance for the 00:00 UTC fidelity check, seconds",
    )
    args = parser.parse_args()

    repo_root = Path(__file__).resolve().parent.parent
    eop_path = (repo_root / args.eop).resolve()

    text = ensure_eop(eop_path, args.insecure, args.url)
    rows = parse_rows(text)
    if len(rows) < 2:
        print("not enough EOP rows", file=sys.stderr)
        return 1

    # Consecutive, daily rows only; skip the final row (it has no next day).
    days = [
        (index, rows[index], rows[index + 1])
        for index in range(len(rows) - 1)
        if rows[index + 1]["mjd"] - rows[index]["mjd"] == 1.0
    ]

    zero_lines = [date_string(row, 0) for _, row, _ in days]
    zero_values = run_bin(repo_root, eop_path, zero_lines)

    worst_zero = 0.0
    worst_label = ""
    checked = 0
    for _, row, _ in days:
        label = date_string(row, 0)
        value = zero_values.get(label, "")
        if value.startswith("ERR") or value.startswith("PARSE"):
            print(f"unexpected failure at {label}: {value}", file=sys.stderr)
            return 1
        difference = abs(float(value) - row["ut1"])
        if difference > worst_zero:
            worst_zero = difference
            worst_label = label
        checked += 1

    print(f"00:00 UTC fidelity: {checked} days, max |crate - table| = {worst_zero:.3e} s")
    if worst_label:
        print(f"  worst day: {worst_label}")
    if worst_zero > args.max_zero_diff:
        print("FAIL: 00:00 UTC does not reproduce the table", file=sys.stderr)
        return 1

    # Leap-boundary physics: big daily steps are positive leap seconds.
    leap_days = [
        (index, row, following)
        for index, row, following in days
        if following["ut1"] - row["ut1"] > 0.5
    ]
    print(f"leap-boundary days detected: {len(leap_days)}")

    if leap_days:
        noon_lines = [date_string(row, 12) for _, row, _ in leap_days]
        noon_values = run_bin(repo_root, eop_path, noon_lines)

        worst_aware = 0.0
        smallest_naive = float("inf")
        for _, row, following in leap_days:
            label = date_string(row, 12)
            value = noon_values.get(label, "")
            if value.startswith("ERR") or value.startswith("PARSE"):
                print(f"unexpected failure at {label}: {value}", file=sys.stderr)
                return 1
            aware = float(value)
            naive = 0.5 * (row["ut1"] + following["ut1"])
            worst_aware = max(worst_aware, abs(aware - row["ut1"]))
            smallest_naive = min(smallest_naive, abs(naive - row["ut1"]))

        print(f"  max |leap-aware noon - day start| = {worst_aware:.3e} s")
        print(f"  min |naive noon - day start|      = {smallest_naive:.3e} s")
        if worst_aware > 0.01:
            print("FAIL: leap-aware value drifts across the day", file=sys.stderr)
            return 1
        if smallest_naive < 0.4:
            print("FAIL: naive midpoint is not clearly wrong", file=sys.stderr)
            return 1

    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
