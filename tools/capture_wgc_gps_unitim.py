"""Capture CSPICE UNITIM intermediate bits in a new output directory."""
import argparse
import hashlib
import json
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output-dir', type=Path, required=True)
args = parser.parse_args()
output = args.output_dir.resolve()
reference = (root / 'tests/reference/wgc-gps').resolve()
if output == reference or reference in output.parents:
    raise SystemExit('Choose an output directory outside the committed reference')
output.mkdir(parents=True, exist_ok=True)
source = reference / 'week-tow-cases.tsv'
lsk = root / 'src/naif0012.tls'
spice.kclear()
spice.furnsh(str(lsk))
rows = []
for line in source.read_text(encoding='utf-8').splitlines()[1:]:
    text, _ = line.split('\t')
    seconds = float(text)
    tai = seconds - 630763181.0
    et = spice.unitim(tai, 'TAI', 'TDB')
    converted = spice.unitim(et, 'TDB', 'TAI') + 630763181.0
    bits = lambda value: struct.pack('>d', value).hex()
    rows.append('\t'.join((bits(seconds), bits(et), bits(converted))))
payload = ('input_bits\tet_bits\toutput_bits\n' + '\n'.join(rows) + '\n').encode()
(output / 'unitim-native.tsv').write_bytes(payload)
meta = {'toolkit': spice.tkvrsn('TOOLKIT'), 'spiceypy': spice.__version__,
        'library_sha256': hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
        'lsk_sha256': hashlib.sha256(lsk.read_bytes()).hexdigest(),
        'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
        'cases': len(rows), 'tsv_sha256': hashlib.sha256(payload).hexdigest()}
(output / 'unitim-native.json').write_text(json.dumps(meta, indent=2) + '\n', encoding='utf-8')
print(json.dumps(meta, indent=2))
spice.kclear()
