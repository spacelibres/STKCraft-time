# Project rules

These rules apply to every change made in this repository.

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
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

Pushing to `main` must keep the GitHub Actions `CI` workflow green. Fix
failures before declaring a change complete.

## Commit messages

- Imperative subject line, 50 characters or fewer.
- Separate the subject from the body with a blank line.
- Only add a body when it provides useful context, and wrap it at 72
  characters.
- Do not repeat information from the subject line in the body.
