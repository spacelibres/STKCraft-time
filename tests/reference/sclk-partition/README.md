# MRO partitioned spacecraft-clock reference

`native.tsv` contains 224 SCDECD inputs and 111 SCENCD strings for the
31-partition MRO clock, captured using CSPICE N0067 through SpiceyPy 8.3.0.
Inputs and encoded tick outputs use exact binary64 hexadecimal bits; string
outputs and native short errors are recorded verbatim. The fixture exercises
partition transitions, fractional neighboring ticks, malformed strings and
explicit or omitted partition numbers.

`capture.json` records the native library, clock kernel and output SHA-256
digests and query counts. Reproduce with
`python tools/capture_sclk_partitions.py` from the repository root. Replaying
the same inputs must reproduce the saved bytes; do not regenerate expected
results to accommodate an implementation mismatch.

These cases do not exhaust all type-1 clocks, offsets, delimiter choices,
reset overlaps, error precedence or ET conversion behavior.

Additional independent captures cover parser history and UTF-8 truncation,
seeded clock strings, reset/overlap kernels, all delimiter codes, nonzero
and negative offsets, fractional partition endpoints, ET-to-string rounding,
embedded NUL and partition capacity. `large-field.tsv` adds 6,008 selected
and seeded formatting inputs, `large-offset.tsv` adds 1,004 inputs with a
large positive offset, and `extreme-field.tsv` records the 30-byte native
format buffer boundary. Each adjacent JSON file records hashes and counts;
the matching `tools/capture_sclk_*.py` script describes input generation.
The Python binding replays the large-field corpus through the Rust adapter.

The synthetic and MRO cases are finite evidence, not a claim that all
possible type-1 kernels, parser histories or platforms are covered.

`coefficient-range.tsc` and `coefficient-range.json` record three SCS2E
cases with the first coefficient shifted to encoded tick 10. The first two
produce `SPICE(VALUEOUTOFRANGE)`, and the third maps exactly to ET zero.
`et-error-grid.tsv` adds 660 finite SCE2S cases across MRO, reset, and
shifted-coefficient kernels; `et-error-grid.json` records the inputs' kernel,
LSK, and CSPICE library hashes. `nonfinite-process-windows.json` records
isolated process observations, including native process termination for
infinite values. The library's non-finite guard is explicitly identified as
an STKCraft guard, not attributed to CSPICE.
`single-zero-start.tsc` and its JSON record four native outputs for a
single partition beginning at zero; the partition prefix and padded fields
are required even though the legacy field-vector path is also available.
`tiny-rate.tsc` and `tiny-rate-process-windows.json` record a positive
`1e-320` coefficient rate. At ET +/-1, local CSPICE reports a native range
error; at ET 0, it produced NaN ticks and the Windows SCE2S string
`2/000:00:000`. `nan-finite-et/capture.json` records 12 finite-ET cases
against separate Windows and Linux x86-64 CSPICE N0067 builds, including
single/multiple partitions, field widths, offsets and all delimiters.
Six additional metadata/width cases are Windows-only in `extra-windows.json`.
With SpiceyPy installed, run `python tools/capture_sclk_nan_intermediate.py
--output <path>` to replay all 18 retained kernels in isolated processes;
the output path must be outside the reference directory.
These are finite observations, not an exhaustive or live-WGC claim.

`overflow-rate.json` records four Windows CSPICE SCS2E outputs for the
`overflow-rate.tsc` synthetic clock. A finite clock string can produce an
infinite native ET. Reproduce the observation outside the reference directory
with `python tools/capture_sclk_overflow.py --output <path>`.
