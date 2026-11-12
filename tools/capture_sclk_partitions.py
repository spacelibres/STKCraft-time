"""Capture NAIF SCLK partition string behavior from the real MRO kernel."""
import hashlib
import json
import math
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/MRO_SCLKSCET.00120.tsc'
output = root / 'tests/reference/sclk-partition'
output.mkdir(parents=True, exist_ok=True)
spice.kclear()
spice.furnsh(str(kernel))
starts, ends = spice.scpart(-74)
values = [-1., -0.5, -0.49, 0., 0.49, 0.5, 1.]
offset = 0.
for start, end in zip(starts, ends):
    values.extend(offset + delta for delta in [-1., -0.5, -0.49, 0., 0.49, 0.5, 1.])
    offset += math.floor(end + 0.5) - math.floor(start + 0.5)
values.extend(offset + delta for delta in [-1., -0.5, -0.49, 0., 0.49, 0.5, 1.])
values = list(dict.fromkeys(values))
rows = []
decoded = []
for value in values:
    try:
        result = spice.scdecd(-74, value)
        decoded.append(result)
        outcome = 'S:' + result
    except spice.utils.exceptions.SpiceyError as error:
        outcome = 'E:' + error.short
    rows.append('D\t' + struct.pack('>d', value).hex() + '\t' + outcome)
strings = list(dict.fromkeys(decoded)) + [
    '1/0000000000.000', '0000000000.000', '2/0000000000.000',
    '0/0000000000.000', '32/0000000000.000', '-1/0.0',
    '1/', '1/0', '1/0.0', '1/0.1', '1/0.999', '1/0.1000',
    '1/0.-1', '1/0000000000.000.1', 'not-a-clock', '', ' ',
]
for text in strings:
    try:
        outcome = 'T:' + struct.pack('>d', spice.scencd(-74, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        outcome = 'E:' + error.short
    rows.append('E\t' + text + '\t' + outcome)
payload = ('operation\tinput\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(output / 'native.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'spacecraft_id': -74, 'partitions': len(starts),
          'decode_queries': len(values), 'encode_queries': len(strings),
          'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(output / 'capture.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
