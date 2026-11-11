# Time contracts implementation plan

The user approved the NAIF-default / explicit scientific-model direction. Use test-driven development and independent review for each task; the repository's single versioned release commit governs integration.

- [x] Add checked model-specific types and NAIF context in src/contract.rs, tests/time_contract.rs; first reproduce implicit interchangeability and checked-conversion requirements in failing tests. Keep existing conversion APIs compatible.
- [x] Add observer-dependent SOFA/ERFA dtdb terms and split-date evaluation in src/dtdb.rs, independent observer golden vectors and generator; fail tests first and verify geocentric equivalence.
- [x] Add explicit scientific context with TT(TAI), geocentric/observer TDB, model-domain checks and metadata. Validate against independent ERFA and reject invalid input/overflow.
- [x] Add tools/validate_geocalc_time.py and tool tests. Build a temporary integration client, compare same-ET and same-UTC states with CSPICE, retain hashes, and leave the Geocalc checkout unchanged.
- [x] Document contracts, units, origins, scientific model bounds and unsupported corrections. Update version 0.1.17, changelog, release notes and current-version references.
- [x] Run all twelve release gates and authoritative comparisons, and independently review the whole diff. All gates passed and independent review approved the scope before the release commit.
- Publication follows the verified release commit: push main, require green CI, push the annotated tag, then verify tag CI, Release assets, crates.io and docs.rs.
