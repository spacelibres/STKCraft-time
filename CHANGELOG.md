# Changelog

All notable changes to `stkcraft-time` are documented here.

The format follows Keep a Changelog. This crate uses semantic versioning.

## [0.1.32] - 2026-11-12

### Added

- Exact nominal BeiDou `Bdt` coordinates with checked integer parts, total
  nanoseconds, full signed weeks, UTC/LSK and checked ET conversion. Independent
  ERFA references cover the BDT origin, leap seconds and fractional boundaries.
  The existing public `TimeScale` enum and native NAIF algorithms are unchanged.
- Independent Python extension and WASM/JavaScript/TypeScript packages exposing
  owned time/kernel objects, exact integer transport, typed model selection,
  structured errors, formatting, EOP/UT1, SCLK, BDT, batches and lazy series.
  Public API inventories document language mappings and foreign-input limits.
- Wheel/sdist, Node/browser/Worker, TypeScript, bundler and clean-consumer
  verification jobs. Tagged releases verify binding artifacts before publication
  and attach wheels, the source distribution and npm tarball with checksums.
- A standalone Cortex-M4 UTC/ET example with independent reference checks,
  allocator setup, board memory layout and an on-target DWT cycle harness.
  Cross-linking and host checks do not claim board execution or hardware timing.

### Fixed

- The isolated Geocalc validator patches registry time dependencies to the
  selected local core, preventing duplicate Epoch/LSK types without rewriting
  the developing Geocalc checkout or lockfile.

### Changed

- Binding dependencies and their newer build-toolchain requirements are isolated
  from core Rust 1.75 and no_std dependencies. Exact integers use Python int and
  JS bigint; low-level floating compatibility paths remain explicitly named.
- Registry publication is conditional on configured PyPI/npm credentials;
  missing access is reported explicitly and does not imply published packages.

## [0.1.31] - 2026-11-12

### Added

- Add `contract::NaifDeltet` and `NaifUtcContext::from_raw_parts` for an
  explicitly selected, original-order structured native leap table. All five
  finite DELTET constants are supplied by the caller; no nominal defaults or
  scientific historical policy are inferred.
- Retain an independent four-profile, 7,968-query C capture covering unsorted
  dates, duplicates and invalid entries inside/outside native's 140-entry limit.

### Fixed

- Resolve issue #4's native validation boundary: raw structured tables can now
  reach ET-to-JDUTC and return `SPICE(BADLEAPSECONDS)` at calculation time.
  The combined 26,831-query corpus matches all 22,292 unrounded numeric outputs
  and 4,539 native error symbols, including the original 185 constructor-blocked
  cases. The old typed adapter differed on 8,153 records before the repair.
- Share the original UNITIM inverse/JDUTC arithmetic with the existing
  `TimeContext` route. Scientific/civil constructors keep their validation and
  public signatures; checked raw-date arithmetic prevents integer overflow.
- Document exact API/model boundaries, native evidence and Agent Skill usage.
  Arbitrary raw kernel variables and fractional dates/offsets remain outside
  this structured-entry contract.
- Check the API index recursively across library source groups so the new
  re-exported contract types remain documented and verified.

## [0.1.30] - 2026-11-12

### Added

- Explicit `TimeContext::et_to_spice_julian_utc` and checked `NaifJulianUtc`
  for native historical packet epochs without integer Epoch/civil-calendar
  quantization. Preserve N0067's historical offset, formal leap-second
  behavior, paired-table validation, operation order and 140-entry selection.
- Independent C captures and exact regressions for all 13 original
  integration inputs, 18,493 native values and 185 native error symbols.
  Preserve another 185 malformed-table records as earlier typed-constructor
  rejections, without claiming native symbol equivalence for that path.
- Explicit raw-model and invalid-native-table contract errors, with native
  source/capture provenance and an exact scope record.

### Changed

- Document the direct native JDUTC API, checked type and Agent Skill call
  path. Preserve existing strict civil/scientific APIs and their model
  separation; reject SOFA/frozen policies on the new raw-kernel path.
- Keep broader TTRANS representations, malformed raw-pool inputs and
  GeoCalc historical SPK10 integration outside this time-layer claim.

## [0.1.29] - 2026-11-12

### Changed

- Documentation alignment with the 0.1.28 kernel-date resolver. There is
  no library, kernel-format or numerical change.
