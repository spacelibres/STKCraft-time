"""Capture additional SCLK numeric and delimiter grammar without altering earlier fixtures."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/MRO_SCLKSCET.00120.tsc'
output = root / 'tests/reference/sclk-partition'
spice.kclear()
spice.furnsh(str(kernel))
strings = ['1e/0', '1e+/0', 'PI/0', '1.9/0', '+1/0', ' 1 /0', '1D0/0',
           '1/PI', '1/E', '1/1E+', '1/0..', '1/0  .  0', '1/0:1', '1/0-1',
           '1/0,1', '1/0 1', '1/0..1', '1/.1', '1/0.', '1/0: ',
           '1/0E999', '1/0E1000', '1/0E10000', '1/1E-999999999999',
           '1e-999999999999/0', '1/ 0  ', '1/0/1', '/0', '  /0',
           '1/0\t1', '1/0000000000000000000000000000000000000001',
           '1/4294967296.0', '1/4294967295.256']
rows = []
for text in strings:
    try:
        result = 'T:' + struct.pack('>d', spice.scencd(-74, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append(text.replace('\t', '\\t') + '\t' + result)
payload = ('input\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(output / 'extra.tsv').write_bytes(payload)
record = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(strings), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(output / 'extra.json').write_text(json.dumps(record, indent=2) + '\n', encoding='utf-8')
print(json.dumps(record, indent=2))
spice.kclear()
