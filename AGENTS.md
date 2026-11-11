# Project rules

These rules apply to every change made in this repository.

## Language

- All repository content must be in English: code, comments,
  documentation, changelog entries, commit messages, skills and example
  output.
- Do not add Chinese text or Chinese-localized UI labels.

## Changelog is mandatory

- Every update must add an entry to `CHANGELOG.md` in the same commit.
- Use the Keep a Changelog sections: `Added`, `Changed`, `Deprecated`,
  `Removed`, `Fixed`, `Security`.
- Every update is versioned immediately: record it under a dated section
  such as `## [0.1.2] - 2026-11-10`. Do **not** leave changes under
  `[Unreleased]`; the changelog must make it obvious which version the
  update belongs to.
- Describe the behaviour change and its user-visible impact; do not just
  repeat the commit subject.

## Versioning

- `stkcraft-time` follows Semantic Versioning (`MAJOR.MINOR.PATCH`).
- Every update increments the **patch** component by one
  (`0.1.1` -> `0.1.2` -> ...). The release commit must update
  `Cargo.toml` (`package.version`), `Cargo.lock` and the corresponding
  `CHANGELOG.md` section to the same version.
- An explicitly requested feature release may use a minor bump, and a
  breaking change may use a major bump. While the major version is `0`, a
  breaking change still requires a minor bump and must be called out in
  the changelog.
- Tag releases as `vX.Y.Z` after the release commit.

## Verification before pushing

Run these commands and require success:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --features mcp
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features mcp -- -D warnings
cargo build --no-default-features
cargo build --release --features mcp --bins
cargo test --release --features mcp
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Pushing to `main` must keep the GitHub Actions `CI` workflow green. Fix
failures before declaring a change complete.

## Automatic release and push

- After every update, complete the release without waiting for a separate
  instruction to commit or push:
  1. Increment the patch version and update `Cargo.toml`, `Cargo.lock`
     and the dated `CHANGELOG.md` section.
  2. Run the verification commands above.
  3. Commit with the subject `Release X.Y.Z`.
  4. Push the commit to `main`.
  5. Wait for the GitHub Actions `CI` workflow to pass. If it fails, fix
     the failure and push again.
  6. Create and push the annotated tag `vX.Y.Z`, then wait for the tag
     CI run to pass.
  7. Confirm the `Release` workflow publishes a GitHub Release with
     release-mode binaries for Linux, macOS and Windows and
     `SHA256SUMS.txt`.
- Every release adds `docs/releases/vX.Y.Z.md`; the `Release` workflow
  uses it as the release body.
- Do not leave the working tree with uncommitted changes when the task is
  complete.
- Skip only when the user explicitly says not to commit, not to push, or
  not to tag. If credentials or network access block the push, report the
  blocker and the exact commands needed.

## GitHub metadata

Keep these repository settings current:

- Description: `NAIF/SPICE-aligned astronomical time scales in Rust:`
  `UTC, TAI, TT, TDB/ET, GPS, UT1 and SCLK, with LSK/SCLK/EOP kernels,`
  `no_std support, WebGeocalc parity and an MCP server.`
- Homepage: <https://www.stkcraft.com>
- Topics: `rust`, `time`, `astronomy`, `spice`, `naif`, `utc`, `tai`,
  `tdb`, `ut1`, `gps`, `sclk`, `no-std`, `mcp`, `spacecraft`,
  `astrodynamics`, `leap-seconds`, `iers`, `ephemeris`,
  `time-conversion`, `rust-crate`.

## Commit messages

- Imperative subject line, 50 characters or fewer.
- Separate the subject from the body with a blank line.
- Only add a body when it provides useful context, and wrap it at 72
  characters.
- Do not repeat information from the subject line in the body.
