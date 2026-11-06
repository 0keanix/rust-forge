# rust-forge

A Claude Code plugin that makes Claude reliable at developing Rust: cargo quality gates enforced by hooks, compiler
diagnostics compressed for the model, a crate-verification policy against hallucinated crates and APIs,
Rust-specialised subagents and a core skill (`rust-forge`) that domain skills plug into.

## Install

1. Install the helper binary (hooks, commands and agents call it):

   ```sh
   cargo install --path tools/forge-tools
   ```

   It goes to `~/.cargo/bin`; make sure that directory is on `PATH`. If `forge-tools` is not on `PATH`, every hook
   exits 0 without output, so nothing is blocked, but no gates run either.

2. Add this repository as a marketplace and install the plugin, in Claude Code:

   ```
   /plugin marketplace add 0keanix/rust-forge
   /plugin install rust-forge@rust-forge
   ```

3. Enable marketplace auto-update: run `/plugin`, open **Marketplaces**, select `rust-forge` and enable
   auto-update. The plugin sets no `version`, so Claude Code versions it by commit SHA and every commit on `main`
   is delivered as an update.

## Tools

Required: a Rust toolchain via rustup (`cargo`, `rustfmt`, `clippy`) and `forge-tools`.

Optional, used by the gates; a missing tool makes its gate `SKIPPED`, never passed:

| Tool | Gate | Install |
|---|---|---|
| cargo-nextest | `full` (G3; falls back to `cargo test`) | `cargo install cargo-nextest --locked` |
| cargo-deny | `deps` (G7) | `cargo install cargo-deny --locked` |
| cargo-audit | `deps` (G7) | `cargo install cargo-audit --locked` |
| cargo-machete | `deps` (G7) | `cargo install cargo-machete --locked` |
| miri | `unsafe` (G6) | `rustup toolchain install nightly && rustup +nightly component add miri` |

## Components

- Hooks (`hooks/hooks.json`): PreToolUse on Edit|Write → `forge-tools hook pre-edit-manifest` (blocks hand-added
  dependencies and version changes in `Cargo.toml`); PostToolUse on Edit|Write|MultiEdit →
  `forge-tools hook post-edit` (rustfmt and `cargo check` after `.rs` edits); Stop → `forge-tools hook stop`
  (`gates full` when Rust sources or manifests changed). Written against the Claude Code hooks reference
  (https://code.claude.com/docs/en/hooks) as read on 2026-11-04.
- Commands: `/rust-forge:gates [fast|full|deps|unsafe]`, `/rust-forge:status`.
- Agents: crate-architect, implementer, compile-fixer, test-author, reviewer, unsafe-auditor, deps-guardian.
- Skill: `rust-forge` with references on error codes, LLM antipatterns, workspace layout, testing and crate policy.

## Model tiers

| Tier | Agents | Model |
|---|---|---|
| strong | crate-architect, reviewer, unsafe-auditor | `fable` |
| fixer | compile-fixer | `opus` |
| tests | test-author | `sonnet` |
| main session | implementer | `inherit` |
| cheap | deps-guardian | `haiku` |

If Fable is not available on your account, switch the strong tier to Opus (only the `model:` lines change):

```sh
forge-tools models set --strong opus --plugin-root <path to the installed plugin>
```

`--strong fable` switches back.

## Evals

The suite under `evals/` runs with `claude plugin eval`, following the plugin evals docs
(https://code.claude.com/docs/en/plugin-evals) as read on 2026-11-05. Run it only through the wrapper:

```sh
scripts/plugin-eval.sh                                   # whole suite
scripts/plugin-eval.sh . --case core-borrow-fix --runs 1
scripts/plugin-eval.sh . --tag trigger --ablation none   # trigger cases: single arm
```

The wrapper always passes `--trust-plugin --scaffold --threshold ${FORGE_EVAL_THRESHOLD:-0.8} --no-publish
--max-cost-usd ${FORGE_EVAL_MAX_COST:-15}`, pins `--model`/`--judge-model` from `FORGE_EVAL_MODEL` and
`FORGE_EVAL_JUDGE_MODEL` when set, grants `Bash`, `Edit`, `Write` and `WebFetch` for `index.crates.io`,
`static.crates.io` and `crates.io`, and exits with `claude plugin eval`'s exit code. Run trigger cases with
`--ablation none`: in a two-arm run, Skill graders do not count toward the score.

Cost: every run and every `llm` grader is a real model call billed to your account; the cost ceiling above stops
starting new runs once reached. Case tags are `core`, `trigger` and `domain`.

Sandbox spike (PRD §7.4), 2026-11-05, Claude Code 2.1.318, one run per case with `--ablation none`, about $0.2–0.3
per core case. Findings, and the setup the wrapper and the cases now use:

- `claude plugin eval` refuses a plugin directory that contains hard links, and cargo fills `target/` with them. The
  wrapper therefore evaluates a clean copy of the plugin without `target/`, `.git` and `.loki`, and writes results to
  `target/plugin-eval/<timestamp>/` unless `--output-dir` is given.
- Bash inside a run cannot build Rust. On macOS the sandbox blocks loading rustc's libraries from anywhere under
  `/Users` (a copy in `/Users/Shared` too), and cargo cannot write its registry cache. On a Linux runner (Ubuntu
  24.04, bubblewrap with the AppArmor profile from the sandboxing docs) the whole filesystem, including the case's
  working directory and `$TMPDIR`, was read-only for Bash, against what the docs describe. `git` is missing on macOS.
- The plugin's hooks run outside the sandbox, so they can build. The post-edit hook returns compiler diagnostics and
  the Stop hook runs `gates full` and writes `target/forge/state.json`. The wrapper puts the active toolchain's own
  `bin` directory first on `PATH`, because a run has a temporary HOME and rustup proxies find no toolchain there.
  `forge-tools` must be installed and on `PATH`.
- Compile evidence: `gates-pass` in `core-borrow-fix` reads `target/forge/state.json` and requires `"G3-test": "PASS"`
  and `"green": true`. It passed on a correct fix and failed where the code did not compile. Before this change the
  graders scored 1.00 on code that never compiled, because they counted a call to `forge-tools gates`, not a pass.
- `cargo add` cannot run in a run, so `core-new-dependency` checks the dependency policy only (crate-check before
  `cargo add`, no hand-written version) and has no compile grader.
- Cases that compile tell the agent through `append_system_prompt` that cargo cannot run in Bash and that the hooks
  report compiler results, so it does not spend turns fighting the sandbox.
- CI: `claude plugin eval` works on a GitHub runner with a `CLAUDE_CODE_OAUTH_TOKEN` from `claude setup-token`. There
  the default model was Sonnet while it was Opus locally, so set `FORGE_EVAL_MODEL` when scores must be comparable.

| Case | Date | Command | Score | Cost |
|---|---|---|---|---|
| `domain-cli` | 2026-11-06 | `scripts/plugin-eval.sh . --case domain-cli --runs 1 --ablation none` | 1.00 (skill fired; gates green; `llm` 3/3 PASS). The first run scored 0.50: G4 failed on a binary-only crate (fixed in 2c58cea) and the rubric asked for a visible gate pass | $0.38 |

## Domains

Domain skills (`rust-domain-*`) are loaded by the domain that `forge-tools domains` detects.

- `rust-domain-cli` (domain `cli`, key crate `clap`): clap derive, config layering, errors and exit codes, human
  and `--json` output, binary tests with `assert_cmd` and `predicates`.
