# PRD 01: rust-forge — domain skill `rust-domain-cli`

Builds on the core stages (`docs/prd/00-core-tools.md`, `docs/prd/00-core.md`, already implemented) and on
earlier domain stages. Read `00-core.md` §6–§8 and §10 first: they define the skill rules, the eval case format,
the structure tests this stage must pass and where tests may write.
Add exactly one domain skill. Do not change forge-tools behaviour, hooks, agents, the core skill or other
domain skills; the only edits outside the new skill are the ones listed under Deliver.

## Deliver

- `skills/rust-domain-cli/SKILL.md`, ≤150 lines. Add `references/` files only if SKILL.md would otherwise exceed
  150 lines; name each one in SKILL.md together with the condition for reading it.
- Frontmatter `metadata.forge-verified`: `crate=version` pairs for every key crate the skill gives
  version-specific guidance for, with the exact version you checked the text against on docs.rs (normally the newest
  stable release today). The drift check in a later stage compares these against crates.io.
- Eval case `evals/domain-cli/` in the format of `00-core.md` §7: `prompt.md` with tags `[domain, cli]`,
  `case.yaml` and `scaffold.sh` that writes the starting project (cli-notes) whose `Cargo.toml` declares `clap`;
  graders: `tool_used` Skill `rust-domain-cli`, plus one `llm` grader built from the expected behaviour below.
- A test in `tools/forge-tools/tests/` that runs that `scaffold.sh` in a temp dir, then `forge-tools domains` there,
  and asserts it reports `cli` and `rust-domain-cli`.
- README "Domains" section: add one line for `rust-domain-cli`.

## Skill content

clap derive (subcommands, global flags, value enums), config layering (defaults, config file, environment,
flags), `anyhow` in binaries and `thiserror` in libraries, meaningful exit codes, output that is both
human-readable and scriptable (`--json`), and CLI tests with `assert_cmd` + `predicates` driving the built binary.

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

- prompt (`prompt.md` body): "This repo is a small notes CLI on clap. Add an export subcommand that writes all notes as JSON or CSV, with the default format taken from a config file in the user config dir and overridable by a flag. Cover it with tests that run the binary."
- expected behaviour (rubric for the `llm` grader): "Uses clap derive for the new subcommand and a value enum for the format, layers config file under the flag, returns a non-zero exit code on bad input, and adds assert_cmd tests; gates fast and full pass."

## Acceptance criteria

All verified by `cargo test --workspace`, offline.

- D-1 AC-1 of the core PRD still holds: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  `cargo test --workspace` pass.
- D-2 The core structure test (AC-10) passes unchanged with the new skill present.
- D-3 The detection test for the new fixture passes.
- D-4 The core eval suite test (AC-11) passes unchanged with the new case.
- D-5 Outside `cargo test`: run the new case once with
  `scripts/plugin-eval.sh . --case domain-cli --runs 1 --ablation none` and add its score and cost as a row in the
  README "Evals" table.

## Out of scope

Other domains; changes to forge-tools source other than the new test; compiling the scaffolded project in
`cargo test`.
