# Astronomical audit fixes implementation plan

> Execute task by task with test-driven development and independent review. The repository release rules govern commits and publication.

**Goal:** Resolve the reproduced historical UTC, SCLK-domain and incomplete-LSK defects without changing supported models or inventing reference data.

**Architecture:** Preserve the public API. Use SOFA historical day-rate and mini-leap conventions with split integer epochs, validate encoded SCLK bounds from NAIF partition durations, and require every DELTET constant when parsing an LSK.

**Tech stack:** Rust 1.75+, no_std + alloc, Python ERFA/CSPICE reference tooling.

## Global constraints

All repository content is English. Release 0.1.16 increments the patch version, uses a dated CHANGELOG section and docs/releases/v0.1.16.md, and commits as Release 0.1.16. Run every AGENTS.md verification gate before pushing to main. Wait for main CI, annotated-tag CI, release assets, crates.io and docs.rs. No unverified model expansions or invented authority data.

## Tasks

- [x] Historical UTC: extend tests/pre_1972.rs and add reproducible ERFA-generated transition vectors, first demonstrating failure. Fix src/convert.rs day validation and inverse interval selection. Cover positive/negative mini-leaps, 1960/1972 endpoints and precision. Update historical formatting if required.
- [x] SCLK: add domain boundary regressions in tests/sclk.rs and confirm failure. Fix src/sclk.rs forward/inverse domains using cumulative partition lengths; retain valid extrapolation within the last partition. Compare NASA MRO and ESA JUICE reference kernels.
- [x] LSK: remove individual required assignments from the real bundled kernel in tests/kernel_validation.rs and confirm rejection tests fail. Require DELTA_T_A, K, EB and M in src/lsk.rs; update intentionally synthetic parser fixtures to supply complete constants.
- [x] Documentation: correct the compact TDB model citation and precision claims in src/tdb.rs, README.md and docs/API.md; state UT1 interpolation, EOP uncertainty and omitted observer-dependent terms explicitly. Synchronize affected local skills and all current-version references.
- [x] Release preparation: dated changelog/release notes, targeted independent comparisons, all twelve required build gates and independent diff review completed.

Publication follows the release commit: push main and verify CI, create/push the annotated tag and verify tag CI, then confirm GitHub assets, crates.io and docs.rs. Report any external blocker accurately.

## Verification commands

Use cargo test --test pre_1972, cargo test --test sclk and cargo test --test kernel_validation for red/green cycles. Generate authority vectors with the installed ERFA/CSPICE Python environment; retain upstream provenance. Run the complete twelve-command AGENTS.md gate and compare official MRO/JUICE/IERS inputs before publication.
