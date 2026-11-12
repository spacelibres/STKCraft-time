# WebGeocalc GPS and day-of-year observations

These JSON records were captured on 2026-11-12 from the live NASA WGC2
`TIME_CONVERSION` endpoint over verified HTTPS using kernel set ID 2. Each
file preserves the parsed requests, creation/status responses and completed
results received by the capture. JSON was reserialized with indentation; it
is not a byte-for-byte archive of the HTTP response bodies. Remote calculation IDs may expire. Kernel
set ID 2 is a server-managed configuration and its complete file list and
content hashes have not yet been independently pinned.

- `utc-outputs.json`: seven UTC labels against GPS week/TOW, GPS seconds and
  UTC day-of-year output; SHA-256
  `c257ccc2aa057cff5ff8add894ad3d9be9fe4e61947371fe11ad932cedc5d6e0`.
- `seconds-boundaries.json`: twelve GPS epoch-second inputs to week/TOW;
  SHA-256 `98bb1f5a49dfe33664afd20356006c0866fb572b1840bc9ae06452e60988a853`.
- `week-inputs.json`: eight individual week/TOW inputs, including malformed
  strings; SHA-256 `b5de7e6b18f9ae2c2e054882cb8fc2724c88bd102d41fb596ee0d2b8fe81a165`.

The reported strings and numeric cells are reference observations, not
unrounded internal binary64 values. NASA's displayed numeric fields may
already be rounded. Do not regenerate or change these files to make a local
test pass. Their finite cases do not establish the complete input grammar,
output precision, rounding, kernel-set stability or time-conversion parity.

Further `SECONDS_PAST_GPS0` to `WEEK_TOW` samples cover an 88-row boundary
grid, a 66-row microsecond rounding grid and 200 seeded values. Their
SHA-256 hashes are respectively
`7a9d650675a463204d5c7c35fc3637782c04c8c10961b40c62a7a31afba112d9`,
`641ae32d061a42a44ace823090b7eb85070c7c7ee054ad54107523930e8d23ac`,
and `88ea5761eed064791879138c9a0029fbfc7011ec30ca215015adebfefdc1ff3d`.
`week-tow-cases.tsv` is a lossless extraction of their input strings and
returned week/TOW strings, SHA-256
`e75f9abe9ad80ae6a56dac7af3a0a142c22946dda0febb58290b02bff76fc9f7`.

`kernel-set-2.json` records the server's description of set 2 as the latest
LSK, named `naif0012.tls`. `lsk-comparison.json` records independent CSPICE
kernel-pool values for the embedded snapshot and the NAIF generic LSK:
their raw file hashes differ, while the five `DELTET` arrays match bit for
bit. The server's exact loaded file bytes were not independently obtained.

`unitim-native.tsv` independently captures the exact input, intermediate ET
and reconstructed GPS-second binary64 bits for the same 354 inputs using
CSPICE N0067 and the embedded LSK. Its SHA-256 is
`166112eacddddf4b9e8bad236ef69b3e2fc366af3e3194abc445b28e9675bc31`;
`unitim-native.json` records library and kernel hashes. Reproduce into a new
location with `python tools/capture_wgc_gps_unitim.py --output-dir PATH` and
compare the bytes to the retained reference.

Additional live captures cover half ties, decimal-adjacent values, i32
week-field boundaries, large inputs and a 400-row seeded rounding grid.
An independent reviewer captured another 480 near-half values in
`independent-half-probe.json`, including raw HTTP response text, the request,
CSPICE comparison and generation details. Extracted TSV files are compared
verbatim to their respective JSON records by
`python -m unittest tools.tests.test_wgc_gps_reference`.
`manifest.json` records SHA-256 for every retained data file.

`unitim-altered.tsv` contains four independent CSPICE rows after changing
`DELTET/K`, `DELTET/EB`, and `DELTET/M`; its adjacent JSON records the exact
values, source hash and output hash. This is a native operation-order test,
not a claim that WGC was queried with an altered kernel.

The retained `replay-2026-11-12.json` records a second live WGC2 execution
of 22 saved requests. Comparison covers 1,325 returned rows, columns, and
native short descriptions for failed requests, with zero mismatches. Numeric
JSON cells are compared by their parsed binary64 bits, including signed zero;
strings are compared exactly. This does not expose WGC's unrounded internal
values and does not establish behavior beyond the captured requests.

`nonfinite-inputs.json` retains three additional WGC2 calculations: GPS
seconds inputs `NaN`, `Infinity`, and `-Infinity` each returned the literal
week/TOW string `0, NaN`. This display value is not a valid numeric epoch.
The JSON SHA-256 is
`27c42fccbc3a4b8a994afb67101d5ea1448c82d109c8a20d58f08bc68fbda71f`;
the extracted TSV is verified against it by the reference test. These three
requests were captured after the retained 22-request replay and are not part
of its 1,325-row count.
