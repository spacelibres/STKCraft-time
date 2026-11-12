"""Validate provenance for the additional native SCLK references."""
import csv,hashlib,json,unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
REF=ROOT/'tests/reference/sclk-partition'

def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()

class SclkAddedReferences(unittest.TestCase):
    def test_overflow_rate_capture(self):
        data=json.loads((REF/'overflow-rate.json').read_text(encoding='utf8'))
        self.assertEqual(data['cspice'],'CSPICE_N0067')
        self.assertEqual(data['kernel_sha256'],digest(REF/'overflow-rate.tsc'))
        self.assertEqual(data['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
        self.assertEqual([r['et_binary64_hex'] for r in data['rows']],
                         ['0000000000000000','7fe1ccf385ebc89f',
                          '7ff0000000000000','7ff0000000000000'])

    def test_coefficient_range_capture(self):
        data=json.loads((REF/'coefficient-range.json').read_text(encoding='utf8'))
        self.assertEqual(data['cspice'],'CSPICE_N0067')
        self.assertEqual(data['kernel_sha256'],digest(REF/'coefficient-range.tsc'))
        self.assertEqual(data['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
        self.assertEqual([x['result'] for x in data['rows']],['E:SPICE(VALUEOUTOFRANGE)','E:SPICE(VALUEOUTOFRANGE)','T:0000000000000000'])

    def test_et_grid_sources(self):
        data=json.loads((REF/'et-error-grid.json').read_text(encoding='utf8'))
        paths={'mro':ROOT/'tests/kernels/MRO_SCLKSCET.00120.tsc','reset':ROOT/'tests/kernels/synthetic_partition_reset.tsc','coeff':REF/'coefficient-range.tsc'}
        for name,path in paths.items():self.assertEqual(data['kernel_sha256'][name],digest(path))
        self.assertEqual(data['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
        self.assertEqual(data['tsv_sha256'],digest(REF/'et-error-grid.tsv'))
        with (REF/'et-error-grid.tsv').open(encoding='utf8',newline='') as f:rows=list(csv.DictReader(f,delimiter='\t'))
        self.assertEqual(len(rows),data['rows']);self.assertEqual(len(rows),660)
        self.assertEqual(set(r['kernel'] for r in rows),set(paths))

    def test_single_and_tiny_rate_capture_hashes(self):
        for meta,kernel in [('single-zero-start.json','single-zero-start.tsc'),('tiny-rate-process-windows.json','tiny-rate.tsc')]:
            data=json.loads((REF/meta).read_text(encoding='utf8'))
            self.assertEqual(data['kernel_sha256'],digest(REF/kernel))
            self.assertEqual(data['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
            self.assertEqual(data['cspice'],'CSPICE_N0067')

    def test_nonfinite_process_capture_hashes(self):
        data=json.loads((REF/'nonfinite-process-windows.json').read_text(encoding='utf8'))
        self.assertEqual(data['kernel_sha256'],digest(ROOT/'tests/kernels/MRO_SCLKSCET.00120.tsc'))
        self.assertEqual(data['cspice'],'CSPICE_N0067')
        self.assertEqual([r['input'] for r in data['results']],['nan','inf','-inf'])

    def test_finite_et_nan_intermediate_captures(self):
        directory=REF/'nan-finite-et'
        data=json.loads((directory/'capture.json').read_text(encoding='utf8'))
        self.assertEqual(data['cspice'],'CSPICE_N0067')
        self.assertEqual(data['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
        self.assertEqual(len(data['rows']),12)
        for row in data['rows']:
            self.assertEqual(row['kernel_sha256'],digest(directory/(row['name']+'.tsc')))
            native=row['linux_result']
            self.assertEqual(row['windows_result'],native.get('error',native.get('string')))
        extra=json.loads((directory/'extra-windows.json').read_text(encoding='utf8'))
        self.assertEqual(extra['lsk_sha256'],digest(ROOT/'src/naif0012.tls'))
        self.assertEqual(len(extra['rows']),6)
        for row in extra['rows']:
            self.assertEqual(row['kernel_sha256'],digest(directory/(row['name']+'.tsc')))

if __name__=='__main__':unittest.main()
