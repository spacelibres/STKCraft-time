# Changelog

All notable changes to `stkcraft-time` are documented here.

The format follows Keep a Changelog. This crate uses semantic versioning.

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
