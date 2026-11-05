# LLM antipatterns in Rust

Each entry: what it looks like, how to detect it, how to fix it.

## Redundant `clone()`
Detect: `.clone()` added right where a borrow error was, or clones of values that are not used again.
Fix: borrow instead, move into the last user, or restructure ownership (see error-codes.md).

## `Arc<Mutex<_>>` by default
Detect: shared mutable state wrapped in `Arc<Mutex<_>>` with a single owner or a single writer.
Fix: give the data one owner and pass messages or `&mut`; use shared locking only when several tasks or threads
really mutate the same data.

## Blocking calls in async
Detect: file or network I/O from the standard library, `std::thread::sleep` or heavy CPU work inside `async fn`.
Fix: use the runtime's async equivalents, or move blocking work to the runtime's blocking-task facility.

## Guards held across `.await`
Detect: a lock guard or `RefCell` borrow alive in a scope that contains `.await`; often surfaces as a `Send` error.
Fix: copy out what you need and drop the guard in an inner scope before the `.await`.

## `unwrap()` / `expect()` in non-test code
Detect: `cargo clippy` with `unwrap_used` and `expect_used` denied; grep outside test modules.
Fix: return `Result` and propagate with `?` and context; keep `expect` only for invariants with a stated reason.

## Outdated or invented crate APIs
Detect: E0599, E0425 or E0433 on crate items; methods that do not appear in the docs for the version in
`Cargo.lock`.
Fix: read the docs for the locked version and use what exists there; never guess a signature.

## Tests that mirror the implementation
Detect: expected values computed by the same logic as the code; assertions that only restate a constant from the
implementation; tests that cannot fail.
Fix: derive expectations from the spec or examples; check behaviour at the public boundary; use properties for
invariants.

## `#[allow]` without reason
Detect: `#[allow(...)]` with no `reason = "..."`, especially on a lint that just fired.
Fix: fix the code; if the allow is justified, prefer `#[expect(..., reason = "...")]` so it is removed once
unneeded.

## Weakened or deleted tests
Detect: in the diff, removed assertions, `#[ignore]` added, loosened comparisons, or deleted test functions.
Fix: restore the test and fix the code; change a test only when the spec changed, and say so.
