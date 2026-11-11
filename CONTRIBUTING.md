# Contributing

## Before you start

- Read [`AGENTS.md`](AGENTS.md). It is authoritative for versioning,
  changelog, release and commit rules.
- All repository content must be in English.
- Open an issue before a non-trivial change.

## Requirements

- Rust 1.75 or newer.
- `cargo`, `rustfmt` and `clippy`.

## Development checks

Run the full matrix before opening a pull request:

```sh
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --features mcp
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features mcp -- -D warnings
cargo build --no-default-features
cargo build --release --features mcp --bins
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
```

## Pull requests

- Keep the change focused and explain the behaviour change.
- Add regression tests for boundary conditions, especially leap seconds,
  EOP validation, UT1 coverage and SCLK parsing.
- Update the API guide in `docs/API.md` and the MCP guide in
  `docs/MCP.md` when the public surface changes.
- Update `CHANGELOG.md` under a new dated version section. Do not use
  `[Unreleased]`.
- Increment the patch version in `Cargo.toml`, `Cargo.lock` and the
  changelog.
- Ensure the GitHub Actions `CI` workflow passes.

## Release process

Every update is released. See the "Automatic release and push" section in
[`AGENTS.md`](AGENTS.md). A release includes:

1. A patch version bump and a dated changelog section.
2. A `Release X.Y.Z` commit.
3. A `vX.Y.Z` tag.
4. A GitHub Release with `docs/releases/vX.Y.Z.md` and release-mode
   binaries for Linux, macOS and Windows.

## crates.io publication

The crate is published under the same version as the Git tag. Publishing
is not part of CI; it needs an API token from
<https://crates.io/settings/tokens>.

```sh
cargo login          # once per machine
cargo publish --dry-run
cargo publish
```

`cargo publish` requires a clean working tree and the `vX.Y.Z` tag.
Verify the result on <https://crates.io/crates/stkcraft-time> and wait
for the docs.rs build at <https://docs.rs/stkcraft-time>.

## Commit messages

- Imperative subject line, 50 characters or fewer.
- Separate the subject and body with a blank line.
- Add a body only when it provides useful context, wrapped at 72
  characters.
- Do not repeat the subject line in the body.
