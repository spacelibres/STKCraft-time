---
name: stkcraft-time
description: Use when changing the STKCraft-time Rust crate or converting UTC, TAI, TT, ET/TDB, GPS, UT1 or SCLK values. Covers the mandatory patch-version bump, versioned changelog entry, LSK/SCLK/EOP loading, API calls, and verification.
---

# STKCraft-time development

## Mandatory update workflow

Every update is a release. Do not leave changes under `[Unreleased]`.
Keep every repository file in English; do not add Chinese prose or
Chinese-localized UI labels.

1. Increment the **patch** component in `Cargo.toml`
   (`0.1.1` -> `0.1.2` -> ...). Use a minor bump only for a new
   backward-compatible feature and a major bump for a breaking change;
   while the major version is `0`, call out breaking changes explicitly.
2. Rename or create the matching section in `CHANGELOG.md`, for example
   `## [0.1.2] - 2026-11-10`. The section must describe the behaviour
   change and its user-visible impact, not just repeat the commit subject.
3. Update `Cargo.lock` to the same version.
4. Run the verification commands below and require success.
5. Commit with `Release X.Y.Z`, push to `main`, wait for the `CI`
   workflow to pass, then create and push the `vX.Y.Z` tag and wait for
   the tag CI run to pass. Do this automatically; do not wait for a
   separate commit or push request.
6. Add `docs/releases/vX.Y.Z.md` and confirm the `Release` workflow
   publishes a GitHub Release with release-mode binaries for Linux,
   macOS and Windows and `SHA256SUMS.txt`.
7. Leave no uncommitted changes when the task is complete. Skip the
   commit/push/tag steps only when the user explicitly asks you to.

