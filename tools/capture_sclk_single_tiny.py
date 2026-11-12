"""Capture single-partition and tiny-rate native SCLK behavior in children."""
import argparse,hashlib,json,platform,subprocess,sys
from pathlib import Path
import spiceypy as spice
import spiceypy.utils.libspicehelper as helper
root=Path(__file__).resolve().parents[1];parser=argparse.ArgumentParser()
parser.add_argument('--output-dir',type=Path,required=True);out=parser.parse_args().output_dir.resolve()
reference=(root/'tests/reference').resolve()
if out==reference or reference in out.parents:raise SystemExit('Choose an output outside committed references')
out.mkdir(parents=True,exist_ok=True)
text=(root/'tests/kernels/synthetic_partition_reset.tsc').read_text(encoding='utf8')
single=text.replace('SCLK_PARTITION_START_999    = ( 6000 0 6000 )','SCLK_PARTITION_START_999    = ( 0 )').replace('SCLK_PARTITION_END_999      = ( 12000 6000 12000 )','SCLK_PARTITION_END_999      = ( 12000 )')
tiny=text.replace('SCLK01_COEFFICIENTS_999     = ( 0 0 1 )','SCLK01_COEFFICIENTS_999     = ( 0 0 1e-320 )')
for name,data in [('single-zero-start',single),('tiny-rate',tiny)]:
 path=out/(name+'.tsc');path.write_text(data,encoding='utf8',newline='\n')
 values=('0.0','0.0001','1.0','2.0') if name=='single-zero-start' else ('1.0','-1.0','0.0')
 rows=[]
 for et in values:
  code="import spiceypy as s; s.furnsh(%r); s.furnsh(%r);\ntry: print('RESULT',repr(s.sce2s(-999,%s)))\nexcept Exception as e: print('ERROR',repr(getattr(e,'short',type(e).__name__)))"%(str(root/'src/naif0012.tls'),str(path),et)
  try:
   run=subprocess.run([sys.executable,'-c',code],capture_output=True,text=True,timeout=12)
   rows.append({'et':et,'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr})
  except subprocess.TimeoutExpired:rows.append({'et':et,'timeout_seconds':12})
 meta={'platform':platform.platform(),'cspice':spice.tkvrsn('TOOLKIT'),'spiceypy':spice.__version__,
       'kernel_sha256':hashlib.sha256(path.read_bytes()).hexdigest(),'lsk_sha256':hashlib.sha256((root/'src/naif0012.tls').read_bytes()).hexdigest(),
       'library_sha256':hashlib.sha256(Path(helper.libspice._name).read_bytes()).hexdigest(),'rows':rows}
 (out/(name+'-process.json')).write_text(json.dumps(meta,indent=2)+'\n',encoding='utf8')
