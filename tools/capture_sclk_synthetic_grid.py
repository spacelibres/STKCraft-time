"""Capture synthetic type-1 clocks with fractional boundaries and delimiters."""
import hashlib
import json
import math
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
configs = [
    (901, [1000, 60, 100], [0, 0, 1], 2, [6000.49, 0.49], [12000.51, 6000.51]),
    (902, [100], [0], 1, [10.0, 0.0], [99.0, 99.0]),
    (903, [100, 10], [1, 1], 3, [0.0, 5.0], [100.0, 500.0]),
    (904, [1000, 100], [0, 0], 4, [0.0, 0.0], [1000.0, 1000.0]),
    (905, [1000, 100], [0, 0], 5, [0.0, 500.0], [500.0, 1000.0]),
]
lines = ['KPL/SCLK', '', '\\begindata', 'SCLK_KERNEL_ID=( @2000-JAN-1/12:00:00 )']
for ident, moduli, offsets, delim, starts, ends in configs:
    assignments = {'SCLK_DATA_TYPE': [1], 'SCLK01_TIME_SYSTEM': [1],
                   'SCLK01_N_FIELDS': [len(moduli)], 'SCLK01_MODULI': moduli,
                   'SCLK01_OFFSETS': offsets, 'SCLK01_OUTPUT_DELIM': [delim],
                   'SCLK_PARTITION_START': starts, 'SCLK_PARTITION_END': ends,
                   'SCLK01_COEFFICIENTS': [0, 0, 1]}
    for name, values in assignments.items():
        lines.append(f'{name}_{ident}=(' + ' '.join(map(str, values)) + ')')
lines.append('\\begintext')
kernel = folder / 'synthetic-grid.tsc'
kernel.write_text('\n'.join(lines) + '\n', encoding='ascii', newline='\n')
spice.kclear()
spice.furnsh(str(kernel))
rows = []
for ident, _, _, _, _, _ in configs:
    starts, ends = spice.scpart(-ident)
    offsets = [0.]
    total = 0.
    for start, end in zip(starts, ends):
        total = math.floor(total + end - start + .5)
        offsets.append(total)
    values = list(dict.fromkeys(v + delta for v in offsets for delta in [-1., -.5, -.49, 0., .49, .5, 1.]))
    decoded = []
    for value in values:
        try:
            text = spice.scdecd(-ident, value)
            outcome = 'S:' + text
            decoded.append(text)
        except spice.utils.exceptions.SpiceyError as error:
            outcome = 'E:' + error.short
        rows.append(f'{ident}\tD\t{struct.pack(">d", value).hex()}\t{outcome}')
    for text in list(dict.fromkeys(decoded + [s.split('/', 1)[1] for s in decoded])):
        try:
            outcome = 'T:' + struct.pack('>d', spice.scencd(-ident, text)).hex()
        except spice.utils.exceptions.SpiceyError as error:
            outcome = 'E:' + error.short
        rows.append(f'{ident}\tE\t{text}\t{outcome}')
payload = ('clock\toperation\tinput\tresult\n' + '\n'.join(rows) + '\n').encode('utf-8')
(folder / 'synthetic-grid.tsv').write_bytes(payload)
report = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
          'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
          'kernel_sha256': hashlib.sha256(kernel.read_bytes()).hexdigest(),
          'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(folder / 'synthetic-grid.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
print(json.dumps(report, indent=2))
spice.kclear()
