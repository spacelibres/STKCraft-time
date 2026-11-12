"""Capture SCDECD large positive-offset formatting with CSPICE N0067."""
import hashlib
import json
import random
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
text = (folder / 'large-field.tsc').read_text(encoding='utf-8')
text = text.replace('SCLK01_OFFSETS_902=(0)', 'SCLK01_OFFSETS_902=(9223372036854775807)')
kernel = folder / 'large-offset.tsc'
kernel.write_bytes(text.encode())
spice.kclear()
spice.furnsh(str(kernel))
rng = random.Random(903)
values = [0., 1., 100., 9999999999999998.] + [float(rng.randrange(0,10**16)) for _ in range(1000)]
rows=[]
for value in values:
    try:
        result='S:'+spice.scdecd(-902,value)
    except spice.utils.exceptions.SpiceyError as error:
        result='E:'+error.short
    rows.append(struct.pack('>d',value).hex()+'\t'+result)
payload=('ticks_bits\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'large-offset.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
        'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
        'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
        'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'large-offset.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2))
spice.kclear()
