"""Capture native MRO clock string to ET and ET back to clock string."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
kernel = root / 'tests/kernels/MRO_SCLKSCET.00120.tsc'
lsk = root / 'src/naif0012.tls'
spice.kclear()
spice.furnsh(str(lsk))
spice.furnsh(str(kernel))
strings = []
for row in (folder / 'native.tsv').read_text(encoding='utf-8').splitlines()[1:]:
    operation, _, result = row.split('\t')
    if operation == 'D' and result.startswith('S:') and result[2:] not in strings:
        strings.append(result[2:])
rows = []
for text in strings:
    try:
        et = spice.scs2e(-74, text)
        decoded = spice.sce2s(-74, et)
        rows.append(text + '\t' + struct.pack('>d', et).hex() + '\t' + decoded)
    except spice.utils.exceptions.SpiceyError as error:
        rows.append(text + '\tE:' + error.short + '\t')
payload = ('input\tet_or_error\tdecoded\n' + '\n'.join(rows) + '\n').encode('utf-8')
(folder / 'et.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'lsk_sha256': hashlib.sha256(lsk.read_bytes()).hexdigest(),
          'cases': len(rows), 'errors': sum('\tE:' in row for row in rows),
          'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'et.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
