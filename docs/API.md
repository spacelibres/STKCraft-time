# `stkcraft-time` API guide

This guide shows how to call the public API of `stkcraft-time` 0.1.16. It
is task-oriented: each section answers "which function do I call, and what
data do I need to load first?" For item-level reference documentation,
run:

```sh
cargo doc --open
```

A runnable version of the main examples is
[`examples/api_tour.rs`](../examples/api_tour.rs):

```sh
cargo run --example api_tour
```

Code snippets assume a function returning `Result<_, TimeError>` and that
values introduced in earlier snippets (`lsk`, `eop`, `utc`, ...) are in
scope unless the snippet redefines them.

## 1. Scope and features

`stkcraft-time` is a `no_std`-capable library. It never performs network
I/O and does not bundle mission data. The caller supplies the data files
(leap seconds, spacecraft clock, EOP) at run time.

| Cargo feature | Default | Effect |
|---|---|---|
| `std` | yes | Enables `impl std::error::Error for TimeError` and the CLI binaries. |
| `mcp` | no | Builds the `stkcraft-mcp` stdio MCP server; implies `std` and `serde_json`. |
| *(none)* | — | `no_std` + `alloc`; the library remains available, binaries are skipped. |

```toml
[dependencies]
stkcraft-time = "0.1.16"

# no_std + alloc:
# stkcraft-time = { version = "0.1.16", default-features = false }

# or pin the repository:
# stkcraft-time = { git = "https://github.com/spacelibres/STKCraft-time", tag = "v0.1.16" }
```

The internal epoch representation is integer seconds plus nanoseconds;
no floating-point accumulation is used for epoch or duration arithmetic.

## 2. What a SPICE kernel is

In the JPL WebGeocalc Time Conversion UI, **Kernel selection** chooses the
SPICE data files for the session. A SPICE *kernel* is a data file, not an
operating-system kernel. Only a subset participates in time conversion:

| WebGeocalc item | Kernel type | Purpose | `stkcraft-time` API |
|---|---|---|---|
| Latest leap-seconds kernel | LSK (`KPL/LSK`) | UTC–TAI leap seconds, `DELTET` constants | `LeapSecondTable::builtin` / `LeapSecondTable::from_lsk` |
| Spacecraft clock | SCLK (`KPL/SCLK`) | Mission clock ticks/strings ↔ ET/TAI | `SclkKernel::parse` |
| IERS EOP (not shown on that screen) | `finals2000A.all` | `UT1 - UTC` | `EopTable::from_finals2000a*` |
| Solar-system kernels | SPK/PCK | Trajectories and planetary constants | Not used for time conversion |
| Ground-station kernel set | Station kernels | Station locations | Not used for time conversion |
| `SPK/MK` | MK (meta-kernel) | A manifest listing other kernels | Not loaded; it is only a file list |

So the statement "each time system has its own kernel" is not quite right:
UTC/TT/TDB/GPS share the **LSK**, and only the spacecraft-clock encoding
needs a mission-specific **SCLK**. SPK/PCK/FK/IK contain geometry and
trajectory data; a pure time conversion neither loads nor needs them.
UT1 is the exception among time scales: it needs an independent **EOP**
table because it measures the actual rotation of the Earth.

## 3. Quick start

```rust
use stkcraft_time::convert::{et_to_utc, utc_to_et};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

fn main() -> Result<(), stkcraft_time::TimeError> {
    // Bundled snapshot of naif0012.tls; use from_lsk in production.
    let lsk = LeapSecondTable::builtin()?;

    let utc = UtcDateTime::new(2000, 1, 1, 12, 0, 0.0);
    let et = utc_to_et(&utc, Calendar::Gregorian, &lsk)?;
    let back = et_to_utc(et, Calendar::Gregorian, &lsk)?;

    assert_eq!(stkcraft_time::format::iso8601(&back, 3), "2000-01-01T12:00:00.000Z");
    Ok(())
}
```

## 4. Loading and selecting a leap-second kernel (LSK)

### 4.1 Bundled snapshot vs explicit kernel

```rust
use stkcraft_time::{LeapSecondTable, TimeError};

fn load_lsk(path: &str) -> Result<LeapSecondTable, TimeError> {
    let text = std::fs::read_to_string(path).map_err(|_| TimeError::LskParse)?;
    LeapSecondTable::from_lsk(&text)
}
```

- `LeapSecondTable::builtin()` parses the bundled `src/naif0012.tls`.
  Its last entry is `2017-01-01`, so it is a convenience snapshot, not a
  privileged source of truth.
- `LeapSecondTable::from_lsk(text)` parses the `DELTET/*` variables of any
  complete `KPL/LSK` kernel. This is the production entry point. Missing
  `DELTA_T_A`, `K`, `EB`, `M` or `DELTA_AT` returns `LskMissing`; no nominal
  constant is silently substituted for the caller's data.
- `stkcraft_time::lsk::parse_lsk(text)` is the module-level function behind
  `from_lsk`; both are public.

### 4.2 Coverage and kernel selection

