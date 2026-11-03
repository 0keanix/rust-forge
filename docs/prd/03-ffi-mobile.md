# PRD 03: rust-forge — domain skill `rust-domain-ffi-mobile`

Builds on the core stage (`docs/prd/00-core.md`, already implemented) and on earlier domain stages.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-ffi-mobile/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- One new eval appended to `evals/evals.json` (keep existing entries and ids; next free id) with its fixture
  project in `evals/fixtures/ffi-notes-core/`, whose `Cargo.toml` declares `uniffi`.
- A test in `tools/forge-tools/tests/` that runs `forge-tools domains` on that fixture and asserts it reports
  `ffi-mobile` and `rust-domain-ffi-mobile`.
- README "Domains" section: add one line for `rust-domain-ffi-mobile`.

## Skill content

UniFFI for Kotlin and Swift bindings (proc-macro or UDL, chosen per the version in use), cbindgen for C headers,
cross-compiling for Android with cargo-ndk and for iOS targets with an XCFramework, keeping the FFI surface thin over
a pure Rust core crate that has its own tests, error and ownership mapping across the boundary, and what to verify on
the host versus on a device or simulator.

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

- prompt: "evals/fixtures/ffi-notes-core is a workspace with a pure Rust notes core crate and an empty ffi crate that already depends on uniffi. Expose NoteStore (add, list, search) to Kotlin and Swift through the ffi crate, and make it build for aarch64-linux-android and aarch64-apple-ios."
- expected_output: "Keeps the ffi crate a thin layer over the core, maps errors to an FFI error type, keeps logic in the core with host tests, and documents the Android and iOS build commands; gates pass on the host."
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
