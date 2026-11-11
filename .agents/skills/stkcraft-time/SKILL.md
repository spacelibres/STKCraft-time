---
name: stkcraft-time
description: Use when changing the STKCraft-time Rust crate or converting UTC, TAI, TT, ET/TDB, GPS, UT1 or SCLK values. Covers the mandatory patch-version bump, versioned changelog entry, LSK/SCLK/EOP loading, API calls, and verification.
---

# STKCraft-time development

## Mandatory update workflow

Every update is a release. Do not leave changes under `[Unreleased]`.

1. Increment the **patch** component in `Cargo.toml`
   (`0.1.1` -> `0.1.2` -> ...). Use a minor bump only for a new
   backward-compatible feature and a major bump for a breaking change;
   while the major version is `0`, call out breaking changes explicitly.
2. Rename or create the matching section in `CHANGELOG.md`, for example
   `## [0.1.2] - 2026-11-10`. The section must describe the behaviour
   change and its user-visible impact, not just repeat the commit subject.
3. Update `Cargo.lock` to the same version.
4. Run the verification commands below and require success.
5. Commit, push to `main`, and require the GitHub Actions `CI` workflow to
   pass. Tag the release as `vX.Y.Z` after the release commit.

## Verification

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --features mcp
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features mcp -- -D warnings
cargo build --no-default-features
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## API orientation

- Load an LSK with `LeapSecondTable::from_lsk(text)` in production.
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
- SCLK support is type 1 only. Partitions are parsed but not applied.

## Edge cases that must stay correct

- `23:59:60.x` is legal only on a positive-leap-second (86401-second) day.
- `23:59:59.x` does not exist on a shortened (86399-second) day and must
  return `InvalidLeapSecond`, not alias the next day's `00:00:00`.
- Pre-1972 UTC requires `LeapSecondSource::Sofa`; the default
  `IersOnly` source returns `UtcOutOfRange`.
- EOP samples with `NaN` or infinity must return `EopParse`, never panic.
- `EopTable` stores `UT1 - UTC`; UT1 conversion needs the UTC day and the
  following day in the table.

## References

- `docs/API.md` — full API guide and function index.
- `docs/MCP.md` — MCP server tool reference.
- `tests/naif_alignment.rs`, `tests/ut1.rs`, `tests/mcp.rs` — boundary and
  protocol tests.
- `AGENTS.md` — repository rules.
