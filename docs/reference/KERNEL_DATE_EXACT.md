# Native kernel-date arithmetic evidence

The original N0067 FURNSH/GDPOOL capture contains 163 literals: 158 accepted
numeric constants and five native errors. Before the arithmetic repair,
the Time parser accepted 59 of those literals: 52 matched exactly and seven
differed by one ULP. It explicitly rejected the other 99 native-valid
inputs. The arithmetic repair described here fixed the seven ULP
mismatches. The 99 coverage failures were subsequently resolved by the
grammar repair documented in
[the grammar scope](KERNEL_DATE_GRAMMAR.md); neither repair widens the
claim beyond the exercised corpus.

The seven counterexamples are independently pinned in
`tests/kernel_date_fractional_exact.rs`. The test failed with all seven
original bit mismatches before changing production arithmetic. N0067 TPARSE
evaluates the day offset, hour, minute and second additions separately.
Reproducing that order fixes all seven without changing their inputs or
native expectations. The original eleven-literal regression is retained.

Original evidence is in `docs/reference/kernel-date-native/`: native stdin,
stdout and stderr, all 163 literal/results records, capture metadata, the
actual C oracle source and the original generator snapshot. The raw stdin
retains the original absolute capture paths as evidence. A reproduction
should create its own input kernels, not edit that capture.

The official N0067 archive SHA256 is
`98d60b814b412fa55294aeaaeb7dab46d849cc87a8b709ffe835d08de17625dc`.
The native library and selected C source files were byte-checked against it.
The manifest records the actual MSVC compiler, linked static UCRT and oracle
executable hashes. Native compilation used `/Od /fp:strict /MT`, Windows SDK
10.0.26100.0 and the default FMA setting of 1. No LSK or time-scale conversion
is used for these bare kernel constants.

To reproduce, compile the captured `oracle.c.txt` as C against the official
CSPICE include directory and static library with that profile. For each literal,
write `KPL/FK`, a `\begindata` block containing `EPOCH = <literal>` and
`\begintext`. Send `C`, `L|<kernel path>` and `D|EPOCH` on separate lines to
the oracle. Compare the returned hexadecimal binary64 bits directly.

The C reference is independent and is never linked into the production crate.
This evidence establishes the exercised arithmetic only. See
[the numerical issue](https://github.com/spacelibres/STKCraft-time/issues/1)
and [the separate grammar gap](https://github.com/spacelibres/STKCraft-time/issues/2).
