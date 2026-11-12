"""Capture native SCLK error for an 11-field type-1 clock."""
import hashlib
import json
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root=Path(__file__).resolve().parents[1]
folder=root/'tests/reference/sclk-partition'
text=(folder/'synthetic-grid.tsc').read_text(encoding='utf-8')
text=text.replace('SCLK01_N_FIELDS_902=(1)','SCLK01_N_FIELDS_902=(11)')
text=text.replace('SCLK01_MODULI_902=(100)','SCLK01_MODULI_902=('+' '.join(['10']*11)+')')
text=text.replace('SCLK01_OFFSETS_902=(0)','SCLK01_OFFSETS_902=('+' '.join(['0']*11)+')')
kernel=folder/'eleven-fields.tsc'
kernel.write_bytes(text.encode())
spice.kclear();spice.furnsh(str(kernel))
rows=[]
for op in ['D','E']:
    try:
        result='S:'+spice.scdecd(-902,0) if op=='D' else 'T:'+str(spice.scencd(-902,'1/0'))
    except spice.utils.exceptions.SpiceyError as error:
        result='E:'+error.short
    rows.append(op+'\t'+result)
payload=('op\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'eleven-fields.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'eleven-fields.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2));spice.kclear()
