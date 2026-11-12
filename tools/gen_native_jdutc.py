"""Capture N0067 JDUTC using the pinned GeoCalc native compiler profile.

Requires an explicit GeoCalc checkout with the official N0067 archive/library
and Windows SDK provisioned. Production and Rust tests do not require it.
"""
from pathlib import Path
import argparse,datetime,hashlib,importlib.util,json,math,random,re,struct,subprocess,sys,zipfile

ROOT=Path(__file__).resolve().parents[1]
def bits(value):return struct.pack('>d',float(value)).hex()
def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--geocalc-source',type=Path,required=True)
    parser.add_argument('--output-dir',type=Path,required=True)
    args=parser.parse_args();geo=args.geocalc_source.resolve();dest=args.output_dir.resolve();dest.mkdir(exist_ok=False,parents=True)
    sys.path.insert(0,str(geo/'tools'))
    spec=importlib.util.spec_from_file_location('native_compile',geo/'tools/gen_kernel_pool.py')
    native=importlib.util.module_from_spec(spec);spec.loader.exec_module(native)
    native.SOURCE=ROOT/'tools/reference/native_jdutc_oracle.c'
    record=native.compile_native(dest)
    nominal=[32.184,0.001657,0.01671,6.239996,1.99096871e-7]
    lsk=(geo/'target/kernels/naif0012.tls').read_text(encoding='ascii')
    months={name:i+1 for i,name in enumerate(['JAN','FEB','MAR','APR','MAY','JUN','JUL','AUG','SEP','OCT','NOV','DEC'])}
    official=[(datetime.date(int(y),months[m.upper()],int(d)).toordinal()+1721425,int(offset)) for offset,y,m,d in re.findall(r'(\d+)\s*,\s*@(\d+)-([A-Za-z]+)-(\d+)',lsk)]
    assert len(official)==28
    cases=[('official',nominal,official),('nonstandard_constants',[39.135,.00234,.04,.4,1e-7],official),
           ('no_periodic_term',[0.,0.,0.,0.,0.],official),
           ('negative_leap',nominal,[(2441318,10),(2441500,11),(2441700,10)]),
           ('future_first',nominal,[(2460677,100),(2461042,101)]),
           ('negative_offsets',nominal,[(2441318,-50),(2441500,-49)]),
           ('ancient_first',nominal,[(1721426,5),(1721791,6)]),
           ('invalid_tai_order',nominal,[(2441318,100000),(2441319,0)])]
    for count in [139,140,141,142]:
        cases.append((f'capacity_{count}',nominal,[(2441318+i*365,10+i) for i in range(count)]))
    cases.append(('adjacent_change_days',nominal,[(2441318,10),(2441319,11)]))
    rows=[];rng=random.Random(706720261112)
    historical=[-4e9,-2e9,-1.5e9,-1e9,-9e8,-883656100.,-883656010.,-883656000.,-883655958.,-883655957.816,-883655957.,-883655900.,0.]
    offsets=[-2.,-1.,-.5,-.125,-1e-6,0.,1e-6,.125,.5,1.,2.]
    for name,constants,entries in cases:
        setup=['C',name,str(len(entries)),*map(bits,constants)]
        for jdn,delta in entries:setup.extend([str(jdn),str(delta)])
        rows.append('|'.join(setup))
        epochs=[*historical,*[rng.uniform(-2e11,2e11) for _ in range(120)]]
        rows.extend('Q|'+bits(value) for value in epochs)
        last=entries[0][1]-1
        for jdn,delta in entries:
            formal=float(jdn-2451545)*86400.-43200.
            for boundary in [formal-86400.+last,formal+delta]:
                values=[boundary+offset for offset in offsets]
                values.extend([math.nextafter(boundary,-math.inf),math.nextafter(boundary,math.inf)])
                rows.extend('B|'+bits(value) for value in values)
            last=delta
    raw=('\n'.join(rows)+'\n').encode('ascii');(dest/'native-stdin.bin').write_bytes(raw)
    result=subprocess.run([str(dest/'kernel-pool-oracle.exe')],input=raw,cwd=geo,capture_output=True)
    if result.returncode or not result.stderr.startswith(b'CRT_FMA:1\r\n'):raise RuntimeError('Native capture failed')
    for stream,data in [('stdout',result.stdout),('stderr',result.stderr)]:(dest/f'native-{stream}.bin').write_bytes(data)
    output=result.stdout.decode('ascii').splitlines()
    assert len(output)==len(rows)
    with zipfile.ZipFile(geo/'target/reference/cspice-n0067.zip') as archive:
        additional={}
        for name in ['ttrans.c','unitim.c','rmaind.c']:
            path=geo/'target/reference/n0067/cspice/src/cspice'/name
            assert path.read_bytes()==archive.read('cspice/src/cspice/'+name)
            additional[name]=native.digest(path)
    record.update(captured_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
        generator_sha256=native.digest(Path(__file__)),compiler_helper_sha256=native.digest(geo/'tools/gen_kernel_pool.py'),
        profiles=len(cases),queries=sum(row.startswith('R|') for row in output),
        numeric=sum('|N|' in row for row in output),errors=sum('|E|' in row for row in output),
        stdin_sha256=native.digest(dest/'native-stdin.bin'),stdout_sha256=native.digest(dest/'native-stdout.bin'),
        stderr_sha256=native.digest(dest/'native-stderr.bin'),source_sha256_additional=additional,
        official_lsk_sha256=native.digest(geo/'target/kernels/naif0012.tls'))
    (dest/'capture.json').write_text(json.dumps(record,indent=2)+'\n',encoding='utf-8')
    print(json.dumps({key:record[key] for key in ['profiles','queries','numeric','errors']}))
if __name__=='__main__':main()
