# Changelog

All notable changes to `stkcraft-time` are documented here.

The format follows Keep a Changelog. This crate uses semantic versioning.

## [0.1.13] - 2026-11-11

### Added

- The `Release` workflow publishes each tag to crates.io with
  `cargo publish` when the `CARGO_REGISTRY_TOKEN` repository secret is
  configured. The job skips itself when the secret is absent and when the
  version is already on the registry, so a re-run or a tag that was
  published by hand is still green.

### Changed

- `README.md` states the install command directly below the badge and
  link block, and the installation section puts `cargo add stkcraft-time`
  before the manifest snippet.

## [0.1.12] - 2026-11-11

### Fixed

- The leap-second day length was applied to every minute of the day, so
  `2016-12-31T12:34:59.9999` at precision 3 printed `12:34:60.000` and
  `23:58:59.9999` printed `23:58:60.000`. Only `23:59` can run past 59 s;
  every other minute now ends at `59.999` and rounds into the next
  minute. The same over-eager limit ended ordinary minutes early on a
  shortened (86 399-second) day.

## [0.1.11] - 2026-11-11

### Fixed

- The 0.1.10 release notes stated the wrong test totals. The suite runs
  104 tests by default and 116 with the `mcp` feature.

### Changed

- The `stkcraft-time` agent skill documents the leap-second aware
  formatting entry points and the per-partition SCLK validation, so
  agents do not fall back to the day-length assumption.

## [0.1.10] - 2026-11-11

### Added

- `format::FormatOptions` and the matching `iso8601_with`, `rfc3339_with`,
  `spice_calendar_with`, `strftime_with` and `Formatter::with_options`
  entry points. They take an optional leap-second table and a calendar,
  so rounding knows the length of the day and how a day carry moves the
  date.
- The MCP `format_utc` tool accepts `calendar` and `lsk_path`, and every
  conversion result's `iso8601` field is now leap-second aware.

### Fixed

- SCLK partition bounds are validated per partition only. A clock that
  jumps ahead or regresses - the reason partitions exist - no longer
  makes the whole kernel fail to load, so encoded ticks↔ET/TAI remain
  available. Mismatched array lengths and a reversed `start > end` are
  still rejected.
- Formatting a value such as `2016-12-31T23:59:59.9999` at precision 3
  now produces `2016-12-31T23:59:60.000` when a leap-second table says
  the day is 86 401 s long, instead of the following day. Without a
  table the documented 86 400-second assumption still applies.
- Day carries use the calendar selected in `FormatOptions` instead of
  always Gregorian, so a Julian date rolls from `1500-02-28` to
  `1500-02-29`. `%j`, `%a` and `%A` follow the selected calendar too.
- UT1 reverse conversion works on the last interpolated day and at the
  start of the table. The initial guess uses the measured `UT1 - UTC`
  value instead of a fixed 32 s offset, and is clamped to the interval
  the interpolation actually supports.
- `DELTET/DELTA_AT` follows NAIF assignment rules: `=` replaces the
  accumulated leap-second table and `+=` appends to it, so a kernel that
  restates the full table in a later data block is valid instead of
  failing with duplicate dates. Any other operator is a parse error.

## [0.1.9] - 2026-11-11

### Added

- crates.io publication. `Cargo.toml` now lists `date-and-time` and
  `parsing` alongside `aerospace`, `science` and `no-std`, so the crate
  appears under the registry categories users browse for time handling
  and kernel parsing.
- `README.md` carries crates.io, docs.rs and download badges, installs
  with `cargo add stkcraft-time`, and documents the `stkcraft-mcp`
  install command.
- `CONTRIBUTING.md` and `AGENTS.md` describe the crates.io publishing
  step: `cargo login`, `cargo publish --dry-run`, `cargo publish`, and
  the docs.rs build check.

### Changed

- Every documentation link in `README.md` is absolute, so the crate page
  on crates.io and the README on docs.rs resolve to the repository
  instead of a relative path inside the package.

### Notes

- No library API, kernel format or numerical result changes in this
  release.

