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

- Load an LSK with `LeapSecondTable::from_lsk(text)` in production.
  All `DELTET` constants must be supplied; missing constants return
  `LskMissing` rather than silently using compiled-in defaults.
  `LeapSecondTable::builtin()` is a snapshot of `naif0012.tls` and stops at
  the `2017-01-01` leap second.
- Every conversion is a pure function of the supplied kernel.
  `coverage()` reports the trusted range; dates before the first entry are
  rejected, dates after the last entry keep the last `TAI - UTC`.
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
- Pre-1972 UTC requires `LeapSecondSource::Sofa`; the default
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
- `docs/MCP.md` — MCP server tool reference.
- `tests/naif_alignment.rs`, `tests/ut1.rs`, `tests/mcp.rs` — boundary and
  protocol tests.
- `AGENTS.md` — repository rules.