- `README.md`, `docs/API.md`, `docs/TIME_CONTRACT.md` and the project wiki
  now state N0067 `TPARTV` token precedence explicitly instead of implying
  a `DD-MON-YY` reading: `@31-JAN-1987` is 1987-01-31, while `@19-JAN-87`
  is 2019-03-28. The API guide also states that a month name is any
  prefix of at least three letters and that a decimal is accepted only in
  the final numeric component of a literal.

### Fixed

- `docs/reference/KERNEL_DATE_EXACT.md` still described the 99 native-valid
  rejections as open. The 0.1.28 grammar repair resolved them; the
  document now separates the arithmetic repair it evidences from that
  later repair and keeps both claims bounded to the captured corpus.
- Wiki pages still advertised 0.1.26, the superseded 163/176 test counts
  and the previous claim that two-digit years are rejected. They now match
  the current release, the measured 165/178 counts, and the exercised
  grammar.
- Wiki `Troubleshooting`, `Design-and-Non-Goals` and `API-Map` no longer
  claim that no kernel date parser exists, and `Reproducing the
  comparisons` records the native kernel-date corpus and its commands.

## [0.1.28] - 2026-11-12

### Fixed

- Kernel-date parsing now follows native token precedence, the default 1969--2068 year window, month prefixes, mixed separators, date/clock normalization and the original integer calendar expression. The 99 native-valid rejections in issue #2, including the real RSSD FK epoch, now resolve to exact native values.
- Reject integer clock/year ambiguities and malformed ISO forms exposed by seventeen additional native counterexamples; retain decimal-token distinctions.
- Replace eight incorrectly classified rejection assumptions with independent native value checks for those same inputs. LSK and civil UTC validation remain strict and unchanged.
- Synchronize the citation release date with this dated release.

### Added

- A 416-input independent C regression: 345 exact native binary64 values and 71 explicit rejections, plus preserved original captures, source provenance, rounding boundaries and red/green logs.
- Documented parser limits and source attribution; complete TPARSE grammar conformance remains unclaimed.

## [0.1.27] - 2026-11-12

### Fixed

- Kernel-date fractional seconds now preserve N0067 TPARSE's day/hour/minute/second addition order. Seven supported literals previously differed by one binary64 ULP; their original native expectations now match exactly.

### Added

- An independent native regression observed failing for all seven counterexamples before the repair, with original C captures and source/toolchain provenance.
- Explicit scope records distinguishing corrected arithmetic from the 99 native-valid unsupported literals still tracked in issue #2. Complete TPARSE conformance is not claimed.

## [0.1.26] - 2026-11-12

### Fixed

- `kernel_date::parse_kernel_date` could reinterpret two malformed
  literals as plausible dates instead of rejecting them. A numeric year
  in the middle field (`@01-2000-02`) now fails, because a middle year
  would force a month-day-year reading that N0067 does not define. A
  fractional-second field with no digits (`@2000-JAN-01-12:00:.`) now
  fails as well; it previously parsed as an exact zero. Both return
  `KernelDateUnsupported`, and `tests/kernel_dates.rs` covers them.

## [0.1.25] - 2026-11-12

### Added

- `kernel_date::parse_kernel_date` resolves a SPICE text-kernel `@date`
  literal with N0067 TPARSE semantics. It returns the exact seconds past
  J2000 together with the normalized proleptic-Gregorian fields, and
  covers `Y-MON-D`, `D-MON-Y` and `MON-D-Y`, numeric month positions,
  full or abbreviated month names, `-` or `/` date separators, and an
  optional `HH:MM[:SS[.fff]]` time of day. A day past the end of its month
  rolls forward, as NAIF normalizes `@2001-FEB-29` to `2001-03-01`.
- `TimeError::KernelDateUnsupported` reports a literal the parser cannot
  resolve. It is deliberately separate from `LskMissing`/`SclkMissing`,
  so present but unsupported data is never reported as an absent kernel
  variable and never replaced by a default epoch.
- `tests/kernel_dates.rs` pins all eleven native literals from the
  STKCraft-Geocalc frame-kernel capture: the exact IEEE-754 bit patterns
  for the eight accepted values, and the explicit rejection of
  `@2000-060`, `@`, and `@NOT_A_DATE`.

### Fixed

- A text kernel's date constants had no public resolver, so a caller that
  needed an `EPOCH = @date` value could only classify the value as a
  missing numeric constant. Kernel date constants now have an explicit
  path and need no leap-second kernel, so a pre-1972 literal is an
  ordinary value rather than `HistoricalUtcRequiresExplicitModel`.
