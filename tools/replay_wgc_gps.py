"""Replay committed WGC GPS requests without overwriting references."""
import argparse
import json
import time
import urllib.request
from pathlib import Path

root = Path(__file__).resolve().parents[1]
reference = (root / 'tests/reference/wgc-gps').resolve()
parser = argparse.ArgumentParser()
parser.add_argument('--output', type=Path, required=True)
parser.add_argument('--timeout', type=float, default=90)
args = parser.parse_args()
output = args.output.resolve()
if output == reference or reference in output.parents:
    raise SystemExit('Choose an output outside the committed reference')
base = 'https://wgc2.jpl.nasa.gov:8443/webgeocalc/api'

def request(url, payload=None):
    data = None if payload is None else json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(url, data=data, headers={
        'Content-Type': 'application/json', 'Accept': 'application/json'},
        method='GET' if payload is None else 'POST')
    with urllib.request.urlopen(req, timeout=30) as response:
        return json.loads(response.read().decode('utf-8'))

records = []
for filename in ['utc-outputs.json', 'seconds-boundaries.json', 'week-inputs.json',
                 'seconds-grid.json', 'rounding-grid.json', 'seeded-grid.json',
                 'half-ties.json', 'large-seconds.json', 'integer-limits.json',
                 'decimal-ties.json', 'decimal-adjacent.json', 'seeded-half-grid.json',
                 'independent-half-probe.json', 'nonfinite-inputs.json']:
    old = json.loads((reference / filename).read_text(encoding='utf-8'))
    for entry in old if isinstance(old, list) else [old]:
        payload = entry['request']
        created = request(base + '/calculation/new', payload)
        current = {'source': filename, 'request': payload, 'created': created}
        if created.get('status') == 'OK':
            calc = created['calculationId']
            deadline = time.monotonic() + args.timeout
            while True:
                status = request(base + '/calculation/' + calc)
                phase = status.get('result', {}).get('phase')
                if phase in ('COMPLETE', 'FAILED', 'CANCELLED', 'EXPIRED'):
                    break
                if time.monotonic() >= deadline:
                    raise TimeoutError('WGC2 calculation did not finish: ' + calc)
                time.sleep(0.5)
            current['status'] = status
            if phase == 'COMPLETE':
                current['results'] = request(base + '/calculation/' + calc + '/results')
        records.append(current)
        print(filename, created.get('status'), flush=True)
output.parent.mkdir(parents=True, exist_ok=True)
output.write_text(json.dumps(records, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
