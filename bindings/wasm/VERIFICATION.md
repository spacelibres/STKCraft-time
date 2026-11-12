# WASM verification record, 2026-11-12

Local verified runtime/compiler versions: rustc 1.99.1
(58cf8f9c3 2026-10-13), wasm-bindgen 0.2.129, Node 24.15.0,
Chrome 157.0.8103.78, TypeScript 5.9.3, esbuild 0.27.3.
Core is built with its default std backend. No core arithmetic was changed
by this adapter. Numerical target agreement was measured, not inferred.

Commands passing: cargo fmt -- --check; wasm-target cargo clippy with
-D warnings; npm run build; npm run test:node; npm run test:browser;
npm run test:types; npm run test:consumer. Both actual WASM runtimes run
the shared suite (79,520 assertions):

| Independent fixture | Rows | Result |
|---|---:|---|
| CSPICE checked UTC contract | 910 | Exact f64 bits |
| Existing UTC/ET vectors | 82 | Maximum error 0 seconds |
| CSPICE SCLK both directions | 10,871 | Exact f64 bits, including TDT rounding counterexamples |
| ERFA geocentric TDB-TT | 1,218 | Maximum error 4.553649124439119e-18 seconds |
| ERFA observer TDB-TT | 724 | Error below existing 1e-15 second gate |
| ERFA TT/TCG/TCB and TDB coordinates | 950 | Maximum error 2.384185791015625e-7 seconds; TT/TDB/TCB < 1e-6, TCG < 5e-7 |

Transport tests cover >2^53, both i64 endpoints, exact negative normalization,
i128 overflow and duration endpoint totals, one-nanosecond arithmetic,
NaN/infinity/fraction/range rejection before ABI narrowing, raw JsValue boundary
bypasses, schema origin/units, runtime model separation, disposal, leap labels,
EOP scalar/batch UT1, BDT origin/negative weeks, lazy series and formatting.
Browser proof includes module Workers (compiled Module initialization), default
URL initialization and esbuild bundling; Node uses byte initialization.
Clean npm tarball installation runs outside the source checkout and validates
WASM asset resolution and independent golden UTC plus exact bigint transport.

Original fixture SHA256 identities:

- MRO_SCLKSCET.00120.tsc: 3d7d62528624421bdb5b74cfa842bb5eac7cbeab89211a4377c1c500fee76078
- juice_step_250412_v01.tsc: a4a987d81ee3be7c1586ad8d6d270cddb1b1d4fb356510847d961cbd208edfc1
- LSK source identity is recorded in tests/golden/sclk.csv; runtime uses the core builtin naif0012 snapshot.

Limits: verification is on the stated local Windows runtimes. Firefox, Safari,
other JS engines and other compiler versions have not been measured here.
Full source/API inventory is API_INVENTORY.md. Civil and numeric calendar
formatting have documented conservative boundary guards; exact arithmetic
retains the core i64-second range. The Rust adapter floor is declared 1.88
but the compiler used here is 1.99.1. Registry publication and npm ownership
are separate release tasks; no publication was performed by these tests.

Review-driven hostile-input regressions reject traps explicitly: UT1 inverse
at Number.MAX_VALUE and 1e300, pathological finite DELTET offsets/anomalies,
SCLK parallel overflow/epoch range, oversized structured leap/EOP dates and
maximum-i32-year formatting carry. Legacy float outputs are checked for
finiteness; inverse magnitude/anomaly preconditions protect infallible core
constructors without substituting alternate calculations. All 60,000
millisecond labels in a minute round-trip through JS Date exactly; genuine
sub-millisecond labels still reject by default.

Additional review regression: finite MAX/-MAX EOP offsets remain accepted,
but interpolation, UT1 offset and forward scalar/batch overflow return
NonFinite. Inverse scalar/batch rejects unrepresentable iteration bounds
as Ut1OutOfRange before any infallible core constructor. Batch codes/classes
match scalar failures and include index. Custom-MAX-anomaly legacy UTC/ET
returns a structured NonFinite error, never an unmarked NaN.

Both public and raw ABI regressions cover 1e19 DELTET amplitude inverse
representability and SCLK i64::MAX offset reconstruction. Valid zero-tick
fields retain i64::MAX; the first overflowing decoded field returns Overflow
before delegation, without rejecting the whole clock kernel.

The coordinate reference comparisons retain the native error < 1e-6 second
gate for TT/TCB and apply the tighter Python error < 5e-7 gate to TCG.
TDB coordinates also satisfy error < 1e-6 seconds; no gate was relaxed. Separate paired conversions
exercise every public Scale (TT, GPS, TCG, TCB.IAU, TDB.Compact, TDB.IAU);
the 950 reference epochs additionally round-trip TT, TDB.IAU, TCG and
TCB.IAU below 1e-6 seconds. Reference agreement and round trips are distinct
assertions. The former TCB.Naif inverse spelling is corrected to TCB.IAU.

Active adjacent integer LSK jumps outside i32 subtraction return Overflow
before civil inverse delegation for epochToUtc, checked etToUtc, legacy
etToUtc and indexed etToUtcBatch. Frozen inactive entries and the SOFA
historical path remain usable; the original-order raw NaifUtcContext still
reports native InvalidNaifLeapSeconds rather than scientific validation.
The npm package carries the exact GitHub repository/subdirectory metadata
required for provenance, and Bdt.seconds maps to the approximate toSeconds
method in both the inventory and its generator.
