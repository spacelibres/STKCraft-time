"""Capture a deterministic CSPICE TIMOUT day-of-year reference corpus.

Requires SpiceyPy 8.3.0 and CSPICE N0067. Results are not live WebGeocalc
responses. The supplied repository LSK is loaded explicitly.
"""
import argparse
import random
import struct
from pathlib import Path
import spiceypy as spice

root = Path(__file__).resolve().parents[1]
spice.furnsh(str(root / 'src/naif0012.tls'))
assert spice.tkvrsn('TOOLKIT') == 'CSPICE_N0067'
rng = random.Random(1211122026)
base = [-6e9, -1e9, -630763181.0, -1.0, 0.0, 1.0, 5.36500868e8, 1e9, 6e9]
values = [base[i % len(base)] + rng.uniform(-1e7, 1e7) for i in range(3000)]
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
out = parser.parse_args().output.resolve()
reference = (root / 'tests/reference').resolve()
if out == reference or reference in out.parents:
    raise SystemExit('Choose an output outside committed references')
out.parent.mkdir(parents=True, exist_ok=True)
with out.open('w', encoding='ascii', newline='\n') as stream:
    stream.write('system\tet_bits\toutput\n')
    for value in values:
        bits = struct.pack('>d', value).hex()
        for system in ('UTC', 'TDB', 'TT', 'GPS'):
            source = spice.unitim(value, 'TDB', 'TAI') - 19.0 if system == 'GPS' else value
            scale = 'TDT' if system == 'TT' else 'TDB' if system == 'GPS' else system
            picture = 'YYYY-DOY/HR:MN:SC.###### ::' + scale + ' ::RND'
            result = spice.timout(source, picture) + ' ' + system
            stream.write(f'{system}\t{bits}\t{result}\n')
