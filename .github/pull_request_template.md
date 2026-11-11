## Summary

Describe the behaviour change and why it is needed.

## Verification

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo test --all-targets`
- [ ] `cargo test --all-targets --features mcp`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo clippy --all-targets --features mcp -- -D warnings`
- [ ] `cargo build --no-default-features`
- [ ] `cargo build --release --features mcp --bins`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps`

## Release checklist

- [ ] Patch version incremented in `Cargo.toml` and `Cargo.lock`.
- [ ] Dated section added to `CHANGELOG.md` (no `[Unreleased]`).
- [ ] API/MCP documentation updated when the public surface changes.
- [ ] Regression tests added for boundary conditions.
- [ ] All repository text is English.
