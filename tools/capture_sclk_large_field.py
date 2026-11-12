"""Capture SCDECD large-field formatting against CSPICE N0067."""
import hashlib
import json
import random
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
base = (folder / 'synthetic-grid.tsc').read_text(encoding='utf-8')
large = base.replace('SCLK01_MODULI_902=(100)', 'SCLK01_MODULI_902=(10000000000000000)')
large = large.replace('SCLK_PARTITION_START_902=(10.0 0.0)', 'SCLK_PARTITION_START_902=(0.0)')
large = large.replace('SCLK_PARTITION_END_902=(99.0 99.0)', 'SCLK_PARTITION_END_902=(10000000000000000.0)')
kernel = folder / 'large-field.tsc'
kernel.write_bytes(large.encode())
spice.kclear()
spice.furnsh(str(kernel))
rows = []
rng = random.Random(902)
values = [0.0, 12345678901234., 123456789012344., 123456789012345., 123456789012346., 999999999999995., 999999999999999., 9999999999999998.]
values += [float(rng.randrange(0, 10**16)) for _ in range(2000)]
for exponent in range(8, 16):
    values += [float(rng.randrange(10**exponent, 10**(exponent + 1))) for _ in range(500)]
for ticks in values:
    try:
        result = 'S:' + spice.scdecd(-902, ticks)
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append(struct.pack('>d', ticks).hex() + '\t' + result)
payload = ('ticks_bits\tresult\n' + '\n'.join(rows) + '\n').encode()
(folder / 'large-field.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'large-field.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
