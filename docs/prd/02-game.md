# PRD 02: rust-forge — domain skill `rust-domain-game`

Builds on the core stage (`docs/prd/00-core.md`, already implemented) and on earlier domain stages.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-game/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- One new eval appended to `evals/evals.json` (keep existing entries and ids; next free id) with its fixture
  project in `evals/fixtures/game-stamina/`, whose `Cargo.toml` declares `bevy`.
- A test in `tools/forge-tools/tests/` that runs `forge-tools domains` on that fixture and asserts it reports
  `game` and `rust-domain-game`.
- `skills/rust-domain-game/references/bevy-testing.md`: building a headless `App`, advancing time
  deterministically, sending input in tests, asserting on component state.
- Fixture ships a `Cargo.lock` pinning a Bevy version (generate it with `cargo generate-lockfile` while developing;
  tests stay offline).
- README "Domains" section: add one line for `rust-domain-game`.

## Skill content

Bevy: ECS structure (components as data, systems as behaviour, resources for global state, events or messages for
decoupling), plugins as the unit of organisation, schedules and system ordering, states, the `dynamic_linking`
feature in dev builds only, asset loading, and testing systems headlessly by building an `App` with minimal plugins
and stepping it (`references/bevy-testing.md`). Migration across Bevy versions: follow the official migration
guide for each version step, one step at a time, with `cargo check` after each.

## Rules for this skill

- `description`: third person, 1–1024 characters, first sentence names the trigger (the crates above and the kind
  of project), states what the skill does and when to use it, slightly pushy because skills tend to undertrigger.
  Mention that it complements the core `rust-forge` skill.
- First step of the body: run `forge-tools domains`, take the key crate's version from its output, and use docs.rs
  and the official guides for exactly that version. Explain why in one sentence: this ecosystem changes between
  minor versions, and version drift is the main source of wrong code.
- Accuracy rule: write principles and procedures. Include a concrete API name or code example only if you verified
  it against docs.rs for a stated version; otherwise leave it out. A wrong API example in a skill is worse than none.
- Imperative mood; explain why instead of piling up MUST/ALWAYS.
- Do not restate what the core skill already covers (gates, crate policy, error codes); refer to it.

## Eval

- prompt: "evals/fixtures/game-stamina is a Bevy prototype with a moving player. Add stamina: sprinting drains it, it regenerates after a short idle delay, and sprint is disabled at zero. Add headless tests for the drain and regeneration rules."
- expected_output: "Adds a stamina component and systems in a plugin with explicit ordering, keeps tunables in a resource, and tests the rules by stepping a headless App; no rendering in tests; gates pass."
- files: the fixture directory above.

## Acceptance criteria

All verified by `cargo test --workspace`, offline.

- D-1 AC-1 of the core PRD still holds: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` pass.
- D-2 The core structure test (AC-10) passes unchanged with the new skill present.
- D-3 The detection test for the new fixture passes.
- D-4 `evals/evals.json` passes the core evals test (AC-11) with the new entry.
- D-5 `forge-tools domains` on the game fixture prints the Bevy version from its `Cargo.lock` (run as a test).

## Out of scope

Other domains; changes to forge-tools source other than the new test; compiling this fixture in tests unless
listed above.
