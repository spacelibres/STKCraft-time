# Native raw JDUTC implementation plan

> **For agentic workers:** Use executing-plans to implement the tasks in this session.

**Goal:** Resolve Time issue #4 with native calculation-stage error parity and preserve issue #3's exact historical values.

**Architecture:** Borrow structured entries and explicit constants through a dedicated raw NAIF context. Reuse one numeric JDUTC/UNITIM path; leave scientific constructors unchanged.

**Tech Stack:** Rust 2021, MSRV 1.75, no_std, Python evidence checks, independent N0067 C oracle.

## Global constraints

- English maintained content; patch 0.1.31 with synchronized metadata and dated changelog.
- No changed native expectation, tolerance, input adjustment or reordered raw entry.
- Original 13-profile capture remains immutable; broader raw-pool/time representations remain open.
- All repository-mandated checks, main/tag CI and actual release/registry/docs verification must pass.

## Task 1: Independent regression

- Capture unsorted dates, duplicate dates and selected/ignored capacity-tail inputs with `tools/gen_native_jdutc_raw.py`, using the unchanged C oracle and pinned native compiler helper.
- Add `tests/native_jdutc_raw_exact.rs`: parse every original and new native record, construct the raw context and compare every output bit or exact SHORT symbol.
- Run `cargo test --test native_jdutc_raw_exact -- --nocapture` before implementation and preserve the absent-API failure and immutable native evidence.
- Preserve the existing typed constructor's 185 `LskParse` checks separately.

## Task 2: Shared native arithmetic and explicit API

- Create `src/contract/naif_utc.rs` for `NaifDeltet` and `NaifUtcContext`; re-export them from `src/contract.rs`.
- `NaifUtcContext::from_raw_parts(entries: &[LeapEntry], deltet: NaifDeltet)` returns `Result<Self, ContractError>` and rejects only an empty entry slice or nonfinite constants. It preserves entry order and defers paired-table validation to conversion.
- `NaifUtcContext::et_to_spice_julian_utc(NaifEt)` returns `Result<NaifJulianUtc, ContractError>`.
- Refactor `src/naif/utc.rs` to consume the borrowed entries and five explicit constants. Share inverse arithmetic in `src/naif.rs`; check integer subtraction before accepting an out-of-domain raw date.
- Run both JDUTC test targets and require all native values and errors to match exactly.

## Task 3: Documentation, review and release

- Update README, API, TIME_CONTRACT, NATIVE_JDUTC scope and the Agent Skill with raw-versus-scientific constructor semantics and an explicit usage example.
- Preserve complete raw captures, provenance and before/after logs; add Python integrity validation.
- Synchronize Cargo.toml, Cargo.lock, CHANGELOG.md, CITATION.cff and docs/releases/v0.1.31.md.
- Run all 13 release checks, Python evidence validation and independent review; freeze sources during the final gate.
- Commit `Release 0.1.31`, push main, require successful exact-commit CI, then create the annotated tag and verify all actual assets, crates.io and docs.rs.
- Close #4 with exact evidence after publication. Complete #3's separate GeoCalc CI/publication verification before closing it.
