# JavaScript, TypeScript and WebAssembly

Independent adapter crate and npm candidate `@stkcraft/time`, version 0.1.33.
The package name is a candidate until registry ownership is confirmed.
Rust build floor: 1.88; tested Node 24, Chrome, and module Workers. Core
retains its independent Rust 1.75 contract. Adapter uses the core's `std`
math backend; measured browser/Node oracle results are recorded by the tests.
Nanosecond storage does not promise nanosecond physical accuracy.

```js
import { init, TimeContext, UtcDateTime, Duration } from '@stkcraft/time/node';
await init();
const context = new TimeContext(); // Explicit bundled naif0012 snapshot.
const epoch = context.utcToEpoch(new UtcDateTime(2016, 12, 31, 23, 59, 60.5));
const next = epoch.add(new Duration(1n));
console.log(next.seconds, next.nanoseconds); // bigint plus number, exact.
next.free(); epoch.free(); context.free();
```

Browser/bundler applications import `@stkcraft/time` and call `await init()`.
Initialization accepts a WASM URL/string, bytes, or compiled WebAssembly.Module.
Bundle tools must copy `pkg/stkcraft_time_bg.wasm` and may supply its asset URL
explicitly. No inline base64 WASM or unsafe eval is required. Module Workers
use the same entry. Node's `/node` entry reads the installed WASM asset locally.
Loading kernels remains application controlled; no conversion fetches data.

`Epoch.seconds`, BDT seconds, duration total nanoseconds and SCLK integer fields
are bigint. Numeric ET and SCLK ticks remain binary64 numbers. Exact integers
are checked in both the JS facade and Rust JsValue boundary before any ABI
narrowing. No BigInt prototype modifications or total-float Epoch transport.
`toTaiSeconds`, `toSeconds` and floating scale methods explicitly lose precision.
JSON schemas record origin, scale and units, using decimal strings for integers.

`TimeContext` owns its LSK snapshot; `select` returns another snapshot. Metadata
is a detached copy. `NaifEt`, `NaifJulianUtc` and `IauTdb` are distinct runtime
classes. Scientific methods never fall back to NAIF. Coordinate scale names
are explicit (`TCB.IAU`, `TDB.Compact`, `TDB.IAU`). `ScientificContext` checks
model bounds. BDT is nominal TAI-33 s at its own 2006 origin; GPS elapsed seconds
have a different origin and cannot be converted by subtracting only 14 s.

Owned WASM objects provide `free()`; JS guards reject later access as `Disposed`.
Use `try/finally` for deterministic cleanup. TimeSeries allocates one result per
step; consumers own yielded Epoch objects. Breaking iteration frees the iterator.
No borrowed memory view escapes. Batch methods copy their input/output and abort
with an error `index`. Errors expose stable `.code` strings named for core errors
or boundary categories (`Type`, `Range`, `Overflow`, `NonFinite`, `Disposed`,
`Schema`, `NotInitialized`). See generated declarations for all signatures.

JS Date adapters require an explicit loss choice for sub-millisecond values;
default rejects loss. Leap labels always reject Date conversion. Civil
constructors check field ranges; the context validates actual dates and leap data.

Build and verify (from this directory):

```sh
npm ci
npm run build
npm run test:node
npm run test:browser
npm run test:types
npm run test:consumer
```

Build requires Rust wasm32-unknown-unknown and wasm-bindgen CLI 0.2.129.
All Cargo and npm dependencies are locked. Browser tests use installed Chrome
by default; set STKCRAFT_BROWSER_CHANNEL=chromium for Playwright's Chromium.
Generated `pkg` is ignored in git and included in npm tarballs after building.
`npm run test:consumer` checks the actual tarball in an independent temporary
project. No npm publication is implied by a successful packaging test.

Boundary limits: legacy inverses use conservative magnitude and kernel-anomaly
preconditions before infallible core calls. Unsupported combinations return
EpochOutOfRange or NonFinite; all returned calculations still come from core.
EOP samples require |MJD| <= 7e11; structured
leap JDNs require |JDN| <= 7e11. Scientific TDB/TCB inverse entry points
require the documented model interval. Maximum-i32-year final-minute formats
reject possible year carry. These explicit foreign-boundary limits do not
change core Rust behavior. TCB.IAU delegates the core IAU TCB model.

Finite EOP offsets are accepted without a physical amplitude clamp. Raw
interpolation and UT1 forward results reject overflow as NonFinite. Inverse
UT1 preflights the sum of input/offset magnitudes against core Epoch
representability before iteration. Batch failures retain scalar codes and
TimeError classification and add index.
