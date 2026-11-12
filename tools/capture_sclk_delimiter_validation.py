"""Capture native SCLK delimiter metadata validation."""
import hashlib
import json
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root=Path(__file__).resolve().parents[1]
folder=root/'tests/reference/sclk-partition'
kernel=folder/'large-field.tsc'
rows=[]
for code in ['missing','-1','0','1','2','3','4','5','6']:
    spice.kclear();spice.furnsh(str(kernel))
    if code=='missing':spice.dvpool('SCLK01_OUTPUT_DELIM_902')
    else:spice.pipool('SCLK01_OUTPUT_DELIM_902',[int(code)])
    for op in ['D','E']:
        try:
            result='S:'+spice.scdecd(-902,0) if op=='D' else 'T:'+str(spice.scencd(-902,'1/0'))
        except spice.utils.exceptions.SpiceyError as error:
            result='E:'+error.short
        rows.append('\t'.join((code,op,result)))
payload=('delimiter\top\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'delimiter-validation.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'delimiter-validation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2));spice.kclear()
