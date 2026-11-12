<p align="center">
  <img src="https://raw.githubusercontent.com/spacelibres/STKCraft_web/main/public/brand/albatross.png" alt="STKCraft — Albatross" width="72">
</p>

<p align="center">
  <a href="https://crates.io/crates/stkcraft-time">crates.io</a> ·
  <a href="https://docs.rs/stkcraft-time">API reference</a> ·
  <a href="https://github.com/spacelibres/STKCraft-time">source</a> ·
  <a href="https://github.com/spacelibres/STKCraft-time/wiki">wiki</a> ·
  <a href="https://www.stkcraft.com">STKCraft</a>
</p>

# Time contracts for STKCraft geometry and science

## Default geometry convention

`contract::TimeContext::new(&lsk)` selects NAIF/SPICE ET explicitly and
has no automatic scientific-model fallback. `NaifEt` is finite TDB seconds
past `2000-01-01T12:00:00 TDB`. It is not UTC seconds, TT seconds, Unix
time or the internal TAI epoch. `Epoch` remains integer TAI seconds plus
nanoseconds since `2000-01-01T00:00:00 TAI`.

```rust
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};
use stkcraft_time::contract::TimeContext;

let lsk = LeapSecondTable::builtin()?; // Select a mission LSK in production.
let context = TimeContext::new(&lsk);
let et = context.utc_to_et(
    &UtcDateTime::new(2016, 12, 31, 23, 59, 60.5), Calendar::Gregorian,
)?;
let seconds_for_spice = et.seconds(); // Explicit boundary to a legacy f64 API.
let selected_settings = context.metadata();
# Ok::<(), Box<dyn std::error::Error>>(())
```

Geocalc's existing `state_at_epoch` methods already call the NAIF
`convert::tai_to_et` path. New integration code can use the typed context
and pass `et.seconds()` to `SpkKernel::state`. Existing f64 interfaces
remain low-level: their callers are responsible for declaring the scale.
There is no implicit conversion between `IauTdb` and `NaifEt`.

The checked UTC entry point follows N0067 TTRANS's paired leap-transition
anchors and UNITIM arithmetic. The continuous `Epoch` entry point follows
UNITIM for explicitly supplied numeric TAI seconds past J2000. Those input
representations can quantize an instant differently: a UTC string and an
already rounded binary64 TAI value need not yield identical ET bits.
Reference validation must compare each path with its corresponding CSPICE
entry point. Convert once and reuse the resulting ET throughout a geometry
calculation. Do not alternate UTC and numeric-TAI routes within one query.
Legacy low-level functions retain their earlier evaluation path, including
the ET fixed-point approximation; the new context records the stricter
N0067 contract without silently changing those existing interfaces.
SCLK TDT clocks also use the shared direct N0067 TT/ET arithmetic, retaining
the original parallel time rather than detouring through an integer Epoch.
Their inverse follows native reciprocal-rate evaluation order, which matters
for nonbinary clock moduli and zero-tolerance CK queries.

NAIF context civil UTC conversion supports the post-1972 LSK convention.
Physical historical UTC uses the explicit low-level SOFA source API.
The separately named `TimeContext::et_to_spice_julian_utc` instead selects
N0067 TTRANS's scalar formal-JDUTC convention, including the native
pre-first-entry offset. It returns checked `NaifJulianUtc` directly from
ET and a raw IERS-only/TableDriven LSK. It does not change existing civil
coverage or infer a scientific historical model. See
[the exact native scope](reference/NATIVE_JDUTC.md) before consuming formal
Julian UTC at leap seconds or historical epochs.

