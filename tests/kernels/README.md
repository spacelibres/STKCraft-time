# Independent SCLK test inputs

These original kernels are test fixtures, not runtime mission data defaults.
Files retain the bytes downloaded from their authoritative publishers.
Git attributes preserve their original line endings and whitespace, including
upstream trailing spaces, so checkout does not invalidate the source hashes.

| File | Official source | SHA-256 |
| --- | --- | --- |
| MRO_SCLKSCET.00120.tsc | https://naif.jpl.nasa.gov/pub/naif/MRO/kernels/sclk/MRO_SCLKSCET.00120.tsc | 3d7d62528624421bdb5b74cfa842bb5eac7cbeab89211a4377c1c500fee76078 |
| juice_step_250412_v01.tsc | https://spiftp.esac.esa.int/data/SPICE/JUICE/kernels/sclk/juice_step_250412_v01.tsc | a4a987d81ee3be7c1586ad8d6d270cddb1b1d4fb356510847d961cbd208edfc1 |

`python tools/gen_sclk_golden.py` generates the expected outputs exclusively
with CSPICE N0067 through SpiceyPy 8.3.0. Query grids and rounding
counterexamples are diagnostic inputs, not fabricated mission observations.
See ATTRIBUTION.md for NAIF source notices.
