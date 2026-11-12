# WebGeocalc GPS Week/TOW Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the observed WGC GPS-seconds to week/TOW output path with preserved native operation order.

**Architecture:** Keep the new compatibility function separate from exact integer time representations. Use the existing NAIF UNITIM arithmetic and an explicit LSK, then split and format as observed. Bind Python directly to the Rust core.

**Tech Stack:** Rust 1.75, no_std + alloc, PyO3, captured WGC2/CSPICE references.

## Global Constraints

- Preserve raw WGC response values and their hashes; do not use tolerances.
- Keep legacy APIs and the supported no_std/MSRV contract.
- Declare finite test coverage and unresolved input/format behavior explicitly.
- Synchronize patch version, English changelog, release notes and bindings.

### Task 1: Independent reference corpus

**Files:** `tests/reference/wgc-gps/*`, `tests/wgc_gps_reference.rs`

- [x] Capture 354 GPS-seconds inputs and output strings from WGC2.
- [x] Record kernel set metadata and LSK comparison, with source hashes.
- [x] Show that direct float formatting fails a native boundary case.
- [ ] Add native UNITIM binary64 cross-checks and hash verification.

### Task 2: Rust compatibility path

**Files:** `src/wgc_gps.rs`, `src/lib.rs`, `tests/wgc_gps_reference.rs`

- [ ] Add an explicit checked function for the observed GPS-seconds output.
- [ ] Preserve native UNITIM order, signed zero and no carry after formatting.
- [ ] Compare every captured row without tolerance; test invalid inputs.
- [ ] Document extrapolation and unsupported inputs without broader claims.

### Task 3: Python and inventories

**Files:** `bindings/python/src/lib.rs`, `bindings/python/python/stkcraft_time/*`, `bindings/python/tests/*`, `bindings/wasm/API_INVENTORY.md`

- [ ] Bind the Rust function without reimplementing numerical arithmetic.
- [ ] Test the installed Python wheel against the same NASA output corpus.
- [ ] Regenerate inventories and label unsupported JavaScript capabilities.

### Task 4: Release verification

**Files:** version metadata, `CHANGELOG.md`, `docs/releases/v0.1.34.md`

- [ ] Complete independent review, required Rust gates and adapter tests.
- [ ] Commit and publish only after earlier 0.1.33 integration is verified.
- [ ] Verify CI, tag, release assets, crates.io, PyPI and docs.rs.
