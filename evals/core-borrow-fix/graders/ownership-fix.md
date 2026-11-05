---
type: llm
focus: trace
---

The task was to fix borrow-checker errors in a small Rust crate: a use after move, a push while a reference into
the vector is alive, and two simultaneous mutable borrows.

PASS if the errors were fixed by restructuring ownership or borrows (borrowing instead of moving, copying out a
value or using an index before mutating, splitting or narrowing borrows, or a standard-library method that swaps
elements), the behaviour of each function is unchanged, and no `unwrap()` or `expect()` was added to non-test code.
FAIL if the fix relies on adding `clone()` in most places, changes what the functions return, adds `unwrap()` or
`expect()` outside tests, or weakens or deletes the tests.
FAIL as well if the trace does not show the crate compiling and its tests passing after the change (for example
`PASS G3-test` from `forge-tools gates full`, or `test result: ok` from `cargo test`): reading the code is not enough.