`NaifUtcContext::from_raw_parts` is a separate structured raw NAIF entry point.
It borrows nonempty `LeapEntry` data without sorting and requires all five
finite `NaifDeltet` constants explicitly. Calculation validates the first 140
native paired-table entries and preserves `SPICE(BADLEAPSECONDS)` at that
boundary. The scientific `LeapSecondTable` constructor keeps its earlier
`LskParse` rejection. This resolves the captured constructor-blocked profile
without mixing models or changing civil behavior. The combined 17-profile
regression compares all 22,292 numeric values and 4,539 native error symbols.
Its integral-date/whole-offset contract does not model arbitrary raw kernel
variables, fractional entries or a new MCP input mode. Empty/nonfinite guards
are explicit checked-input errors; out-of-domain raw dates are checked before
integer arithmetic. See [the raw API example](API.md#raw-native-leap-table-errors).

## Scientific convention

`ScientificContext` must be selected explicitly. It realizes TT through
`TT(TAI) = TAI + 32.184 SI seconds`, independently of any caller-modified
LSK constants. Its TDB model is the SOFA/ERFA Fairhead-Bretagnon series,
with separately selected geocentric or observer-dependent terms.

```rust
use stkcraft_time::{Epoch, contract::ScientificContext};
let context = ScientificContext::new();
let tai = Epoch::from_parts(43_167, 816_000_000); // J2000 TT.
let tt_seconds = context.tt_seconds(tai)?;
let geocentric_tdb = context.to_tdb(tai)?;
let model_used = geocentric_tdb.model();
# Ok::<(), Box<dyn std::error::Error>>(())
```

The checked scientific model is restricted to TT dates from 1950-01-01
through 2050-01-01, including the endpoints. This is the documented model
validation interval, not an assertion that the approximation is exact
within it. Legacy low-level functions retain their existing extrapolation
behavior. Absolute f64 coordinate seconds do not preserve 1 ns resolution
at modern epochs, even when the periodic model is more precise.

Observer-dependent evaluation requires all of the following inputs:

| Input | Meaning |
| --- | --- |
| UT1 fraction | Fraction of a UT1 day starting at midnight, in `[0, 1)` |
| Longitude | East-positive longitude, radians |
| `u` | Distance from the Earth's spin axis, km, nonnegative |
| `v` | Signed distance north of the Earth's equatorial plane, km |

These quantities must refer to the same observer and instant. Obtain UT1
from an appropriate IERS data/model chain; obtain the observer coordinates
from the appropriate terrestrial reference system. UTC must not be
substituted for UT1 without an explicitly assessed approximation.
The API performs no hidden geodetic conversion, EOP download or UT1 guess.
The observer correction is a terrestrial TDB-TT model, not a spacecraft
proper-time conversion, barycentric light-travel-time correction or
complete Earth-orientation solver. The original geocentric DE405-relative
accuracy statement must not be advertised as a universal observer or
mission accuracy guarantee.

## Standards and supported capabilities

| Reference | Current capability | Limit |
| --- | --- | --- |
| NAIF/SPICE | LSK-based UTC/TAI/TT/ET, GPS, supported SCLK type 1 paths | Kernel and explicitly documented API coverage apply |
| IAU/SOFA/ERFA | Geocentric and observer-dependent TDB-TT, TCG/TCB APIs | Model scope and numerical representation apply |
| IERS | Leap-aware interpolation of daily EOP UT1-UTC samples | Subdaily ocean-tide/libration and uncertainty propagation are not implemented |
| BIPM/IAU | TT realized through TAI plus 32.184 seconds | This is not TT(BIPM) or a correction for UTC(k) clock realization |

TT(BIPM) requires the corresponding published BIPM realization data.
Complete terrestrial/celestial frame transformation also needs the
appropriate precession/nutation, polar motion and Earth-rotation model.
Neither capability can be inferred merely from a numerical TDB result.

### Kernel date constants

A text-kernel literal such as `@2000-JAN-01-12:00:00` is resolved by
N0067 `TPARSE` into seconds past J2000 using proleptic-Gregorian calendar
arithmetic. It is a raw kernel constant:

- no UTC, TT or TDB conversion is performed;
- no leap-second kernel is required or consulted, so pre-1972 dates are
  ordinary values rather than `HistoricalUtcRequiresExplicitModel`;
- calendar and clock overflow follow native normalization. Year tokens in
  `0..=99` expand over the default 1969--2068 interval, and token
  precedence is that of N0067 `TPARTV`, so a `DD-MON-YY` reading must not
  be assumed (`@19-JAN-87` is 2019-03-28);
- mixed dash/slash separators, month-name prefixes, `JD`-marked Julian
  dates and the exercised ISO ordinal productions are supported, while
  standalone `@2000-060` is a native rejection. Complete TPARSE grammar
  conformance is not claimed; see
  [`reference/KERNEL_DATE_GRAMMAR.md`](reference/KERNEL_DATE_GRAMMAR.md);
- an unsupported literal returns `KernelDateUnsupported`, never a
  missing-kernel error and never a substitute epoch.

A leap-second kernel's `DELTET/DELTA_AT` dates are a different case: they
must name a real calendar day, so an impossible date makes the LSK
invalid (`LskParse`) instead of rolling forward. Do not reuse the
kernel-date roll-forward rule for leap-second data.

## Reproducibility and acceptance

Record the library/revision and every input file's source URL, SHA-256,
load order and supported interval. For LSK, distinguish the dates of
leap entries from an announcement or validity horizon: the latest entry
is not a guaranteed expiry date. The default table-driven future behavior
is an assumption conditional on the selected file, not a prediction of
future leap seconds. `NaifMetadata` records selected constants, entry
dates, source and policy; file hashes must be recorded by the integration
layer. Do not label a caller-supplied file name as verified provenance.

Use three separate acceptance checks:

1. Time semantics: scales, origins, units, positive/negative adjustments,
   coverage and invalid inputs.
2. Reference-engine agreement: CSPICE for NAIF behavior; ERFA for the
   explicitly selected scientific model. Same kernels and conventions are
   mandatory. Compare every output component and reject non-finite data.
3. Application error budget: geometry, frame model, observation data and
   search tolerance. Engine agreement does not establish physical truth.

`tools/validate_geocalc_time.py` checks the UTC-to-ET-to-SPK chain using
an isolated client, so Geocalc's developing checkout is not rewritten.
It compares equal-ET geometry separately from equal-UTC end-to-end results
and records the supplied kernel hashes.

## Primary references

- [NAIF time subsystem](https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/time.html)
- [Pinned ERFA dtdb](https://github.com/liberfa/erfa/blob/1a8044cde5b7763295d472a6443387239127c6c8/src/dtdb.c)
- [IERS Conventions chapter 5](https://iers-conventions.obspm.fr/content/chapter5/icc5.pdf)
- [BIPM time metrology and TT realizations](https://www.bipm.org/en/time-metrology)
