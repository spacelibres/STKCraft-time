"""Capture state after a truncated multibyte SCLK component."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
kernel = root / 'tests/kernels/MRO_SCLKSCET.00120.tsc'
folder = root / 'tests/reference/sclk-partition'
spice.kclear()
spice.furnsh(str(kernel))
rows = []
for text in ['1/0.0', '1/0.' + '0' * 29 + 'é', '1/.1', '1/0.0', '1/.1']:
    try:
        result = 'T:' + struct.pack('>d', spice.scencd(-74, text)).hex()
    except spice.utils.exceptions.SpiceyError as error:
        result = 'E:' + error.short
    rows.append(text + '\t' + result)
payload = ('input\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(folder / 'utf8-history.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'utf8-history.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
