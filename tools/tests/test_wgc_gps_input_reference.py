"""Verify GPS input reference integrity and NASA-response extraction."""
import hashlib
import gzip
import json
import re
import struct
import unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
REF=ROOT/'tests/reference/wgc-gps-input'

class WgcInputReferences(unittest.TestCase):
    def test_output_format_matrix_retains_all_observed_combinations(self):
        rows=json.loads((REF/'output-format-matrix.json').read_text(encoding='utf8'))
        systems={'UTC','TDB','TDT','TT','GPS'}
        formats={'CALENDAR','CALENDAR_YMD','CALENDAR_DOY','CUSTOM','JULIAN',
                 'SECONDS_PAST_GPS0','SECONDS_PAST_J2000',
                 'SPACECRAFT_CLOCK_STRING','SPACECRAFT_CLOCK_TICKS','WEEK_TOW'}
        pairs={(r['request']['outputTimeSystem'],r['request']['outputTimeFormat']) for r in rows}
        self.assertEqual(len(rows),50)
        self.assertEqual(pairs,{(system,fmt) for system in systems for fmt in formats})
        for row in rows:
            request=row['request']
            self.assertEqual(request['times'],['2000-01-01T12:00:00'])
            self.assertEqual(request['kernels'],[{'type':'KERNEL_SET','id':2}])
            self.assertEqual(request['outputTimeCustomFormat'],'YYYY-DOY/HR:MN:SC.######')
            self.assertEqual(row['created']['status'],'OK')
            self.assertEqual(row['results']['status'],'OK')
            self.assertEqual(len(row['results']['rows']),1)
            self.assertEqual(len(row['results']['columns']),2)

    def test_hashes_cover_every_data_file(self):
        rows=[line.split('  ',1) for line in (REF/'SHA256SUMS.txt').read_text(encoding='ascii').splitlines()]
        expected={name:digest for digest,name in rows}
        actual={p.name for p in REF.iterdir() if p.suffix in ('.json','.tsv','.tls','.gz')}
        self.assertEqual(set(expected),actual)
        for name,digest in expected.items():
            self.assertEqual(hashlib.sha256((REF/name).read_bytes()).hexdigest(),digest,name)

    def test_week_input_derived_output_strings_and_numeric_bits(self):
        record=json.loads((REF/'seeded.json').read_text(encoding='utf8'))
        inputs=record['request']['times']; rows=record['results']['rows']
        self.assertEqual(len(inputs),200); self.assertEqual(len(rows),200)
        displays='input\tweek_tow\n'+''.join(f'{x}\t{r[0]}\n' for x,r in zip(inputs,rows))
        numeric='input\toutput_bits\n'+''.join(f"{x}\t{struct.pack('>d',float(r[1])).hex()}\n" for x,r in zip(inputs,rows))
        self.assertEqual((REF/'seeded-week-tow.tsv').read_bytes(),displays.encode())
        self.assertEqual((REF/'week-seconds-seeded.tsv').read_bytes(),numeric.encode())

    def test_numeric_gps_rows_are_extracted_from_wgc(self):
        for name,count in [('gps-seconds-numeric',330),('gps-numeric-rounding-grid',1200),('gps-signed-zero',10)]:
            record=json.loads((REF/(name+'.json')).read_text(encoding='utf8'))
            inputs=record['request']['times'];rows=record['results']['rows']
            self.assertEqual(len(inputs),count);self.assertEqual(len(rows),count)
            expected='input\toutput_bits\n'+''.join(f"{x}\t{struct.pack('>d',float(r[1])).hex()}\n" for x,r in zip(inputs,rows))
            self.assertEqual((REF/(name+'.tsv')).read_bytes(),expected.encode(),name)

    def test_calendar_rows_are_extracted_from_wgc(self):
        for source,suffix,count in [('doy-systems-seeded.json','doy-seeded',240),
                                    ('doy-epochs-seeded.json','doy-epochs',261),
                                    ('doy-leaps.json','doy-leaps',336),
                                    ('ymd-epochs.json','ymd-epochs',261)]:
            for record in json.loads((REF/source).read_text(encoding='utf8')):
                inputs=record['request']['times'];rows=record['results']['rows']
                self.assertEqual(len(inputs),count);self.assertEqual(len(rows),count)
                name=record['request']['outputTimeSystem'].lower()+'-'+suffix+'.tsv'
                expected='et\toutput\n'+''.join(f'{x}\t{r[1]}\n' for x,r in zip(inputs,rows))
                self.assertEqual((REF/name).read_bytes(),expected.encode(),name)
        record=json.loads((REF/'utc-doy-seeded.json').read_text(encoding='utf8'))
        expected='et\tutc_doy\n'+''.join(f'{x}\t{r[1]}\n' for x,r in zip(record['request']['times'],record['results']['rows']))
        self.assertEqual((REF/'utc-doy-seeded.tsv').read_bytes(),expected.encode())

    def test_grammar_matrix_is_extracted_from_wgc(self):
        rows=json.loads((REF/'grammar-matrix.json').read_text(encoding='utf8'))
        self.assertEqual(len(rows),59)
        expected='input_utf8_hex\taccepted\n'+''.join(r['request']['times'][0].encode('utf8').hex()+'\t'+('1' if r['created']['status']=='OK' else '0')+'\n' for r in rows)
        self.assertEqual((REF/'grammar-matrix-bytes.tsv').read_bytes(),expected.encode())

    def test_unicode_grammar_is_extracted_from_wgc(self):
        entries=json.loads((REF/'grammar-unicode.json').read_text(encoding='utf8'))
        batch=json.loads((REF/'grammar-unicode-digits.json').read_text(encoding='utf8'))
        self.assertEqual(len(entries),43)
        self.assertEqual(len(batch['request']['times']),370)
        items=[]
        for entry in entries:
            value=entry['request']['times'][0]
            if entry['created']['status']=='OK':
                bits=struct.pack('>d',float(entry['results']['rows'][0][1])).hex()
                items.append((value,'1',bits))
            else:items.append((value,'0','-'))
        for value,row in zip(batch['request']['times'],batch['results']['rows']):
            items.append((value,'1',struct.pack('>d',float(row[1])).hex()))
        self.assertEqual(len(items),413)
        expected='input_utf8_hex\taccepted\toutput_bits\n'+''.join(value.encode('utf8').hex()+'\t'+accepted+'\t'+bits+'\n' for value,accepted,bits in items)
        self.assertEqual((REF/'grammar-unicode.tsv').read_bytes(),expected.encode())

    def test_independent_review_tables_match_retained_native_outputs(self):
        meta=json.loads((REF/'review-metadata.json').read_text(encoding='utf8'))
        for original,compressed in [('probe.py','review-probe.py.gz'),('wide-results.jsonl','review-wide-results.jsonl.gz'),('altered-leaps-results.jsonl','review-altered-leaps-results.jsonl.gz')]:
            with gzip.open(REF/compressed,'rb') as source:
                self.assertEqual(hashlib.sha256(source.read()).hexdigest(),meta['files'][original])
        prefix=re.compile(r'^\s*(?:-?\d+|\*+)(?: BC| AD)?-')
        year_pattern=re.compile(r'^\s*(\d+)(?:\s+(B\.C\.|A\.D\.))?\s')
        wide=['system\tstyle\tet_bits\tyear\tnative_suffix\n']
        with gzip.open(REF/'review-wide-results.jsonl.gz','rt',encoding='utf8') as source:
            for line in source:
                row=json.loads(line);match=year_pattern.match(row['native_year_reference'])
                self.assertIsNotNone(match)
                year=1-int(match[1]) if match[2]=='B.C.' else int(match[1])
                suffix=prefix.sub('',row['native_timout'],count=1)
                self.assertNotEqual(suffix,row['native_timout'])
                bits=struct.pack('>d',float.fromhex(row['et_hex'])).hex()
                wide.append(f"{row['system']}\t{row['style']}\t{bits}\t{year}\t{suffix}\n")
        self.assertEqual(len(wide),20491)
        self.assertEqual((REF/'review-wide.tsv').read_bytes(),''.join(wide).encode())
        altered=['offset\tstyle\tet_bits\toutput\n']
        with gzip.open(REF/'review-altered-leaps-results.jsonl.gz','rt',encoding='utf8') as source:
            for line in source:
                row=json.loads(line);bits=struct.pack('>d',float.fromhex(row['et_hex'])).hex()
                altered.append(f"{row['offset_2017']}\t{row['style']}\t{bits}\t{row['native_timout']}\n")
        self.assertEqual(len(altered),49249)
        self.assertEqual((REF/'review-altered-leaps.tsv').read_bytes(),''.join(altered).encode())

if __name__=='__main__':
    unittest.main()