```rust
use stkcraft_time::calendar::gregorian_to_jdn;
use stkcraft_time::LeapSecondTable;

let lsk = LeapSecondTable::builtin()?;

let (first_jdn, last_jdn) = lsk.coverage();
println!("kernel covers JDN {first_jdn}..={last_jdn}");

let jdn = gregorian_to_jdn(2017, 1, 1);
assert_eq!(lsk.delta_at_for_jdn(jdn), Some(37));
```

Every conversion is a pure function of the supplied kernel. Two kernels
that disagree about a leap second give different results after the first
disagreement, exactly as selecting a different kernel in WebGeocalc does.
UTC dates before the first active entry return `TimeError::UtcOutOfRange`;
dates after the last entry keep the last `TAI - UTC` value.

### 4.3 Policies and the pre-1972 source

```rust
use stkcraft_time::calendar::gregorian_to_jdn;
use stkcraft_time::{
    LeapSecondPolicy, LeapSecondSource, LeapSecondTable,
};

let text = std::fs::read_to_string("naif0012.tls")?;
let lsk = LeapSecondTable::from_lsk(&text)?
    .with_source(LeapSecondSource::Sofa)
    .with_policy(LeapSecondPolicy::FrozenAfter {
        jdn: gregorian_to_jdn(2035, 1, 1),
    });
```

- `LeapSecondSource::IersOnly` (default) matches the NAIF LSK and handles
  integer leap seconds from 1972 onward. UTC before 1972 is rejected.
- `LeapSecondSource::Sofa` adds the IAU SOFA `iauDat` drift model for
  1960–1972 and returns fractional `TAI - UTC`.
- `LeapSecondPolicy::TableDriven` (default) keeps the last table entry in
  force forever.
- `LeapSecondPolicy::FrozenAfter { jdn }` ignores entries after the chosen
  date. `LeapSecondPolicy::frozen_after(year, month, day)` is a
  convenience constructor. This is an explicit caller assumption; the
  default path never infers a future policy from a date.

### 4.4 Building a table from entries

```rust
use stkcraft_time::calendar::gregorian_to_jdn;
use stkcraft_time::{LeapEntry, LeapSecondTable};

let lsk = LeapSecondTable::new(vec![
    LeapEntry { jdn: gregorian_to_jdn(1972, 1, 1), delta_at: 10 },
    LeapEntry { jdn: gregorian_to_jdn(2017, 1, 1), delta_at: 37 },
])?;
```

`LeapSecondTable::new` sorts entries by JDN, rejects duplicate effective
dates, and rejects a table whose TAI start instants are not strictly
increasing. `with_constants(delta_t_a, k, eb, m0, m1)` replaces the
`DELTET` constants when they come from an external source.

## 5. UTC and leap seconds

```rust
use stkcraft_time::convert::{tai_to_utc, utc_to_tai};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

let lsk = LeapSecondTable::builtin()?;

// Positive leap second: legal only at the end of a 86401-second day.
let leap = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
let tai = utc_to_tai(&leap, Calendar::Gregorian, &lsk)?;
let back = tai_to_utc(tai, Calendar::Gregorian, &lsk)?;
assert!((back.second - 60.5).abs() < 1e-9);
```

Boundary behaviour:

| Input | Result |
|---|---|
| `23:59:60.x` on a positive-leap-second day | Accepted; the day has 86401 seconds. |
| `23:59:60.x` on any other day | `TimeError::InvalidLeapSecond` |
| `23:59:59.x` on a shortened (86399-second) day | `TimeError::InvalidLeapSecond` |
| `second` outside `0.0..61.0` | `TimeError::InvalidSecond` |
| Invalid month/day | `TimeError::InvalidDate` |
| `hour > 23` or `minute > 59` | `TimeError::InvalidTime` |
| UTC before the first active LSK entry | `TimeError::UtcOutOfRange` |

A negative leap second removes `23:59:59` from the preceding day. The
conversion rejects that label rather than aliasing it with the following
day's `00:00:00`.

## 6. Time-scale conversions

| From → To | Function | Requires |
|---|---|---|
| UTC → TAI | `convert::utc_to_tai` | LSK |
| TAI → UTC | `convert::tai_to_utc` | LSK |
| UTC → ET | `convert::utc_to_et` | LSK |
| ET → UTC | `convert::et_to_utc` | LSK |
| TAI → TT | `convert::tai_to_tt` | LSK |
| TT → TAI | `convert::tt_to_tai` | LSK |
| TAI → ET | `convert::tai_to_et` | LSK |
| ET → TAI | `convert::et_to_tai` | LSK |
| TAI → GPS | `convert::tai_to_gps` | — |
| GPS → TAI | `convert::gps_to_tai` | — |
| TAI → UT1 | `convert::tai_to_ut1` | LSK + EOP |
| UT1 → TAI | `convert::ut1_to_tai` | LSK + EOP |
| UTC → UT1 | `convert::utc_to_ut1` | LSK + EOP |
| UT1 → UTC | `convert::ut1_to_utc` | LSK + EOP |
| TAI → TCG | `coord::tai_to_tcg` | LSK |
| TCG → TAI | `coord::tcg_to_tai` | LSK |
| TAI → TCB | `coord::tai_to_tcb` | LSK |
| TCB → TAI | `coord::tcb_to_tai` | LSK |
| TAI → ESA TDB | `tdb::tai_to_tdb` | LSK |
| TAI → IAU TDB | `dtdb::tai_to_tdb_iau` | LSK |

