"""Capture SCFM01 native field widths around integer powers of ten."""
import hashlib
import json
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root=Path(__file__).resolve().parents[1]
folder=root/'tests/reference/sclk-partition'
kernel=folder/'large-field.tsc'
rows=[]
values=sorted(set([10**n+d for n in range(1,17) for d in (-2,-1,0,1,2) if 10**n+d>0]))
for modulus in values:
    spice.kclear();spice.furnsh(str(kernel));spice.pdpool('SCLK01_MODULI_902',[float(modulus)])
    try:result='S:'+spice.scdecd(-902,0.)
    except spice.utils.exceptions.SpiceyError as error:result='E:'+error.short
    rows.append(str(modulus)+'\t'+result)
payload=('modulus\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'width-boundaries.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'width-boundaries.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2));spice.kclear()
