"""Verify retained WGC GPS evidence hashes and unmodified output extractions."""
import hashlib
import json
import struct
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REF = ROOT / 'tests/reference/wgc-gps'

class WgcGpsReferenceTest(unittest.TestCase):
    def test_manifest_matches_evidence_bytes(self):
        manifest = json.loads((REF / 'manifest.json').read_text(encoding='utf-8'))
        self.assertEqual(manifest['algorithm'], 'sha256')
        self.assertGreaterEqual(len(manifest['files']), 20)
        for name, expected in manifest['files'].items():
            with self.subTest(name=name):
                self.assertEqual(hashlib.sha256((REF / name).read_bytes()).hexdigest(), expected)

    def test_reference_rows_are_exact_transcriptions(self):
        groups = {
            'week-tow-cases.tsv': ['seconds-grid', 'rounding-grid', 'seeded-grid'],
            'week-tow-edge-cases.tsv': ['half-ties', 'large-seconds', 'integer-limits'],
            'decimal-ties.tsv': ['decimal-ties'],
            'decimal-adjacent.tsv': ['decimal-adjacent'],
            'seeded-half-grid.tsv': ['seeded-half-grid'],
            'independent-half-probe.tsv': ['independent-half-probe'],
        }
        for output, sources in groups.items():
            rows = ['input_gps_seconds\tweek_tow']
            for name in sources:
                record = json.loads((REF / (name + '.json')).read_text(encoding='utf-8'))
                values = record['request']['times']
                result = record['results']['rows']
                self.assertEqual(len(values), len(result), name)
                rows.extend(str(value) + '\t' + row[1] for value, row in zip(values, result))
            with self.subTest(output=output):
                self.assertEqual((REF / output).read_bytes(), ('\n'.join(rows) + '\n').encode())

    def test_nonfinite_output_extraction(self):
        entries = json.loads((REF / 'nonfinite-inputs.json').read_text(encoding='utf-8'))
        self.assertEqual(len(entries), 3)
        lines = ['input_gps_seconds\tweek_tow']
        for entry in entries:
            self.assertEqual(entry['results']['status'], 'OK')
            self.assertEqual(len(entry['results']['rows']), 1)
            lines.append(entry['request']['times'][0] + '\t' + entry['results']['rows'][0][1])
        self.assertEqual((REF / 'nonfinite-inputs.tsv').read_bytes(), ('\n'.join(lines) + '\n').encode())

    def test_recorded_replay_matches_original_observations(self):
        replay = json.loads((REF / 'replay-2026-11-12.json').read_text(encoding='utf-8'))
        self.assertEqual(len(replay), 22)
        indices = {}
        row_count = 0
        for record in replay:
            name = record['source']
            original = json.loads((REF / name).read_text(encoding='utf-8'))
            entries = original if isinstance(original, list) else [original]
            index = indices.get(name, 0)
            indices[name] = index + 1
            prior = entries[index]
            self.assertEqual(record['request'], prior['request'])
            before = prior['created']
            after = record['created']
            self.assertEqual(before['status'], after['status'])
            if before['status'] != 'OK':
                self.assertEqual(before['error']['shortDescription'], after['error']['shortDescription'])
                continue
            expected = prior['results']
            observed = record['results']
            self.assertEqual(expected['columns'], observed['columns'])
            self.assertEqual(len(expected['rows']), len(observed['rows']))
            for left, right in zip(expected['rows'], observed['rows']):
                row_count += 1
                self.assertEqual(len(left), len(right))
                for first, second in zip(left, right):
                    if isinstance(first, (int, float)) and isinstance(second, (int, float)):
                        self.assertEqual(struct.pack('>d', float(first)), struct.pack('>d', float(second)))
                    else:
                        self.assertEqual(first, second)
        self.assertEqual(row_count, 1325)

if __name__ == '__main__':
    unittest.main()
