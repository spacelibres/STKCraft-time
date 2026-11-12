"""Capture native ET-to-SCLK results and error symbols for finite inputs."""
import argparse,csv,hashlib,json,math,random,struct,platform
from pathlib import Path
import spiceypy as s
import spiceypy.utils.libspicehelper as helper
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('--output-dir',type=Path,required=True)
out=parser.parse_args().output_dir.resolve()
reference=(root/'tests/reference').resolve()
if out==reference or reference in out.parents:raise SystemExit('Choose an output outside committed references')
out.mkdir(parents=True,exist_ok=True)
base=root/'tests/reference/sclk-partition'
kernels={'mro':(root/'tests/kernels/MRO_SCLKSCET.00120.tsc',-74),'reset':(root/'tests/kernels/synthetic_partition_reset.tsc',-999),'coeff':(base/'coefficient-range.tsc',-999)}
r=random.Random(433941); records=[]
for name,(path,id) in kernels.items():
 s.kclear();s.furnsh(str(root/'src/naif0012.tls'));s.furnsh(str(path))
 if name=='mro':
  with (base/'et.tsv').open(encoding='utf8',newline='') as f: source=list(csv.DictReader(f,delimiter='\t'))
  anchors=[struct.unpack('>d',bytes.fromhex(row['et_or_error']))[0] for row in source]
 else:anchors=[-2.,-1.,-0.001,-0.0001,0.,0.0001,0.001,1.,2.,3.,3.0001,4.]
 values=[]
 for v in anchors:
  values.extend((v,math.nextafter(v,-math.inf),math.nextafter(v,math.inf)))
 values.extend(r.uniform(-1e12,1e12) for _ in range(100))
 values.extend((1e308,-1e308))
 for et in values:
  try: result='S:'+s.sce2s(id,et)
  except s.utils.exceptions.SpiceyError as e: result='E:'+e.short
  records.append((name,struct.pack('>d',et).hex(),result))
with (out/'et-error-grid.tsv').open('w',encoding='utf8',newline='\n') as f:
 f.write('kernel\tet_bits\tresult\n');f.writelines('\t'.join(row)+'\n' for row in records)
meta={'platform':platform.platform(),'spiceypy':s.__version__,'cspice':s.tkvrsn('TOOLKIT'),
 'library_sha256':hashlib.sha256(Path(helper.libspice._name).read_bytes()).hexdigest(),
 'lsk_sha256':hashlib.sha256((root/'src/naif0012.tls').read_bytes()).hexdigest(),
 'kernel_sha256':{k:hashlib.sha256(p.read_bytes()).hexdigest() for k,(p,_) in kernels.items()},'rows':len(records),'seed':433941,
 'tsv_sha256':hashlib.sha256((out/'et-error-grid.tsv').read_bytes()).hexdigest()}
(out/'et-error-grid.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf8')
print('rows',len(records))
