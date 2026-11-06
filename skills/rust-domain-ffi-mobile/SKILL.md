---
name: rust-domain-ffi-mobile
description: Guides exposing Rust to Kotlin, Swift and C from projects that use uniffi or cbindgen - a thin FFI crate over a pure Rust core crate, UniFFI proc-macro or UDL bindings chosen per version, C headers with cbindgen, error and ownership mapping across the boundary, cross-compiling for Android with cargo-ndk and for iOS into an XCFramework, and what to verify on the host versus on a device or simulator. Use it whenever a Rust project depends on uniffi or cbindgen, or the user asks to expose Rust to Android, iOS, Kotlin, Swift or C, build for aarch64-linux-android or aarch64-apple-ios, or ship a mobile library, even if they only say "the bindings", "the ffi crate" or "the mobile app". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: uniffi=0.32.2
---

# rust-domain-ffi-mobile

Domain guidance for Rust behind a foreign-language boundary (Kotlin, Swift, C). The core `rust-forge` skill owns
the working loop, the gates, the crate policy and compiler-error strategies; follow it and use this skill only for
what is specific to FFI and mobile builds.

## 1. Pin the version first

Run `forge-tools domains` and read the `ffi-mobile` line: `crate=uniffi version=<x>`. Read docs.rs and the UniFFI
user guide for exactly that version before writing code. UniFFI changes its macros, its bindgen command and its
supported types between minor releases (0.x), and version drift is the main source of wrong FFI code. If the
output says `version=unknown`, the lock file is missing; take the requirement from `Cargo.toml` and resolve it
before relying on details.

The UniFFI names below were checked against uniffi 0.32.2. For another version, confirm each name in that
version's docs before using it. Commands for cbindgen, cargo-ndk and Apple tools are given as procedures; check
their `--help` and documentation for the installed version before running them.

## 2. Layout: thin FFI over a pure core

- Keep all logic in a core crate with no FFI dependency and its own unit tests. It builds and tests on any host,
  and the bindings cannot hide bugs in it.
- Put the boundary in a separate ffi crate that depends on the core and only converts types, maps errors and
  forwards calls. If a function in the ffi crate has a branch that is not about conversion, move it to the core.
- Give the ffi crate `crate-type = ["lib", "cdylib", "staticlib"]`: `cdylib` for Android `.so` files and for
  library-mode bindgen, `staticlib` for iOS, `lib` so Rust tests and tools can link it.
- Do not add FFI attributes or derives to core types to save a wrapper. Mirror them in the ffi crate with
  `From` conversions; the core stays free of binding concerns and the exported surface stays deliberate.

## 3. UniFFI: proc-macro or UDL

- On current versions prefer proc-macros: call `uniffi::setup_scaffolding!()` once in the ffi crate's `lib.rs`,
  mark exported functions and `impl` blocks with `#[uniffi::export]`, and derive `uniffi::Record` (plain data
  passed by value), `uniffi::Enum`, `uniffi::Object` (a handle with methods) and `uniffi::Error`.
- Use UDL only when the project already does, or the pinned version lacks a feature you need in proc-macros. A UDL
  project keeps a `.udl` file, generates scaffolding from a build script with the `build` feature, and pulls it in
  with `uniffi::include_scaffolding!`. Do not mix the two styles for one type.
- Generate bindings in library mode from the built `cdylib` (`generate <lib> --language kotlin|swift
  --out-dir <dir>`; in 0.32 a library path is detected and `--library` is deprecated), through a small
  `uniffi-bindgen` binary that calls `uniffi::uniffi_bindgen_main()` with the
  `cli` feature, as the user guide for the version describes. Library mode reads the metadata compiled into the
  library, so the bindings match the code that ships.
- Generated Kotlin and Swift files are build outputs. Regenerate them in the build script or documented commands
  instead of editing them by hand.

## 4. Ownership across the boundary

- Exported objects live behind `Arc` and are shared with the foreign side, which may call them from any thread.
  Methods take `&self`, so wrap mutable core state in a `Mutex` (or `RwLock`) inside the object and keep the lock
  scope to one core call.
- Records are copied across on every call. Use them for small values (a note, a result); for large or long-lived
  state, return an object handle instead of copying it back and forth.
- Do not hand out references or raw pointers into Rust memory; return owned values. The foreign garbage collector
  or reference counter cannot see Rust lifetimes.

## 5. Errors across the boundary

- Define one FFI error enum in the ffi crate deriving `uniffi::Error` (and `Debug` plus `Display`, for example
  with `thiserror`), with a `From<CoreError>` conversion, and return `Result<T, FfiError>` from exported functions.
  Kotlin then sees an exception class and Swift a thrown error the caller can match on.
- Never let a panic or `unwrap` stand in for an error. A panic across the boundary is at best a generic internal
  error on the foreign side and at worst an abort of the app; map every expected failure explicitly.
- Keep messages and variants stable and meaningful to app code; do not pass core error strings through as the only
  information.

## 6. C headers with cbindgen

- Use cbindgen when consumers want a plain C API: export `extern "C"` functions with `#[unsafe(no_mangle)]` in
  edition 2024 (`#[no_mangle]` before it) and `#[repr(C)]` types, configure it with `cbindgen.toml`, and generate
  the header from a build script or a documented command so it cannot drift from the code.
- In a C API you own the memory rules: pair every function that returns an allocated value with a free function in
  the same crate, use opaque pointers for handles, and return status codes plus out-parameters instead of
  unwinding. Document each pointer's ownership in the header comment; review new `unsafe` per the core skill.

## 7. Cross-compiling

- Android: install the targets with `rustup target add` (for example `aarch64-linux-android`), the Android NDK and
  `cargo-ndk`, then build the ffi crate with `cargo ndk -t arm64-v8a -o <app>/src/main/jniLibs build --release`
  (add one `-t` per ABI). cargo-ndk finds the NDK through `ANDROID_NDK_HOME` and sets the linker and sysroot,
  which plain `cargo build --target` does not.
- iOS: add `aarch64-apple-ios` for devices and `aarch64-apple-ios-sim` (plus `x86_64-apple-ios` for Intel Macs)
  for simulators, build the `staticlib` for each, merge the simulator slices with `lipo`, and package device and
  simulator libraries with the headers and module map into one XCFramework with `xcodebuild -create-xcframework`.
  An XCFramework is how Xcode picks the right slice; a single fat library cannot hold both device and simulator
  arm64.
- Write these commands into a script or the README of the ffi crate, so the build is reproducible and reviewable.
  iOS builds need macOS with Xcode; say so instead of claiming they were run elsewhere.

## 8. What to verify where

- On the host: the core gates from `rust-forge` (fmt, clippy, tests) over the whole workspace, core unit tests
  for every rule, and ffi-crate Rust tests that call the exported functions and check error mapping. These run
  anywhere and catch almost all logic bugs.
- On the host, when the tooling exists: `cargo check --target <triple>` for each mobile target with the target
  installed, and bindings generation from the host `cdylib` to confirm the exported surface is supported.
- On a device or simulator: loading the library, the generated Kotlin and Swift calling it, threading and
  lifecycle in the app. Report these as not verified unless they were actually run, and name the command that
  would run them.
