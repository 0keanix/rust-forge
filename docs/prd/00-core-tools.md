# PRD 00a: rust-forge core tools — `forge-tools`

This is stage 1 of a staged build of `rust-forge`, a Claude Code plugin for Rust development. This stage builds only
the Rust binary that the plugin's hooks and commands call. The next stage (`docs/prd/00-core.md`) adds the plugin
files around it: hooks.json, agents, commands, the core skill, evals and README. Do not create those here.

## 1. Goal

A single, well-tested CLI that gives Claude deterministic Rust tooling: compact compiler diagnostics, cargo quality
gates, crate verification against crates.io, project domain detection, Claude Code hook handlers, and model-tier
switching for the plugin's agents.

## 2. Repository layout (deliver exactly this in this stage)

```
rust-forge/
├── Cargo.toml                      # cargo workspace, members = ["tools/forge-tools"],
│                                   # exclude = ["evals/fixtures/*", "tools/forge-tools/tests/fixtures/*"]
└── tools/forge-tools/
    ├── Cargo.toml
    ├── src/...
    └── tests/
        ├── fixtures/...
        └── *.rs
```

## 3. forge-tools (Rust binary)

Single binary, installed with `cargo install --path tools/forge-tools`. Edition 2024, stable toolchain.
Allowed dependencies: `clap` (derive), `serde`, `serde_json`, `toml_edit`, `anyhow`, `ureq`.
Add them with `cargo add`. Add nothing else without a written reason in a comment in `tools/forge-tools/Cargo.toml`.

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
and follow it for input fields, output JSON and exit codes; note the doc version used in a comment at the top of
the hook module (the next stage copies it into README).
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

## 4. Acceptance criteria

All verified by `cargo test --workspace` unless stated. Tests run offline; every cargo invocation in tests on a
fixture uses `--offline`.

- T-1 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- T-2 `diag`: recorded cargo JSON fixtures cover dedupe, ordering, caps with the omitted line, help suffix,
  rendered blocks for the first 3 errors, non-JSON input lines, exit codes.
- T-3 `gates`: on a fixture crate with a formatting error, G0 fails and later gates do not run; on a clean
  fixture `fast` passes; a missing tool yields SKIPPED, not PASS; `state.json` is written with per-gate results.
- T-4 `crate-check`: recorded crates.io responses produce OK, STALE, NOT_FOUND; a transport error produces UNKNOWN.
- T-5 `domains`: a fixture workspace with `clap` and `bevy` reports cli and game with skill names and the Bevy
  version from its `Cargo.lock` (a hand-written lock file is fine); each of the 8 domains is detected from a
  minimal fixture manifest declaring its key crate.
- T-6 `hook pre-edit-manifest`: blocks a new dependency and a version bump, allows a feature change, a removal,
  a non-dependency edit and a git dependency marked `# forge: allow-git`.
- T-7 `hook stop`: blocks on a failing fixture, passes on a clean one, and lets the stop through after 3
  consecutive blocks.
- T-8 `hook` handlers exit 0 with no output outside a cargo project.
- T-9 `models set --strong opus` on a fixture copy of an `agents/` directory (three strong agents plus one other)
  changes only the `model:` line of the three strong agents.

## 5. Out of scope

Plugin files (`.claude-plugin/`, `hooks/`, `agents/`, `commands/`, `skills/`, `evals/`, README): next stage.

## 6. Constraints

- No network access in tests. Tests write only into a fresh directory under the system temp directory
  (`std::env::temp_dir()`) and remove it afterwards; they never write inside the repository, `target/` included.
  loki treats manifests that appear in the tree during a run as changed test configuration and will not verify
  the run. This rule applies to every later stage.
- Do not commit `target/`.
- Keep files minimal: no subcommands or options beyond this PRD.
