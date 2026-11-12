"""Capture SCLK sign, C-string and partition-count boundary behavior."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/synthetic_partition_reset.tsc'
folder = root / 'tests/reference/sclk-partition'
rows = []
spice.kclear()
spice.furnsh(str(kernel))
spice.pipool('SCLK01_OFFSETS_999', [0, -1, 1])
try:
    result = 'S:' + spice.scdecd(-999, 0.)
except spice.utils.exceptions.SpiceyError as error:
    result = 'E:' + error.short
rows.append('negative-offset\t' + result)
spice.pipool('SCLK01_OFFSETS_999', [0, 0, 1])
for text in ['1/1\0garbage', '\0ignored']:
    try:
        result = 'T:' + struct.pack('>d', spice.scencd(-999, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append('nul:' + text.replace('\0', '\\0') + '\t' + result)
spice.pdpool('SCLK_PARTITION_START_999', [0.] * 10000)
spice.pdpool('SCLK_PARTITION_END_999', [1.] * 10000)
for action in ['decode', 'encode']:
    try:
        value = spice.scdecd(-999, 0.) if action == 'decode' else spice.scencd(-999, '1/0')
        result = 'SUCCESS:' + str(value)
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append('many:' + action + '\t' + result)
payload = ('case\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(folder / 'special.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'base_kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'special.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
