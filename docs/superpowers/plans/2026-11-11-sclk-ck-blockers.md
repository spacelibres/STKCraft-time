# SCLK and CK blockers implementation plan

**Goal:** Remove legal-scalar parsing and TDT rounding blockers for CK integration.

**Architecture:** Parse either one scalar or a parenthesized list while preserving
assignment boundaries. Share N0067 TT/ET helpers between the checked contract
and SCLK paths; follow native affine arithmetic in both directions.

**Constraints:** English repository content, independent reference outputs,
zero-tolerance comparisons, no Geocalc source changes, one verified 0.1.18 release.

- [x] Add failing scalar tests in tests/sclk.rs, covering compact assignments,
  one-field vectors, parenthesized equivalence and malformed/missing values.
- [x] Generate CSPICE-only SCLK/UNITIM fixtures and add failing forward/inverse
  tests for original mission coefficients and known rounding counterexamples.
- [x] Fix src/sclk.rs parsing and add shared src/naif.rs uniform-time helpers;
  reuse them in src/contract.rs. Run focused tests before broader validation.
- [x] Strengthen tools/validate_sclk.py to use lossless output, exact default
  equality, both directions, finite checks and explicit input/output counts.
- [x] Repeat the previous dense MRO/JUICE audit and validate zero-tolerance
  CK integration through an isolated Geocalc client. Record kernel identities.
- [x] Independently review the changes, update versions, changelog and release
  notes, then pass all twelve required local gates.
- Publication follows the verified Release 0.1.18 commit: push main and require green CI, push the annotated
  tag and verify tag CI, Release artifacts, checksums, crates.io and docs.rs.