## [0.1.8] - 2026-11-11

### Added

- `tools/fetch_eop.py`, `tools/validate_ut1.py` and
  `tools/gen_dtdb_table.py` accept `--url`, and the `STKCRAFT_EOP_URL`
  and `STKCRAFT_DTDB_URL` environment variables, so a mirror or local
  copy can replace the upstream host when it is slow or unreachable.
  `--url` wins over the environment variable, and a pre-existing
  `--eop` / `--src` file still skips the download entirely.

### Fixed

- The download scripts now write the response bytes verbatim. On
  Windows, text-mode writes translated line endings, so the saved
  `finals2000A.all` was not byte-identical to the upstream file.

## [0.1.7] - 2026-11-11

### Fixed

- SCLK `SCLK01_OFFSETS` now follows NAIF: encode as `field - offset`,
  decode as `encoded + offset`. Nonzero-offset clocks such as Voyager no
  longer gain or lose ticks.
- `Epoch` arithmetic is checked end to end. `from_parts`, floating-point
  construction, addition and subtraction no longer wrap in release;
  checked variants are available for untrusted input.
- MCP `precision` is limited to `0..=9` in the schema and execution layer,
  and tool panics are isolated so a bad request cannot terminate the
  service.
- Formatting rounding now carries into minutes and days, clamps precision
  to nanoseconds, and never emits `010.000` or a nonexistent `60.000`.
- `etcal` splits ET into whole days and seconds before formatting, so
  microseconds survive at J2000.
- UT1 inverse conversion works at the first EOP day; the fixed-point
  initial guess is clamped to the EOP-supported interval.
- UT1 interpolation uses continuous `UT1 - TAI`, so missing EOP samples
  across a leap second no longer reintroduce a ~0.25 s step error.
- `Duration` checked negation, addition and subtraction use i128 totals
  and accept representable extrema, including `a.checked_sub(a)`.
- Non-finite and out-of-range floats are rejected instead of becoming
  zero or saturated epochs/durations; EOP queries reject NaN.
- LSK and SCLK parsers read all data blocks, validate calendar dates,
  reject non-finite constants, require increasing ticks/seconds and
  positive rates, and validate partition arrays.
- SCLK partition-aware field/string conversion returns
  `SclkPartitionsUnsupported` instead of silently ignoring offsets;
  encoded ticks↔ET/TAI remain available.
- MCP accepts continuous SCLK ticks; optional fields and clock strings are
  `null` for non-integral ticks.
- Invalid JSON-RPC requests receive `-32600`; the server advertises only
  MCP protocol version `2024-11-05`.
- `TimeSeries` uses checked cursor advancement and delivers the last
  representable epoch before ending.
- `tools/gen_dtdb_table.py` uses TLS verification by default, pins the
  ERFA source commit and SHA-256, and requires an explicit `--insecure`.

### Added

- `Epoch::checked_from_parts`, `checked_from_tai_seconds`, `checked_add`,
  `checked_sub`, and `Duration::checked_from_seconds_f64`.
- `TimeError::SclkPartitionsUnsupported`.
- MCP `sec`/`nanos` epoch input and schema precision limits.

### Changed

- CI now checks formatting, rustdoc, MSRV 1.75, wasm32 and
  thumbv7em-none-eabi `no_std` builds.
- Property tests use an independent i128 oracle at integer extremes.
- TCG/TCB tolerance now protects the IAU TDB model instead of allowing
  the old NAIF-model error.
- Documentation and skills describe checked APIs, SCLK offsets, partition
  behavior and UT1 interpolation.

## [0.1.6] - 2026-11-10

### Fixed

- The `Release` workflow now runs macOS builds on the supported Intel
  runner and cross-builds an arm64 binary, so both macOS architectures
  are published. The previous arm64 runner label was unavailable.

### Changed

- `docs/releases/v0.1.6.md` is the release body. Version 0.1.5 was
  tagged but not published because its release workflow could not start
  the arm64 macOS runner; 0.1.6 is the first published GitHub Release.

## [0.1.5] - 2026-11-10