## Verification

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --features mcp
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features mcp -- -D warnings
cargo build --no-default-features
cargo build --release --features mcp --bins
cargo test --release --features mcp
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo +1.75.0 check --locked --all-targets --features mcp
cargo build --locked --no-default-features --target wasm32-unknown-unknown
cargo build --locked --no-default-features --target thumbv7em-none-eabi
```

## API orientation

- Prefer `contract::TimeContext::new(&lsk)` for new geometry integration.
  It returns checked `NaifEt`; only `.seconds()` crosses into legacy f64
  geometry APIs. Scientific TDB is an explicit `ScientificContext` with a
  distinct `IauTdb` type and no implicit conversion to NAIF ET.
- For a native SPK10 packet epoch, use
  `TimeContext::new(&lsk).et_to_spice_julian_utc(et)?.days()` to obtain
  TTRANS JDUTC directly. This explicit path requires IERS-only/TableDriven
  LSK settings, preserves native historical extrapolation and formal
  leap-second behavior, and returns checked `NaifJulianUtc`. Do not detour
  through integer Epoch, civil-calendar fields or SOFA. Read
  `docs/reference/NATIVE_JDUTC.md` for the exact corpus and error boundaries.
- The checked UTC path follows pinned TTRANS/UNITIM arithmetic. Validate it
  against STR2ET; validate supplied numeric TAI against UNITIM. Those inputs
  can quantize differently, so do not compare every representation against
  one UTC-string oracle or mix conversion routes inside a geometry query.
- Scientific TT is TT(TAI), always TAI + 32.184 seconds independent of LSK
  constants. It is not TT(BIPM). Checked scientific TDB uses the 1950-2050
  comparison interval and rejects invalid inputs. Observer-dependent terms
  require supplied UT1 fraction, east longitude radians and u/v kilometres.
  Never substitute UTC for UT1 or invent missing observer/EOP data.

- Load an LSK with `LeapSecondTable::from_lsk(text)` in production.
  All `DELTET` constants must be supplied; missing constants return
  `LskMissing` rather than silently using compiled-in defaults.
  `LeapSecondTable::builtin()` is a snapshot of `naif0012.tls` and stops at
  the `2017-01-01` leap second.
- Every conversion is a pure function of the supplied kernel.
  `coverage()` reports the trusted range for civil UTC conversions; dates
  before the first entry are rejected there. The explicit native JDUTC
  path selects a separate N0067 historical convention. Dates after the
  last entry keep the last `TAI - UTC`.
- Use `LeapSecondPolicy::FrozenAfter` only when the caller explicitly
  chooses a future policy. The default `TableDriven` policy infers
  nothing from a date.
- UTC -> TAI: `convert::utc_to_tai(&UtcDateTime, Calendar, &lsk)`.
- TAI -> UTC: `convert::tai_to_utc(epoch, Calendar, &lsk)`.
- TAI -> ET: `convert::tai_to_et(epoch, &lsk)`; ET -> TAI:
  `convert::et_to_tai(et, &lsk)`.
- TAI -> TT: `convert::tai_to_tt`; TAI -> GPS:
  `convert::tai_to_gps`; GPS -> TAI: `convert::gps_to_tai`.
- UT1 needs an EOP table: `EopTable::from_finals2000a(text)` and
  `convert::utc_to_ut1(&utc, Calendar, &lsk, &eop)`. Use
  `convert::ut1_minus_utc` for the leap-aware offset;
  `EopTable::ut1_minus_utc` is raw table interpolation.
- SCLK: `SclkKernel::parse(text, spacecraft_id)`, then
  `ticks_to_et`, `et_to_ticks`, `ticks_to_tai`, `tai_to_ticks`,
  `parse_string`, `format_string`, `ticks_from_fields`,
  `fields_from_ticks`.
- Legal SCLK numeric scalars may omit parentheses. TDT `ticks_to_et`
  and `et_to_ticks` use direct N0067 UNITIM TT/ET arithmetic; do not route
  those f64 paths through integer TAI. Preserve native reciprocal-rate
  evaluation and reverse modulus accumulation for bit-exact CK boundaries.
  `tools/validate_sclk.py` defaults to exact comparisons in both directions.
- SCLK support is type 1 only. Partition durations bound encoded ticks;
  field/string conversion returns `SclkPartitionsUnsupported` for
  partitioned clocks, while encoded ticks↔ET/TAI remain available.
  Partition readouts are rounded to integral ticks before accumulating
  `end - start`; clock resets are allowed. Reject ticks before the first
  coefficient or beyond that cumulative domain. A standalone map without
  partition metadata has no declared upper bound.
- Formatting: `format::iso8601(&utc, precision)` assumes a
  86 400-second day. When a carry could reach a leap second or a calendar
  day boundary, use
  `format::iso8601_with(&utc, &FormatOptions::new(p).with_leap_seconds(&lsk).with_calendar(cal))`;
  `rfc3339_with`, `spice_calendar_with`, `strftime_with` and
  `Formatter::with_options` take the same options.
- Kernel `@date` literals: `kernel_date::parse_kernel_date(text)` returns
  seconds past J2000 with normalized calendar fields. It is N0067 TPARSE
  arithmetic only: no UTC/TT/TDB conversion, no LSK, pre-1972 dates are
  valid constants. Token precedence, mixed separators, the default 1969--2068
  short-year window and native calendar/clock normalization are covered.
  Consult docs/reference/KERNEL_DATE_GRAMMAR.md for exercised forms and
  residual limits; do not infer exhaustive TPARSE support. Unsupported
  literals return `KernelDateUnsupported`; never report them as
  `LskMissing`/`SclkMissing` and never substitute an epoch. LSK
  `DELTET/DELTA_AT` dates stay strict: an impossible day is `LskParse`, not
  a roll-forward.
- `SCLK01_OFFSETS` is the first displayed value: encode as
  `field - offset`, decode as `encoded + offset`.
- UT1 interpolation is performed on continuous `UT1 - TAI`, so missing
  days across a leap second do not smear the step.

## Edge cases that must stay correct

- Untrusted numeric input must use `Epoch::checked_from_parts`,
  `checked_from_tai_seconds`, `checked_add`/`checked_sub`, or the
  `Duration::checked_*` methods. The infallible constructors panic rather
  than wrapping or returning a fake zero.
- From 1972, `23:59:60.x` is legal only on a positive-leap-second day.
  In SOFA historical mode, the final minute can instead include a positive
  mini-leap or lose a fractional second; validate against that day's jump.
- `23:59:59.x` does not exist on a shortened (86399-second) day and must
  return `InvalidLeapSecond`, not alias the next day's `00:00:00`.
- Physical historical civil UTC requires `LeapSecondSource::Sofa`; the default
  `IersOnly` source returns `UtcOutOfRange`.
- Historical inverse conversion keeps split integer epoch arithmetic and
  actual midnight ownership. SOFA drift/mini-leap scaling has few-nanosecond
  overlaps/gaps; unrepresentable negative-gap epochs return `UtcOutOfRange`.
- EOP samples with `NaN` or infinity must return `EopParse`, never panic.
- UT1 is daily EOP interpolation, without subdaily ocean-tide/libration
  corrections or propagated measurement/prediction uncertainty. Do not
  advertise it as a complete instantaneous Earth-orientation model.
- `EopTable` stores `UT1 - UTC`; UT1 conversion needs bracketing EOP
  samples and does not support the last sample day as an epoch boundary.

## References

- `docs/API.md` — full API guide and function index.
- `docs/TIME_CONTRACT.md` — model contracts, units, origins and limits.
- `docs/MCP.md` — MCP server tool reference.
- [Project wiki](https://github.com/spacelibres/STKCraft-time/wiki) —
  narrative explanations and the verification record.
- `tests/naif_alignment.rs`, `tests/ut1.rs`, `tests/mcp.rs` — boundary and
  protocol tests.
- `AGENTS.md` — repository rules.
