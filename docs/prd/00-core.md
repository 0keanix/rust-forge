# PRD 00: rust-forge core — Claude Code plugin for Rust development

This is stage 1 of a staged build. Later PRDs in `docs/prd/` add one domain skill each (`01-cli.md` …
`08-embedded.md`). Do not create any `rust-domain-*` skill in this stage.

## 1. Goal

Build the core of `rust-forge`, a Claude Code plugin that makes Claude reliable at developing Rust applications:
deterministic cargo quality gates enforced by hooks, compiler diagnostics compressed for the model,
a crate-verification policy against hallucinated crates and APIs, Rust-specialised subagents, and
a core skill that later domain skills plug into.

The plugin does not orchestrate long autonomous runs (that is loki-mode's job). It is the Rust layer:
knowledge, gates and diagnostics that any Claude Code session (or a loki run) uses.

Primary user: a solo developer building Rust CLIs, Bevy games, offline-first apps with a Rust core,
and local LLM inference, on macOS (Apple Silicon).

## 2. Repository layout (deliver exactly this in this stage)

```
rust-forge/
├── Cargo.toml                      # cargo workspace, members = ["tools/forge-tools"]
├── .claude-plugin/plugin.json
├── hooks/hooks.json
├── agents/
│   ├── crate-architect.md
│   ├── implementer.md
│   ├── compile-fixer.md
│   ├── test-author.md
│   ├── reviewer.md
│   ├── unsafe-auditor.md
│   └── deps-guardian.md
├── commands/
│   ├── gates.md                    # /rust-forge:gates [fast|full|deps|unsafe]
│   └── status.md                   # /rust-forge:status
├── skills/
│   └── rust-forge/
│       ├── SKILL.md
│       └── references/
│           ├── error-codes.md
│           ├── llm-antipatterns.md
│           ├── workspace-layout.md
│           ├── testing.md
│           └── crate-policy.md
├── tools/forge-tools/              # Rust binary used by hooks and commands
│   ├── Cargo.toml
│   ├── src/...
│   └── tests/
│       ├── fixtures/...
│       └── *.rs
├── evals/
│   ├── evals.json
│   └── fixtures/...
└── README.md
```

## 3. forge-tools (Rust binary)

Single binary, installed with `cargo install --path tools/forge-tools`. Edition 2024, stable toolchain.
Allowed dependencies: `clap` (derive), `serde`, `serde_json`, `toml_edit`, `anyhow`, `ureq`.
Add them with `cargo add`. Add nothing else without a written reason in README.

Lints in `tools/forge-tools/Cargo.toml` (the tool must follow the rules it enforces):
`clippy::unwrap_used`, `clippy::expect_used`, `clippy::panic`, `clippy::dbg_macro`, `clippy::todo`
are `deny`; tests may opt out with `#[expect(..., reason = "...")]` or a `#![cfg_attr(test, ...)]` attribute.

### 3.1 `forge-tools diag`

Reads cargo `--message-format=json` lines from stdin, prints a compact summary for an LLM.

- Keep only `reason == "compiler-message"` with level `error` or `warning`. Ignore everything else, including
  non-JSON lines.
- Deduplicate by (code, primary file, line, column, message). Same error from lib and test targets counts once.
- Order: errors first, then warnings, each in file/line order.
- One line per diagnostic: `<CODE|level> <file>:<line>:<col> <message>` plus ` | help: <first help child>` when present.
- Cap: 20 errors and 10 warnings; then one line `... <N> more errors, <M> more warnings omitted`.
- For the first 3 errors, append the `rendered` field, indented.
- Last line: `errors=<n> warnings=<m>`.
- Exit code: 1 if any error, else 0.
- Do not use `--message-format=json-render-diagnostics` anywhere: it hides `compiler-message` entries.

### 3.2 `forge-tools gates <level>`

Runs gates from the cargo workspace root (find it with `cargo locate-project --workspace --message-format plain`).
Stops at the first failing gate. Full raw output goes to `target/forge/logs/<unix-ts>-<gate>.log`;
stdout gets only the compact summary (via the `diag` logic for check/clippy).

| Level | Gates in order |
|---|---|
| `fast` | G0 `cargo fmt --all -- --check` · G1 `cargo check --workspace --all-targets --message-format=json` · G2 `cargo clippy --workspace --all-targets --all-features --message-format=json -- -D warnings` |
| `full` | `fast` + G3 `cargo nextest run --workspace --all-features` (fallback `cargo test --workspace --all-features` when nextest is not installed) + G4 `cargo test --workspace --doc` |
| `deps` | G7 `cargo deny check`, `cargo audit`, `cargo machete` |
| `unsafe` | G6 `cargo +nightly miri test` for workspace members whose sources contain `unsafe` |

- A gate whose tool is not installed prints `SKIPPED <gate>: <tool> not installed (install: <command>)`.
  SKIPPED is never reported as passed.
- Writes `target/forge/state.json`: level, per-gate result (PASS/FAIL/SKIPPED), git HEAD sha, timestamp.
- Exit code: 0 all passed or skipped, 1 a gate failed, 2 not a cargo project.

### 3.3 `forge-tools crate-check <name>`

- GET `https://crates.io/api/v1/crates/<name>` with a descriptive `User-Agent` naming the tool and its repository
  (crates.io requires one; do not put a personal email in it).
- Print: verdict, newest stable version, last update date, recent downloads, repository URL.
- Verdicts: `NOT_FOUND` (exit 1), `STALE` (newest stable release older than 18 months: likely unmaintained, worth
  looking for an alternative; exit 0), `OK` (exit 0). Network failure: `UNKNOWN` (exit 3), never `OK`.
- HTTP goes through a trait so tests use recorded JSON fixtures. No network in tests.

### 3.4 `forge-tools domains`

- Runs `cargo metadata --format-version 1 --no-deps --offline`, collects declared dependencies of workspace members.
- Mapping: `clap` → cli; `bevy` → game; `axum` → web; `tauri` → desktop; `leptos`, `yew`, `wasm-bindgen` → wasm;
  `uniffi` → ffi-mobile; `candle-core`, `burn`, `llama-cpp-2` → ml; `embassy-executor`, `cortex-m` → embedded.
- Print each detected domain with the skill to load (`rust-domain-<domain>`), whether or not that skill exists yet.
- For each detected domain, also print the resolved version of its key crate from `Cargo.lock` when present
  (the crate from the mapping above that matched), so domain skills can pick docs for exactly that version.

### 3.5 `forge-tools hook <event>`

Hook handlers. Read the hook JSON from stdin and answer in the format Claude Code expects.
Before implementing, read the current Claude Code hooks reference (https://code.claude.com/docs/en/hooks)
and follow it for input fields, output JSON and exit codes; note in README which doc version was used.
Every handler exits 0 silently when the working directory is not inside a cargo project.

- `post-edit` (PostToolUse on Edit|Write|MultiEdit): only for `*.rs` files. Run `rustfmt` on the file, then G1
  for the workspace. On errors, return the compact diag to Claude as feedback. On success, stay silent.
  Respect a 120 s budget: on timeout report `check deferred`, never block.
- `stop` (Stop): run `gates full` only if `*.rs` or `Cargo.toml`/`Cargo.lock` changed since the HEAD recorded
  in the last green `state.json` (or no green state exists). On failure, block with the compact summary as reason.
  Loop guard: after 3 consecutive blocks in one session (counter in `target/forge/state.json`), let the stop
  through with a warning that gates are still red.
- `pre-edit-manifest` (PreToolUse on Edit|Write for files named `Cargo.toml`): apply the proposed change in memory
  (Edit: replace old_string with new_string; Write: new content), parse before/after with `toml_edit`, and compare
  all dependency tables (`dependencies`, `dev-dependencies`, `build-dependencies`, `workspace.dependencies`,
  target-specific tables). Block a new dependency or a changed version with reason
  `use cargo add <name> after forge-tools crate-check <name>`. Allow feature changes, removals, and anything outside
  dependency tables. Allow a git dependency only when the line carries the comment `# forge: allow-git`.

### 3.6 `forge-tools models set --strong <fable|opus> [--plugin-root <path>]`

Rewrites the `model:` frontmatter value of the strong-tier agents (crate-architect, reviewer, unsafe-auditor) and
touches nothing else in those files. Exists as a fallback for when Fable is unavailable.

## 4. hooks/hooks.json

Wire PostToolUse → `forge-tools hook post-edit`, Stop → `forge-tools hook stop`,
PreToolUse → `forge-tools hook pre-edit-manifest`, with matchers and timeouts per the hooks reference.
If `forge-tools` is not on PATH the hooks must fail open (no block); README explains installation.

## 5. Subagents

Frontmatter: `name`, `description` (when the main session should delegate), `tools`, `model`.
Bodies are short imperative instructions that explain why, not rule lists.

| Agent | model | tools | Job |
|---|---|---|---|
| crate-architect | `fable` | Read, Grep, Glob, Bash | Workspace and crate boundaries, ownership model, feature flags, crate choice per crate-policy. Also the escalation target when compile-fixer fails 3 times: repeated borrow-checker failures usually mean the ownership design is wrong. |
| implementer | `inherit` | Read, Edit, Write, Grep, Glob, Bash | Small increments; `cargo check` must pass after each. |
| compile-fixer | `opus` | Read, Edit, Grep, Bash | Input: compact diag only. Strategy by error code from references/error-codes.md. Max 3 iterations, then return `UNRESOLVED` with the remaining diag. |
| test-author | `sonnet` | Read, Edit, Write, Grep, Bash | Unit, integration, doctests, `proptest` for invariants, `insta` for snapshots. Tests must check behaviour, not restate the implementation. |
| reviewer | `fable` | Read, Grep, Glob | Blind review against the spec and references/llm-antipatterns.md. Returns findings with severity; never edits. |
| unsafe-auditor | `fable` | Read, Grep, Bash | Only when the diff contains `unsafe` or FFI: `// SAFETY:` comments, invariants, `gates unsafe`. |
| deps-guardian | `haiku` | Bash, Read | Runs `crate-check` and `gates deps`, reports. |

## 6. Core skill

Follow the Agent Skills spec: `name` = directory name, lowercase-hyphen, ≤64 chars; `description` ≤1024 chars,
third person, states what the skill does and when to use it, key trigger in the first sentence,
slightly pushy against undertriggering. SKILL.md bodies in imperative mood. Every file under `references/`
is named in its SKILL.md together with the condition for reading it.

- `rust-forge` (≤200 lines): the working loop (domains → plan → implement in small steps → gates fast → tests →
  gates full → review), loading the `rust-domain-*` skill named by `forge-tools domains`, when to delegate to each
  subagent, compile-fixer → crate-architect escalation, crate policy summary (`cargo add` only, `crate-check` first,
  read docs for the version in `Cargo.lock` before first use of an unfamiliar API), recommended `[workspace.lints]`
  block, and how to read `target/forge/state.json`.
  Triggers: any work in a project with `Cargo.toml` or `.rs` files, Rust compile errors, cargo, crates.
- `references/error-codes.md`: strategies for E0382, E0499, E0502, E0505, E0597, E0106, E0277, E0308, E0599, E0425.
  Order of moves: restructure ownership (owned data, indices instead of references, split borrows) before adding
  lifetimes or `clone()`.
- `references/llm-antipatterns.md`: redundant `clone()`, `Arc<Mutex<_>>` by default, blocking calls in async,
  guards held across `.await`, `unwrap()`/`expect()` in non-test code, outdated or invented crate APIs, tests that
  mirror the implementation, `#[allow]` without reason, weakened or deleted tests. Each with a detection hint and
  the fix.
- `references/workspace-layout.md`, `references/testing.md`, `references/crate-policy.md`: as named.

Accuracy rule for all skill text: write principles and procedures. Include a concrete API name or code example only
if you verified it against docs.rs for a stated version; otherwise leave it out. Wrong API examples in a skill are
worse than none.

## 7. evals

`evals/evals.json` in the Skill Factory format (`skill_name`, `evals[]` with `id`, `prompt`, `expected_output`,
`files`). This stage adds two realistic multi-step evals for the core skill, each with a fixture under
`evals/fixtures/`:
1. Fix a crate that fails the borrow checker in several places (fixture has no dependencies and must fail
   `cargo check` as shipped).
2. Add a feature that needs a new dependency to a small crate (the expected behaviour is `crate-check` then
   `cargo add`, never a hand-written version in `Cargo.toml`).

Later stages append one eval per domain. Fixtures are excluded from the root workspace
(`exclude = ["evals/fixtures/*", "tools/forge-tools/tests/fixtures/*"]`).

## 8. Acceptance criteria

All verified by `cargo test --workspace` unless stated. Tests run offline; every cargo invocation in tests on a
fixture uses `--offline`.

- AC-1 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- AC-2 `diag`: recorded cargo JSON fixtures cover dedupe, ordering, caps with the omitted line, help suffix,
  rendered blocks for the first 3 errors, non-JSON input lines, exit codes.
- AC-3 `gates`: on a fixture crate with a formatting error, G0 fails and later gates do not run; on a clean
  fixture `fast` passes; a missing tool yields SKIPPED, not PASS; `state.json` is written with per-gate results.
- AC-4 `crate-check`: recorded crates.io responses produce OK, STALE, NOT_FOUND; a transport error produces UNKNOWN.
- AC-5 `domains`: a fixture workspace with `clap` and `bevy` reports cli and game with skill names and the Bevy
  version from its `Cargo.lock` (a hand-written lock file is fine); each of the 8 domains is detected from a
  minimal fixture manifest declaring its key crate.
- AC-6 `hook pre-edit-manifest`: blocks a new dependency and a version bump, allows a feature change, a removal,
  a non-dependency edit and a git dependency marked `# forge: allow-git`.
- AC-7 `hook stop`: blocks on a failing fixture, passes on a clean one, and lets the stop through after 3
  consecutive blocks.
- AC-8 `hook` handlers exit 0 with no output outside a cargo project.
- AC-9 `models set --strong opus` changes only the `model:` line of the three strong agents.
- AC-10 Structure test, written so later stages reuse it unchanged: plugin.json and hooks.json parse; every agent
  has name/description/tools/model with model in {fable, opus, sonnet, haiku, inherit}; for every directory under
  `skills/`: `name` matches the directory, description is 1–1024 chars, every file under its `references/` is
  mentioned in its SKILL.md, SKILL.md has ≤200 lines for `rust-forge` and ≤150 lines for `rust-domain-*`; no file
  under `.claude-plugin/`, `agents/`, `commands/`, `hooks/`, `skills/` or in `README.md` contains `TODO`, `TBD`
  or `[insert`.
- AC-11 `evals/evals.json` parses, ids are unique, every path in each eval's `files` exists; the borrow-checker
  fixture fails `cargo check --offline`.
- AC-12 README: install (`cargo install --path tools/forge-tools`, plugin install), required and optional tools
  (nextest, cargo-deny, cargo-audit, cargo-machete, miri) with install commands, model tiers and the
  `models set` fallback, the hooks doc version used, and a "Domains" section listing the domain skills that exist.

## 9. Out of scope

`rust-domain-*` skills (later stages); own orchestrator or task queue; MCP server; loki-mode integration code;
gates G5 (cargo-hack) and G8 (semver-checks); perf and release agents; Windows support; publishing to a marketplace.

## 10. Constraints

- No network access in tests; no writes outside the repo and `target/` in tests.
- Do not commit `target/`.
- Keep files minimal: no sections or options beyond this PRD.
