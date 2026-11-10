<p align="center">
  <img src="https://raw.githubusercontent.com/spacelibres/STKCraft_web/main/public/brand/albatross.png" alt="STKCraft — Albatross" width="180">
</p>

# STKCraft-time

**STKCraft project:** <https://www.stkcraft.com>
**Repository:** <https://github.com/spacelibres/STKCraft-time>

A clean-room, NAIF/SPICE-aligned time library written in Rust.

`stkcraft-time` implements the time-scale model documented by the NASA
Navigation and Ancillary Information Facility (NAIF) in the **SPICE Time
Subsystem Required Reading** and driven by a **Leapseconds Kernel (LSK)**.
It does not link CSPICE and does not port CSPICE source code.

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
   day. The `TDB - UTC = 32.184 + DeltaAT + K*sin(E)` expression is *not*
   used to convert across a leap-second instant.

4. **Scales are distinct types.**
   UTC, TAI, TT, TDB/ET and GPS are never interchangeable. Conversion is
   only possible through explicit constructors.

5. **No future leap-second policy is encoded.**
   Leap seconds are data. The library follows the supplied LSK. To model
   a policy assumption, construct a table whose entries reflect that
   assumption. The crate never infers a policy from a date.

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
- UTC dates **before** the first kernel entry are rejected
  (`UtcOutOfRange`); nothing is guessed.
- UTC dates **after** the last entry hold the last `TAI - UTC` value.
  This is NAIF's documented behaviour, not data invented here: a kernel
  cannot know about a leap second announced after it was released.
  Update the kernel to extend trustworthy coverage.

Two kernels that differ in `DELTET/DELTA_AT` therefore give different
results; `tests/kernel_selection.rs` asserts this explicitly.

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
| TDB (ESA) | `TT + 0.001658*sin(g) + 0.000014*sin(2g)` |
| GPS | `TAI - 19 s`, zero at `1980-01-06T00:00:00 UTC` |
| UT1 | Earth-rotation scale, `UTC + (UT1 - UTC)`; requires IERS EOP |
| TCG | `TT + L_G*(JD_TT - T0)*86400` (IAU coordinate scale) |
| TCB | `TDB + L_B*(JD_TDB - T0)*86400 - TDB0` (IAU coordinate scale) |

`ET` is **TDB seconds past the J2000 epoch** (`2000-01-01T12:00:00 TDB`),
matching the SPICE convention. `ET` and the ESA `TDB` model are kept
distinct on purpose: use `ET` for NAIF/SPICE interoperability, and the
`tdb` module for the higher-fidelity dynamical model.

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
- IAU TDB model, Fairhead & Bretagnon 787-term series (`dtdb` module)
- SCLK type-1 kernel parsing and tick/time conversion (`sclk` module)
- ISO 8601, RFC 3339, RFC 2822 and C89 `strftime`-style formatting
- `etcal` (no kernel required)

Not implemented in this version:

- Full SPICE `str2et` token grammar (this version provides typed
  constructors only)
- SCLK partition boundaries (parsed but not applied) and clock types
  other than type 1
- Automatic policy selection without an explicit date (the caller
  supplies `LeapSecondPolicy`)

## Spacecraft clocks (SCLK)

`SclkKernel::parse(text, id)` reads a `KPL/SCLK` type-1 kernel and
converts between encoded ticks, the parallel time system, and ET/TAI.
The clock may use TDB or TT coefficients (`SCLK01_TIME_SYSTEM`); the LSK
is used only for the TT-to-ET step.

The mapping was verified against CSPICE `sct2e` with a real mission
kernel (`MRO_SCLKSCET`): over 64 sample ticks spanning the whole kernel,
the maximum difference is **1.8e-7 s**. That is the spacing of an `f64`
tick count near `3.8e11`, not a model error.

```bash
python tools/validate_sclk.py --sclk target/mro_sclk.tsc --id -74
```

Scope: encoded-tick conversions and field/string encoding are exact
within one partition. Partition boundaries are parsed but not applied;
a partition-aware reformatting layer is future work. Only clock type 1
is supported.

## Formatting

```rust
use stkcraft_time::format::{rfc2822, rfc3339, strftime};
use stkcraft_time::UtcDateTime;

let dt = UtcDateTime::new(2000, 2, 29, 14, 57, 29.000_000_037);
assert_eq!(rfc3339(&dt, 9), "2000-02-29T14:57:29.000000037+00:00");
assert_eq!(rfc2822(&dt), "Tue, 29 Feb 2000 14:57:29 +0000");
assert_eq!(
    strftime(&dt, "%Y-%m-%d %H:%M:%S").unwrap(),
    "2000-02-29 14:57:29"
);
```

Supported C89 tokens: `%Y %y %m %d %e %H %M %S %j %a %A %b %B %z %Z %f %%`.

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
`LeapSecondSource::IersOnly` matches the NAIF LSK and rejects pre-1972
UTC. Selecting `LeapSecondSource::Sofa` additionally enables the IAU
SOFA `iauDat` drift model for 1960-1972, returning fractional
`TAI - UTC`.

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

UT1 is an Earth-rotation angle, not a uniform coordinate time. It is kept
separate from ET and TDB on purpose and is never mixed into the dynamical
conversions.

## TCG and TCB

TCG and TCB are the IAU 2000/2006 coordinate scales. They are linear in
TT and TDB and need no kernel:

| Scale | Definition |
|-------|------------|
| TCG | `TT + L_G*(JD_TT - T0)*86400` |
| TCB | `TDB + L_B*(JD_TDB - T0)*86400 - TDB0` |

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
cargo test --test spice_golden      # 82 CSPICE reference vectors
cargo build --no-default-features   # no_std + alloc build
cargo clippy --all-targets -- -D warnings
python tools/wgc2_compare.py        # live WGC2 (CSPICE front-end) check
```

The library is `no_std` compatible with `alloc`:

```toml
stkcraft-time = { version = "0.1", default-features = false }
```

In `no_std` builds, `libm` supplies `sin`, `floor`, `round` and `fmod`;
calendar and epoch arithmetic remain integer-based. The `std` feature is
enabled by default and only adds `std::error::Error`.

### Alignment with NAIF (release criterion)

The release criterion is numerical alignment with NASA NAIF/SPICE:

- 82 CSPICE golden vectors: max UTC-to-ET difference `1.1e-12 s`.
- Live WGC2 `TIME_CONVERSION`: max `2.847e-7 s` (its output rounding).
- Kernel selection: `naif0012`/`naif0011`/`naif0010` reproduced exactly
  (see *Measured against the live WGC2 API*).
- SCLK: max `1.5e-7 s` against CSPICE `sct2e`.

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
- NAIF `ET` and the higher-fidelity ESA `TDB` model are kept as separate
  functions on purpose.
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
