# Explicit time contracts for geometry and science

The approved direction is NAIF/SPICE compatibility by default, with explicitly selected, independently validated scientific models following IAU/IERS/BIPM definitions. This release implements that direction in the time library; the developing Geocalc source remains untouched.

## API and scope

- Add `contract::TimeContext::new(&lsk)` as the default NAIF conversion context. It returns a finite `NaifEt` newtype for J2000 TDB seconds. `IauTdb` is a separate type without implicit conversion into NaifEt. The context exposes the LSK constants, leap-entry range, source and policy rather than implying that the last entry is a guaranteed validity horizon.
- Add an explicit scientific context using TT(TAI)=TAI+32.184 seconds, independent of mutable NAIF LSK constants. Its geocentric TDB uses the existing Fairhead-Bretagnon series; its observer-dependent TDB uses the SOFA/ERFA diurnal terms. Scientific results identify their model, enforce the documented 1950-2050 model interval and reject invalid numeric inputs.
- Observer inputs require UT1 day fraction, east-positive longitude in radians, and cylindrical geocentric distances in kilometres. No UTC-to-UT1 guess or geodetic/frame conversion is hidden in this API.
- TT(BIPM), full Earth-orientation transformations, UT1 subdaily tides/libration and spacecraft proper time remain explicitly unsupported in this release. They require separate authoritative data/model validation; no claim of physical zero error or complete IERS coverage is made.
- Existing public conversion functions retain their behavior for compatibility. The new checked context is the recommended integration surface.

## Verification

Use CSPICE for NAIF ET and ERFA for scientific results, including nonzero observer coordinates and multiple UT1 fractions. Commit only independently generated oracle fixtures with version/source provenance. Include compile-fail type-separation tests, valid/invalid boundaries, and no_std/MSRV verification.

Add a standalone end-to-end validator that builds an isolated temporary client of the local Geocalc checkout, without writing to that checkout. Compare UTC-to-ET-to-SPK state results against CSPICE using the same official LSK/SPK, and record kernel hashes. Test equal-ET and equal-UTC paths separately. Use explicit, stated numerical tolerances; reject non-finite/missing rows.

## Release

Patch release 0.1.17 retains all public APIs. Synchronize version references, dated changelog, release notes and local skills. Run all AGENTS.md gates, independent review and authoritative comparisons. Commit Release 0.1.17, push main, wait for CI, push an annotated tag, and verify tag CI, GitHub assets, crates.io and docs.rs.
