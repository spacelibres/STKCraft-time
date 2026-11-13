<p align="center">
  <img src="https://raw.githubusercontent.com/spacelibres/STKCraft_web/main/public/brand/albatross.png" alt="STKCraft — Albatross" width="128">
</p>

# STKCraft-time

**STKCraft project:** <https://www.stkcraft.com>
**Repository:** <https://github.com/spacelibres/STKCraft-time>

[![CI](https://github.com/spacelibres/STKCraft-time/actions/workflows/ci.yml/badge.svg)](https://github.com/spacelibres/STKCraft-time/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/stkcraft-time.svg)](https://crates.io/crates/stkcraft-time)
[![docs.rs](https://docs.rs/stkcraft-time/badge.svg)](https://docs.rs/stkcraft-time)
[![downloads](https://img.shields.io/crates/d/stkcraft-time.svg)](https://crates.io/crates/stkcraft-time)
[![GitHub release](https://img.shields.io/github/v/release/spacelibres/STKCraft-time)](https://github.com/spacelibres/STKCraft-time/releases)
[![API guide](https://img.shields.io/badge/docs-API%20guide-blue.svg)](https://github.com/spacelibres/STKCraft-time/blob/main/docs/API.md)
[![MCP](https://img.shields.io/badge/MCP-stdio%20server-purple.svg)](https://github.com/spacelibres/STKCraft-time/blob/main/docs/MCP.md)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/spacelibres/STKCraft-time/blob/main/LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue.svg)](https://github.com/spacelibres/STKCraft-time/blob/main/Cargo.toml)
[![no_std](https://img.shields.io/badge/no__std-supported-green.svg)](https://docs.rs/stkcraft-time/latest/stkcraft_time/)

A NAIF/SPICE-aligned time library written in Rust.

`stkcraft-time` implements the time-scale model documented by the NASA
Navigation and Ancillary Information Facility (NAIF) in the **SPICE Time
Subsystem Required Reading** and driven by a **Leapseconds Kernel (LSK)**.
It does not link CSPICE. Text-kernel decimal arithmetic follows NAIF's
NPARSD operation order; see [ATTRIBUTION.md](ATTRIBUTION.md) for provenance.

- **API guide:** [`docs/API.md`](https://github.com/spacelibres/STKCraft-time/blob/main/docs/API.md)
- **API reference:** <https://docs.rs/stkcraft-time>
- **Wiki:** <https://github.com/spacelibres/STKCraft-time/wiki>
- **MCP server:** [`docs/MCP.md`](https://github.com/spacelibres/STKCraft-time/blob/main/docs/MCP.md)
- **Runnable tour:** [`examples/api_tour.rs`](https://github.com/spacelibres/STKCraft-time/blob/main/examples/api_tour.rs)
- **Agent skills:** [`.agents/skills/`](https://github.com/spacelibres/STKCraft-time/tree/main/.agents/skills)
- **crates.io:** <https://crates.io/crates/stkcraft-time>
- **Reference:** `cargo doc --open`

**Install:** `cargo add stkcraft-time`, or add
`stkcraft-time = "0.2.1"` to `[dependencies]`. With `no_std`, use `default-features = false`.

## Project status

- Latest release: **v0.2.1** on GitHub Releases, including release-mode
  binaries for Linux, macOS (x86_64 and arm64) and Windows.
- Published on crates.io as **`stkcraft-time` 0.2.1**, with API docs on
  [docs.rs](https://docs.rs/stkcraft-time).
- Minimum supported Rust version: **1.75**.
- The library is `no_std` compatible with `alloc`; the optional MCP server
  requires `std`.

## Installation

```sh
cargo add stkcraft-time
```

```toml
[dependencies]
stkcraft-time = "0.2.1"
```

`no_std` plus `alloc`:

```toml
stkcraft-time = { version = "0.2.1", default-features = false }
```

The MCP server and the validation binaries are optional:

```sh
cargo install stkcraft-time --features mcp --bin stkcraft-mcp
```

To track the repository instead of a crates.io release, pin a tag:

```toml
stkcraft-time = { git = "https://github.com/spacelibres/STKCraft-time", tag = "v0.2.1" }
```

## Releases

Every `vX.Y.Z` tag publishes a GitHub Release with optimized binaries
built by `cargo build --release --features mcp --bins`:

| Platform | Archive |
|---|---|
| Linux x86_64 | `stkcraft-time-vX.Y.Z-linux-x86_64.tar.gz` |
| macOS x86_64 | `stkcraft-time-vX.Y.Z-macos-x86_64.tar.gz` |
| macOS arm64 | `stkcraft-time-vX.Y.Z-macos-aarch64.tar.gz` |
| Windows x86_64 | `stkcraft-time-vX.Y.Z-windows-x86_64.zip` |

Each archive contains `stkcraft-mcp`, `sclk-check`, `ut1-check`,
`tdb-compare`, `wgc-compare`, `README.md`, `LICENSE` and
`CHANGELOG.md`. `SHA256SUMS.txt` is attached separately. Release notes
are in
[`docs/releases/`](https://github.com/spacelibres/STKCraft-time/tree/main/docs/releases).

## Design rules

1. **The LSK is the only source of truth.**
   `DELTET/DELTA_T_A`, `DELTET/K`, `DELTET/EB`, `DELTET/M` and
   `DELTET/DELTA_AT` are read from the kernel. No leap-second value is
   hard-coded in the conversion logic.

2. **TAI is the internal canonical scale.**
   `Epoch` stores an integer count of TAI seconds plus nanoseconds since
   `2000-01-01T00:00:00 TAI`. UTC is a civil *representation*, not an
   internal time scale. This keeps the representation monotonic across
   leap seconds.

3. **UTC conversion uses the day-length table, not the `DeltaET` formula.**
   For every UTC day affected by a leap second, the day is 86400, 86401
   or 86399 seconds long. `23:59:60.x` is legal only on a 86401-second
   day; conversely `23:59:59.x` does not exist on a 86399-second day.
   The `TDB - UTC = 32.184 + DeltaAT + K*sin(E)` expression is *not*
   used to convert across a leap-second instant.

4. **Scales are distinct types.**
   UTC, TAI, TT, TDB/ET and GPS are never interchangeable. Conversion is
   only possible through explicit constructors.

5. **No future leap-second policy is inferred by default.**
   Leap seconds are data. The default `TableDriven` policy follows the
   supplied LSK and freezes `TAI - UTC` after its last entry. A caller
   that wants to model an explicit assumption can select
   `LeapSecondPolicy::FrozenAfter`; the crate never infers a policy from
   a date on its own.

## Kernel selection

Time conversion results depend on the leap-second kernel you load. This
mirrors the **Kernel selection** control in the JPL WebGeocalc Time
Conversion tool: the same UTC instant maps to a different `TAI - UTC`
after a leap second if two kernels disagree about that leap second.

- `LeapSecondTable::from_lsk(text)` loads an explicit `KPL/LSK` kernel.
  This is the recommended entry point in production. Every conversion is
  a pure function of that kernel.
- `LeapSecondTable::builtin()` is a snapshot of `naif0012.tls` bundled
  for convenience. Its last leap-second entry is `2017-01-01`. It is not
  a privileged source of truth.
- `LeapSecondTable::coverage()` returns the `(first, last)` effective
  dates the kernel actually covers.
- Civil UTC dates **before** the first kernel entry are rejected
  (`UtcOutOfRange`). The explicitly named native JDUTC path below selects
  N0067's separate historical convention.
- UTC dates **after** the last entry hold the last `TAI - UTC` value.
  This is NAIF's documented behaviour, not data invented here: a kernel
  cannot know about a leap second announced after it was released.
  Update the kernel to extend trustworthy coverage.

Two kernels that differ in `DELTET/DELTA_AT` therefore give different
results; `tests/kernel_selection.rs` asserts this explicitly.

### Kernel types in WebGeocalc and in this crate

In WebGeocalc's **Kernel selection**, a "kernel" is a SPICE data file,
not an operating-system kernel. Only some of the loaded files participate
in time conversion:

| WebGeocalc item | Kernel | Purpose | `stkcraft-time` API |
|---|---|---|---|
| Latest leap-seconds kernel | LSK | UTC–TAI leap seconds, `DELTET` constants | `LeapSecondTable::builtin()` / `from_lsk()` |
| Spacecraft clock | SCLK | Mission clock ticks/strings ↔ ET/TAI | `SclkKernel::parse()` |
| IERS EOP (not shown on that screen) | `finals2000A.all` | `UT1 - UTC` | `EopTable::from_finals2000a*()` |
| Solar-system kernels | SPK/PCK | Trajectories and planetary constants | Not used for time conversion |
| Ground-station kernel set | Station kernels | Station locations | Not used for time conversion |
| `SPK/MK` | MK (meta-kernel) | A list of files to load | Not loaded; it is only a manifest |

UTC/TT/TDB/GPS conversions share the **LSK**; only the spacecraft-clock
encoding needs a mission-specific **SCLK**. UT1 is the exception: it needs
an independent **EOP** table because it measures the actual rotation of
the Earth. SPK/PCK/FK/IK contain geometry and trajectory data and are not
needed by a pure time conversion.

```rust
use stkcraft_time::convert::{tai_to_et, utc_to_tai};
use stkcraft_time::{Calendar, LeapSecondTable, UtcDateTime};

let text = std::fs::read_to_string("naif0012.tls").expect("read LSK");
let lsk = LeapSecondTable::from_lsk(&text).expect("parse LSK");

let utc = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
let tai = utc_to_tai(&utc, Calendar::Gregorian, &lsk).expect("valid UTC");
let et = tai_to_et(tai, &lsk);
```

For complete call examples, including SCLK and EOP/UT1, see
[`docs/API.md`](https://github.com/spacelibres/STKCraft-time/blob/main/docs/API.md) and the runnable
[`examples/api_tour.rs`](https://github.com/spacelibres/STKCraft-time/blob/main/examples/api_tour.rs).

### Kernel date literals

A text kernel can write an epoch as `@2000-JAN-01-12:00:00`. NAIF
resolves that literal with `TPARSE` into seconds past J2000, without any
UTC, TT or TDB conversion and without a leap-second kernel.
`stkcraft_time::kernel_date::parse_kernel_date` performs the same
resolution:

```rust
use stkcraft_time::kernel_date::parse_kernel_date;

let epoch = parse_kernel_date("@2001-FEB-29").expect("supported literal");
assert_eq!(epoch.seconds_past_j2000, 36_676_800.0);
assert_eq!((epoch.year, epoch.month, epoch.day), (2001, 3, 1));
```

A day past the end of its month rolls forward as NAIF does. Year selection
follows N0067 `TPARTV` token precedence, not an assumed `DD-MON-YY`
reading: an integer token of at least 1000 is a year, and a year token in
`0..=99` expands over the native 1969--2068 window. `@31-JAN-1987` is
1987-01-31, while `@19-JAN-87` is 2019-03-28: token `19` is the year and
token `87` is an out-of-range day that rolls forward, exactly as native
SPICE resolves it.

Mixed dash/slash separators, month-name prefixes of at least three
letters, normalized month/day/clock overflow, `JD`-marked Julian dates and
the exercised ISO ordinal productions are supported. Standalone
`@2000-060`, empty literals and unsupported grammar return
`KernelDateUnsupported`, never a missing-kernel error and never a
substitute epoch. See the [exact grammar scope](docs/reference/KERNEL_DATE_GRAMMAR.md)
for the 416-input native corpus and the residual limits; full TPARSE
grammar conformance is not claimed.

### Measured against the live WGC2 API

`tools/wgc2_kernel_sweep.py` selects individual LSK kernels through the
WGC2 `kernels` parameter (`{"type": "KERNEL", "path": "pds/wgc/kernels/lsk/naif0012.tls"}`)
and converts the same UTC epochs to TDB. Measured on 2026-11-10 against
WGC2 2.11.2 (build `5520 N67`):

| UTC | naif0012 ET | naif0011 ET | naif0010 ET |
|-----|-------------|-------------|-------------|
| `2017-06-01T00:00:00` | 549547269.184903 | 549547268.184903 | 549547267.184903 |
| `2015-06-01T00:00:00` | 486388867.184915 | 486388867.184915 | 486388867.184915 |
| `2016-12-31T23:59:60.5` | 536500868.68393 | 536500868.68393 | 536500867.68393 |
| `2024-01-01T00:00:00` | 757339269.183906 | 757339268.183906 | 757339267.183906 |

After the 2015-07-01 leap second `naif0011` is one second behind
`naif0012`, and `naif0010` is two seconds behind — exactly the leap
seconds each kernel lacks. This crate reproduces every comparable value
to WGC2's rounding (≤ 5e-7 s).

For `2016-12-31T23:59:60.5` under `naif0011`/`naif0010`, WGC2 still
returned a number, but those kernels do not declare that leap second.
This crate rejects the input with `InvalidLeapSecond`, which is SPICE's
documented treatment of a leap-second label missing from the kernel. The
result is not guessed.

The CSV is committed as `tests/golden/wgc2_kernel_selection.csv`, so the
check runs offline. Regenerate it with:

```bash
python tools/wgc2_kernel_sweep.py --insecure   # --insecure only if needed
```

## Implemented scales

| Scale | Definition used here |
|-------|----------------------|
| UTC | Civil representation with leap seconds |
| TAI | Continuous atomic time |
| TT | `TAI + 32.184 s` |
| ET (NAIF) | `TT + K*sin(E)`, `E = M + EB*sin(M)`, `M = M0 + M1*t` |
| TDB (compact) | `TT + 0.001658*sin(g) + 0.000014*sin(2g)` |
| GPS | `TAI - 19 s`, zero at `1980-01-06T00:00:00 UTC` |
| UT1 | Earth-rotation scale, `UTC + (UT1 - UTC)`; requires IERS EOP |
| TCG | `TT + L_G/(1-L_G)*(JD_TT - T0)*86400` (IAU coordinate scale) |
| TCB | `TDB + (L_B*(JD_TDB - T0)*86400 - TDB0)/(1-L_B)` (IAU coordinate scale) |

`ET` is **TDB seconds past the J2000 epoch** (`2000-01-01T12:00:00 TDB`),
matching the SPICE convention. Use `ET` for NAIF/SPICE interoperability,
`tdb` for the compact [NASA HEASARC approximation](https://heasarc.gsfc.nasa.gov/docs/xte/abc/time_tutorial.html),
and `dtdb` for the 787-term geocentric IAU/SOFA model. The compact model
is not a guaranteed improvement over NAIF: a daily 1950-2050 comparison
with ERFA has a maximum residual of about 36.5 us.

## Accuracy statement

Timekeeping does not have an absolute accuracy independent of a
reference. This crate targets the following verifiable properties:

- Internal storage is integer seconds plus nanoseconds. No floating-point
  accumulation is used for epoch arithmetic. The nanosecond field is a
  *representation* step, not a claim of nanosecond accuracy.
- UTC/TAI conversion matches the LSK for every leap-second boundary,
  including `23:59:60.x`.
- TT/TDB conversion follows NAIF's published model, whose stated accuracy
  is about **30 microseconds** (small-period terms are ignored).
- ET is an `f64` by definition, exactly as in SPICE; the `f64` boundary
  therefore bounds ET precision.

The only accuracy figures quoted here are either published by NAIF (about
30 µs for the TDB model) or measured against a reference engine (CSPICE).
No accuracy is asserted that the underlying data does not support.

Bit-for-bit equality with CSPICE is validated by 82 golden vectors
generated offline with `tools/gen_golden.py`. The current maximum
UTC-to-ET difference is **1.1e-12 s**. CSPICE is not a build or runtime
dependency.

### WGC2 API comparison

WGC2 exposes a REST API at
`https://wgc2.jpl.nasa.gov:8443/webgeocalc/api/` and performs time
conversion through `calculationType: "TIME_CONVERSION"`. A live request
for two representative epochs returned:

| UTC | WGC2 ET (s) | CSPICE / this crate ET (s) | Difference |
|-----|-------------|----------------------------|------------|
| `2016-12-31T23:59:60.5` | `536500868.68393` | `536500868.6839298` | `2.0e-7 s` |
| `2000-01-01T12:00:00` | `64.183927` | `64.18392728473108` | `2.8e-7 s` |

The differences are WGC2's output rounding. WGC2 is a CSPICE front-end;
the underlying model is identical. This crate agrees with CSPICE to
`1.1e-12 s`. It is therefore **numerically aligned with NASA's engine**,
not physically more accurate; its difference is an integer-nanosecond
internal representation and output formatting that prints more digits
than WebGeocalc's rounded values. More printed digits are not more
accuracy.

Reproduce the comparison with:

```bash
python tools/wgc2_compare.py            # TDB against WGC2, then UT1
python tools/wgc2_compare.py --insecure # only if TLS verification fails
```

The script produces one report with three sections:

1. **TDB conversion** — submits `TIME_CONVERSION` to WGC2, runs
   `cargo run --bin wgc-compare` for the same epochs, and compares both
   against WGC2 and against `tests/golden/utc_to_et.csv`.
2. **TDB model: NAIF ET vs IAU** — shows the difference between the NAIF
   model used for SPICE interoperability and the IAU Fairhead-Bretagnon
   TDB. It is epoch-dependent: `+26.570 us` at `2000-01-01T12:00:00 UTC`,
   and across a full year it ranges about `-29` to `+29 us`, consistent
   with NAIF's stated ~30 us bound. The printed decimals are arithmetic,
   not an accuracy claim. Reproduce the independent reference with
   `python tools/validate_et_vs_iau.py --year 2000` (CSPICE + ERFA).
3. **UT1** — probes WGC2 to show that it has no UT1 time system, then
   validates UT1 against the real IERS data and leap physics through
   `tools/validate_ut1.py`. See *No external UT1 engine* above.

A representative run reports `max WGC2 vs crate difference: 2.847e-07 s`,
`max crate vs CSPICE golden difference: 9.095e-13 s`,
`largest ET-vs-IAU-TDB over these epochs: 26.570 us`, and
`OVERALL: PASS`. Pass `--skip-ut1` or `--skip-tdb-model` to omit a section,
or `--eop <path>` to point the UT1 section at a specific
`finals2000A.all`.

## Scope

For new integration code, read [Time contracts](docs/TIME_CONTRACT.md).
`contract::TimeContext::new(&lsk)` is the default NAIF/SPICE path and
returns typed `NaifEt`. `ScientificContext` is an explicit TT(TAI)/ERFA
scientific path returning a separate `IauTdb`, with geocentric or supplied
observer-dependent terms. Existing low-level APIs retain their behavior.

Native consumers needing historical scalar JDUTC, such as SPK type-10
initialization, can explicitly call
`TimeContext::et_to_spice_julian_utc` and use `NaifJulianUtc::days()`.
This preserves TTRANS's raw arithmetic, historical offset and formal
leap-second behavior without an integer Epoch/civil-calendar round trip.
It requires IERS-only/TableDriven LSK settings; see
[the exact native scope](docs/reference/NATIVE_JDUTC.md). It is distinct
from a physically realized historical UTC/SOFA model.

Original-order structured native entries can instead use
`contract::NaifUtcContext::from_raw_parts(&entries, deltet)?`, with all five
`NaifDeltet` constants supplied explicitly. This separate Rust context preserves
native calculation-stage `SPICE(BADLEAPSECONDS)` errors for malformed TAI
tables. It does not sort entries or weaken the scientific constructor. The
17-profile raw regression matches all 22,292 numeric values and 4,539 native
errors. Arbitrary pool variables and a new MCP input mode are outside its scope;
see [the API example](docs/API.md#raw-native-leap-table-errors).

Implemented:

- Gregorian, Julian and Mixed calendars
- Julian Date and Modified Julian Date (`MJD = JD - 2400000.5`, explicitly)
- Exact `Duration` arithmetic and `TimeSeries`
- `TimeScale` identifiers
- `no_std` support (`alloc` + `libm`; `std` is the default feature)
- Pre-1972 UTC drift via the optional IAU SOFA `iauDat` model
- LSK parsing (`KPL/LSK` text kernel subset)
- UTC <-> TAI <-> TT <-> TDB/ET <-> GPS
- UT1 via an IERS Earth Orientation Parameters table (`eop` module)
- TCG and TCB IAU coordinate scales (`coord` module)
- IAU TDB model, Fairhead & Bretagnon 787-term series and explicitly
  supplied observer-dependent SOFA/ERFA terms (`dtdb` module)
- SCLK type-1 kernel parsing and tick/time conversion (`sclk` module)
- ISO 8601, RFC 3339, RFC 2822 and C89 `strftime`-style formatting
- `etcal` (no kernel required)
- Explicit, sample-verified WGC GPS seconds and week/TOW input/output paths,
  plus UTC/TDB/TT/TDT/GPS calendar YMD and day-of-year displays
- Kernel `@date` literal parsing (`kernel_date` module, exercised N0067
  `TPARTV`/`TPARSE` profile)

Not implemented in this version:

- Full SPICE `str2et` token grammar for arbitrary civil text. The
  `kernel_date` module resolves the exercised kernel `@date` literals;
  civil UTC still uses typed constructors.
- SCLK clock types other than type 1 and partition-aware legacy field-vector
  helpers; use the native string methods for partitioned clock strings
- Automatic policy selection without an explicit date (the caller
  supplies `LeapSecondPolicy`)

## Spacecraft clocks (SCLK)

`SclkKernel::parse(text, id)` reads a `KPL/SCLK` type-1 kernel and
converts between encoded ticks, the parallel time system, and ET/TAI.
The clock may use TDB or TT coefficients (`SCLK01_TIME_SYSTEM`); the LSK
is used only for the TT-to-ET step.

Scalar assignments such as `SCLK01_N_FIELDS_74 = 2` and equivalent
parenthesized values are accepted. TDT clocks retain the original parallel
TT through the N0067 UNITIM arithmetic in both directions, without an
intermediate integer Epoch or origin shift.

Independent CSPICE references exercise both `sct2e` and `sce2c`, including
coefficient neighbors and the formerly failing 0.93 ns and 119.21 ns cases.
The validator defaults to bit-exact acceptance, uses lossless numeric output
and loads the same selected LSK on both sides. Finite-sample equality is
not a physical clock-calibration accuracy guarantee.

```bash
python tools/validate_sclk.py --sclk target/mro_sclk.tsc --id -74
```

Scope: legacy field-vector and string helpers support no partition metadata
or one zero-start partition. Native-style string methods support partitioned
clocks, with an explicit parser context for native call history. Encoded
tick/time conversion bounds use rounded accumulated raw partition durations;
SCENCD string encoding rounds endpoints separately. Clock resets are allowed.
A standalone map without partition metadata has no declared upper bound.
Only clock type 1 is supported. Captured MRO and synthetic reference cases
do not establish exhaustive SCLK parity.

## Formatting

```rust
use stkcraft_time::format::{iso8601_with, rfc2822, rfc3339, strftime, FormatOptions};
use stkcraft_time::{LeapSecondTable, UtcDateTime};

let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.000_000_037);
assert_eq!(rfc3339(&dt, 9), "2000-02-29T14:57:29.000000037+00:00");
assert_eq!(rfc2822(&dt), "Tue, 29 Feb 2000 14:57:29 +0000");
assert_eq!(
    strftime(&dt, "%Y-%m-%d %H:%M:%S").unwrap(),
    "2000-02-29 14:57:29"
);

// Rounding that reaches a leap second needs the day length.
let lsk = LeapSecondTable::builtin().unwrap();
let options = FormatOptions::new(3).with_leap_seconds(&lsk);
let last_second = UtcDateTime::new(2016, 12, 31, 23, 59, 59.9999);
assert_eq!(
    iso8601_with(&last_second, &options),
    "2016-12-31T23:59:60.000Z"
);
```

Supported C89 tokens: `%Y %y %m %d %e %H %M %S %j %a %A %b %B %z %Z %f %%`.
`FormatOptions` also selects the calendar used when a carry moves the
date, so Julian dates roll into `1500-02-29` rather than `1500-03-01`.

## Leap-second policy note

Leap seconds are announced by the International Earth Rotation and
Reference Systems Service (IERS), and NAIF publishes an updated LSK when
one is needed. The 27th CGPM (2022), Resolution 4, set a target of **no
new leap seconds by 2035**, with the maximum `|UT1 - UTC|` allowed to
grow.

Any other date, including `2027-05-20`, must be treated as a **policy
assumption supplied by the caller**, with a citation to the BIPM, IERS or
NAIF document that establishes it. This crate does not encode such a
date.

Before 1972, UTC seconds were not SI seconds. The default
`LeapSecondSource::IersOnly` civil path matches the NAIF LSK and rejects pre-1972
UTC. Selecting `LeapSecondSource::Sofa` additionally enables the IAU
SOFA `iauDat` drift model for 1960-1972, including positive and negative
mini-leaps in the final minute and fractional `TAI - UTC`. Historical
conversions are checked against independent ERFA vectors at every model
transition. The SOFA drift/mini-leap convention has overlaps/gaps of a few
nanoseconds at some day ends; inverse conversion gives actual midnight
to the new date and returns `UtcOutOfRange` for unrepresentable gaps.
Integer nanosecond storage does not make these historical labels an exact
bijection at those boundaries.

## UT1 and IERS EOP

UT1 is the Earth-rotation time scale. It is **not** derivable from the
Leapseconds Kernel: the LSK knows when civil UTC was stepped, but only an
IERS Earth Orientation Parameters (EOP) product knows how the Earth
actually rotated. The LSK and the EOP data are two independent data sets.

This crate reads the `UT1 - UTC` series of the IERS `finals2000A.all`
product into an `EopTable` and exposes UT1 through
`convert::ut1_minus_utc`, `convert::tai_to_ut1`, `convert::ut1_to_tai`,
`convert::utc_to_ut1` and `convert::ut1_to_utc`. Both the Bulletin A and
Bulletin B series are supported (see below).

### Data is deployment-time only

No EOP file is bundled. Fetch it once at deployment time:

```bash
python tools/fetch_eop.py --out data/finals2000A.all
# --insecure only if TLS verification is unavailable in the target environment
```

If the IERS endpoint is slow or unreachable from your network, point the
script at a mirror or a local copy. The endpoint can be set per run or
once per shell:

```bash
python tools/fetch_eop.py --out data/finals2000A.all \
    --url https://mirror.example/finals2000A.all
export STKCRAFT_EOP_URL=https://mirror.example/finals2000A.all
python tools/fetch_eop.py --out data/finals2000A.all   # uses the env var
```

`tools/validate_ut1.py` accepts the same `--url` flag and
`STKCRAFT_EOP_URL` variable, and skips the download entirely when
`--eop` already points at an existing file.

Then load it from disk:

```rust
use stkcraft_time::eop::EopTable;

let text = std::fs::read_to_string("data/finals2000A.all")?;
let eop = EopTable::from_finals2000a(&text)?;
```

The crate performs no network I/O at runtime. `finals2000A.all` is
refreshed daily by IERS; re-run the script to update it.

### Fixed-column format, not whitespace-delimited

`finals2000A.all` is fixed-column. When the month or day has two digits
the year, month and day run together (`73 110` is `1973-01-10`), so a
whitespace split shifts every later column and silently reads a different
quantity. This parser uses the documented columns:

| Field | Columns (1-based) |
|-------|-------------------|
| Year / Month / Day | 1-2 / 3-4 / 5-6 |
| MJD | 7-15 |
| `I`/`P` flag (Bulletin A) | 58 |
| `UT1 - UTC` Bulletin A (s) | 59-68 |
| `UT1 - UTC` Bulletin B (s) | 156-165 |

Rows with a blank `UT1 - UTC` column (future placeholders) are skipped.
Predicted rows (`P`) are retained and flagged; use
`EopTable::observed_only` to restrict a table to IERS observations, which
makes the supported interval explicit.

### Two UT1-UTC series: Bulletin A and Bulletin B

`finals2000A.all` carries two `UT1 - UTC` series. Bulletin A is the
rapid-service and predicted series (columns 59-68); Bulletin B is the
final solution (columns 156-165). They can differ by several milliseconds
for old data. Choose with `EopSeries`:

- `EopTable::from_finals2000a` / `from_finals2000a_series(text, EopSeries::BulletinA)`
  — the default; matches the product name.
- `EopSeries::BulletinB` — the final solution where available.
- `EopSeries::FinalPreferred` — Bulletin B where present, otherwise
  Bulletin A. This matches the combined `UT1_UTC` column used by
  astropy's IERS tables.

Compatibility with astropy's IERS implementation was measured over 1,744
samples spanning the file: ordinary days agree to `1.1e-16 s` and leap
days to `0 s`. Reproduce with `python tools/validate_ut1_astropy.py`.

### Leap seconds are removed before interpolating

`UT1 - UTC` jumps by one second when a positive leap second is inserted,
because UTC is delayed while UT1 keeps running. Naively interpolating the
daily samples across a leap-second day smears that one-second step over
the whole day, giving a half-second error at local noon.

`convert::ut1_minus_utc` removes the known step (from the LSK) before
interpolating, using the true UTC day length of 86400, 86401 or 86399
seconds. On real `finals2000A.all` data at `2016-12-31T12:00:00` this
gives `-0.408239 s`; the naive interpolation gives `+0.091761 s`, a
difference of `0.5 s`. The raw `EopTable::ut1_minus_utc` remains
available for table inspection and for use away from leap boundaries.

### No external UT1 engine; validation against the data

The JPL WebGeocalc / CSPICE have no UT1 time system. The WGC2 API rejects
both `"timeSystem": "UT1"` and `"outputTimeSystem": "UT1"` with
`No enum constant ...TimeStandard.UT1`, and `UT1` appears nowhere in the
Time Conversion documentation. UT1 therefore cannot be compared against
WGC2, and no such comparison is claimed.

UT1 is validated instead against the real IERS file and against an
independent implementation:

- **Data fidelity** (`tools/validate_ut1.py`): on all **20,037** usable
  days, the value at `00:00:00 UTC` reproduces the table entry exactly
  (max `|crate - table|` = `0 s`).
- **Leap physics**: on all **25** leap-boundary days, the leap-aware value
  at `12:00` stays within **1.7 ms** of the day-start value (the LOD
  drift), while naive interpolation is at least **0.498 s** wrong.
- **Independent implementation** (`tools/validate_ut1_astropy.py`): the
  `FinalPreferred` series agrees with astropy's combined IERS column over
  1,744 samples to `1.1e-16 s` on ordinary days and `0 s` on leap days.

```bash
python tools/validate_ut1.py            # downloads the EOP file if absent
python tools/validate_ut1_astropy.py    # vs astropy's IERS implementation
```

### Scope note

UT1 is an Earth-rotation time scale. This API provides leap-aware daily
EOP interpolation, not a complete instantaneous Earth-orientation model.
[IERS Conventions, section 5.5.3.1](https://iers-conventions.obspm.fr/content/chapter5/icc5.pdf)
requires additional subdaily ocean-tide and libration terms for the
corresponding high-accuracy calculation; these are not implemented. EOP
measurement and prediction uncertainties are not propagated. Select
`observed_only()` when predictions are inappropriate. Matching a table
interpolator does not establish zero physical error.

## TCG and TCB

TCG and TCB are the IAU 2000/2006 coordinate scales. They are linear in
TT and TDB and need no kernel:

| Scale | Definition |
|-------|------------|
| TCG | `TT + L_G/(1-L_G)*(JD_TT - T0)*86400` |
| TCB | `TDB + (L_B*(JD_TDB - T0)*86400 - TDB0)/(1-L_B)` |

with `T0 = 1977-01-01T00:00:00 TAI` (`JD 2443144.5003725` in TT),
`L_G = 6.969290134e-10`, `L_B = 1.550519768e-8` and `TDB0 = -6.55e-5 s`.

Neither the WebGeocalc API nor CSPICE exposes TCG/TCB: the WGC2 API
rejects both with `No enum constant ...TimeStandard.TCG/TCB`. They are
validated against ERFA/SOFA through astropy. Measured over **950 TAI
epochs from 1972 to 2050**:

- TT agrees with ERFA exactly (0 s).
- TCG agrees to `2.384e-7 s`, which is the `f64` spacing of a value near
  `1.6e9` seconds at 2050, not a model error.
- TCB agrees to `2.384e-7 s`, the same `f64` limit.

TCB uses the IAU TDB model from the `dtdb` module, not the NAIF `ET`
approximation. That model is the 787-term Fairhead & Bretagnon series:
it agrees with ERFA `dtdb` to `4.6e-18 s` over 1218 samples from 1950 to
2050, and ERFA states the series is accurate to about 3 ns in that
interval. NAIF `ET`, accurate to about 30 us, remains available separately
for SPICE interoperability; the two are not interchangeable.

That 30 us to 3 ns improvement is the point of the port. It matters when
the reference chain is IAU/SOFA (VLBI, pulsar timing, relativistic time
transfer, comparing against ERFA-based pipelines) or when TDB feeds an
ephemeris at the microsecond level. It does not matter for ordinary
mission planning, where 30 us is far below every other error term.

Coordinate scales are `f64` seconds past `JD 2451545.0`; at modern epochs
that spacing is about 60 ns.

```bash
pip install astropy pyerfa
python tools/gen_tcg_tcb_golden.py     # rewrites tests/golden/tcg_tcb.csv
python tools/gen_dtdb_golden.py        # rewrites tests/golden/dtdb.csv
```

## Usage

```rust
use stkcraft_time::calendar::Calendar;
use stkcraft_time::convert::{tai_to_utc, utc_to_tai};
use stkcraft_time::{LeapSecondTable, UtcDateTime};

let lsk = LeapSecondTable::builtin()?;

let dt = UtcDateTime::new(2016, 12, 31, 23, 59, 60.5);
let tai = utc_to_tai(&dt, Calendar::Gregorian, &lsk)?;
let back = tai_to_utc(tai, Calendar::Gregorian, &lsk)?;
assert_eq!(back.second, 60.5);
# Ok::<(), stkcraft_time::TimeError>(())
```

## MCP server

The optional `mcp` feature builds a stdio MCP server that exposes the
conversion API as JSON-RPC tools:

```sh
cargo build --release --features mcp --bin stkcraft-mcp
```

Example client configuration:

```json
{
  "mcpServers": {
    "stkcraft-time": {
      "command": "/absolute/path/to/target/release/stkcraft-mcp",
      "args": []
    }
  }
}
```

See [`docs/MCP.md`](https://github.com/spacelibres/STKCraft-time/blob/main/docs/MCP.md) for the tool list and schemas. Agent
skills are in [`.agents/skills/`](https://github.com/spacelibres/STKCraft-time/tree/main/.agents/skills).

## References

- NAIF, *SPICE Time Subsystem Required Reading* (`time.req`)
- NAIF, *Leapseconds Kernel* (`lsk.req`)
- NAIF, `naif0012.tls`
- IERS Bulletin C (leap-second announcements)
- IERS, `finals2000A.all` (Earth Orientation Parameters)
- BIPM, 27th CGPM (2022), Resolution 4

## Build and verification

```bash
cargo test                          # unit and integration tests
cargo test --features mcp           # MCP protocol tests
cargo test --test spice_golden      # 82 CSPICE reference vectors
cargo build --no-default-features   # no_std + alloc build
cargo build --no-default-features --target wasm32-unknown-unknown
cargo build --no-default-features --target thumbv7em-none-eabi
cargo build --release --features mcp --bins
cargo +1.75.0 check --all-targets --features mcp
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features mcp -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
python tools/wgc2_compare.py        # live WGC2 (CSPICE front-end) check
```

The library is `no_std` compatible with `alloc`:

```toml
stkcraft-time = { version = "0.2.1", default-features = false }
```

In `no_std` builds, `libm` supplies `sin`, `cos`, `floor` and `round`;
calendar and epoch arithmetic remain integer-based. The `std` feature is
enabled by default and only adds `std::error::Error`.

### Alignment with NAIF (release criterion)

The release criterion is numerical alignment with NASA NAIF/SPICE:

- 82 CSPICE golden vectors: max UTC-to-ET difference `1.1e-12 s`.
- Live WGC2 `TIME_CONVERSION`: max `2.847e-7 s` (its output rounding).
- Kernel selection: `naif0012`/`naif0011`/`naif0010` reproduced exactly
  (see *Measured against the live WGC2 API*).
- SCLK: 10,871 independent forward/inverse CSPICE cases match bit for bit.
  Explicit Epoch/TAI conversions still quantize to integer nanoseconds.

Scales that NAIF does not define are not part of this criterion: UT1 is
validated against IERS data, and TCG/TCB against ERFA. Neither CSPICE nor
WebGeocalc exposes them.

### Optional: formal verification (Kani)

Kani is **not** a release gate. It is an optional extra for the integer
representation layer; the release criterion above is NAIF alignment.

`src/verification.rs` contains `#[kani::proof]` harnesses for `Duration`
and `Epoch` normalization. Status, stated precisely:

- Kani 0.68.0 publishes binaries only for Linux and macOS. There is no
  Windows build, and this project's development host is
  `x86_64-pc-windows-msvc`, so **the harnesses have not been executed on
  this machine**.
- They can be run on `ubuntu-latest` by manually dispatching
  `.github/workflows/kani.yml`.
- `tests/properties.rs` exercises the same invariants with 200,000
  deterministic samples under `cargo test`. Those checks **execute here**,
  but they are runtime tests, not proofs.

To run the proofs locally on a supported platform:

```bash
cargo install --locked kani-verifier
cargo kani setup
cargo kani
```

## License

MIT License. See `LICENSE`.

The bundled `src/naif0012.tls` is a NAIF data product and is included
with attribution. It is not covered by the crate's MIT license.

## Contributing

See [`CONTRIBUTING.md`](https://github.com/spacelibres/STKCraft-time/blob/main/CONTRIBUTING.md) for development setup, release
procedures and validation commands. Security reports go through
[`SECURITY.md`](https://github.com/spacelibres/STKCraft-time/blob/main/SECURITY.md); all participation is covered by
[`CODE_OF_CONDUCT.md`](https://github.com/spacelibres/STKCraft-time/blob/main/CODE_OF_CONDUCT.md). Support resources are listed
in [`SUPPORT.md`](https://github.com/spacelibres/STKCraft-time/blob/main/SUPPORT.md).

## Citation

See [`CITATION.cff`](https://github.com/spacelibres/STKCraft-time/blob/main/CITATION.cff).

## Precision and representation

- `Epoch` is an integer count of TAI seconds plus nanoseconds since
  `2000-01-01T00:00:00 TAI`. The integer fields can address about
  ±2.9×10¹¹ years, and the nanosecond field is a 1 ns *representation*
  step. Addressability is not accuracy: a conversion is only defined
  where the loaded kernel, the calendar rules and (for UT1) the EOP
  table support the date.
- `Duration` is exact integer arithmetic; no floating-point accumulation
  is used for epoch or duration arithmetic.
- TAI is the glitch-free internal reference. UTC is a civil
  representation, not an internal scale.
- NAIF `ET`, the compact two-term `TDB` approximation and the 787-term
  geocentric IAU `TDB` model are separate functions with different limits.
- Against 82 CSPICE reference vectors, the maximum UTC-to-ET difference
  is currently **1.1×10⁻¹² s**. This is agreement with a reference
  engine, not a physical accuracy claim.

## Leap-second regime

The library distinguishes three concepts:

- the **table** (`LEAP` entries from the LSK);
- the **source** (`IersOnly` or `Sofa` for pre-1972);
- the **policy** that decides what happens after a chosen epoch.

The default policy is `TableDriven`: the last table entry remains in
force indefinitely, so if no further leap second is announced, `TAI−UTC`
freezes automatically. A caller that wants to model the CGPM decision to
stop leap seconds can use `LeapSecondPolicy::frozen_after(year, month,
day)` (or the `FrozenAfter { jdn }` variant); from that date onward,
later table entries are ignored and the last value is kept. No future
date is hard-coded in the default path.


## Python, JavaScript and BeiDou interfaces

Independent Python and WASM/JavaScript/TypeScript adapters call this same
Rust kernel. See [language interfaces and support matrix](docs/BINDINGS.md)
for installation artifacts, exact-integer transport, model selection,
verification and release status. [BeiDou BDT](src/bdt.rs) provides checked
integer parts and full-week coordinates without changing existing NAIF
calls. The [Cortex-M example](examples/cortex-m-utc-et/README.md) demonstrates
bare-metal linking and provides an on-target cycle-measurement harness.
