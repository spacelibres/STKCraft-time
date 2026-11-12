# Python, WebAssembly and JavaScript binding design

Status: approved by the user on 2026-11-12, including full Python/WASM/JS and BeiDou BDT support.
Local baseline: stkcraft-time 0.1.31, commit a7b3d22 (verify full SHA before implementation).
Reference: hifitime commit e258220164f6f6a488e2e1d7083587a540990567.

## Decision

Use two independent adapter crates, bindings/python and bindings/wasm, each
with its own empty [workspace], Cargo.lock and explicit toolchain policy.
Both depend directly on the same exact version of the root Rust crate.
Keep the root crate dependency graph, public arithmetic and existing MSRV
and no_std contracts intact. No PyO3 or wasm-bindgen attributes go on core
structs; no calculation is reimplemented in Python or JavaScript.

Python uses PyO3 and maturin. WASM uses wasm-bindgen plus a thin JavaScript
package and TypeScript declarations. JavaScript consumes this same WASM;
no independent JavaScript time engine and no Node native addon are needed.
Use a single npm package with explicit entry points for browser/bundler and
Node. The default browser entry uses explicit asynchronous initialization
accepting a URL, bytes or a compiled module; an optional inline entry may
be added only after testing CSP, bundle size and startup cost. Package names
are proposals until registry ownership is verified: Python distribution
stkcraft-time with import stkcraft_time, npm @stkcraft/time.

Rejected alternatives: putting binding dependencies/features in the core
risks feature unification, MSRV and no_std changes; JSON-only RPC loses
natural object APIs, introduces integer serialization pitfalls and extra
copy overhead. JSON is an explicit interchange facility, not the ABI.

## What to learn from hifitime

Adopt class-oriented Epoch/Duration APIs, iterators, type declarations,
separate platform packaging and tests which install built artifacts.
Its Python registration exposes real classes and errors, and its WASM
wrapper delegates to Rust. Do not copy its epoch origin or time-scale
semantics. Our Epoch remains TAI since 2000-01-01 00:00:00 TAI, and NAIF
ET remains TDB seconds past J2000 noon. Do not import its precision claims
as our own guarantees. Reimplement the adapter design for this MIT project;
any actual upstream code reuse requires preserving applicable MPL notices.

## Public coverage contract

Create a reviewed API inventory linking every public core capability to
its Python name, JS/TS name, supported input types and test. Every item must
be classified as wrapped, represented by a language protocol, deliberately
low-level, or unavailable with an explicit reason. No hidden omissions.

| Core capability | Python surface | JavaScript/TypeScript surface |
| --- | --- | --- |
| Epoch | immutable class, checked parts, comparison, arithmetic | checked class, bigint seconds, number nanoseconds |
| Duration | exact integer nanoseconds, arithmetic/operators | bigint total nanoseconds, named checked methods |
| UTC/calendar | UtcDateTime and Calendar | UtcDateTime and Calendar |
| Typed NAIF results | NaifEt and NaifJulianUtc | same distinct classes/types |
| TimeContext | owned immutable LSK snapshot, checked methods | owned immutable LSK snapshot, checked methods |
| ScientificContext | separate IauTdb and model identifiers | separate IauTdb and model identifiers |
| Leap data/policy/source | text loading, entries, metadata, immutable selection | text loading, entries, metadata, immutable selection |
| SCLK | kernel, coefficients, partitions, fields and conversion methods | same, with exact integer fields |
| EOP/UT1 | table/series/observed-only selection and conversions | same from text/bytes, no automatic fetch |
| TCG/TCB, compact TDB and ERFA | explicitly named model modules | explicitly named model modules |
| Formatting | options, standard/custom formats | options, standard/custom formats |
| Kernel dates | existing parse_kernel_date contract | existing parse_kernel_date contract |
| TimeSeries | lazy iterator | lazy iterable, not materialized by default |
| Legacy conversion functions | explicit compatibility namespace | explicit compatibility namespace |
| Errors/constants/version | documented classes, codes, units, version | Error subclasses/codes, units, version |

