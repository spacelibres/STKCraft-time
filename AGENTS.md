# Project rules

These rules apply to every change made in this repository.

## Changelog is mandatory

- Every user-visible change must add an entry to `CHANGELOG.md` in the
  same commit.
- Use the Keep a Changelog sections: `Added`, `Changed`, `Deprecated`,
  `Removed`, `Fixed`, `Security`.
- While a version is unreleased, add the entry under `## [Unreleased]`.
  When preparing a release, rename that section to the new version and
  date.
- Describe the behaviour change and its user-visible impact; do not just
  repeat the commit subject.

## Versioning

- `stkcraft-time` follows Semantic Versioning (`MAJOR.MINOR.PATCH`).
- Every release increments at least the patch component.
- Choose the smallest necessary component:
  - `PATCH` for backward-compatible fixes, documentation, CI and tests.
  - `MINOR` for backward-compatible features.
  - `MAJOR` for breaking changes. While the major version is `0`, a
    breaking change still requires a minor bump and must be called out in
    the changelog.
- A release commit must update `Cargo.toml` (`package.version`),
  `Cargo.lock` and the corresponding `CHANGELOG.md` section to the same
  version.
- Tag releases as `vX.Y.Z` after the release commit.

## Verification before pushing

Run these commands and require success:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
cargo build --no-default-features
```

Pushing to `main` must keep the GitHub Actions `CI` workflow green. Fix
failures before declaring a change complete.

## Commit messages

- Imperative subject line, 50 characters or fewer.
- Separate the subject from the body with a blank line.
- Only add a body when it provides useful context, and wrap it at 72
  characters.
- Do not repeat information from the subject line in the body.
