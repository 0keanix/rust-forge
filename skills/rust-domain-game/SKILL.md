---
name: rust-domain-game
description: Guides building and changing Bevy games and prototypes in Rust - ECS structure with components, systems, resources and messages, plugins as the unit of organisation, schedules and system ordering, states, asset loading, the dynamic_linking feature for dev builds, headless tests that step an App, and migration between Bevy versions. Use it whenever a Rust project depends on bevy or the user asks to add a gameplay mechanic, system, plugin, state, asset or game test, or to upgrade Bevy, even if they only say "the game", "the player" or "the prototype". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: bevy=0.20.1
---

# rust-domain-game

Domain guidance for Bevy games. The core `rust-forge` skill owns the working loop, the gates, the crate policy and
compiler-error strategies; follow it and use this skill only for what is specific to Bevy.

## 1. Pin the version first

Run `forge-tools domains` and read the `game` line: `crate=bevy version=<x>`. Read docs.rs, the official Bevy
examples and the migration guides for exactly that version before writing code. Bevy renames and reshapes core
APIs between minor releases (0.x), and version drift is the main source of wrong Bevy code. If the output says
`version=unknown`, the lock file is missing; take the requirement from `Cargo.toml` and resolve it before relying
on details.

The API names below were checked against bevy 0.20.1. If the project uses another version, confirm each name in
that version's docs before you use it; for example, older releases call messages "events" and use different
method names for them.

## 2. ECS structure

- Components are plain data on entities: small `#[derive(Component)]` structs with no behaviour. Split by what
  systems query together; a `Stamina` component next to `Player` is easier to reuse and test than a field buried
  in a large `Player` struct.
- Systems are ordinary functions whose parameters (`Query`, `Res`, `ResMut`, `Commands`, `MessageWriter`,
  `MessageReader`) declare what they read and write. Keep each system to one rule; the scheduler can then run
  independent systems in parallel and tests can target one rule at a time.
- Resources hold global state and tunables (`#[derive(Resource)]`). Put gameplay numbers (drain rate, regen
  delay, speeds) in a resource with a `Default` impl instead of constants inside systems, so tests and designers
  can change them without touching logic.
- Messages decouple systems: one system writes (`MessageWriter::write`), others read, and neither knows about the
  other. Register each message type with `App::add_message`. Use them for "something happened" signals (sprint
  started, stamina exhausted); use components or resources for state that persists.
- Use `Commands` to spawn, despawn or insert components from systems; the changes apply at a sync point, so do
  not expect them to be visible later in the same system.
- Use `Time` (`Res<Time>`, `delta_secs()`) for every rate, never a per-frame constant, so behaviour does not
  depend on frame rate and tests can control time.

## 3. Plugins

- Make each feature a plugin: a struct implementing `Plugin` whose `build` registers its resources, messages,
  states and systems on the `App`. `main` then reads as a list of `add_plugins` calls, and a test can add exactly
  the plugin under test to a minimal `App`.
- Keep a plugin's types, systems and registration in one module. A plugin should not depend on rendering or
  windowing unless that is its purpose; gameplay rules that do not need them stay testable headlessly.
- When one plugin needs another's types, depend on its public types and system sets, not on its private systems.

## 4. Schedules and ordering

- Put startup work in `Startup`, per-frame logic in `Update`, and physics-like rules that must be frame-rate
  independent in `FixedUpdate`. Mixing them silently changes how often a rule runs.
- Systems in the same schedule have no order unless you give one, and an unordered read-after-write is a
  one-frame lag that shows up as flaky behaviour. Order explicitly: `.chain()` for a fixed sequence,
  `.before(...)`/`.after(...)` for single edges, and a `#[derive(SystemSet)]` enum with `App::configure_sets` when
  several plugins must agree on phases (input, then simulation, then presentation).
- Gate systems with `.run_if(...)` conditions instead of early returns, so the schedule shows when a system is
  active.

## 5. States

- Model screens and modes (menu, playing, paused) as a `#[derive(States)]` enum registered with
  `App::init_state`. Run setup in `OnEnter(...)`, teardown in `OnExit(...)`, and gate per-frame systems with
  `run_if(in_state(...))`.
- Change state through `ResMut<NextState<S>>` and `set`; the transition applies on the next state-transition
  step, not immediately.
- In a headless test with `MinimalPlugins`, add `StatesPlugin` before calling `init_state`, because the state
  machinery is not part of the minimal set.

## 6. Assets

- Load through `AssetServer::load`, which returns a handle at once and loads in the background. Keep the handle
  (in a component or resource) for as long as the asset is needed; dropping all handles unloads it.
- Before using an asset, check its load state or wait in a loading state; do not assume it is ready on the frame
  you requested it.
- Keep asset paths relative to the `assets/` directory and in one place, so a rename breaks one line.
- Gameplay rules should not need assets; keep them separate so tests run without an asset server.

## 7. Fast dev builds

- The `dynamic_linking` feature of `bevy` speeds up incremental builds a lot, but it must not reach a release
  build: the shipped binary would depend on a separate Bevy dynamic library. Enable it only for local development,
  for example with `cargo run --features bevy/dynamic_linking`, rather than in the default features of
  `[dependencies]`.
- Follow the Bevy setup guide for the project's version for other dev-profile settings (optimising dependencies
  in the dev profile); add them to `Cargo.toml` profiles, not to code.

## 8. Testing headlessly

Test rules by building an `App` with `MinimalPlugins` and the plugin under test, setting up entities and
resources, calling `App::update` to step it, and asserting on component and resource state. Never add
`DefaultPlugins` or anything that opens a window or renders in tests: they need a display and a GPU, and fail
or hang in CI.

Read `references/bevy-testing.md` before writing or changing any Bevy test: it covers building the headless `App`,
advancing time deterministically, sending input and asserting on component state.

## 9. Migrating between Bevy versions

- Upgrade one minor version at a time (0.19 to 0.20, not 0.18 to 0.20), following the official migration guide
  for that step. Each guide assumes the previous one was applied, and skipping one hides renames you then cannot
  find.
- After each step, run `cargo check` (and the core gates) and fix everything before the next step, so each error
  maps to one guide.
- Update Bevy ecosystem crates (third-party plugins) to the releases that target the same Bevy version in the same
  step; mismatched releases pull in two copies of Bevy and fail with confusing type errors.