### Added

- GitHub `Release` workflow that runs release-mode tests, builds
  optimized binaries for Linux x86_64, macOS arm64 and Windows x86_64,
  generates `SHA256SUMS.txt` and publishes a GitHub Release.
- `docs/releases/v0.1.5.md` as the release body.
- Community health files: `SECURITY.md`, `CONTRIBUTING.md`,
  `CODE_OF_CONDUCT.md`, `SUPPORT.md`, `CITATION.cff`, issue templates
  and a pull request template.
- README badges, project status, installation and release-asset
  sections.
- `[package.metadata.docs.rs]` with `all-features = true`, an `authors`
  field and the `aerospace` category.

### Changed

- `Cargo.toml` description now covers all supported scales, kernel
  types, `no_std` support and the MCP server.
- Release profile uses thin LTO, one codegen unit and stripped binaries.
- MCP build and client configuration examples now use release binaries
  instead of debug binaries.
- `AGENTS.md` and the `stkcraft-time` skill now require a GitHub Release
  and release notes for every tag, and record the canonical repository
  description, homepage and topics.
- Repository description, homepage and topics are now populated on
  GitHub.

The time-conversion algorithms are unchanged in this release.

## [0.1.4] - 2026-11-10

### Changed

- `AGENTS.md` and the `stkcraft-time` skill now require the agent to
  commit, push and tag every update automatically, and to wait for both
  the main and tag GitHub Actions runs.

## [0.1.3] - 2026-11-10

### Changed

- Replace the Chinese-localized WebGeocalc UI labels in `README.md` and
  `docs/API.md` with their English equivalents. Repository content is now
  English-only, and `AGENTS.md` records that rule.

## [0.1.2] - 2026-11-10

### Added

- `docs/API.md`: a task-oriented API guide covering LSK/SCLK/EOP kernel
  loading, WebGeocalc kernel parity, time-scale conversions, formatting
  and error handling.
- `examples/api_tour.rs`: a runnable example of the documented calls.
- Optional `mcp` feature and `stkcraft-mcp` stdio MCP server exposing
  UTC/TAI/TT/ET/GPS/UT1/SCLK conversions as JSON-RPC tools. The feature
  enables `serde_json` and is not part of the `no_std` build.
- `docs/MCP.md`: MCP client setup, protocol flow, tool schemas and
  examples.
- Project skills in `.agents/skills/stkcraft-time` and
  `.agents/skills/stkcraft-mcp` for API and MCP workflows.

### Changed

- The README **Kernel selection** section now explains what WebGeocalc
  calls a kernel, maps each kernel type to the corresponding
  `stkcraft-time` API, and links to the new API guide. A new MCP section
  documents the optional server.
- `AGENTS.md` now requires every update to increment the patch version and
  record a dated `CHANGELOG.md` section; `[Unreleased]` is no longer used.
- CI now also runs tests and Clippy with `--features mcp` and builds the
  MCP server.

### Fixed

- The `sclk-check` binary's usage comment no longer produces rustdoc
  invalid-HTML warnings.

## [0.1.1] - 2026-11-10

### Added

- GitHub Actions CI (`.github/workflows/ci.yml`) running the test suite,
  Clippy with `-D warnings`, and the `--no-default-features` (`no_std`)
  build on every push and pull request.
- `AGENTS.md` documenting the mandatory changelog and semantic-version
  rules.

### Fixed

- `convert::utc_to_tai` now rejects `23:59:59.x` on a shortened
  (86399-second) UTC day instead of mapping it to the same TAI epoch as
  the following day's `00:00:00`. Round trips through TAI are therefore
  no longer ambiguous at a negative leap second.
- `eop::EopTable::new` rejects non-finite MJD or `UT1 - UTC` samples with
  `TimeError::EopParse`. Sorting now uses `f64::total_cmp` instead of
  `partial_cmp(...).unwrap()`, so malformed input cannot panic.
- `TimeError::InvalidLeapSecond` now documents and reports both positive
  and negative leap-second boundary cases.

### Changed

