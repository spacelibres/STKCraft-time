# SCLK parsing and CK precision blockers

The user approved fixing both blockers in the independent time repository,
with the existing zero-tolerance CK acceptance criterion retained.

Accept a single numeric text-kernel value with or without parentheses,
including one-field moduli, offsets and partition bounds. Keep required
field counts and numeric validation; do not consume the following assignment
when a value is missing. This is focused SCLK parsing, not a general SPICE
kernel-pool implementation.

For TDT clocks, convert the original parallel TT directly through the N0067
UNITIM evaluation order, and convert ET directly back to TT for the inverse.
No integer Epoch or origin shift belongs in either f64-to-f64 path. Share
the direct TT/ET calculation with the checked NAIF context. Retain explicit
Epoch conversions for callers requesting integer TAI. Match SCTE01/SCEC01
rate evaluation order for the piecewise affine map.

Prove the failures before implementation. Use independent CSPICE outputs,
original NASA/ESA mission kernels, raw floating-point comparisons, coefficient
neighbors, cancellation boundaries and CK queries with zero tolerance.
Artificial parser/CK fixtures are test inputs and must be labeled as such;
scientific reference outputs must come from CSPICE. Leave the developing
Geocalc source and lockfile untouched during isolated integration checks.

Release 0.1.18 follows all AGENTS.md gates and publication rules.
