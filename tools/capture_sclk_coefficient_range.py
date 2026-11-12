"""Capture N0067 SCS2E behavior when encoded ticks precede coefficients."""
import argparse
import hashlib,json,platform,struct
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as helper
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--output-dir',type=Path,required=True)
out=parser.parse_args().output_dir.resolve()
reference=(root/'tests/reference').resolve()
if out==reference or reference in out.parents:raise SystemExit('Choose an output outside committed references')
out.mkdir(parents=True,exist_ok=True)
text=(root/'tests/kernels/synthetic_partition_reset.tsc').read_text(encoding='utf-8')
old='SCLK01_COEFFICIENTS_999     = ( 0 0 1 )';new='SCLK01_COEFFICIENTS_999     = ( 10 0 1 )'
assert text.count(old)==1
fixture=out/'coefficient-range.tsc';fixture.write_text(text.replace(old,new),encoding='utf-8',newline='\n')
spice.furnsh(str(root/'src/naif0012.tls'));spice.furnsh(str(fixture));rows=[]
for clock in ('1/001:00:001','1/001:00:002','1/001:00:011'):
 try:result='T:'+struct.pack('>d',spice.scs2e(-999,clock)).hex()
 except spice.utils.exceptions.SpiceyError as error:result='E:'+error.short
 rows.append({'clock':clock,'result':result})
library=Path(helper.libspice._name)
record={'platform':platform.platform(),'spiceypy':spice.__version__,'cspice':spice.tkvrsn('TOOLKIT'),
        'kernel_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),
        'lsk_sha256':hashlib.sha256((root/'src/naif0012.tls').read_bytes()).hexdigest(),
        'library_sha256':hashlib.sha256(library.read_bytes()).hexdigest(),'rows':rows}
(out/'coefficient-range.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
