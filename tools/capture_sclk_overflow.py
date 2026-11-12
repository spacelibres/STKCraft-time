"""Capture extreme-rate SCS2E results with a local CSPICE build."""
import argparse
import hashlib
import json
import platform
import struct
from pathlib import Path

import spiceypy as spice
import spiceypy.utils.libspicehelper as helper

ROOT = Path(__file__).resolve().parents[1]
REFERENCE = ROOT / 'tests/reference'
KERNEL = REFERENCE / 'sclk-partition/overflow-rate.tsc'
LSK = ROOT / 'src/naif0012.tls'


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    output = parser.parse_args().output.resolve()
    if output == REFERENCE.resolve() or REFERENCE.resolve() in output.parents:
        raise SystemExit('Choose an output outside committed references')
    spice.furnsh(str(LSK))
    spice.furnsh(str(KERNEL))
    rows = []
    for clock in ('1/000', '1/001', '1/002', '1/003'):
        value = spice.scs2e(-777, clock)
        rows.append({'clock': clock, 'et_binary64_hex': struct.pack('>d', value).hex()})
    data = {'platform': platform.platform(), 'cspice': spice.tkvrsn('TOOLKIT'),
            'spiceypy': spice.__version__, 'kernel_sha256': digest(KERNEL),
            'lsk_sha256': digest(LSK),
            'library_sha256': digest(Path(helper.libspice._name)), 'rows': rows}
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(data, indent=2) + '\n', encoding='utf8')


if __name__ == '__main__':
    main()