- Every existing kernel reader was re-audited for a silent date fallback.
  `parse_lsk` and `SclkKernel::parse` already fail loudly on a non-numeric
  token, and LSK `DELTET/DELTA_AT` dates remain strict: an impossible
  calendar day is `LskParse`, not a roll-forward.

## [0.1.24] - 2026-11-12

### Added

- `docs/API.md`, `docs/MCP.md` and `docs/TIME_CONTRACT.md` now open with
  the project mark and a link row (crates.io, API reference, source, wiki,
  STKCraft), so a reader who lands on a single guide page can still reach
  the registry and the full documentation set.

### Notes

- No library, kernel-format or numerical change. The project wiki uses the
  SVG brand mark on its home page, the raster mark in the sidebar, and the
  same link set in a footer.

## [0.1.23] - 2026-11-12

### Added

- `tests/api_index.rs` keeps the public API index in `docs/API.md`
  complete. It parses every `pub fn`/`struct`/`enum`/`const`/`type`/`trait`
  in `src/` and fails with the exact names when an item is missing from the
  index, or when the index lists a name that no longer exists. Crate-private
  and test-only items are ignored. `CONTRIBUTING.md` and the index heading
  now state that the list is enforced.

### Notes

- No library, kernel-format or numerical change.

## [0.1.22] - 2026-11-12

### Fixed

- `docs/API.md` claimed the crate bundles no leap-second data while also
  documenting the compiled-in `naif0012.tls` snapshot. The scope section
  now states that the snapshot is bundled and that mission SCLK and EOP
  data are not.
- `README.md` listed `fmod` among the `libm` functions used by `no_std`
  builds. The crate uses `sin`, `cos`, `floor` and `round`; the list is now
  correct.
- `docs/MCP.md` and the MCP skill stated that every tool accepts
  `lsk_path`. GPS tools and `etcal` take no leap-second kernel; the text
  now says which tools accept it.
- `docs/MCP.md` and the MCP skill did not mention that SCLK tools bound
  encoded ticks by the cumulative partition duration.

### Added

- The `docs/API.md` index now covers the `contract` module (`TimeContext`,
  `ScientificContext`, `NaifEt`, `IauTdb`, `NaifMetadata`, `ContractError`),
  the `error` and `mcp` modules, `lsk::parse_lsk`, `SclkClock`/`SclkMap`,
  `DtdbError` and the observer `dtdb` functions, and
  `Formatter::with_options`.
- `docs/API.md` names the three documentation sources and their roles, so
  the in-repository contract, the model contracts and the wiki cannot drift
  silently.

### Notes

- No library, kernel-format or numerical change. The SCLK figures in the
  wiki now match the committed fixtures: 10 871 bit-exact CSPICE cases
  across the MRO and JUICE kernels, not the earlier 64-sample tolerance
  check.

## [0.1.21] - 2026-11-12

### Added

- `README.md` links to the project wiki, which documents the time-scale
  definitions, kernel formats, leap-second rules, SCLK model, TDB models,
  `no_std` support, the MCP server and the verification results with
  reproduction commands.

### Notes

- No library, kernel-format or numerical change.

## [0.1.20] - 2026-11-11

### Removed

- The container image, `Dockerfile`, `.dockerignore`, the `publish-image`
  workflow job and the related README, MCP guide and contributing
  documentation. The `ghcr.io/spacelibres/stkcraft-time` package was
  deleted. 0.1.19 is yanked on crates.io; its only change was the
  withdrawn feature.

## [0.1.19] - 2026-11-11 (yanked)

### Added

- A `Dockerfile` and a `publish-image` job. Every tag builds
  `linux/amd64` and `linux/arm64` images at
  `ghcr.io/spacelibres/stkcraft-time`, tagged `X.Y.Z` and `latest`, with
  `stkcraft-mcp` and the four validation binaries. The image runs as UID
  10001, uses `/data` as the working directory and performs no network
  I/O, so the MCP server can be launched without a Rust toolchain.
- `README.md` and `docs/MCP.md` document the container invocation and an
  MCP client configuration that starts the image.

Withdrawn in 0.1.20.

## [0.1.18] - 2026-11-11

### Fixed

- SCLK parsing accepts legal numeric scalar assignments with or without
  parentheses, including one-field moduli, offsets and partition bounds.
  Kernel integer fields accept NAIF decimal/exponent notation; formatted
  clock strings retain their integer-field validation.
- TDT clock conversions preserve original parallel TT and use direct N0067
  UNITIM arithmetic in both directions. Removing the intermediate integer
  Epoch eliminates observed subnanosecond and larger rounding discrepancies
  that could make zero-tolerance CK attitude queries miss valid coverage.
