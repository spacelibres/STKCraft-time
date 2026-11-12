"""Protect independent native JDUTC evidence and its original expectations."""
from pathlib import Path
import hashlib,json,unittest

ROOT=Path(__file__).resolve().parents[2]
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

class NativeJdutcEvidence(unittest.TestCase):
    def test_expanded_capture_preserves_all_original_records(self):
        original=ROOT/'docs/reference/native-jdutc'
        expanded=original/'expanded'
        record=json.loads((expanded/'capture.json').read_text(encoding='utf-8'))
        for stream in ['stdin','stdout','stderr']:
            self.assertEqual(digest(expanded/f'native-{stream}.bin'),record[f'{stream}_sha256'])
        raw=(expanded/'native-stdout.bin').read_bytes()
        self.assertTrue(raw.startswith((original/'native-stdout.bin').read_bytes()))
        self.assertEqual(raw,(ROOT/'tests/reference/native-jdutc/native.tsv').read_bytes())
        self.assertEqual((record['profiles'],record['queries'],record['numeric'],record['errors']),(13,18863,18493,370))
        self.assertEqual(digest(expanded/'native_jdutc_oracle.c'),record['source_sha256'])
        self.assertEqual((expanded/'native_jdutc_oracle.c').read_bytes(),(ROOT/'tools/reference/native_jdutc_oracle.c').read_bytes())
        self.assertEqual(digest(expanded/'gen_native_jdutc.py'),record['generator_sha256'])
        self.assertEqual((expanded/'gen_native_jdutc.py').read_bytes(),(ROOT/'tools/gen_native_jdutc.py').read_bytes())
        earlier=json.loads((original/'capture.json').read_text(encoding='utf-8'))
        self.assertEqual(digest(original/'native-stdout.bin'),earlier['stdout_sha256'])
        self.assertEqual(digest(original/'gen_native_jdutc.py'),earlier['generator_sha256'])

    def test_original_lsk_historical_reproduction_is_unchanged(self):
        initial=ROOT/'docs/reference/native-jdutc/initial-historical-gap'
        reference=ROOT/'tests/reference/native-jdutc'
        record=json.loads((initial/'capture.json').read_text(encoding='utf-8'))
        self.assertEqual(digest(initial/'native-stdout.bin'),record['stdout_sha256'])
        self.assertEqual(digest(initial/'oracle.c'),record['source_sha256'])
        self.assertEqual(digest(reference/'naif0012.tls'),record['lsk_sha256'])
        self.assertEqual((initial/'native-stdout.bin').read_bytes(),(reference/'original-lsk.tsv').read_bytes())
        self.assertEqual(len((reference/'original-lsk.tsv').read_bytes().splitlines()),13)
