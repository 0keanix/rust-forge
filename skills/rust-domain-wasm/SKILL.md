---
name: rust-domain-wasm
description: Guides building Rust for the browser on wasm32-unknown-unknown - Leptos and Yew client-side rendered apps, wasm-bindgen bindings, trunk builds, feature-gating dependencies that do not build for wasm, logic kept in a shared crate tested on the host, and wasm-bindgen-test for browser-only behaviour. Use it whenever a Rust project depends on leptos, yew or wasm-bindgen, has a trunk index.html or Trunk.toml, or the user asks for a web frontend, a component, a signal, a browser app or anything compiled to WebAssembly, even if they only say "the app", "the page" or "in the browser". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: leptos=0.8.23,yew=0.23.0,wasm-bindgen=0.2.130
---

# rust-domain-wasm

Domain guidance for Rust compiled to WebAssembly for the browser. The core `rust-forge` skill owns the working
loop, the gates, the crate policy and compiler-error strategies; follow it and use this skill only for what is
specific to wasm.

## 1. Pin the version first

Run `forge-tools domains` and read the `wasm` line: `crate=<leptos|yew|wasm-bindgen> version=<x>`. Read docs.rs and
the official guides (the Leptos book, the Yew docs, the wasm-bindgen guide, trunkrs.dev) for exactly that version
before writing code. This ecosystem renames reactive primitives, features and macros between minor versions, and
version drift is the main source of wrong wasm code. If the output says `version=unknown`, the lock file is
missing; take the requirement from `Cargo.toml` and resolve it before relying on details. Guides and answers for
older Leptos (`create_signal`, a `cx: Scope` argument) or older Yew do not apply to current releases; ignore them.

The names below were checked against leptos 0.8.23, yew 0.23.0, wasm-bindgen 0.2.130, wasm-bindgen-test 0.3.80
and getrandom 0.4.3. For any other version, confirm each name in that version's docs before using it.

## 2. Structure: shared crate, thin app crate

- Put the logic (types, filtering, validation, parsing, state transitions) in a plain library crate with no
  Leptos, Yew, `wasm-bindgen` or `web-sys` dependency. It then builds and tests on the host with `cargo test`,
  which is fast, runs in CI without a browser, and is what the core skill's gates check.
- The app crate depends on the shared crate and contains only components, signals or hooks, event wiring and the
  mount call. A component reads input, calls one shared function and renders the result; nothing else.
- Keep the shared API in domain terms (`filter(&[Note], query) -> Vec<Note>` or an iterator), not in DOM terms,
  so the same tests cover every frontend and the logic survives a framework change.
- When you add behaviour to the app, add the host tests for it in the shared crate first: the empty input, case
  handling, no match, ordering. Those tests carry most of the confidence; browser tests only prove the wiring.

## 3. Target and build with trunk

- Install the target with `rustup target add wasm32-unknown-unknown`. Plain `cargo build` builds for the host;
  build for the browser with `cargo build --target wasm32-unknown-unknown` or through trunk.
- trunk builds a binary crate from an `index.html` next to its `Cargo.toml`: `trunk serve` for development,
  `trunk build --release` for a `dist/` folder. trunk runs `wasm-bindgen` itself, so the `wasm-bindgen` CLI it
  downloads must match the `wasm-bindgen` version in `Cargo.lock`; a mismatch fails at the bindgen step. Check
  the asset directives (`<link data-trunk rel="rust">` and friends) and `Trunk.toml` keys in the trunk docs for
  the installed trunk version.
- The app crate is a binary with `fn main()` that mounts the root component. Do not keep a `[lib]` with
  `crate-type = ["cdylib"]` unless you use wasm-pack instead of trunk; mixing the two setups is a common cause of
  "nothing renders".
- Add `dist/` to `.gitignore`; it is build output.
- Do not commit a `.cargo/config.toml` that sets `build.target` for the whole workspace: it moves the host gates to
  wasm and breaks host tests. Pass the target to trunk or the command instead.

## 4. Leptos (client-side rendering)

