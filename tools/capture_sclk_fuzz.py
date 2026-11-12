"""Capture seeded SCLK string grammar cases without changing previous records."""
import hashlib
import json
import random
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/MRO_SCLKSCET.00120.tsc'
output = root / 'tests/reference/sclk-partition'
spice.kclear()
spice.furnsh(str(kernel))
rng = random.Random(20261114)
components = ['0', '1', '2', '255', '256', '1000', '4294967295', 'PI',
              'E', '1E', '1E+', '1D2', '0E1000', '0E10000', 'UNKNOWN', '', '  ']
prefixes = ['1/', '2/', '31/', '32/', '0/', '', '1e/', 'PI/', '1.9/']
rows = []
for _ in range(3000):
    text = rng.choice(prefixes) + rng.choice(components)
    for _ in range(rng.randrange(3)):
        text += rng.choice(['.', ':', '-', ',', ' ', '  ']) + rng.choice(components)
    try:
        result = 'T:' + struct.pack('>d', spice.scencd(-74, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append(text + '\t' + result)
payload = ('input\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(output / 'fuzz.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(rows), 'seed': 20261114, 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(output / 'fuzz.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