The common core:

```rust
use stkcraft_time::convert::{
    et_to_tai, gps_to_tai, tai_to_et, tai_to_gps, tai_to_tt, tt_to_tai,
    utc_to_tai,
};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

let lsk = LeapSecondTable::builtin()?;
let utc = UtcDateTime::new(2016, 12, 31, 12, 0, 0.0);
let tai = utc_to_tai(&utc, Calendar::Gregorian, &lsk)?;

let tt = tai_to_tt(tai, &lsk);          // TAI + 32.184 s
let et = tai_to_et(tai, &lsk);          // NAIF ET, TDB s past J2000
let gps = tai_to_gps(tai);              // GPS s past 1980-01-06

let tai_again = tt_to_tai(tt, &lsk);
let tai_from_et = et_to_tai(et, &lsk);
let tai_from_gps = gps_to_tai(gps);
```

`GPS` is a continuous scale defined as `TAI - 19 s`; GPS↔TAI itself needs
no leap-second table. Conversions that involve UTC still need the LSK.

### 6.1 Which TDB model should I use?

| API | Model | Use when |
|---|---|---|
| `convert::tai_to_et` / `et_to_tai` | NAIF one-term `K*sin(E)` | You need NAIF/SPICE/CSPICE compatibility. |
| `tdb::tai_to_tdb` / `tdb_to_tai` | NASA HEASARC compact two-term | You need this approximate relation; not a guaranteed improvement over NAIF. |
| `dtdb::tai_to_tdb_iau` / `tdb_iau_to_tai` | IAU Fairhead–Bretagnon 787-term | You need IAU/SOFA `dtdb` compatibility. |

