"""Verify wheel notices/typing, bundled core hashes, and external sdist builds.

Run with the isolated test environment's interpreter. Network access is
required for first-time Rust/build dependency downloads.
"""
from pathlib import Path
import hashlib, json, subprocess, sys, tarfile, tempfile, zipfile

def main():
    wheel,sdist=map(lambda s:Path(s).resolve(),sys.argv[1:3])
    root=Path(__file__).resolve().parents[3]
    with zipfile.ZipFile(wheel) as z:
        names=z.namelist()
        for suffix in ('stkcraft_time/py.typed','stkcraft_time/__init__.pyi','stkcraft_time/compat.pyi','licenses/PYTHON-LICENSE','licenses/CORE-ATTRIBUTION.md'):
            assert any(n.endswith(suffix) for n in names),suffix
    with tarfile.open(sdist) as t:
        names=t.getnames();prefix=names[0].split('/')[0]
        for source in (root/'src').rglob('*'):
            if source.is_file():
                bundled=t.extractfile(prefix+'/src/'+source.relative_to(root/'src').as_posix()).read()
                assert bundled==source.read_bytes(),source
        assert any(n.endswith('/bindings/python/Cargo.lock') for n in names)
        assert any(n.endswith('/python/stkcraft_time/__init__.pyi') for n in names)
    external=Path(tempfile.mkdtemp(prefix='stkcraft-clean-source-'))
    subprocess.run([sys.executable,'-m','pip','wheel',str(sdist),'--no-deps','--no-cache-dir','--wheel-dir',str(external/'wheels')],cwd=external,check=True)
    rebuilt=list((external/'wheels').glob('*.whl'));assert len(rebuilt)==1
    subprocess.run([sys.executable,'-m','pip','install','--force-reinstall',str(rebuilt[0])],cwd=external,check=True)
    subprocess.run([sys.executable,'-m','pytest',str(root/'bindings/python/tests'),'-q'],cwd=external,check=True)
    print(json.dumps({'external_build':str(external),'wheel_sha256':hashlib.sha256(wheel.read_bytes()).hexdigest(),'sdist_sha256':hashlib.sha256(sdist.read_bytes()).hexdigest(),'rebuilt_wheel':str(rebuilt[0])},indent=2))
if __name__=='__main__':main()
