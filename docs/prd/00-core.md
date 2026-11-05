# PRD 00: rust-forge core — Claude Code plugin for Rust development

This is stage 2 of a staged build. Stage 1 (`docs/prd/00-core-tools.md`) already built `forge-tools`; this stage
adds the plugin around it. Later PRDs in `docs/prd/` add one domain skill each (`01-cli.md` … `08-embedded.md`),
then automation (`09-automation.md`) and the improvement journal (`10-journal.md`).
Do not create any `rust-domain-*` skill in this stage.

## 1. Goal

Build the core of `rust-forge`, a Claude Code plugin that makes Claude reliable at developing Rust applications:
deterministic cargo quality gates enforced by hooks, compiler diagnostics compressed for the model,
a crate-verification policy against hallucinated crates and APIs, Rust-specialised subagents, a core skill that
later domain skills plug into, and an eval suite that later stages use to gate changes automatically.

The plugin does not orchestrate long autonomous runs (that is loki-mode's job). It is the Rust layer:
knowledge, gates and diagnostics that any Claude Code session (or a loki run) uses.

Primary user: a solo developer building Rust CLIs, Bevy games, offline-first apps with a Rust core,
and local LLM inference, on macOS (Apple Silicon).

## 2. Repository layout after this stage

```
rust-forge/
├── Cargo.toml                      # cargo workspace, members = ["tools/forge-tools"]
├── .claude-plugin/
│   ├── plugin.json
│   └── marketplace.json            # this repo is its own marketplace with one plugin
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
├── tools/forge-tools/              # exists from stage 1
├── evals/                          # `claude plugin eval` suite, see §7
│   └── <case>/
│       ├── prompt.md
│       ├── case.yaml               # only when the case needs a scaffold
│       ├── scaffold.sh             # writes the starting project into the empty run workspace
│       └── graders/*.md
├── scripts/plugin-eval.sh          # the one place that knows how to run the suite
└── README.md
```

## 3. forge-tools

Built in stage 1; its behaviour is specified in `docs/prd/00-core-tools.md` §3. Use it from hooks, commands, agents
and the skill exactly as specified there. Change its source only to fix a defect, and then with a test that fails
before the fix. In the root `Cargo.toml`, set `exclude = ["evals", "tools/forge-tools/tests/fixtures"]`.

## 4. Plugin manifest and hooks

- `.claude-plugin/plugin.json` and `.claude-plugin/marketplace.json`: follow the current Claude Code plugin and
  marketplace docs (https://code.claude.com/docs/en/plugins). Do not set a `version` field in either file: Claude
  Code then versions the plugin by commit SHA, so every commit on `main` is a release and marketplace auto-update
  delivers it. Later stages rely on this.
- `hooks/hooks.json`: wire PostToolUse → `forge-tools hook post-edit`, Stop → `forge-tools hook stop`,
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

## 7. Eval suite (`claude plugin eval`)

Read https://code.claude.com/docs/en/plugin-evals before writing cases and follow it for file formats; note the doc
date in README. Facts from that page this design relies on: each run starts in an empty workspace with a temporary
HOME; a case seeds files only through `case.yaml` `context.scaffold_script`, which runs only with `--scaffold`;
`Bash`, `Edit`, `Write` must be granted with `--allow-tools`; granted Bash runs in an OS sandbox where the real home
directory is unreadable and network reaches only domains granted as `WebFetch(domain:…)`; only allowlisted and
`EVAL_*` environment variables reach the run.

### 7.1 Case rules

- One directory per case under `evals/`. `prompt.md` body is the user request, phrased the way a person would type
  it, never naming the skill. Frontmatter sets `tags`, `max_turns`, `timeout_seconds`, `allowed_tools`.
- A case that needs a project gets `case.yaml` with `context.scaffold_script: scaffold.sh`. `scaffold.sh` writes the
  whole fixture with heredocs (no reads from the case directory, so it works wherever the runner puts it), runs
  `git init` and one commit so the run's diff is inspectable, and every fixture `Cargo.toml` contains an empty
  `[workspace]` table so it never joins another workspace.
- Every case has at least one grader. Prefer graders that cost nothing (`tool_used`, `tool_order`, `regex`,
  `file_exists`); add one `llm` grader only where behaviour cannot be checked mechanically.
- Tags: exactly one of `core`, `trigger`, `domain`; domain cases also carry the domain name.
- Bash inside a run is sandboxed and cannot run cargo, while the plugin's hooks run outside the sandbox (README
  "Evals"). A case that must prove the code builds uses a `regex` grader on the file `target/forge/state.json`,
  written by the Stop hook's `gates full`, requiring `"G3-test": "PASS"` and `"green": true`; its `prompt.md` sets
  `append_system_prompt` to say that cargo cannot run in Bash and that the hooks report compiler results.
  `cargo add` cannot run in a run either, so a case about adding a dependency checks the policy only.

### 7.2 Cases in this stage

| Case | Tags | What it checks |
|---|---|---|
| `core-borrow-fix` | core | Scaffold: a dependency-free crate that fails the borrow checker in at least three places with different error codes. Graders: `rust-forge` skill fired; the Stop hook's `gates full` ended green (`state.json`); `llm`: errors fixed by restructuring ownership rather than cloning everywhere, no `unwrap` added |
| `core-new-dependency` | core | Scaffold: small crate; request needs a JSON library. Graders: `tool_order` `forge-tools crate-check` before `cargo add`; `tool_used` Edit on `Cargo.toml` adding a version: max 0 |
| `trigger-rust-positive-1..3` | trigger | Three different natural Rust requests (compile error pasted in chat; "add a subcommand"; "why is cargo test slow"). Grader: `rust-forge` skill fired |
| `trigger-rust-negative-1..2` | trigger | Near misses that must not fire it (a Python script; a Go module's go.mod error). Grader: `tool_used` Skill matching `rust-forge`, min 0, max 0 |

### 7.3 `scripts/plugin-eval.sh`

Wraps `claude plugin eval` so CI, the pipeline and people run the suite the same way. Arguments: plugin path
(default `.`), then any extra `claude plugin eval` flags (`--case`, `--tag`, `--ablation`, `--json`).
It always passes `--trust-plugin --scaffold --threshold ${FORGE_EVAL_THRESHOLD:-0.8} --no-publish
--max-cost-usd ${FORGE_EVAL_MAX_COST:-15}`, pins `--model` and `--judge-model` from `FORGE_EVAL_MODEL` and
`FORGE_EVAL_JUDGE_MODEL` when set, and grants exactly the tools the cases need, including
`WebFetch(domain:index.crates.io)`, `WebFetch(domain:static.crates.io)` and `WebFetch(domain:crates.io)` so cargo
and crate-check can reach crates.io from the sandbox. It exits with `claude plugin eval`'s exit code.
Trigger cases are meant to run with `--ablation none`: in a two-arm run Skill graders do not count toward the score.

### 7.4 Sandbox spike

Done on 2026-11-05 by the owner; the results and the setup that came out of them (clean copy of the plugin, the
toolchain's own bin directory on `PATH`, compile evidence through the Stop hook) are in README "Evals" and
`scripts/plugin-eval.sh`.

## 8. Acceptance criteria

All verified by `cargo test --workspace` unless stated. Tests run offline; every cargo invocation in tests on a
fixture uses `--offline`. Tests never call `claude`.

- AC-1 `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass.
- AC-2–AC-8: specified in `docs/prd/00-core-tools.md` as T-2–T-8; they must still pass.
- AC-9 `forge-tools models set --strong opus` on a copy of the real `agents/` directory in a temp dir
  changes only the `model:` line of crate-architect, reviewer and unsafe-auditor.
- AC-10 Structure test, written so later stages reuse it unchanged:
  - plugin.json and marketplace.json parse, and neither has a `version` field; hooks.json parses;
  - every agent has name/description/tools/model with model in {fable, opus, sonnet, haiku, inherit};
  - for every directory under `skills/`: `name` matches the directory, description is 1–1024 chars, every file
    under its `references/` is mentioned in its SKILL.md, SKILL.md has ≤200 lines for `rust-forge` and ≤150 lines
    for `rust-domain-*`;
  - every `rust-domain-*` skill has frontmatter `metadata.forge-verified` matching `crate=version(,crate=version)*`
    with exact versions, and each crate is a key crate of that domain in the `forge-tools domains` mapping;
  - no file under `.claude-plugin/`, `agents/`, `commands/`, `hooks/`, `skills/`, `evals/` or in `README.md`
    contains `TODO`, `TBD` or `[insert`.
- AC-11 Eval suite test, reused unchanged by later stages: every directory under `evals/` (except `results/`) that
  holds `prompt.md` or `case.yaml` has at least one `graders/*.md`; `prompt.md` frontmatter uses only keys
  documented for `prompt.md`; every grader's `type` is one of regex, tool_used, tool_order, file_exists, llm,
  baseline; every `tool_used` grader on `Skill` names an existing skill directory; every case has exactly one of the
  tags core/trigger/domain; every `scaffold_script` named in a `case.yaml` exists and passes `bash -n`; there are at
  least 3 positive and 2 negative trigger cases.
- AC-12 Running `core-borrow-fix/scaffold.sh` in a temp dir produces a crate that fails `cargo check --offline` with
  at least three distinct error codes; running `core-new-dependency/scaffold.sh` produces a crate that passes it.
- AC-13 README: install (`cargo install --path tools/forge-tools`, adding this repo as a marketplace and installing
  the plugin), enabling marketplace auto-update, required and optional tools (nextest, cargo-deny, cargo-audit,
  cargo-machete, miri) with install commands, model tiers and the `models set` fallback, the hooks doc version used,
  "Evals" (how to run `scripts/plugin-eval.sh`, cost note, the spike result from §7.4), and a "Domains" section
  listing the domain skills that exist.

## 9. Out of scope

`rust-domain-*` skills (later stages); CI workflows, drift checks and automerge (`09-automation.md`); journal and
retro (`10-journal.md`); own orchestrator or task queue; MCP server; loki-mode integration code; gates G5
(cargo-hack) and G8 (semver-checks); perf and release agents; Windows support.

## 10. Constraints

- No network access in tests. Tests write only into a fresh directory under the system temp directory
  (`std::env::temp_dir()`) and remove it afterwards; they never write inside the repository, `target/` included.
  loki treats manifests that appear in the tree during a run as changed test configuration and will not verify
  the run. This rule applies to every later stage.
- Do not commit `target/` or `evals/results/`.
- Keep files minimal: no sections or options beyond this PRD.
