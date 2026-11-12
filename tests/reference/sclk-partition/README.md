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
