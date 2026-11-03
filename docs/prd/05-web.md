# PRD 05: rust-forge — domain skill `rust-domain-web`

Builds on the core stage (`docs/prd/00-core.md`, already implemented) and on earlier domain stages.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-web/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- One new eval appended to `evals/evals.json` (keep existing entries and ids; next free id) with its fixture
  project in `evals/fixtures/web-notes-api/`, whose `Cargo.toml` declares `axum`.
- A test in `tools/forge-tools/tests/` that runs `forge-tools domains` on that fixture and asserts it reports
  `web` and `rust-domain-web`.
- README "Domains" section: add one line for `rust-domain-web`.

## Skill content

axum + tokio + tower: routing, extractors, shared application state without `Arc<Mutex<_>>` by default, error types
that map to responses, middleware with tower layers, sqlx (offline query data with `cargo sqlx prepare`, migrations),
graceful shutdown, `tracing`, and integration tests that call the router directly without binding a port.

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

- prompt: "evals/fixtures/web-notes-api is an axum service. Add POST /notes and GET /notes/{id} backed by SQLite through sqlx with a migration, keep the query data offline-ready, and add router tests that do not bind a port."
- expected_output: "Adds handlers with typed extractors and an error type that maps to status codes, a sqlx migration and offline query data, and tests via the router as a tower service; gates pass."
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
