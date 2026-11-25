---
name: rust-domain-desktop
description: Guides building and changing Tauri 2 desktop apps in Rust - commands and IPC between the webview and Rust, capabilities and permissions kept to least privilege, managed state, business logic in a Rust crate that does not depend on Tauri so it is unit-testable, platform build prerequisites, and what can be tested without opening a window. Use it whenever a Rust project depends on tauri or tauri-build, has a src-tauri directory or tauri.conf.json, or the user asks to add a command, capability, permission, window, plugin or state to a desktop app, even if they only say "the app", "the frontend calls" or "invoke". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: tauri=2.12.7
---

# rust-domain-desktop

Domain guidance for Tauri 2 desktop apps. The core `rust-forge` skill owns the working loop, the gates, the crate
policy and compiler-error strategies; follow it and use this skill only for what is specific to Tauri.

## 1. Pin the version first

Run `forge-tools domains` and read the `desktop` line: `crate=tauri version=<x>`. Read docs.rs and the official
guides (v2.tauri.app, "Develop" and "Security") for exactly that version, and for the `tauri-build` version in
`Cargo.lock`, before writing code. Tauri reshapes its permission model, config keys, plugin APIs and test helpers
between minor versions, and version drift is the main source of wrong desktop code. If the output says
`version=unknown`, the lock file is missing; take the requirement from `Cargo.toml` and resolve it before relying
on details. Tauri 1 material (the `allowlist` in `tauri.conf.json`) does not apply to Tauri 2; ignore it.

The names below were checked against tauri 2.12.7 and tauri-build 2.7.6. For any other version, confirm each name
in that version's docs before using it.

## 2. Structure: core crate, thin app crate

- Put the business logic (types, validation, storage, rules) in a plain library crate with no `tauri` dependency.
  It then builds and tests on any machine, in CI without a display, and fast, because it does not pull the webview
  stack.
- The app crate (usually `src-tauri`) depends on the core and contains only commands, state wiring, window and
  plugin setup. Each command extracts its arguments, calls one core function and maps the result; nothing else.
- Keep the core's API in domain terms (`add(&mut self, text) -> Result<Note, NoteError>`), not in IPC terms. The
  app crate owns the serialization shape, so the core does not need to change when the frontend does.

## 3. Commands and IPC

- Declare a command with `#[tauri::command]` and register every command in one
  `tauri::generate_handler![...]` passed to `Builder::invoke_handler`. A second `invoke_handler` call replaces the
  first, so all commands go in the same list; a command missing from it fails only at runtime.
- Arguments and return values cross IPC as JSON: they must implement `serde::Deserialize` and `serde::Serialize`.
  The frontend passes arguments in camelCase by default (`noteId` for `note_id`), a common source of "missing
  argument" errors; check the argument-naming rules in the pinned version's command docs.
- Return `Result<T, E>` from fallible commands so the frontend's promise rejects instead of the app panicking. `E`
  must be `Serialize`; define a small error type in the app crate that converts from the core error (with
  `thiserror` per the core skill) and serializes to a message or a tagged object. Do not leak internal details.
- Make a command `async` when it does I/O or long work, so it does not block the main thread and freeze the
  window. Blocking CPU work belongs on a blocking task, per the core skill's async rules.
- Keep the IPC surface small: one command per user action, with typed arguments, rather than a generic
  "execute" command. Every command is reachable by any script running in a window it is allowed for.

## 4. Capabilities and permissions (least privilege)

- In Tauri 2 a window can call only what a capability grants it. Capabilities are JSON or TOML files in
  `src-tauri/capabilities/`; each names an `identifier`, the `windows` (labels) it applies to, and a list of
  `permissions`. Check the exact schema in the pinned version's security guide; `tauri-build` generates a JSON
  schema under `gen/schemas/` that editors can validate against.
- Grant only the permissions the app's commands and plugins use, to the windows that need them. Prefer individual
  `allow-*` permissions over a plugin's `default` set or wildcard grants, and add scopes for file system, shell or
  HTTP access instead of granting everything; a compromised or injected script can call whatever is granted.
- For the app's own commands, list them in `build.rs` with
  `tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[...]))` and call
  `tauri_build::try_build(attributes)`. tauri-build then generates `allow-<command>` and `deny-<command>`
  permissions, and the capability can grant exactly those commands. Without this step app commands are not gated
  by permissions; check the pinned version's docs for the current default.
- One capability per trust level (for example the main window versus a window that loads remote content) keeps
  grants reviewable. Never grant permissions to remote URLs unless the task requires it.
- Review a capability change like a security change: say in the reply what each new permission allows.

## 5. State

- Register shared state once with `Builder::manage(value)` and take it in commands as `tauri::State<'_, T>`. A
  command asking for a type that was never managed fails at runtime, so register state next to `generate_handler!`.
- Managed state is shared across threads; wrap mutable data in `std::sync::Mutex` (or an `RwLock`) inside the
  managed type. Hold the lock only for the synchronous call into the core, and never across `.await`.
- Keep the state type thin: a lock around the core's store or service. Persistence and rules stay in the core.
- Do not use global statics for app data; managed state is visible in signatures and replaceable in tests.

## 6. Configuration and build prerequisites

- `tauri.conf.json` (or `Tauri.toml`) sets the identifier, windows, bundle settings and the frontend location
  (`build.frontendDist`, `build.devUrl`). Check key names in the config reference for the pinned version; they
  moved between 1.x and 2.x.
- `tauri::generate_context!()` reads that config at compile time, so the app crate needs it and its `build.rs`
  (calling `tauri_build::build()` or `try_build`) to compile. The frontend dist folder and the window icon (by
  default `icons/icon.png`, RGBA) must exist at build time.
- System prerequisites (see "Prerequisites" in the official guide for the pinned version): Linux needs WebKitGTK
  4.1 and its development packages; macOS needs the Xcode Command Line Tools; Windows needs the MSVC build tools
  and the WebView2 runtime. If the gates fail on the app crate with missing system libraries, say so and confirm
  that the core crate passes, instead of changing code to work around the environment.
- Mobile targets (Android, iOS) need their SDKs and `tauri android init` / `tauri ios init`; treat them as a
  separate task.

## 7. Testing without a window

- Test the business logic in the core crate with ordinary unit and integration tests (see the core skill's
  testing guidance). This is where most tests belong, and they run anywhere. When you expose a core operation as a
  command, add or extend core tests for exactly the behaviour the command relies on (success, rejected input,
  ordering), so that behaviour is pinned where it lives and does not depend on the app crate building.
- Commands that only delegate can be tested by calling the core functions they wrap. When a command adds mapping
  logic (error conversion, argument shaping), extract that into a plain function in the app crate and test it
  directly.
- For IPC-level tests, tauri's `test` feature provides `tauri::test::mock_builder`, `tauri::test::mock_context`
  and `tauri::test::get_ipc_response`, which run commands through a mock runtime without a real window. Enable the
  feature only for tests and check the helpers' signatures for the pinned version before use.
- End-to-end tests of the real window (WebDriver with `tauri-driver`) need a display and platform drivers; keep
  them out of `cargo test` and say what was not covered.
- Validate capability files by building the app crate: tauri-build checks that every referenced permission exists.
