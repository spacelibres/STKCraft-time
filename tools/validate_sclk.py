#!/usr/bin/env python3
"""Validate the Rust SCLK type-1 implementation against CSPICE.

`src/bin/sclk_check.rs` converts encoded ticks to ET with this crate.
This script does the same with CSPICE `sct2e` and compares the two, for
sample ticks drawn from the kernel's own coefficient boundaries.

Offline tooling only; needs `spiceypy`.

Usage:
    python tools/validate_sclk.py --sclk target/mro_sclk.tsc --id -74
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

import spiceypy as spice

NUMBER = re.compile(r"[-+]?\d+(?:\.\d*)?(?:[EeDd][-+]?\d+)?")


def coefficient_ticks(text: str, magnitude: int) -> list[float]:
    match = re.search(
        rf"SCLK01_COEFFICIENTS_{magnitude}\s*=\s*\((.*?)\)", text, re.S
    )
    if not match:
        raise SystemExit(f"no SCLK01_COEFFICIENTS_{magnitude} found")
    values = []
    for token in NUMBER.findall(match.group(1)):
        try:
            values.append(float(token.replace("D", "E").replace("d", "e")))
        except ValueError:
            continue
    if len(values) < 3 or len(values) % 3 != 0:
        raise SystemExit("coefficient count is not a multiple of three")
    return values[0::3]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sclk", required=True)
    parser.add_argument("--id", type=int, required=True, help="signed clock id")
    parser.add_argument("--lsk", default="src/naif0012.tls")
    parser.add_argument("--samples", type=int, default=64)
    parser.add_argument("--tolerance", type=float, default=1e-6)
    args = parser.parse_args()

    text = Path(args.sclk).read_text(encoding="utf-8", errors="replace")
    ticks = coefficient_ticks(text, abs(args.id))
    first, last = ticks[0], ticks[-1]
    step = (last - first) / max(args.samples - 1, 1)
    samples = [first + step * index for index in range(args.samples)]

    repo_root = Path(__file__).resolve().parent.parent
    command = [
        "cargo",
        "run",
        "--quiet",
        "--bin",
        "sclk-check",
        "--",
        args.sclk,
        str(args.id),
    ]
    process = subprocess.run(
        command,
        cwd=repo_root,
        input="\n".join(f"{value!r}" for value in samples),
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
        tick_text, et_text = line.split(",")
        ours[float(tick_text)] = float(et_text)

    spice.furnsh(args.lsk)
    spice.furnsh(args.sclk)

    worst = 0.0
    worst_ticks = 0.0
    for value in samples:
        expected = spice.sct2e(args.id, value)
        actual = ours[value]
        difference = abs(actual - expected)
        if difference > worst:
            worst = difference
            worst_ticks = value

    spice.kclear()

    print(f"samples: {len(samples)}, ticks {first:.0f} .. {last:.0f}")
    print(f"max |crate - CSPICE sct2e| = {worst:.3e} s at ticks {worst_ticks:.0f}")
    if worst > args.tolerance:
        print("FAIL", file=sys.stderr)
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