- SCLK affine conversions preserve native reciprocal-rate evaluation and
  modulus accumulation order, including nonbinary tick scales.

### Added

- 10,871 independently generated CSPICE forward/inverse SCLK references,
  original NASA MRO and ESA JUICE test kernels with source hashes, and
  scalar/malformed-input and rounding-counterexample regressions.

### Changed

- SCLK validation uses lossless numeric output and the selected LSK in both
  engines, checks both directions, rejects malformed/non-finite/missing
  output, and defaults to bit-exact comparisons with separate explicit
  seconds/ticks tolerances. Passing a loose threshold no longer implies
  zero-difference compatibility.

## [0.1.17] - 2026-11-11

### Added

- Checked `TimeContext` defaults to the NAIF/SPICE contract with distinct
  `NaifEt` and scientific `IauTdb` types. UTC and numeric TAI entry points
  reproduce the corresponding N0067 TTRANS/UNITIM input arithmetic.
  Kernel settings and model identifiers make the selected convention explicit.
- `ScientificContext` uses TT(TAI) independently of mutable LSK constants,
  preserves split-date scientific evaluation and rejects epochs outside the
  documented 1950-2050 model interval, including nanosecond boundary cases.
- Observer-dependent SOFA/ERFA TDB-TT terms with supplied UT1 fraction,
  east longitude and cylindrical observer coordinates. Invalid inputs are
  rejected; 724 independently generated ERFA vectors exercise the model.
- A source-pinned generator and 910 independent CSPICE UTC input vectors,
  including fractional-second and leap boundaries. An isolated Geocalc
  validator separates equal-ET, UTC and numeric-TAI reference paths and
  records kernel/source hashes without changing the developing checkout.

### Fixed

- The checked civil-time inverse rejects dates beyond the calendar's i32
  year range rather than silently wrapping the year.
- Coordinate-scale documentation now states the exact affine rate factors
  used by the implementation and identifies UT1 as a time scale.

### Changed

- New integration guidance distinguishes SPICE compatibility, scientific
  model accuracy and application error budgets. TT(BIPM), IERS subdaily
  corrections, full Earth orientation and spacecraft proper time remain
  explicitly outside the implemented contract.

## [0.1.16] - 2026-11-11

### Fixed

- Historical UTC conversions in SOFA mode now handle positive and negative
  fractional mini-leaps and day-rate changes. The inverse preserves split
  epoch precision and the civil date of extended final minutes; historical
  formatting also uses the correct fractional day-end limit.
- Encoded SCLK/time conversion rejects ticks before the first coefficient
  or beyond the cumulative partition domain, including clock resets and
  NAIF's rounding of partition readouts. Valid extrapolation inside the
  last partition remains available.
- Incomplete LSK files return the missing DELTET variable instead of
  silently substituting nominal constants, including through MCP tools.
- Text-kernel decimal arithmetic now follows NAIF NPARSD operation order.
  This fixes wrong-segment selection at some official ESA JUICE clock
  coefficient boundaries, where ordinary decimal parsing could produce
  time differences of about 0.666 seconds.
- Compact TDB documentation now identifies the NASA HEASARC source and
  removes unsupported claims of improved accuracy. UT1 documentation
  explicitly identifies daily interpolation, omitted subdaily corrections
  and unpropagated EOP uncertainty.

### Added

- Release binary archives include `ATTRIBUTION.md`, retaining the numeric
  parser's source provenance and original NAIF notice with the binaries.
- Reproducible PyERFA historical UTC vectors at every 1960-1972 model
  transition, with positive/negative adjustment boundaries and nanosecond
  precision checks. SCLK validation now checks every coefficient boundary,
  encoded-domain endpoints and out-of-domain rejection against CSPICE.

## [0.1.15] - 2026-11-11

### Added

- `tests/version_sync.rs` fails the build when `README.md`, `docs/API.md`,
  `CITATION.cff` or the bug report template references any version other
  than the one in `Cargo.toml`, and when an install snippet stops pinning
  that exact version. Release bookkeeping can no longer silently leave a
  page advertising the previous version.

## [0.1.14] - 2026-11-11

### Fixed

- The `Release` workflow's crates.io lookup sent no `User-Agent`, so the
  API answered `403` and the job fell through to `cargo publish` on a
  version that was already published, failing the tag run. The check now
  sends a `User-Agent` and publishes only on an authoritative `404`,
  failing loudly on any other status.

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