Traits and Rust borrowing do not map one-to-one: SclkClock/SclkMap behavior
is represented by an owned clock/context object. Formatter stores owned
configuration rather than a borrowed temporary Rust object. MCP protocol
internals are not a second public scientific binding API.

## Precision and input boundary

1. Never convert Epoch/Duration through total f64 seconds merely to cross
   a language boundary. Use exact (seconds, nanos) components or total
   integer nanoseconds, preserving the core normalization rules.
2. Python int is range-checked before extracting i64/i128. Reject booleans
   where numeric time fields are required. Float paths are explicit and
   require finite values; integer fields reject fractional values.
3. JS seconds/total-nanoseconds are bigint. Small fields are validated
   finite integers with explicit ranges before narrowing. BigInt values
   must be range-checked before wasm-bindgen converts to i64/i128: generated
   ABI conversion can wrap. Enforce this in Rust JsValue entry points as
   well as ergonomic JS wrappers, so bypassing TypeScript cannot bypass it.
4. NaifEt and IauTdb remain separate at runtime as well as in declarations.
   No implicit scale selection, no automatic scientific fallback and no
   Date/datetime conversion hidden in an Epoch constructor.
5. Python datetime and JS Date are convenience adapters only. Their reduced
   resolution and inability to represent leap-second labels require
   explicit loss policy; the default rejects an unrepresentable conversion.
6. JSON uses a versioned schema with scale, origin and units. i64/i128
   values are decimal strings, not JSON numbers. Do not redefine
   BigInt.prototype.toJSON globally. Numeric ET cannot be called exact TAI.
7. Preserve raw numeric ET/SCLK bit patterns through the ABI. No string
   formatting roundtrip, float32 conversion, alternate operation order,
   clipping, tolerance inflation or result-specific corrections.
8. A nanosecond storage field is not a physical accuracy guarantee.

## Ownership, errors and batch operations

A language TimeContext owns a shared immutable LeapSecondTable; each call
creates a short-lived native borrowed TimeContext. No self-referential
structure or unsafe lifetime extension is needed. Changing policy creates
another immutable object. EOP and SCLK inputs are likewise owned snapshots.
Pass kernel text/bytes to Rust parsers. Native Python may provide explicit
file helpers; browsers provide fetch/File externally. Record caller-provided
source identity as such, never as independently verified provenance.

Map TimeError, ContractError and DtdbError to stable error codes and native
language exceptions. Preserve details such as missing kernel field and
coverage failure. Expected invalid input must not panic or trap, silently
become zero or return an unmarked NaN. Check before invoking legacy
infallible core constructors/operators. Unexpected panics/traps remain
reported internal failures, not fabricated successful outputs.

Batch UTC/ET, SCLK and UT1 operations should cross the ABI once per batch,
reuse parsed data and preserve scalar equivalence. Default errors abort
with index/context; partial results need an explicitly distinct API.
Python accepts ordinary sequences; NumPy acceleration can be optional with
an explicit dtype/shape contract. JS uses Float64Array for f64 data and
exact integer arrays/components for integer time. No borrowed view survives
WASM memory growth or object disposal. Python may release the GIL only after
converting inputs to owned Rust data. JS bulk work can run in a Worker.

## Packaging and platform contracts

Core: keep Rust 1.75, no_std + alloc, wasm32 and thumbv7em gates unchanged.
Bindings: pin compatible dependencies/CLI versions and document their own
build MSRV. PyO3 0.29 currently requires Rust >=1.83; this cannot silently
raise the core MSRV. Initial proposed Python floor is CPython 3.10; prove
abi3 compatibility before selecting it, and test each claimed runtime.
Free-threaded Python/PyPy are separate tested support claims, not assumed.

Python wheels: Linux x86_64/aarch64, macOS x86_64/arm64, Windows x86_64,
plus an sdist that contains the exact core source needed for a clean build.
A path dependency outside a standalone sdist is not an acceptable release.
Use an embedded source layout generated from the exact tagged core and
verify its hashes, or a pinned registry core dependency with an explicit
online/offline build policy. Test the sdist outside the repository.

