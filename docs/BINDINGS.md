# Python, WebAssembly and JavaScript interfaces

The Rust calculation library is the single implementation. Independent
adapter crates live in bindings/python and bindings/wasm; they have separate
Cargo locks and build toolchain floors. Binding dependencies are excluded
from the root package/dependency graph. The core remains Rust 1.75 and
no_std + alloc. Language adapters do not change the existing TimeScale enum
or the NAIF/LSK/SCLK arithmetic used by Geocalc.

## Language packages and complete coverage

| Consumer | Package / entry | Implementation and inventory |
| --- | --- | --- |
| Python >=3.10 | stkcraft-time / import stkcraft_time | [Python guide](../bindings/python/README.md), [inventory](../bindings/python/API_COVERAGE.md) |
| Browser / bundler | @stkcraft/time | [WASM guide](../bindings/wasm/README.md), [inventory](../bindings/wasm/API_INVENTORY.md) |
| Node >=24 | @stkcraft/time/node | The same WASM module, not an independent JS time engine |
| Embedded Rust | default-features=false | [Cortex-M example](../examples/cortex-m-utc-et/README.md) |

Inventories classify public capabilities and document language protocols in
place of Rust borrowing and traits. MCP transport internals are outside the
language scientific API. Use the per-package guide for exact method spellings,
installation, initialization, error codes, batch behavior and limitations.
Registry installation commands apply only after the corresponding registry
publication is confirmed; verified wheel/sdist/npm artifacts are also attached
to the GitHub Release. Missing credentials are reported as a publication
blocker, not as successful PyPI/npm availability.

## Stable scientific contract

- Epoch is integer TAI seconds and nanoseconds since 2000-01-01 00:00:00 TAI.
- NaifEt is finite binary64 TDB seconds past J2000 TDB noon. New geometry
  integration selects TimeContext explicitly. IauTdb is a distinct result
  requiring ScientificContext. Conversion is never selected by guessing.
- Legacy compatibility namespaces preserve the literal legacy Rust API
  evaluation paths; their rounding can differ from checked TimeContext.
- Python integers and JS bigint transport exact Epoch/Duration/BDT values.
  Total integer nanoseconds must not pass through f64 or JSON numbers.
- Numeric inputs are checked before narrowing. In particular, native WASM
  integer conversion must not silently wrap oversized bigint inputs or turn
  NaN into a valid calendar field. Expected input errors are exceptions.
- Date/datetime conveniences are explicit and disclose/reject reduced
  resolution or leap labels. Exact JSON schemas include scale, origin and
  units; integer counts use decimal strings.
- LSK, EOP and SCLK contexts own snapshots. No global kernel pool, implicit
  download or hidden update is introduced. Browser applications supply text
  or bytes; ordinary Rust/Python code can select local files explicitly.
- EOP daily interpolation and scientific model limits remain the core limits.
  Bindings do not imply new subdaily Earth-orientation or TT(BIPM) capabilities.
- Civil calendar guards in adapters may reject extreme values before legacy
  narrowing; consult the inventories for explicit foreign-boundary limits.
  Exact continuous integer arithmetic retains the documented core range.

## BeiDou time

Bdt is an additive nominal coordinate type: continuous SI seconds from
2006-01-01 00:00:00 UTC, without leap-second insertion. The nominal calendar
relation is TAI minus BDT = 33 seconds. GPS and BDT calendar coordinates differ
by 14 seconds, but their elapsed-second origins differ: GPS elapsed minus
BDT elapsed is 1356 * 604800 + 14 seconds. Do not confuse those quantities.

Integer parts, total nanoseconds and full signed week/nanosecond coordinates
are checked. Weeks are not truncated broadcast fields and do not roll over.
Negative values extend the coordinate mathematically before the operational
epoch. UTC conversions inherit the chosen LSK and future policy. ET paths
reuse TimeContext; they do not define a new TDB approximation.

This nominal coordinate conversion does not apply broadcast satellite clock
corrections or measure physical BDT clock realization error. See the primary
[ITU account by BeiDou timing specialists](https://www.itu.int/hub/2023/06/the-beidou-navigation-satellite-system-and-the-utc-leap-second/)
and tools/gen_bdt_golden.py for independent ERFA epoch/leap references.

## Verification and release gates

Every release retains the twelve existing core checks and adds language CI:
wheel installation/tests on Linux x86_64/aarch64, macOS x86_64/aarch64 and
Windows x86_64; CPython version checks; external sdist installation; actual
WASM Node/browser/Worker execution; strict TypeScript compilation; and a
clean npm tarball consumer. The tag release waits for binding verification
before publishing its combined artifacts.

Core CSPICE/ERFA golden files are reused as independent references. Tests
include >2^53 integers, full i64 boundaries, negative normalization, leap
labels, SCLK coefficient neighbors, earlier rounding counterexamples,
model/coverage errors, malformed data, exact transport and scalar/batch
agreement. Cross-runtime mathematical agreement is measured, not inferred
from successful compilation. No loose tolerance, model substitution or
result-specific correction is introduced to hide a binding difference.

The embedded example is linked for Cortex-M and has host reference checks.
It includes an on-target DWT harness, but no hardware timing or flight
qualification is claimed without actual board measurements.
