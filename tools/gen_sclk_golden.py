#!/usr/bin/env python3
"""Generate independent SCLK bit-pattern references with CSPICE N0067.

The two mission kernels are original official files, verified by SHA-256.
Diagnostic query values are inputs, never manufactured reference results.
"""
import csv
import hashlib
import io
import math
from pathlib import Path
import struct
import argparse
import spiceypy as spice

ROOT = Path(__file__).resolve().parents[1]
KERNELS = [
    ("MRO_SCLKSCET.00120.tsc", -74, "3d7d62528624421bdb5b74cfa842bb5eac7cbeab89211a4377c1c500fee76078",
     [159438732146.9826, 436452997776.49994]),
    ("juice_step_250412_v01.tsc", -28, "a4a987d81ee3be7c1586ad8d6d270cddb1b1d4fb356510847d961cbd208edfc1",
     [194529601196.84076, 70368791422407.77]),
]

def bits(value):
    return struct.pack(">d", value).hex()

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", type=Path, default=ROOT/"tests/golden/sclk.csv")
    args = parser.parse_args()
    if spice.__version__ != "8.3.0" or spice.tkvrsn("TOOLKIT") != "CSPICE_N0067":
        raise SystemExit("Oracle requires SpiceyPy 8.3.0 and CSPICE N0067")
    output = io.StringIO(newline="")
    output.write("# Oracle: SpiceyPy 8.3.0, CSPICE_N0067 SCT2E/SCE2C\n")
    output.write("# See tests/kernels/README.md for original kernel sources.\n")
    output.write("# Inputs are test queries; all expected bits are independent CSPICE outputs.\n")
    lsk = ROOT/"src/naif0012.tls"
    output.write(f"# LSK SHA256: {hashlib.sha256(lsk.read_bytes()).hexdigest()}\n")
    for name, _, digest, _ in KERNELS:
        if hashlib.sha256((ROOT/"tests/kernels"/name).read_bytes()).hexdigest() != digest:
            raise SystemExit(f"Kernel hash mismatch: {name}")
        output.write(f"# {name} SHA256: {digest}\n")
    writer = csv.writer(output, lineterminator="\n")
    writer.writerow(["clock", "direction", "input_bits", "expected_bits"])
    count = 0
    try:
        for name, clock, _, regressions in KERNELS:
            spice.kclear(); spice.furnsh(str(lsk)); spice.furnsh(str(ROOT/"tests/kernels"/name))
            coefficients = spice.gdpool(f"SCLK01_COEFFICIENTS_{abs(clock)}", 0, 1000000)
            starts, ends = spice.scpart(clock)
            maximum = sum(math.floor(float(e)+.5)-math.floor(float(b)+.5) for b, e in zip(starts, ends))
            first, last = float(coefficients[0]), float(coefficients[-3])
            ticks = set(regressions + [first, float(maximum)])
            ticks.update(first+(last-first)*i/1000 for i in range(1001))
            ticks.update(x for c in coefficients[::3] for x in
                         [math.nextafter(float(c), -math.inf), float(c), math.nextafter(float(c), math.inf)]
                         if first <= x <= maximum)
            for tick in sorted(ticks):
                et = float(spice.sct2e(clock, tick))
                writer.writerow([clock, "forward", bits(tick), bits(et)]); count += 1
                for query in [math.nextafter(et, -math.inf), et, math.nextafter(et, math.inf)]:
                    try:
                        encoded = float(spice.sce2c(clock, query))
                    except spice.utils.exceptions.SpiceVALUEOUTOFRANGE:
                        continue
                    writer.writerow([clock, "inverse", bits(query), bits(encoded)]); count += 1
    finally:
        spice.kclear()
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(output.getvalue(), encoding="utf-8", newline="\n")
    print(f"Wrote {count} independent SCLK cases to {args.out}")

if __name__ == "__main__":
    main()
