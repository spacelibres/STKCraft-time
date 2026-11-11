#!/usr/bin/env python3
"""Generate decimal parsing fixtures using CSPICE PRSDP via SpiceyPy.

Pass official SCLK kernel paths to add all their numeric data-block tokens.
The output records source-file checksums and exact reference binary64 bits.
"""

from __future__ import annotations

import argparse
import hashlib
import re
import struct
from pathlib import Path

import spiceypy as spice

TOKENS = [
    "0", "-0", "+0", "0E999", "-0e-999999", "1E308",
    "1.7976931348623157E308", "1E-308", "1E-320", "5E-324", "1E-324",
    "0.0000000000000001234567890123456789", "1.234567890123456789",
    "12345678901234565", "12345678901234566", "99999999999999995",
    "0.99999999999999995", "1.00000000000000005", "1E-11",
    "4.9260090394032E+13", ".5", "1.", "+1.2D-3",
]
NUMBER = re.compile(r"(?<![A-Za-z_])[-+]?\d+(?:\.\d*)?(?:[EeDd][-+]?\d+)?")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kernels", nargs="*", type=Path)
    parser.add_argument("--output", type=Path, default=Path("tests/golden/naif_numbers.csv"))
    args = parser.parse_args()
    lines = [f"# CSPICE {spice.tkvrsn('TOOLKIT')} PRSDP; SpiceyPy {spice.__version__}"]
    tokens = list(TOKENS)
    for path in args.kernels:
        blob = path.read_bytes()
        lines.append(f"# {path.name} SHA256 {hashlib.sha256(blob).hexdigest()}")
        for block in blob.decode().split("\\begindata")[1:]:
            tokens.extend(NUMBER.findall(block.split("\\begintext", 1)[0]))
    for token in dict.fromkeys(tokens):
        bits = struct.unpack(">Q", struct.pack(">d", spice.prsdp(token)))[0]
        lines.append(f"{token},{bits:016x}")
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
