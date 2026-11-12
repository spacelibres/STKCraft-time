"""Capture CSPICE SCLK continuous and discrete conversion boundaries."""
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
reset = root / 'tests/kernels/synthetic_partition_reset.tsc'
grid = folder / 'synthetic-grid.tsc'
lsk = root / 'src/naif0012.tls'
spice.kclear()
for kernel in (lsk, reset, grid):
    spice.furnsh(str(kernel))
rows = []
for op, sc, values in (
    ('D', -999, [-0.0001, -0.00008, 0.0, 3.0, 3.00008, 3.0000816666666665, 3.0001]),
    ('T', -901, [11999.0, 12000.0, 12000.01, 12001.0, 12002.0]),
):
    for value in values:
        try:
            if op == 'D':
                result = 'S:' + spice.sce2s(sc, value)
            else:
                result = 'F:' + struct.pack('>d', spice.sct2e(sc, value)).hex()
        except spice.utils.exceptions.SpiceyError as error:
            result = 'E:' + error.short
        rows.append(f'{op}\t{sc}\t{struct.pack(">d", value).hex()}\t{result}')
payload = ('operation\tid\tinput_bits\tresult\n' + '\n'.join(rows) + '\n').encode()
(folder / 'et-boundaries.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'reset_sha256': hashlib.sha256(reset.read_bytes()).hexdigest(),
          'grid_sha256': hashlib.sha256(grid.read_bytes()).hexdigest(),
          'lsk_sha256': hashlib.sha256(lsk.read_bytes()).hexdigest(),
          'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'et-boundaries.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
