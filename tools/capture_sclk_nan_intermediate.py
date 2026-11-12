"""Capture finite-ET SCE2S behavior in isolated CSPICE processes.

Run with SpiceyPy installed and --output outside tests/reference. Each child
loads the pinned LSK and one retained synthetic kernel. A crashing or hanging
native library is recorded without terminating the capture process.
"""
import argparse
import hashlib
import json
import platform
import subprocess
import sys
from pathlib import Path

import spiceypy as spice
import spiceypy.utils.libspicehelper as helper

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'tests/reference'
CASES = REFERENCE / 'sclk-partition/nan-finite-et'
LSK = ROOT / 'src/naif0012.tls'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    output = parser.parse_args().output.resolve()
    if output == REFERENCE.resolve() or REFERENCE.resolve() in output.parents:
        raise SystemExit('Choose an output outside committed references')
    rows = []
    for source in ('capture.json', 'extra-windows.json'):
        metadata = json.loads((CASES / source).read_text(encoding='utf8'))
        for case in metadata['rows']:
            kernel = CASES / (case['name'] + '.tsc')
            code = (
                'import json,spiceypy as s\n'
                f's.furnsh({str(LSK)!r});s.furnsh({str(kernel)!r})\n'
                'try: print(json.dumps({"string":s.sce2s(-777,0.0)}))\n'
                'except Exception as e: print(json.dumps({"error":getattr(e,"short",type(e).__name__)}))\n'
            )
            row = {'name': case['name'], 'kernel_sha256': digest(kernel)}
            try:
                result = subprocess.run([sys.executable, '-c', code],
                                        capture_output=True, text=True, timeout=12)
                row.update(exit_code=result.returncode, stdout=result.stdout,
                           stderr=result.stderr)
            except subprocess.TimeoutExpired:
                row['timeout_seconds'] = 12
            rows.append(row)
    result = {'platform': platform.platform(), 'cspice': spice.tkvrsn('TOOLKIT'),
              'spiceypy': spice.__version__, 'lsk_sha256': digest(LSK),
              'library_sha256': digest(Path(helper.libspice._name)), 'rows': rows}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + '\n', encoding='utf8')


if __name__ == '__main__':
    main()
