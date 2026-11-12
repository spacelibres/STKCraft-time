"""Capture native validation order for fields, delimiter, and partitions."""
import hashlib
import json
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as lib
root=Path(__file__).resolve().parents[1]
folder=root/'tests/reference/sclk-partition'
kernel=folder/'large-field.tsc'
rows=[]
for mode in ('fields_and_delim_missing','delim_invalid_partitions_missing','delim_missing_partitions_missing','delim_invalid_capacity'):
    spice.kclear();spice.furnsh(str(kernel))
    if mode=='fields_and_delim_missing':
        spice.pipool('SCLK01_N_FIELDS_902',[11]);spice.dvpool('SCLK01_OUTPUT_DELIM_902')
    elif mode=='delim_invalid_partitions_missing':
        spice.pipool('SCLK01_OUTPUT_DELIM_902',[6]);spice.dvpool('SCLK_PARTITION_START_902');spice.dvpool('SCLK_PARTITION_END_902')
    elif mode=='delim_missing_partitions_missing':
        spice.dvpool('SCLK01_OUTPUT_DELIM_902');spice.dvpool('SCLK_PARTITION_START_902');spice.dvpool('SCLK_PARTITION_END_902')
    else:
        spice.pipool('SCLK01_OUTPUT_DELIM_902',[6]);spice.pdpool('SCLK_PARTITION_START_902',[0.]*10000);spice.pdpool('SCLK_PARTITION_END_902',[1.]*10000)
    for op,text in [('D','0'),('E',''),('E','/'),('E','bad/0'),('E','1/0')]:
        try: result='S:'+spice.scdecd(-902,0) if op=='D' else 'T:'+str(spice.scencd(-902,text))
        except spice.utils.exceptions.SpiceyError as error: result='E:'+error.short
        rows.append('\t'.join((mode,op,text,result)))
payload=('mode\top\tinput\tresult\n'+'\n'.join(rows)+'\n').encode()
(folder/'metadata-precedence.tsv').write_bytes(payload)
report={'toolkit':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
'library_sha256':hashlib.sha256(Path(lib.libspice._name).read_bytes()).hexdigest(),
'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
'cases':len(rows),'tsv_sha256':hashlib.sha256(payload).hexdigest()}
(folder/'metadata-precedence.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8')
print(json.dumps(report,indent=2));spice.kclear()
