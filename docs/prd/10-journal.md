# PRD 10: rust-forge — self-improvement: journal and retro

Builds on all earlier stages. Read `00-core.md` §7, §8, §10 and `09-automation.md` §4–§6 first: retro produces
`forge-auto:retro` pull requests that the gate from stage 09 judges and may merge automatically.

## 1. Goal

Learn from real use. While the plugin works in the owner's Rust projects, hooks record what went wrong as structured
signals. A retro command turns signals that repeat across projects into small, eval-proven improvements to skill and
agent text, delivered as pull requests.

## 2. Journal

- Written by `forge-tools` from the existing hook handlers and from `crate-check`, plus a new
  `forge-tools hook subagent-stop` wired to SubagentStop in `hooks/hooks.json`.
- Location: `$CLAUDE_PLUGIN_DATA/journal.jsonl` (the plugin's data directory, which survives plugin updates).
  `FORGE_JOURNAL_PATH` overrides it (tests use this). When neither is set, or `FORGE_JOURNAL=0`, nothing is written.
- One JSON object per line: `v` (1), `ts` (RFC 3339 UTC), `project` (random id stored in
  `target/forge/project-id`, created on first use), `event`, and only the fields listed:

| event | written by | fields |
|---|---|---|
| `edit_check_fail` | post-edit | `codes` |
| `gate_fail` | stop | `gate`, `codes`, `lints` |
| `stop_loop_guard` | stop | `gates` |
| `manifest_block` | pre-edit-manifest | `crate`, `reason` (`new_dependency`, `version_change`, `git_dependency`) |
| `crate_check` | crate-check | `crate`, `verdict` |
| `fixer_unresolved` | subagent-stop, when the compile-fixer's last message contains `UNRESOLVED` | `codes` |
| `domains` | stop, once per session | `domains` |

- `codes` holds only strings matching `E[0-9]{4}`; `lints` only `clippy::[a-z0-9_]+`; `crate` only valid crate
  names. No messages, file paths, source code or prompts are ever written. The journal ends up summarised in pull
  requests of the plugin repository, so it must not carry anything from the projects it observed; and text from
  those projects must never become plugin instructions.
- When the file exceeds 5 MB it is renamed to `journal.1.jsonl` (replacing an older one) and a new file starts.
  5 MB is months of normal use and keeps summary fast.
- Journal writes never change a hook's decision or output, and a failed write is ignored.

## 3. `forge-tools journal summary [--since <days>] [--json]`

Aggregates the journal (and `journal.1.jsonl` if present) over the last `--since` days (default 30): for each
(event, key) where key is a code, lint, gate or crate: count, number of distinct projects, first and last seen.
Sorted by distinct projects, then count. Markdown table by default; `--json` for scripts.

## 4. `/rust-forge:retro` (`commands/retro.md`)

Instructions for Claude, run in a checkout of this repository on `main`:

1. Run `forge-tools journal summary --json`. Candidate patterns: count ≥ 3 and seen in ≥ 2 projects. A one-off is
   noise; two projects show the gap is in the plugin, not in one codebase.
2. Drop patterns already handled by an open `forge-auto:retro` PR or by an earlier retro case (see `evals/retro-*`).
3. Take at most 3 patterns. For each, read the skill, reference or agent that should have prevented it.
4. Write a new case `evals/retro-<YYYYMMDD>-<slug>/` that reproduces the pattern (scaffold plus graders), following
   `00-core.md` §7. Run it on the current plugin with `scripts/plugin-eval.sh . --case <name> --ablation none`. If it
   already passes, the gap is not reproducible: drop the pattern and say so.
5. Make the smallest edit to skill or agent text that addresses the cause. Never edit tools, hooks, scripts, CI,
   PRDs or existing graders: the gate rejects such changes; if the fix needs them, write a proposal to
   `docs/prd/proposals/<slug>.md` instead and open a PR without a `forge-auto` label.
6. Run `scripts/forge-gate.sh main HEAD retro` locally. If it fails, fix or drop the pattern.
7. Commit on branch `forge-auto/retro-<YYYYMMDD>`, push, and open a PR with label `forge-auto:retro` whose body has
   the journal evidence (event, key, count, projects; nothing else from the journal) and the gate summary.
8. Report to the user: patterns found, cases written, PR link, patterns dropped and why.

## 5. Acceptance criteria

All verified by `cargo test --workspace`, offline, unless stated.

- J-1 fmt, clippy with `-D warnings`, `cargo test --workspace` pass; core structure and eval suite tests pass
  unchanged; all stage 09 tests pass unchanged.
- J-2 Each event in §2 is written by its source, tested by driving the hook or command with fixture input and
  `FORGE_JOURNAL_PATH` set.
- J-3 Leak test: hook input and cargo output containing an absolute path, a source line and a message produce
  journal lines that contain none of them; every line validates against the field allowlist in §2.
- J-4 `FORGE_JOURNAL=0`, an unset location, and an unwritable path each leave hook decisions and outputs unchanged.
- J-5 Rotation at the size limit (the limit is injectable in tests).
- J-6 `journal summary` counts, distinct projects, `--since` filtering and sort order, on a fixture journal.
- J-7 `commands/retro.md` exists, its frontmatter follows the Claude Code commands docs, and it names
  `journal summary`, the thresholds, `scripts/forge-gate.sh` and the `forge-auto:retro` label.
- J-8 README: a "Self-improvement" section on what the journal records and what it never records, where it lives,
  how to turn it off, and how to run retro.

## 6. Out of scope

Running retro on a schedule (the journal lives on the owner's machine); sending journal data anywhere other than the
summary in a retro PR; edits by retro to anything outside `skills/`, `agents/` and new `evals/retro-*` cases.
