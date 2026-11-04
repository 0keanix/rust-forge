# PRD 02: rust-forge — domain skill `rust-domain-game`

Builds on the core stages (`docs/prd/00-core-tools.md`, `docs/prd/00-core.md`, already implemented) and on
earlier domain stages. Read `00-core.md` §6–§8 and §10 first: they define the skill rules, the eval case format,
the structure tests this stage must pass and where tests may write.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-game/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- Frontmatter `metadata.forge-verified`: `crate=version` pairs for every key crate the skill gives
  version-specific guidance for, with the exact version you checked the text against on docs.rs (normally the newest
  stable release today). The drift check in a later stage compares these against crates.io.
- Eval case `evals/domain-game/` in the format of `00-core.md` §7: `prompt.md` with tags `[domain, game]`,
  `case.yaml` and `scaffold.sh` that writes the starting project (game-stamina) whose `Cargo.toml` declares `bevy`;
  graders: `tool_used` Skill `rust-domain-game`, plus one `llm` grader built from the expected behaviour below.
- A test in `tools/forge-tools/tests/` that runs that `scaffold.sh` in a temp dir, then `forge-tools domains` there,
  and asserts it reports `game` and `rust-domain-game`.
- `skills/rust-domain-game/references/bevy-testing.md`: building a headless `App`, advancing time
  deterministically, sending input in tests, asserting on component state.
- `scaffold.sh` also writes a minimal `Cargo.lock` that pins `bevy` to the version in `metadata.forge-verified`.
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

## Eval case

- prompt (`prompt.md` body): "This repo is a Bevy prototype with a moving player. Add stamina: sprinting drains it, it regenerates after a short idle delay, and sprint is disabled at zero. Add headless tests for the drain and regeneration rules."
- expected behaviour (rubric for the `llm` grader): "Adds a stamina component and systems in a plugin with explicit ordering, keeps tunables in a resource, and tests the rules by stepping a headless App; no rendering in tests; gates pass."

## Acceptance criteria

All verified by `cargo test --workspace`, offline.

- D-1 AC-1 of the core PRD still holds: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` pass.
- D-2 The core structure test (AC-10) passes unchanged with the new skill present.
- D-3 The detection test for the new fixture passes.
- D-4 The core eval suite test (AC-11) passes unchanged with the new case.
- D-5 Outside `cargo test`: run the new case once with
  `scripts/plugin-eval.sh . --case domain-game --runs 1 --ablation none` and add its score and cost as a row in the
  README "Evals" table.
- D-6 `forge-tools domains` on the scaffolded game project prints that Bevy version (run as a test).

## Out of scope

Other domains; changes to forge-tools source other than the new test; compiling the scaffolded project in
`cargo test`.