- The README now states that the default leap-second policy infers
  nothing from a date, while `LeapSecondPolicy::FrozenAfter` is an
  explicit caller choice.
- Verification commands are documented as required checks in `AGENTS.md`.

## [0.1.0] - 2026-11-10

Initial release. NAIF/SPICE-aligned time scales for the STKCraft project.

### Scales and conversions

- UTC, TAI, TT, NAIF ET (TDB), GPS, UT1, TCG, TCB.
- `Epoch` stores integer TAI seconds plus nanoseconds since
  `2000-01-01T00:00:00 TAI`; no floating-point accumulation in epoch or
  duration arithmetic.
- Leap seconds are data: read from a NAIF LSK via `LeapSecondTable`.
  `from_lsk` selects an explicit kernel; `coverage` reports its range.
  No future leap-second date is hard-coded. `LeapSecondPolicy` can freeze
  `TAI - UTC` from a caller-supplied date.
- Pre-1972 UTC drift is available with `LeapSecondSource::Sofa` only; the
  default IERS-only source rejects pre-1972 UTC.

### UT1 and Earth orientation

- `eop::EopTable` parses the fixed-column IERS `finals2000A.all` product:
  Bulletin A (`UT1 - UTC` at columns 59-68, observed/predicted flag at
  column 58) and Bulletin B (columns 156-165). `EopSeries` selects
  `BulletinA`, `BulletinB`, or `FinalPreferred` (B where present, else A,
  matching astropy's combined column).
- `convert::ut1_minus_utc` removes the one-second leap-second step before
  interpolating, so the step is not smeared across a UTC day.
- No EOP data is bundled; `tools/fetch_eop.py` fetches it at deployment
  time.

### Dynamical and coordinate time

- IAU Fairhead & Bretagnon 787-term TDB model (`dtdb`), matching ERFA
  `dtdb` to about `5e-18 s` over 1950-2050.
- TCG and TCB per IAU 2000/2006, using the IAU TDB for TCB.
- NAIF `ET` remains available separately for SPICE interoperability.

### Spacecraft clocks

- `sclk::SclkKernel` parses `KPL/SCLK` type-1 kernels: fields, moduli,
  offsets, coefficients, time system and delimiter.
- Converts encoded ticks, field vectors, clock strings, ET and TAI.
- SCLK partition boundaries are parsed but not applied.

### Formatting

- ISO 8601, RFC 3339, RFC 2822 and C89 `strftime`-style formatting, plus
  `etcal`.

### Verification

The release criterion is numerical alignment with NASA NAIF/SPICE.

- 69 tests pass; `no_std` build passes; `clippy -D warnings` clean.
- UTC-to-ET against 82 CSPICE golden vectors: max `1.1e-12 s`.
- WGC2 live API: max `2.8e-7 s` (its output rounding); kernel selection
  reproduced with `naif0012`/`naif0011`/`naif0010`.
- SCLK against CSPICE `sct2e` on `MRO_SCLKSCET`: `1.5e-7 s` (f64 tick
  spacing).

Scales NAIF does not define are validated against their own authorities:

- UT1 against real IERS data: 20,037 days exact at 00:00 UTC, 25 leap
  days checked. The `FinalPreferred` series additionally agrees with
  astropy's IERS implementation to `1.1e-16 s` on ordinary days and `0 s`
  on leap days over 1,744 samples.
- TCG/TCB against ERFA over 950 TAI epochs (1972-2050): TT exact,
  TCG and TCB both `2.384e-7 s` (the f64 spacing at 1.6e9 s).

### Optional tooling

- Kani proofs in `src/verification.rs` are optional and not a release
  gate. Kani has no Windows build, so they have not run on the Windows
  development host; `.github/workflows/kani.yml` is dispatched manually
  on Linux. `tests/properties.rs` runs the same invariants as executed
  runtime property tests.

### Known limitations

- SCLK partition-aware reformatting is not implemented; only type 1 is
  supported.
- The full SPICE `str2et` token grammar is not implemented.
- UT1 requires external IERS EOP data at runtime.
