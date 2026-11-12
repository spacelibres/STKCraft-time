# WebGeocalc GPS input references

The JSON files contain request payloads and parsed JSON responses captured
from the NASA JPL WebGeocalc 2 API at
`https://wgc2.jpl.nasa.gov:8443/webgeocalc/api` on 2026-11-12 and 2026-11-13
(Asia/Shanghai) using HTTPS
certificate verification and kernel set ID 2. They preserve parsed JSON,
not raw HTTP bytes. The server's kernel bytes were not independently
retrieved; comparison against a local LSK does not establish byte identity.

`seeded-week-tow.tsv` is derived from the 200 paired input strings and output
strings in `seeded.json`; it tests the displayed value, not the server's
unrounded internal state. `grammar*.json` contain both accepted and rejected
inputs; `grammar-results.json` additionally captures results when available.
`grammar-unicode.json` and `grammar-unicode-digits.json` record 43
individual probes and all ten digits in 37 BMP decimal blocks. The retained
TSV includes exact output bits and rejected inputs. The tested supplementary
digit and non-ASCII TOW digit were rejected; untested Unicode versions are
not assumed to match.
`calendar-doy.json` covers six UTC instants and four output systems.
`doy-systems-seeded.json` and `utc-doy-seeded.json` capture 240 TDB inputs
per output system; `doy-epochs-seeded.json` captures 261 wide-range inputs
per system and `doy-leaps.json` captures 336 inputs near LSK boundaries per
system. `ymd-epochs.json` captures 261 YMD inputs per system.
`gps-seconds-numeric.json` and `gps-numeric-rounding-grid.json` capture 330
and 1,200 numeric GPS display rows respectively. `format-combinations.json`
records accepted fallbacks that differ from documented applicability.
`output-format-matrix.json` records 50 observed combinations of five output
systems and ten requested formats for the single UTC input
`2000-01-01T12:00:00`, using server kernel set 2 and the custom picture
`YYYY-DOY/HR:MN:SC.######`. It retains the request, output column metadata,
and returned value for each combination. These observations do not establish
behavior for other inputs or server versions; the generic output-format
dispatcher remains outside the implemented API.
`gps-signed-zero.json` and its derived TSV preserve the observed output
binary64 sign of zero at ten near-zero inputs.
These finite samples do not prove the complete grammar or formatting behavior.

`hex-python.tsv` is independently generated with Python `float.fromhex`,
seed 817623, using `tools/generate_wgc_hex_reference.py --output <path>`.
The script refuses to overwrite committed references. It is not a NASA
reference. Its binary64 bit patterns test numeric parsing and IEEE rounding
separately from the WebGeocalc observations. Scientific claims require both
sources to be considered separately.

`cspice-doy-seeded.tsv` contains 12,000 local CSPICE N0067 TIMOUT `::RND`
strings from 3,000 binary64 ET inputs and four systems, including the GPS
UNITIM conversion. `cspice-doy-metadata.json` records the local LSK and
library hashes and generator seed. Reproduce to a separate path with
`python tools/capture_wgc_doy_cspice.py --output <path>`. These native
results are independent local references, not additional NASA WGC responses.
`altered-lsk-doy-ymd.tsv` adds 3,200 local CSPICE strings generated using
the modified constants in `altered-lsk.tls`; its JSON metadata records their
hashes. `tools/capture_wgc_doy_altered_lsk.py --output-dir <path>` reproduces
the capture without overwriting committed references.

The independent review retained compressed raw outputs for 20,490 wide-epoch
calendar comparisons and 49,248 UTC strings around six altered 2017 leap
offsets. `review-wide.tsv` compares native TIMOUT calendar components and
separately captured ETCAL/ET2UTC years; TIMOUT's fixed-width year text is
not treated as the WGC extended-year presentation. `review-altered-leaps.tsv`
compares full native strings. `review-metadata.json` records source and
adapter hashes and counts; the exact capture script and raw records are
retained as gzip files. These are local CSPICE observations, not live WGC
samples, and the capture script records machine-specific paths.

`SHA256SUMS.txt` covers every JSON, TSV, gzip and LSK data file in this directory.
`python -m unittest discover -s tools/tests -p 'test_wgc_gps_input_reference.py'`
checks those hashes and the TSV extraction from the retained WGC responses.
