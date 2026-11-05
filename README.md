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

Sandbox spike (PRD §7.4): not run yet. It was not run during the stage that built the suite, so whether `cargo`
(rustup keeps toolchains in `~/.rustup`) and `forge-tools` (`~/.cargo/bin`) work inside the eval sandbox on macOS,
and its cost, are unknown. Run `scripts/plugin-eval.sh . --case core-borrow-fix --runs 1 --ablation none` and
record the outcome here. Until then the graders read only the trace (skill invocation, tool calls, tool order, an
`llm` judgement of the edits); none requires a successful build.

## Domains

Domain skills (`rust-domain-*`) are loaded by the domain that `forge-tools domains` detects. None exist yet.
