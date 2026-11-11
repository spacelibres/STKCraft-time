#!/usr/bin/env python3
"""Validate the Rust SCLK type-1 implementation against CSPICE.

`src/bin/sclk_check.rs` converts encoded ticks to ET with this crate.
This script does the same with CSPICE `sct2e` and compares the two, for
sample ticks drawn from the kernel's own coefficient boundaries. The inverse
path is independently referenced to CSPICE SCE2C, not to the original ticks.
Both directions default to bit-exact acceptance; tolerance units are separate.

Offline tooling only; needs `spiceypy`.

Usage:
    python tools/validate_sclk.py --sclk target/mro_sclk.tsc --id -74
"""

from __future__ import annotations

import argparse
import math
import re
import subprocess
import sys
from pathlib import Path

import struct

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


def parse_results(output, samples):
    expected = {struct.pack(">d", x) for x in samples}
    rows = {}
    for line in output.splitlines():
        fields = line.split(",")
        if len(fields) != 2:
            raise ValueError("expected two numeric columns")
        key, value = map(float, fields)
        if not math.isfinite(key) or not math.isfinite(value):
            raise ValueError("non-finite client result")
        encoded = struct.pack(">d", key)
        if encoded in rows:
            raise ValueError("duplicate client input")
        rows[encoded] = value
    if set(rows) != expected:
        raise ValueError("missing or unexpected client input")
    return [rows[struct.pack(">d", x)] for x in samples]


def finite_nonnegative(value):
    number = float(value)
    if not math.isfinite(number) or number < 0:
        raise argparse.ArgumentTypeError("tolerance must be finite and nonnegative")
    return number


def main() -> int:
    import spiceypy as spice
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--sclk", required=True)
    parser.add_argument("--id", type=int, required=True, help="signed clock id")
    parser.add_argument("--lsk", default="src/naif0012.tls")
    parser.add_argument("--samples", type=int, default=64)
    parser.add_argument("--tolerance", type=finite_nonnegative, default=0.0, help="explicit forward seconds tolerance; default bit-exact")
    parser.add_argument("--tick-tolerance", type=finite_nonnegative, default=0.0, help="explicit inverse ticks tolerance; default bit-exact")
    args = parser.parse_args()
    if args.samples < 2:
        parser.error("samples must be at least two")
    spice.kclear()

    text = Path(args.sclk).read_text(encoding="utf-8", errors="replace")
    ticks = coefficient_ticks(text, abs(args.id))
    spice.furnsh(args.lsk)
    spice.furnsh(args.sclk)
    pool_ticks = [float(value) for value in spice.gdpool(
        f"SCLK01_COEFFICIENTS_{abs(args.id)}", 0, 1_000_000)[0::3]]
    first, last = pool_ticks[0], pool_ticks[-1]
    step = (last - first) / max(args.samples - 1, 1)
    samples = [first + step * index for index in range(args.samples)]
    starts, ends = spice.scpart(args.id)
    # NAIF normalizes each partition readout to integral ticks. The
    # cumulative domain, not the final raw readout, bounds encoded ticks.
    maximum = sum(math.floor(float(end) + 0.5) - math.floor(float(start) + 0.5)
                  for start, end in zip(starts, ends))
    boundaries = [query for value in pool_ticks
                  for query in [math.nextafter(value, -math.inf), value,
                                math.nextafter(value, math.inf)]
                  if first <= query <= maximum]
    samples = sorted({value for value in samples + ticks + boundaries + [first, float(maximum)]
                      if first <= value <= maximum})

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
        "--lsk", str(Path(args.lsk).resolve()),
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

    ours = parse_results(process.stdout, samples)
    expected = [float(spice.sct2e(args.id, value)) for value in samples]
    differences = [abs(actual - reference) for actual, reference in zip(ours, expected)]
    worst = max(differences)
    worst_ticks = samples[differences.index(worst)]
    forward_bit_mismatches = sum(struct.pack(">d", a) != struct.pack(">d", b) for a, b in zip(ours, expected))
    inverse_inputs = sorted(set(expected))
    inverse = subprocess.run(command + ["--inverse"], cwd=repo_root,
                             input="\n".join(map(repr, inverse_inputs)), text=True,
                             capture_output=True, check=True)
    inverse_ours = parse_results(inverse.stdout, inverse_inputs)
    inverse_expected = [float(spice.sce2c(args.id, et)) for et in inverse_inputs]
    inverse_worst = max(abs(a-b) for a, b in zip(inverse_ours, inverse_expected))
    inverse_bit_mismatches = sum(struct.pack(">d", a) != struct.pack(">d", b) for a, b in zip(inverse_ours, inverse_expected))

    for invalid in [math.nextafter(first, -math.inf),
                    math.nextafter(float(maximum), math.inf)]:
        try:
            spice.sct2e(args.id, invalid)
        except spice.utils.exceptions.SpiceVALUEOUTOFRANGE:
            pass
        else:
            raise AssertionError(f"CSPICE unexpectedly accepted out-of-domain tick {invalid!r}")
        rejected = subprocess.run(command, cwd=repo_root, input=f"{invalid!r}\n",
                                  text=True, capture_output=True)
        if rejected.returncode != 1 or "conversion error" not in rejected.stderr:
            raise AssertionError(f"crate did not reject out-of-domain tick {invalid!r}")
    spice.kclear()

    print(f"samples: {len(samples)}, ticks {first:.0f} .. {maximum:.0f}")
    print(f"max |crate - CSPICE sct2e| = {worst:.3e} s at ticks {worst_ticks:.0f}")
    print(f"encoded domain endpoints and out-of-domain rejection checked: {first} .. {maximum}")
    print(f"inverse samples: {len(inverse_inputs)}, max |crate - CSPICE sce2c| = {inverse_worst:.17g} ticks")
    print(f"bit mismatches: forward {forward_bit_mismatches}, inverse {inverse_bit_mismatches}")
    if (worst > args.tolerance or inverse_worst > args.tick_tolerance
            or (args.tolerance == 0 and forward_bit_mismatches)
            or (args.tick_tolerance == 0 and inverse_bit_mismatches)):
        print("FAIL", file=sys.stderr)
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
