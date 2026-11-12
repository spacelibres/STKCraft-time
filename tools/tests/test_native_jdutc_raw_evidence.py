"""Check independent raw-order provenance and retained before/after inputs."""
from pathlib import Path
import hashlib, json, unittest

ROOT = Path(__file__).resolve().parents[2]
def digest(path): return hashlib.sha256(path.read_bytes()).hexdigest()

class NativeRawJdutcEvidence(unittest.TestCase):
    def test_raw_capture_is_independent_complete_and_unchanged(self):
        capture = ROOT/'docs/reference/native-jdutc/raw-order'
        record = json.loads((capture/'capture.json').read_text(encoding='utf-8'))
        for stream in ['stdin','stdout','stderr']:
            self.assertEqual(digest(capture/f'native-{stream}.bin'), record[f'{stream}_sha256'])
        self.assertEqual((capture/'native-stdout.bin').read_bytes(), (ROOT/'tests/reference/native-jdutc/raw.tsv').read_bytes())
        self.assertEqual((record['profiles'],record['queries'],record['numeric'],record['errors']), (4,7968,3799,4169))
        self.assertEqual(digest(capture/'native_jdutc_oracle.c'), record['source_sha256'])
        self.assertEqual((capture/'native_jdutc_oracle.c').read_bytes(), (ROOT/'tools/reference/native_jdutc_oracle.c').read_bytes())
        self.assertEqual(digest(capture/'gen_native_jdutc_raw.py'), record['generator_sha256'])
        self.assertEqual((capture/'gen_native_jdutc_raw.py').read_bytes(), (ROOT/'tools/gen_native_jdutc_raw.py').read_bytes())
        before = (capture/'native-jdutc-raw-before.log').read_text(encoding='utf-8')
        self.assertIn('left: 8153', before)
        self.assertIn('constructor:LskParse', before)
        self.assertIn('FAILED', before)
        self.assertIn('LeapSecondTable::new', (capture/'native_jdutc_raw_before.rs').read_text(encoding='utf-8'))
        self.assertEqual(record['archive_sha256'], '98d60b814b412fa55294aeaaeb7dab46d849cc87a8b709ffe835d08de17625dc')

if __name__ == '__main__': unittest.main()
