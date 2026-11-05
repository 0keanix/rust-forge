# Testing

- Unit tests live next to the code in a `#[cfg(test)]` module and may test private items.
- Integration tests live in `tests/` and use only the public API; each file is a separate test binary, so share
  helpers through a module directory rather than many top-level files.
- Doctests keep documented examples correct; run them with `cargo test --doc` (gate G4).
- Use property tests (`proptest`) for invariants that must hold over many inputs: round-trips, ordering,
  idempotence. Use snapshot tests (`insta`) for large structured output. Read the docs for the version in
  `Cargo.lock` before using either.
- Write each test from the spec: name the behaviour, set up input, assert the observable result. A test that
  cannot fail for a plausible bug is not worth keeping.
- Tests must not touch the network or write into the repository; use a fresh temp directory and remove it.
- Slow `cargo test`: most time is usually compilation and linking, not running. Check with `cargo test --timings`
  or by timing a second run. Reduce test binaries by merging integration test files, avoid rebuilding with
  different feature sets, and use `cargo nextest` (gate G3 uses it when installed) for parallel execution per test.
- Never weaken, ignore or delete a failing test to get green; fix the code or report the spec conflict.
