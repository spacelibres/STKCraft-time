"""Capture eleven-field native validation precedence across partition states."""
import hashlib
import json
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root=Path(__file__).resolve().parents[1]
folder=root/'tests/reference/sclk-partition'
kernel=folder/'eleven-fields.tsc'
rows=[]
for mode in ('normal','missing','capacity'):
    spice.kclear();spice.furnsh(str(kernel))
    if mode=='missing':
        spice.dvpool('SCLK_PARTITION_START_902');spice.dvpool('SCLK_PARTITION_END_902')
    elif mode=='capacity':
        spice.pdpool('SCLK_PARTITION_START_902',[0.0]*10000)
        spice.pdpool('SCLK_PARTITION_END_902',[1.0]*10000)
    for op,text in [('D','0'),('E',''),('E','/'),('E','/0'),('E','1/'),('E','bad/0'),('E','1/UNKNOWN'),('E','1/0')]:
        try:
            result='S:'+spice.scdecd(-902,0) if op=='D' else 'T:'+str(spice.scencd(-902,text))
        except spice.utils.exceptions.SpiceyError as error:
            result='E:'+error.short
        rows.append('\t'.join((mode,op,text,result)))
payload=('mode\top\tinput\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'eleven-precedence.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'eleven-precedence.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2));spice.kclear()
