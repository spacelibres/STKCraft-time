# BeiDou and complete language interfaces implementation plan

User approved the full scope and independent-adapter design on 2026-11-12.
Use test-driven implementation and independent task/final reviews. Repository
release rules override per-task commits: integrate one verified 0.1.32 release.

- [x] Add exact checked BDT type/conversions and independent reference tests;
  preserve every existing public enum and NAIF arithmetic implementation.
- [x] Implement Python classes, functions, errors, complete API inventory,
  typing, batches, wheel/sdist and runtime reference tests in bindings/python.
- [x] Implement WASM/JS/TS equivalents, input validation before ABI narrowing,
  Node/browser/consumer packaging tests in bindings/wasm.
- [x] Independently review each completed task, fixing concrete blockers.
- [x] Record API/support/validation matrix and exercise unchanged Geocalc
  calls through an isolated consumer; retain source/kernel identities.
- [x] Integrate core, Python and WASM/JS CI, common version and artifact
  release workflow; verify clean-install wheel/sdist/npm artifacts.
- [x] Pass all twelve mandatory core checks plus language tests and independent
  adapter review. Python passes 253 tests; Node and Chrome pass 79,520
  assertions each. Original core sources and Geocalc contracts are preserved.

## Release execution

After final integration review, commit Release 0.1.32, push main, wait for CI,
tag, and verify GitHub artifacts, crates.io/docs.rs, PyPI and npm publication
where access is configured. The GitHub Actions runs and registry responses
record publication outcomes; local package tests do not prove publication.
PyPI requires a configured API token or trusted publisher, not an account
password. npm registration/namespace access is pending. Report actual external
publication blockers explicitly.
