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

The crate is published under the same version as the Git tag. The
`Release` workflow runs `cargo publish` when the
`CARGO_REGISTRY_TOKEN` repository secret is set, and skips the job when
it is not or when that version already exists on the registry.

To publish by hand:

```sh
cargo login          # once per machine, or set CARGO_REGISTRY_TOKEN
cargo publish --locked
```

`cargo publish` requires a clean working tree. Verify the result on
<https://crates.io/crates/stkcraft-time> and wait for the docs.rs build
at <https://docs.rs/stkcraft-time>.

## Container image

Every tag also builds `ghcr.io/spacelibres/stkcraft-time` for
`linux/amd64` and `linux/arm64`, tagged `X.Y.Z` and `latest`. The image
contains `stkcraft-mcp` and the four validation binaries; the default
command is the MCP server.

Build it locally with:

```sh
docker build -t stkcraft-time:dev .
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"smoke","version":"0"}}}' \
    | docker run --rm -i stkcraft-time:dev
```

The GitHub Packages entry must be public; set it once with
`gh api --method PATCH users/spacelibres/packages/container/stkcraft-time -f visibility=public`.
GitHub Packages has no Cargo registry, so the library itself is only
published to crates.io.

## Commit messages

- Imperative subject line, 50 characters or fewer.
- Separate the subject and body with a blank line.
- Add a body only when it provides useful context, wrapped at 72
  characters.
- Do not repeat the subject line in the body.
