"""Generate independent Python binary64 hexadecimal parsing vectors.

The rows use Python float.fromhex, not WebGeocalc, and exercise conversion
rounding separately from the NASA request/response fixtures.
"""
import argparse
import random
import struct
from pathlib import Path

rng = random.Random(817623)
rows = []
for _ in range(5000):
    length = rng.randint(1, 80)
    digits = ''.join(rng.choice('0123456789abcdef') for _ in range(length))
    position = rng.randint(0, length)
    text = ('-' if rng.getrandbits(1) else '') + '0x' + digits[:position] + '.' + digits[position:] + 'p' + str(rng.randint(-1300, 1300))
    try:
        number = float.fromhex(text)
    except OverflowError:
        number = float('-inf') if text[0] == '-' else float('inf')
    number = 0.0 + number
    bits = struct.unpack('>Q', struct.pack('>d', number))[0]
    rows.append(f'0, {text}\t{bits:016x}\n')
root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
output = parser.parse_args().output.resolve()
reference = (root / 'tests/reference').resolve()
if output == reference or reference in output.parents:
    raise SystemExit('Choose an output outside committed references')
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text('input\tbits\n' + ''.join(rows), encoding='ascii', newline='\n')
