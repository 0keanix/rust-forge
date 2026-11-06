---
type: llm
focus: trace
---

The task was to expose the `NoteStore` of a pure Rust notes core crate (add, list, search) to Kotlin and Swift
through a separate ffi crate that already depends on uniffi, and to make it build for `aarch64-linux-android` and
`aarch64-apple-ios`.

PASS if the ffi crate stays a thin layer over the core (it wraps or delegates to the core's `NoteStore` instead of
reimplementing add, list or search), core errors are mapped to an error type exported across the FFI boundary
instead of panics or plain strings, the logic and its host tests stay in the core crate, and the change documents
the Android and iOS build commands (for example cargo-ndk for Android, and the iOS target plus an XCFramework for
iOS).
FAIL if note logic moves into the ffi crate, errors cross the boundary as panics, `unwrap` or plain strings, the
core loses its tests, or there are no Android and iOS build instructions.
Whether the code builds and the gates pass on the host is graded separately; do not judge it here.
