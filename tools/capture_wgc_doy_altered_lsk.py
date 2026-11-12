"""Capture calendar output with altered LSK constants using CSPICE N0067."""
import argparse,hashlib,json,random,re,struct
from pathlib import Path
import spiceypy as s
import spiceypy.utils.libspicehelper as helper
root=Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser();parser.add_argument('--output-dir',type=Path,required=True)
out=parser.parse_args().output_dir.resolve();reference=(root/'tests/reference').resolve()
if out==reference or reference in out.parents:raise SystemExit('Choose an output outside committed references')
out.mkdir(parents=True,exist_ok=True)
text=(root/'src/naif0012.tls').read_text(encoding='utf8')
replacements={'DELTET/DELTA_T_A':'30.125','DELTET/K':'1.0D0','DELTET/EB':'2.0D-1','DELTET/M':'( 1.0D-1 5.0D-1 )'}
for key,value in replacements.items():
 text,n=re.subn(r'(?m)^'+re.escape(key)+r'\s*=\s*.*$',key+' = '+value,text);assert n==1
lsk=out/'altered-lsk.tls';lsk.write_text(text,encoding='utf8',newline='\n');s.furnsh(str(lsk))
rng=random.Random(749612);rows=[]
for _ in range(400):
 et=rng.uniform(-1e8,1e8)
 for system in ('UTC','TDB','TT','GPS'):
  source=s.unitim(et,'TDB','TAI')-19 if system=='GPS' else et
  scale='TDT' if system=='TT' else 'TDB' if system=='GPS' else system
  for style,picture in (('DOY','YYYY-DOY/HR:MN:SC.######'),('YMD','YYYY-MM-DD HR:MN:SC.######')):
   rows.append((style,system,struct.pack('>d',et).hex(),s.timout(source,picture+' ::'+scale+' ::RND')+' '+system))
(out/'altered-lsk-doy-ymd.tsv').write_text('style\tsystem\tet_bits\toutput\n'+''.join('\t'.join(row)+'\n' for row in rows),encoding='ascii',newline='\n')
meta={'source':'local CSPICE TIMOUT; not live WGC','seed':749612,'inputs':400,'rows':len(rows),'spiceypy':s.__version__,'cspice':s.tkvrsn('TOOLKIT'),'lsk_sha256':hashlib.sha256(lsk.read_bytes()).hexdigest(),'library_sha256':hashlib.sha256(Path(helper.libspice._name).read_bytes()).hexdigest()}
(out/'altered-lsk-doy-ymd.json').write_text(json.dumps(meta,indent=2)+'\n',encoding='utf8')
