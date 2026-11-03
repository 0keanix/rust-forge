# PRD 06: rust-forge — domain skill `rust-domain-desktop`

Builds on the core stage (`docs/prd/00-core.md`, already implemented) and on earlier domain stages.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-desktop/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- One new eval appended to `evals/evals.json` (keep existing entries and ids; next free id) with its fixture
  project in `evals/fixtures/desktop-notes/`, whose `Cargo.toml` declares `tauri`.
- A test in `tools/forge-tools/tests/` that runs `forge-tools domains` on that fixture and asserts it reports
  `desktop` and `rust-domain-desktop`.
- README "Domains" section: add one line for `rust-domain-desktop`.

## Skill content

Tauri 2: commands and IPC, capabilities and permissions (least privilege), state management, keeping business logic
in a Rust crate independent of Tauri so it is unit-testable, platform build prerequisites, and what can be tested
without a window.

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

- prompt: "evals/fixtures/desktop-notes is a workspace with a notes core crate and a stub Tauri 2 app crate. Expose list_notes and add_note as Tauri commands with capabilities restricted to what those commands need, keeping the logic testable without the app."
- expected_output: "Adds two commands in the app crate that delegate to the core, a capability file granting only what is needed, and tests on the core crate; gates pass for the core."
- files: the fixture directory above.

## Acceptance criteria

All verified by `cargo test --workspace`, offline.

- D-1 AC-1 of the core PRD still holds: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` pass.
- D-2 The core structure test (AC-10) passes unchanged with the new skill present.
- D-3 The detection test for the new fixture passes.
- D-4 `evals/evals.json` passes the core evals test (AC-11) with the new entry.

## Out of scope

Other domains; changes to forge-tools source other than the new test; compiling this fixture in tests unless
listed above.