JS/WASM: Node and real headless browsers, TypeScript strict compilation,
bundler fixture, packaged .wasm asset resolution and Worker initialization.
Publish generated declarations and licenses; verify npm pack contents and
install the tarball in a clean consumer. Test .free/disposal and use-after-
free behavior. Default scientific code performs no network I/O or system
clock access; explicit loading/init is the application boundary.

## Numerical verification gates

Before full wrapping, implement a precision vertical slice: checked Epoch
parts, Duration, text LSK, typed UTC/ET and both SCLK directions. Run actual
Python extension and actual WASM in Node and at least Chromium; use the
existing independently generated CSPICE/ERFA vectors and the known 0.93 ns
and 119.21 ns counterexamples. Verify min/max i64, i128 totals, >2^53,
negative normalization, +/-1 ns, NaN/infinity, fractional and oversized
integer inputs, leap labels and removed labels, model bounds and bad files.

Separate transport exactness (mandatory bit-preserving) from target math
backend behavior. Current core math uses std trig or libm depending on
features. Do not promise native-vs-WASM bit parity from compilation alone.
Measure against the same independent references per runtime. Any mismatch
blocks the stated parity claim and gets a root-cause report; do not change
core trig or relax acceptance to make bindings pass. A necessary core
change requires a separate reviewed decision and full native regression.

Then cover the full API inventory, error paths, EOP/UT1 and scientific
models, batch/scalar agreement, lazy iteration and serialization. Reuse
existing reference data instead of making expected outputs with bindings.
Report counts, runtime/compiler versions, kernel hashes and raw f64 bits.

## Implementation and release sequence

1. Approve this boundary/coverage design and pin the current clean core.
2. Add independent adapter packages and the precision vertical slice; prove
   numeric transport and real runtime reference comparisons before breadth.
3. Complete every API inventory category for Python, then mirror coverage
   in WASM/JS; keep model and error contracts identical.
4. Add wheel, sdist, npm tarball and browser consumer installation tests.
5. Run all existing twelve core release gates plus binding-specific gates;
   independently review kernel diff (target: no src/ arithmetic changes).
6. Version all artifacts from one release identity; keep root Cargo metadata,
   adapter crates, Python metadata, npm metadata and release notes in sync.
   Use the repository patch-release policy unless a minor release is
   explicitly selected. Do not preallocate a version while parallel work
   might advance the repository.
7. Build/test all artifacts before any registry publication. Follow existing
   main/tag/CI/GitHub/crates.io steps and add verified PyPI/npm publication.
   Check registry ownership and available trusted-publishing/token setup;
   never put credentials in source. Registry access failure does not count
   as successful publication. Attach checksums and provenance to artifacts.

## Primary sources examined

- https://github.com/nyx-space/hifitime/tree/e258220164f6f6a488e2e1d7083587a540990567
- https://raw.githubusercontent.com/nyx-space/hifitime/e258220164f6f6a488e2e1d7083587a540990567/src/python.rs
- https://raw.githubusercontent.com/nyx-space/hifitime/e258220164f6f6a488e2e1d7083587a540990567/web/Cargo.toml
- https://raw.githubusercontent.com/nyx-space/hifitime/e258220164f6f6a488e2e1d7083587a540990567/web/README.md
- https://pyo3.rs/v0.29.0/
- https://wasm-bindgen.github.io/wasm-bindgen/reference/types/numbers.html


## Approved BeiDou addition

Add a separate exact Bdt type/module with the nominal BDT reference epoch
2006-01-01 00:00:00 UTC and continuous SI seconds without leap insertion.
Preserve the existing public TimeScale enum to avoid breaking exhaustive
Rust matches in downstream projects. Expose BDT in language-level interfaces
through its explicit type. Do not treat nominal offsets as measured clock
realization accuracy. Validate against official BeiDou documentation and an
independent time engine. Existing Geocalc NAIF/LSK/SCLK calls must retain
identical behavior and compile without source or lockfile edits.
