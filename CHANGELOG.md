# Changelog

All notable changes to `stkcraft-time` are documented here.

The format follows Keep a Changelog. This crate uses semantic versioning.

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