The models are not interchangeable. `ET` is an `f64` by definition.
The compact model follows [NASA HEASARC](https://heasarc.gsfc.nasa.gov/docs/xte/abc/time_tutorial.html);
its maximum residual against geocentric ERFA on a daily 1950-2050 grid
is about 36.5 us. The IAU model is geocentric; observer-dependent
topocentric terms and spacecraft proper-time transformations are not
implemented. Nanosecond model accuracy does not imply nanosecond
resolution for an absolute `f64` seconds value.

## 7. UT1 and IERS Earth Orientation Parameters

UT1 cannot be derived from the LSK. It needs an IERS `finals2000A.all`
table that measures `UT1 - UTC`. This API interpolates daily EOP values.
It omits the subdaily ocean-tide and libration terms required by
[IERS Conventions section 5.5.3.1](https://iers-conventions.obspm.fr/content/chapter5/icc5.pdf)
for high-accuracy instantaneous Earth orientation, and does not propagate
measurement/prediction uncertainties. `observed_only()` filters predictions,
not measurement error.

### 7.1 Parsing an EOP file

```rust
use stkcraft_time::eop::{EopSeries, EopTable};

let text = std::fs::read_to_string("finals2000A.all")
    .map_err(|_| stkcraft_time::TimeError::EopParse)?;

// Bulletin A (the default).
let eop = EopTable::from_finals2000a(&text)?;

// Bulletin B where present, otherwise Bulletin A, matching astropy's
// combined column.
let eop = EopTable::from_finals2000a_series(&text, EopSeries::FinalPreferred)?;

// Restrict the supported interval to measured values.
let observed = eop.observed_only()?;
let (first_mjd, last_mjd) = observed.range();
```

`EopTable::new(samples)` is available when you already have samples:

```rust
use stkcraft_time::eop::{EopSample, EopTable};

let eop = EopTable::new(vec![
    EopSample { mjd: 57_753.0, ut1_minus_utc: -0.4089, observed: true },
    EopSample { mjd: 57_754.0, ut1_minus_utc: 0.5911, observed: true },
    EopSample { mjd: 57_755.0, ut1_minus_utc: 0.5907, observed: true },
])?;
```

`new` requires a non-empty table and rejects non-finite `mjd` or
`ut1_minus_utc` values with `TimeError::EopParse`.

### 7.2 Raw interpolation vs leap-aware UT1

```rust
use stkcraft_time::convert::{tai_to_ut1, ut1_minus_utc, utc_to_tai, utc_to_ut1};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

// `eop` is the EopTable loaded in section 7.1.
let lsk = LeapSecondTable::builtin()?;
let utc = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
let tai = utc_to_tai(&utc, Calendar::Gregorian, &lsk)?;

// Raw table interpolation. It does not remove the leap-second step.
let raw = eop.ut1_minus_utc(57_753.5)?;

// Leap-aware: each EOP sample is converted to continuous UT1-TAI
// before interpolation, so a missing day across a leap second does not
// smear the step.
let offset = ut1_minus_utc(tai, &lsk, &eop)?;

// Continuous UT1 seconds past 2000-01-01T00:00:00 UT1.
let ut1 = tai_to_ut1(tai, &lsk, &eop)?;
let ut1_from_utc = utc_to_ut1(&utc, Calendar::Gregorian, &lsk, &eop)?;
```

Use `convert::ut1_minus_utc` for epochs on the UT1 scale.
Use `EopTable::ut1_minus_utc` only for table inspection or when the
leap-second step is irrelevant.

The leap-aware conversion interpolates `UT1 - TAI`, not raw `UT1 - UTC`.
The EOP table must contain a sample on or before the epoch's UTC day and a
later sample; the last sample day is not usable as an epoch boundary.
Missing days inside the table are handled as a continuous gap, without
reintroducing a leap-second step.

## 8. Spacecraft clock kernels (SCLK)

Only type-1 clocks are supported. No SCLK is bundled; load the mission
kernel you need.

```rust
use stkcraft_time::sclk::{SclkKernel, SclkTimeSystem};
use stkcraft_time::LeapSecondTable;

let lsk = LeapSecondTable::builtin()?;

const SYNTH_SCLK: &str = r#"
KPL/SCLK

\begindata

SCLK_DATA_TYPE_999       = ( 1 )
SCLK01_TIME_SYSTEM_999   = ( 1 )
SCLK01_N_FIELDS_999      = ( 2 )
SCLK01_MODULI_999        = ( 1000 100 )
SCLK01_OFFSETS_999       = ( 0 0 )
SCLK01_OUTPUT_DELIM_999  = ( 2 )
SCLK01_COEFFICIENTS_999  = (
    0.0000000000000E+00     0.0000000000000E+00     1.0000000000000E+00
    1.0000000000000E+05     1.0000000000000E+03     1.0000000000000E+00
)

\begintext
"#;

let sclk = SclkKernel::parse(SYNTH_SCLK, -999)?;
assert_eq!(sclk.time_system(), SclkTimeSystem::Tdb);
assert_eq!(sclk.n_fields(), 2);

let fields = sclk.parse_string("123:45")?;            // [123, 45]
let ticks = sclk.ticks_from_fields(&fields)?;         // 12345.0
let same_fields = sclk.fields_from_ticks(ticks)?;     // [123, 45]
let clock_string = sclk.format_string(&same_fields)?; // "123:45"

let et = sclk.ticks_to_et(ticks, &lsk)?;
let ticks_again = sclk.et_to_ticks(et, &lsk)?;

let tai = sclk.ticks_to_tai(ticks, &lsk)?;
let ticks_from_tai = sclk.tai_to_ticks(tai, &lsk)?;
```

- `spacecraft_id` may be positive or negative; kernel variables are looked
  up with the magnitude (`-41` and `41` both find the `_41` variables).
- `SCLK01_OFFSETS` gives the first displayed value of each field. The
  encoded counter is `field - offset`; decoding adds the offset back. For
  a Voyager-like third field with offset 1, the displayed field
  `[0, 0, 1]` is encoded tick 0, not tick 2.
- `SclkTimeSystem::Tdb` coefficients are ET seconds past J2000 directly.
  `SclkTimeSystem::Tdt` coefficients are TT seconds past J2000 and use the
  LSK for the TT↔ET step.
- `parallel_seconds(ticks)` and `ticks_from_parallel(seconds)` stop at the
  parallel time system; `ticks_to_et`/`et_to_ticks` and
  `ticks_to_tai`/`tai_to_ticks` perform the scale conversion. Continuous
  (non-integral) ticks are valid for all of these conversions.
- `fields_from_ticks` and `ticks_from_fields` operate on discrete encoded
  ticks. `fields_from_ticks` rejects a value more than 1e-6 from an
  integer; callers that only need ET/TAI should use the tick conversion
  functions directly.
- Partition boundaries are exposed through `partitions()` and their
  cumulative duration bounds encoded tick/time conversion. When the kernel
  has more than one partition or a nonzero partition start, field and
  string conversion returns
  `TimeError::SclkPartitionsUnsupported`; encoded ticks↔ET/TAI remain
  available within the encoded domain. Each partition endpoint is rounded
  to integral ticks before its duration is accumulated. Inputs before the
  first coefficient or beyond the domain are rejected. A standalone map
  with no partition metadata has no declared upper bound.
- Only the bounds inside a partition are validated (`start <= end`, both
  finite); the cumulative duration must also be finite. Cross-partition
  ordering is not required: NAIF lets a clock jump ahead or regress,
  which is exactly why a new partition starts.

`DELTET/DELTA_AT` follows NAIF assignment rules: `=` replaces the
accumulated table and `+=` extends it, so a kernel that restates the
leap-second list in a later data block is valid.

## 9. Coordinate scales: TCG and TCB

```rust
use stkcraft_time::convert::utc_to_tai;
use stkcraft_time::coord::{tai_to_tcb, tai_to_tcg, tcb_to_tai, tcg_to_tai};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

let lsk = LeapSecondTable::builtin()?;
let utc = UtcDateTime::new(2016, 12, 31, 12, 0, 0.0);
let tai = utc_to_tai(&utc, Calendar::Gregorian, &lsk)?;

let tcg = tai_to_tcg(tai, &lsk);
let tcb = tai_to_tcb(tai, &lsk);

let tai_from_tcg = tcg_to_tai(tcg, &lsk);
let tai_from_tcb = tcb_to_tai(tcb, &lsk);
```

`tai_to_tcb` uses the IAU Fairhead–Bretagnon TDB model, not NAIF `ET`, so
it matches the IAU/SOFA definition of TCB. Both scales are `f64` seconds
near `5e8` at modern epochs; their spacing is about 60 ns.

## 10. Epochs, durations and time series

```rust
use stkcraft_time::{Duration, Epoch, TimeSeries};

let start = Epoch::from_parts(0, 0);
let step = Duration::from_parts(30, 0);
let end = start + Duration::from_parts(90, 0);

let inclusive: Vec<Epoch> = TimeSeries::inclusive(start, end, step).collect();
assert_eq!(inclusive.len(), 4);

let exclusive: Vec<Epoch> = TimeSeries::exclusive(start, end, step).collect();
assert_eq!(exclusive.len(), 3);
```

`Epoch` stores integer TAI seconds plus nanoseconds since
`2000-01-01T00:00:00 TAI`. `Duration` is exact and normalised. Use
`Epoch::checked_from_parts`, `checked_from_tai_seconds`, `checked_add`
and `checked_sub`, or the `Duration::checked_*` methods, for untrusted
input. The infallible constructors and operators panic on overflow or
non-finite input rather than wrapping or returning a fake zero.
`TimeSeries` yields nothing when `step` is zero or negative, and it uses
checked cursor advancement so the last representable epoch is delivered
before iteration ends.

## 11. Formatting

```rust
use stkcraft_time::calendar::Calendar;
use stkcraft_time::format::{
    etcal, iso8601, iso8601_with, rfc2822, rfc3339, spice_calendar, strftime,
    strftime_with, Format, FormatOptions, Formatter,
};
use stkcraft_time::{LeapSecondTable, UtcDateTime};

let utc = UtcDateTime::new(2000, 1, 1, 12, 0, 0.0);

assert_eq!(iso8601(&utc, 3), "2000-01-01T12:00:00.000Z");
assert_eq!(rfc3339(&utc, 9), "2000-01-01T12:00:00.000000000+00:00");
assert_eq!(rfc2822(&utc), "Sat, 01 Jan 2000 12:00:00 +0000");
assert_eq!(spice_calendar(&utc, 3), "2000 JAN 01 12:00:00.000");
assert_eq!(etcal(0.0, 3), "2000 JAN 01 12:00:00.000");

let pattern = strftime(&utc, "%Y-%m-%d %H:%M:%S.%f")?;
let format = Format::parse("%Y-%m-%dT%H:%M:%SZ")?;
let rendered = Formatter::new(&utc, &format).to_string();

// Leap-second and calendar aware formatting.
let lsk = LeapSecondTable::builtin()?;
let options = FormatOptions::new(3)
    .with_calendar(Calendar::Gregorian)
    .with_leap_seconds(&lsk);

let last_second = UtcDateTime::new(2016, 12, 31, 23, 59, 59.9999);
assert_eq!(
    iso8601_with(&last_second, &options),
    "2016-12-31T23:59:60.000Z"
);
assert_eq!(
    strftime_with(&last_second, "%Y-%m-%dT%H:%M:%S", &options)?,
    "2016-12-31T23:59:60"
);
```

Precision is clamped to 9 fractional digits. Rounding is carry-aware:
`59.9999` with precision 3 becomes the next minute, and `60.9999` on a
leap second becomes the next day. `etcal` splits ET into whole days and
seconds before formatting, so microsecond values survive. A non-finite
ET produces `"INVALID ET"`.

Whether a carry crosses a leap second or moves to the next day depends on
the length of the day, so [`FormatOptions`] takes an optional
`LeapSecondTable`. With the table, `23:59:59.9999` on a 86 401-second day
formats as `23:59:60.000`; without it, a 86 400-second day is assumed,
unless the value already has `second >= 60`. Only the final minute of the
day is affected: every other minute still ends at `59.999`, so
`12:34:59.9999` on a leap-second day rounds to `12:35:00.000`. The same
options select the calendar used for the day carry, so a Julian date
rolls from `1500-02-28` to `1500-02-29` rather than to `1500-03-01`. `%j`
and `%a` use the selected calendar as well.

Supported `strftime` tokens: `%Y %y %m %d %e %H %M %S %j %a %A %b %B %z
%Z %f %%`. `%f` is nanoseconds; `%z` is `+0000`; `%Z` is `UTC`. An
unsupported token returns `TimeError::InvalidFormat`.

## 12. Error handling

`TimeError` is the single error type:

| Variant | Meaning |
|---|---|
| `InvalidDate { year, month, day }` | Date not valid for the selected calendar. |
| `InvalidTime { hour, minute }` | Time of day outside its legal range. |
| `InvalidSecond` | Seconds component outside `0.0..61.0`. |
| `InvalidLeapSecond` | `23:59:60.x` without a positive leap second, or `23:59:59.x` on a shortened day. |
| `UtcOutOfRange` | UTC epoch outside the supplied LSK coverage. |
| `Ut1OutOfRange` | Epoch or bracketing EOP sample outside the table. |
| `EopMissing` | Empty EOP table. |
| `EopParse` | EOP text or sample is invalid, including non-finite values. |
| `SclkParse` / `SclkMissing` / `SclkUnsupported` | SCLK text, missing variable, or non-type-1 clock. |
| `SclkPartitionsUnsupported` | Field/string conversion requested for a partitioned clock; encoded ticks↔ET remain usable. |
| `LskParse` / `LskMissing` | LSK text or required `DELTET` variable. |
| `InvalidFormat` | Unsupported `strftime` token. |
| `Overflow` | Reserved for checked arithmetic. |

With the default `std` feature, `TimeError` implements
`std::error::Error`.

## 13. Binaries and tools

The CLI binaries require the `std` feature:

| Binary | Purpose |
|---|---|
| `wgc-compare` | Compare crate results with recorded WebGeocalc values. |
| `ut1-check` | Check UT1 against an IERS EOP table. |
| `tdb-compare` | Compare NAIF and ESA TDB models. |
| `sclk-check` | Check SCLK conversions against CSPICE output. |
| `stkcraft-mcp` | Optional stdio MCP server (`--features mcp`); see `docs/MCP.md`. |

Data-fetching and validation scripts live in `tools/`; the crate itself
never performs network I/O. No EOP or mission SCLK data is bundled.

Every download is configurable, so a mirror or a local copy can be used
when the upstream host is slow or blocked:

| Script | Flag | Environment variable | Default |
|---|---|---|---|
| `tools/fetch_eop.py` | `--url` | `STKCRAFT_EOP_URL` | IERS `finals2000A.all` |
| `tools/validate_ut1.py` | `--url` | `STKCRAFT_EOP_URL` | IERS `finals2000A.all` |
| `tools/gen_dtdb_table.py` | `--url` / `--src` | `STKCRAFT_DTDB_URL` | pinned ERFA `dtdb.c` |

`--url` wins over the environment variable, and `--eop` / `--src` skip
the download when they point at an existing file. Downloaded data is
validated on arrival: the EOP file is parsed and summarized, and the
ERFA source is checked against its pinned SHA-256, so a mirror cannot
change the result silently.

## 14. Public API index

### `calendar`

```rust
pub const J2000_JDN: i64;
pub const GPS_EPOCH_JDN: i64;
pub const GREGORIAN_SWITCH_JDN: i64;

pub enum Calendar { Gregorian, Julian, Mixed }

pub fn is_gregorian_leap(year: i32) -> bool;
pub fn is_julian_leap(year: i32) -> bool;
pub fn days_in_month(year: i32, month: u8, cal: Calendar) -> Option<u8>;
pub fn gregorian_to_jdn(year: i32, month: u8, day: u8) -> i64;
pub fn julian_to_jdn(year: i32, month: u8, day: u8) -> i64;
pub fn jdn_to_gregorian(jdn: i64) -> (i32, u8, u8);
pub fn jdn_to_julian(jdn: i64) -> (i32, u8, u8);
pub fn jdn_to_calendar(jdn: i64, cal: Calendar) -> (i32, u8, u8);
pub fn calendar_to_jdn(year: i32, month: u8, day: u8, cal: Calendar)
    -> Result<i64, TimeError>;
pub fn calendar_to_jd(
    year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64,
    cal: Calendar,
) -> Result<f64, TimeError>;
pub fn jd_to_mjd(jd: f64) -> f64;
pub fn mjd_to_jd(mjd: f64) -> f64;
```

### `epoch`

```rust
pub struct Epoch { /* private fields */ }

impl Epoch {
    pub fn from_parts(sec: i64, nanos: i64) -> Self;
    pub fn checked_from_parts(sec: i64, nanos: i64) -> Option<Self>;
    pub fn from_tai_seconds(seconds: f64) -> Self;
    pub fn checked_from_tai_seconds(seconds: f64) -> Option<Self>;
    pub fn tai_seconds(&self) -> f64;
    pub fn sec(&self) -> i64;
    pub fn nanos(&self) -> u32;
    pub fn checked_add(self, rhs: Duration) -> Option<Self>;
    pub fn checked_sub(self, rhs: Duration) -> Option<Self>;
}

pub struct UtcDateTime {
    pub year: i32,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: f64,
}

impl UtcDateTime {
    pub fn new(year: i32, month: u8, day: u8, hour: u8, minute: u8, second: f64)
        -> Self;
}
```

`Epoch` supports `Add<Duration>`, `Sub<Duration>` and `Sub<Epoch> ->
Duration`.

### `duration`

```rust
pub struct Duration { /* private fields */ }

impl Duration {
    pub const ZERO: Self;
    pub const NANOSECOND: Self;
    pub const SECOND: Self;

    pub fn from_parts(sec: i64, nanos: i64) -> Self;
    pub fn checked_from_parts(sec: i64, nanos: i64) -> Option<Self>;
    pub fn from_nanoseconds(ns: i128) -> Option<Self>;
    pub fn from_seconds_f64(seconds: f64) -> Self;
    pub fn checked_from_seconds_f64(seconds: f64) -> Option<Self>;
    pub fn sec(&self) -> i64;
    pub fn nanos(&self) -> i32;
    pub fn total_nanoseconds(&self) -> i128;
    pub fn seconds_f64(&self) -> f64;
    pub fn is_zero(&self) -> bool;
    pub fn signum(&self) -> i8;
    pub fn abs(&self) -> Self;
    pub fn checked_neg(&self) -> Option<Self>;
    pub fn checked_add(&self, other: Self) -> Option<Self>;
    pub fn checked_sub(&self, other: Self) -> Option<Self>;
    pub fn checked_mul_i64(&self, rhs: i64) -> Option<Self>;
    pub fn floor(&self, step: Self) -> Self;
    pub fn ceil(&self, step: Self) -> Self;
    pub fn round(&self, step: Self) -> Self;
}
```

`Duration` supports `Add`, `Sub`, `Neg`, `Mul<i64>`, `Div<i64>` and
`Display`.

### `leap`

```rust
pub const DELTA_T_A: f64;
pub const K: f64;
pub const EB: f64;
pub const M0: f64;
pub const M1: f64;
pub const JDN_1960: i64;
pub const JDN_1972: i64;

pub struct LeapEntry {
    pub jdn: i64,
    pub delta_at: i32,
}

pub enum LeapSecondSource { IersOnly, Sofa }

pub enum LeapSecondPolicy {
    TableDriven,
    FrozenAfter { jdn: i64 },
}

impl LeapSecondPolicy {
    pub fn frozen_after(year: i32, month: u8, day: u8) -> Self;
}

pub struct LeapSecondTable {
    pub delta_t_a: f64,
    pub k: f64,
    pub eb: f64,
    pub m0: f64,
    pub m1: f64,
}

impl LeapSecondTable {
    pub fn new(entries: Vec<LeapEntry>) -> Result<Self, TimeError>;
    pub fn with_constants(self, delta_t_a: f64, k: f64, eb: f64, m0: f64, m1: f64)
        -> Self;
    pub fn builtin() -> Result<Self, TimeError>;
    pub fn from_lsk(text: &str) -> Result<Self, TimeError>;
    pub fn with_source(self, source: LeapSecondSource) -> Self;
    pub fn source(&self) -> LeapSecondSource;
    pub fn with_policy(self, policy: LeapSecondPolicy) -> Self;
    pub fn policy(&self) -> LeapSecondPolicy;
    pub fn delta_at_f64(&self, jdn: i64, day_fraction: f64) -> Option<f64>;
    pub fn entries(&self) -> &[LeapEntry];
    pub fn coverage(&self) -> (i64, i64);
    pub fn delta_at_for_jdn(&self, jdn: i64) -> Option<i32>;
}

pub fn sofa_delta_at(mjd: f64) -> Option<f64>;
```

`lsk::parse_lsk(text)` and `lsk::builtin()` are equivalent module-level
entry points.

### `convert`

```rust
pub fn utc_to_tai(dt: &UtcDateTime, calendar: Calendar, lsk: &LeapSecondTable)
    -> Result<Epoch, TimeError>;
pub fn tai_to_utc(epoch: Epoch, calendar: Calendar, lsk: &LeapSecondTable)
    -> Result<UtcDateTime, TimeError>;

pub fn tai_to_et(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn et_to_tai(et: f64, lsk: &LeapSecondTable) -> Epoch;
pub fn tai_to_tt(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tt_to_tai(tt: f64, lsk: &LeapSecondTable) -> Epoch;
pub fn tai_to_gps(epoch: Epoch) -> f64;
pub fn gps_to_tai(gps: f64) -> Epoch;

pub fn utc_to_et(dt: &UtcDateTime, calendar: Calendar, lsk: &LeapSecondTable)
    -> Result<f64, TimeError>;
pub fn et_to_utc(et: f64, calendar: Calendar, lsk: &LeapSecondTable)
    -> Result<UtcDateTime, TimeError>;

pub fn ut1_minus_utc(epoch: Epoch, lsk: &LeapSecondTable, eop: &EopTable)
    -> Result<f64, TimeError>;
pub fn tai_to_ut1(epoch: Epoch, lsk: &LeapSecondTable, eop: &EopTable)
    -> Result<f64, TimeError>;
pub fn ut1_to_tai(ut1: f64, lsk: &LeapSecondTable, eop: &EopTable)
    -> Result<Epoch, TimeError>;
pub fn utc_to_ut1(
    dt: &UtcDateTime, calendar: Calendar, lsk: &LeapSecondTable, eop: &EopTable,
) -> Result<f64, TimeError>;
pub fn ut1_to_utc(
    ut1: f64, calendar: Calendar, lsk: &LeapSecondTable, eop: &EopTable,
) -> Result<UtcDateTime, TimeError>;
```

### `eop`

```rust
pub struct EopSample {
    pub mjd: f64,
    pub ut1_minus_utc: f64,
    pub observed: bool,
}

pub enum EopSeries { BulletinA, BulletinB, FinalPreferred }

pub struct EopTable { /* private fields */ }

impl EopTable {
    pub fn new(samples: Vec<EopSample>) -> Result<Self, TimeError>;
    pub fn from_finals2000a(text: &str) -> Result<Self, TimeError>;
    pub fn from_finals2000a_series(text: &str, series: EopSeries)
        -> Result<Self, TimeError>;
    pub fn observed_only(&self) -> Result<Self, TimeError>;
    pub fn ut1_minus_utc(&self, mjd: f64) -> Result<f64, TimeError>;
    pub fn range(&self) -> (f64, f64);
    pub fn samples(&self) -> &[EopSample];
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### `sclk`

```rust
pub enum SclkTimeSystem { Tdb, Tdt }

pub struct SclkCoefficient {
    pub ticks: f64,
    pub seconds: f64,
    pub rate: f64,
}

pub struct SclkKernel { /* private fields */ }

impl SclkKernel {
    pub fn parse(text: &str, spacecraft_id: i32) -> Result<Self, TimeError>;
    pub fn spacecraft_id(&self) -> i32;
    pub fn data_type(&self) -> i32;
    pub fn time_system(&self) -> SclkTimeSystem;
    pub fn n_fields(&self) -> usize;
    pub fn moduli(&self) -> &[i64];
    pub fn offsets(&self) -> &[i64];
    pub fn coefficients(&self) -> &[SclkCoefficient];
    pub fn partitions(&self) -> (&[f64], &[f64]);
    pub fn ticks_per_msb(&self) -> f64;
    pub fn ticks_from_fields(&self, fields: &[i64]) -> Result<f64, TimeError>;
    pub fn fields_from_ticks(&self, ticks: f64) -> Result<Vec<i64>, TimeError>;
    pub fn parallel_seconds(&self, ticks: f64) -> Result<f64, TimeError>;
    pub fn ticks_from_parallel(&self, seconds: f64) -> Result<f64, TimeError>;
    pub fn ticks_to_et(&self, ticks: f64, lsk: &LeapSecondTable)
        -> Result<f64, TimeError>;
    pub fn et_to_ticks(&self, et: f64, lsk: &LeapSecondTable)
        -> Result<f64, TimeError>;
    pub fn ticks_to_tai(&self, ticks: f64, lsk: &LeapSecondTable)
        -> Result<Epoch, TimeError>;
    pub fn tai_to_ticks(&self, epoch: Epoch, lsk: &LeapSecondTable)
        -> Result<f64, TimeError>;
    pub fn parse_string(&self, sclk: &str) -> Result<Vec<i64>, TimeError>;
    pub fn format_string(&self, fields: &[i64]) -> Result<String, TimeError>;
}
```

### `format`

```rust
pub fn iso8601(dt: &UtcDateTime, precision: usize) -> String;
pub fn iso8601_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String;
pub fn spice_calendar(dt: &UtcDateTime, precision: usize) -> String;
pub fn spice_calendar_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String;
pub fn etcal(et: f64, precision: usize) -> String;
pub fn jd_to_calendar(jd: f64) -> (i32, u8, u8, u8, u8, f64);
pub fn rfc3339(dt: &UtcDateTime, precision: usize) -> String;
pub fn rfc3339_with(dt: &UtcDateTime, options: &FormatOptions<'_>) -> String;
pub fn rfc2822(dt: &UtcDateTime) -> String;
pub fn strftime(dt: &UtcDateTime, pattern: &str) -> Result<String, TimeError>;
pub fn strftime_with(
    dt: &UtcDateTime,
    pattern: &str,
    options: &FormatOptions<'_>,
) -> Result<String, TimeError>;

pub struct FormatOptions<'a> { /* private fields */ }
impl<'a> FormatOptions<'a> {
    pub fn new(precision: usize) -> Self;
    pub fn with_calendar(self, calendar: Calendar) -> Self;
    pub fn with_leap_seconds(self, lsk: &'a LeapSecondTable) -> Self;
    pub fn precision(&self) -> usize;
    pub fn calendar(&self) -> Calendar;
}

pub struct Format { /* private fields */ }
impl Format {
    pub fn parse(pattern: &str) -> Result<Self, TimeError>;
}

pub struct Formatter<'a> { /* private fields */ }
impl<'a> Formatter<'a> {
    pub fn new(dt: &'a UtcDateTime, format: &'a Format) -> Self;
}
```

`Formatter` implements `core::fmt::Display`.

### `coord`

```rust
pub const L_G: f64;
pub const L_B: f64;
pub const TDB0: f64;

pub fn tcg_minus_tt(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tai_to_tcg(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tcg_to_tai(tcg: f64, lsk: &LeapSecondTable) -> Epoch;
pub fn tcb_minus_tdb(tdb: f64) -> f64;
pub fn tai_to_tcb(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tcb_to_tai(tcb: f64, lsk: &LeapSecondTable) -> Epoch;
```

### `tdb` and `dtdb`

```rust
pub fn tdb_minus_tt_seconds(jd_tt: f64) -> f64;
pub fn tai_to_tdb(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tdb_to_tai(tdb: f64, lsk: &LeapSecondTable) -> Epoch;

pub fn dtdb_minus_tt(jd_tt: f64) -> f64;
pub fn tai_to_tdb_iau(epoch: Epoch, lsk: &LeapSecondTable) -> f64;
pub fn tdb_iau_to_tai(tdb: f64, lsk: &LeapSecondTable) -> Epoch;
```

### `series` and `timescale`

```rust
pub struct TimeSeries { /* private fields */ }
impl TimeSeries {
    pub fn inclusive(start: Epoch, end: Epoch, step: Duration) -> Self;
    pub fn exclusive(start: Epoch, end: Epoch, step: Duration) -> Self;
}
impl Iterator for TimeSeries { type Item = Epoch; }

pub enum TimeScale {
    Utc, Tai, Tt, Et, Gps, Ut1, Tcg, Tcb,
}
```
