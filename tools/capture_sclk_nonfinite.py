"""Capture process-isolated CSPICE SCDECD non-finite behavior on this host."""
import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path
import spiceypy
import spiceypy.utils.libspicehelper as helper

root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--output-dir',type=Path,required=True)
out=parser.parse_args().output_dir.resolve()
reference=(root/'tests/reference').resolve()
if out==reference or reference in out.parents:raise SystemExit('Choose an output outside committed references')
out.mkdir(parents=True,exist_ok=True)
kernel=root/'tests/kernels/MRO_SCLKSCET.00120.tsc'
library=Path(helper.libspice._name)
results=[]
for value in ('nan','inf','-inf'):
    code=("import spiceypy as s; s.furnsh(%r); print(repr(s.scdecd(-74,float(%r))))" % (str(kernel),value))
    try:
        run=subprocess.run([sys.executable,'-c',code],capture_output=True,text=True,timeout=12)
        results.append({'input':value,'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr})
    except subprocess.TimeoutExpired:
        results.append({'input':value,'timeout_seconds':12})
record={'platform':platform.platform(),'python':platform.python_version(),'spiceypy':spiceypy.__version__,
        'cspice':spiceypy.tkvrsn('TOOLKIT'),'kernel_sha256':hashlib.sha256(kernel.read_bytes()).hexdigest(),
        'library_sha256':hashlib.sha256(library.read_bytes()).hexdigest(),'results':results}
(out/'nonfinite-process.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