- Enable only the `csr` feature on `leptos` for a client-side app. `hydrate` and `ssr` are for server rendering
  with a server crate (cargo-leptos); do not enable them together with `csr` or in a CSR-only app.
- Leptos 0.8: `#[component]` functions return `impl IntoView` and take no scope argument; `view! { ... }` builds
  the tree; `signal(initial)` returns a `(ReadSignal, WriteSignal)` pair; `leptos::mount::mount_to_body(App)` mounts
  the root from `main`. Read the input with `event_target_value(&ev)` in an `on:input` handler and set the signal.
  `use leptos::prelude::*;` brings these in.
- Derive the visible list from the signal with a closure or a derived signal that calls the shared filter;
  do not store the filtered list in a second signal that you keep in sync by hand.
- For lists, check the pinned version's docs for `<For>` and its `each`/`key` props; a stable key (the note id)
  avoids re-creating rows on every keystroke.

## 5. Yew (client-side rendering)

- Enable the `csr` feature on `yew` for a client-side app; `ssr` and `hydration` are for server rendering.
- yew 0.23: mount the root from `main` with `yew::Renderer::<App>::new().render()`. Function components
  (`#[function_component]`) with hooks are the current style; check hook names and the `html!` syntax in the
  pinned version's docs before use.
- As with Leptos, compute the filtered list from state through the shared crate inside the component.

## 6. wasm-bindgen and browser APIs

- Use `wasm-bindgen` for bindings and `web-sys`/`js-sys` for browser APIs. Each `web-sys` type is behind its own
  cargo feature (named after the type); a missing feature shows up as "cannot find type" in `web_sys`.
- Keep `#[wasm_bindgen]` exports and `JsValue` handling at the edge of the app crate; convert to domain types
  before calling the shared crate.
- On stable Rust a panic on wasm32-unknown-unknown aborts the module and shows as `unreachable` in the console. Return
  errors from fallible code paths and show them in the UI instead of panicking.

## 7. Dependencies that do not build for wasm

- Many crates assume an OS: threads, files, sockets, `tokio` with the full runtime, native TLS, system clocks.
  Before adding a dependency to a crate that compiles for wasm, check its docs for wasm32-unknown-unknown support;
  use the core skill's crate policy for the rest.
- Put host-only dependencies under `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` or behind an
  optional cargo feature that the app crate does not enable, and gate the code that uses them with the same
  `cfg` or `#[cfg(feature = "...")]`. That keeps the shared crate building on both targets.
- `getrandom` (pulled in by `rand`, `uuid` and others) needs a browser backend on wasm32-unknown-unknown. For
  getrandom 0.4 that is the `wasm_js` feature, plus the backend selection described in its docs for that version;
  older major versions use different names, so read the docs of the version in `Cargo.lock`.
- When a wasm build fails deep in a dependency, run `cargo tree --target wasm32-unknown-unknown -i <crate>` to find
  who pulls it in, then gate that edge instead of patching the dependency.

## 8. Testing

- Host first: the shared crate's unit tests run with the core gates and need no browser.
- Browser-only behaviour (DOM rendering, events, `web-sys` calls) goes in `wasm-bindgen-test` tests in the app
  crate, as a dev-dependency. Mark tests with `#[wasm_bindgen_test]` and call
  `wasm_bindgen_test_configure!(run_in_browser);` once in the test crate so they run in a browser instead of Node.
- Gate those tests with `#![cfg(target_arch = "wasm32")]` (or put them in a module behind that `cfg`), so host
  `cargo test` and clippy skip them instead of failing to link.
- Run them with `wasm-pack test --headless --firefox` (or `--chrome`), or with `cargo test --target
  wasm32-unknown-unknown` and `wasm-bindgen-test-runner` configured as the target runner; the runner version must
  match `wasm-bindgen`. These need a browser and driver, so they are not part of the host gates; say in the reply
  which commands run them and whether you ran them.
- Keep browser tests few and about wiring: mount, type, check the rendered list. Filtering rules belong in the
  shared crate's host tests.
