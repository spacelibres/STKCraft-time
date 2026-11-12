"""Capture reset/overlap and offset-field SCLK behavior for a synthetic clock."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/synthetic_partition_reset.tsc'
output = root / 'tests/reference/sclk-partition'
spice.kclear()
spice.furnsh(str(kernel))
values = [-1., -0.5, -0.49, 0., 0.49, 0.5, 1., 5999., 5999.5,
          6000., 6000.5, 6001., 11999., 11999.5, 12000., 12000.5,
          12001., 17999., 17999.5, 18000., 18000.5, 18001.]
strings = ['1/001:00:001', '1/002:00:001', '2/000:00:001',
           '2/001:00:001', '3/001:00:001', '001:00:001',
           '002:00:001', '000:00:001', '1/001:00:000',
           '1/001:00:101', '1/001:60:001', '0/001:00:001',
           '4/001:00:001', '3/000:00:001', 'not-clock', '']
rows = []
for value in values:
    try:
        outcome = 'S:' + spice.scdecd(-999, value)
    except spice.utils.exceptions.SpiceyError as error:
        outcome = 'E:' + error.short
    rows.append('D\t' + struct.pack('>d', value).hex() + '\t' + outcome)
for text in strings:
    try:
        outcome = 'T:' + struct.pack('>d', spice.scencd(-999, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        outcome = 'E:' + error.short
    rows.append('E\t' + text + '\t' + outcome)
payload = ('operation\tinput\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(output / 'reset.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'decode_cases': len(values), 'encode_cases': len(strings),
          'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(output / 'reset.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
