"""Capture SCDECD behavior when DPSTRF's fixed buffer loses its decimal."""
import hashlib
import json
import math
import struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root = Path(__file__).resolve().parents[1]
folder = root / 'tests/reference/sclk-partition'
text = (folder / 'large-field.tsc').read_text(encoding='utf-8')
text = text.replace('SCLK_PARTITION_END_902=(10000000000000000.0)', 'SCLK_PARTITION_END_902=(1E100)')
kernel = folder / 'extreme-field.tsc'
kernel.write_bytes(text.encode())
spice.kclear()
spice.furnsh(str(kernel))
values=[0.,1e16,1e20,1e25,1e26,1e27,9.99999999999994e27,9.99999999999995e27,math.nextafter(1e28,0.0),1e28,math.nextafter(1e28,math.inf),1e29,1e30,1e50,1e99,1e100]
rows=[]
for value in values:
    try:
        result='S:'+spice.scdecd(-902,value)
    except spice.utils.exceptions.SpiceyError as error:
        result='E:'+error.short
    rows.append(struct.pack('>d',value).hex()+'\t'+result)
payload=('ticks_bits\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'extreme-field.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
        'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
        'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
        'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'extreme-field.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2))
spice.kclear()
