# PRD 07: rust-forge — domain skill `rust-domain-wasm`

Builds on the core stage (`docs/prd/00-core.md`, already implemented) and on earlier domain stages.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-wasm/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- One new eval appended to `evals/evals.json` (keep existing entries and ids; next free id) with its fixture
  project in `evals/fixtures/wasm-filter/`, whose `Cargo.toml` declares `leptos`, `yew` or `wasm-bindgen`.
- A test in `tools/forge-tools/tests/` that runs `forge-tools domains` on that fixture and asserts it reports
  `wasm` and `rust-domain-wasm`.
- README "Domains" section: add one line for `rust-domain-wasm`.

## Skill content

`wasm32-unknown-unknown` target, wasm-bindgen, Leptos and Yew (client-side rendering first), trunk, feature-gating
dependencies that do not build for wasm, keeping logic in a shared crate that is tested on the host, and
`wasm-bindgen-test` for browser-only behaviour.

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

- prompt: "evals/fixtures/wasm-filter is a workspace with a note-filtering crate and an empty Leptos app crate. Build a client-side app with trunk that filters notes as you type, keeping the filter logic in the shared crate with host tests and adding a wasm-bindgen-test for the browser part."
- expected_output: "Keeps filtering in the shared crate with host tests, builds the Leptos CSR app with trunk for wasm32, gates non-wasm dependencies behind features, and adds a wasm-bindgen-test; host gates pass."
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
