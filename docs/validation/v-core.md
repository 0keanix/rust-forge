# Validation: v-core

Target: docs/prd/00-core.md · Date: 2026-11-05 · HEAD before fixes: e2a2d91d6940e13fdd6f76dfc760391a0fa71e1a

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (47 passed; 40 before this run + 7 added in `tests/v_core.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| AC-1 | the three gate commands, run before and after this run | PASS | green both times |
| AC-2–AC-8 | tests/diag.rs, tests/gates.rs, tests/crate_check.rs, tests/domains.rs, tests/hook.rs, tests/validation.rs (see docs/validation/v-tools.md, T-2–T-8) | PASS | no test or source in these files changed since 89cff85 |
| AC-9 | tests/plugin.rs `models_set_on_real_agents_changes_only_strong_model_lines` | PASS | copies the real `agents/` to a temp dir; exactly one changed line (`model: fable` → `model: opus`) for the three strong agents, others byte-identical |
| AC-10 | tests/plugin.rs `manifests_parse_without_version`, `agents_have_required_frontmatter`, `skills_follow_spec`, `no_placeholders` | PASS | `skills_follow_spec` checks name, length, description 1–1024, every `references/` file named in SKILL.md, line budgets and `metadata.forge-verified` against `domains::MAPPING` |
| AC-11 | tests/plugin.rs `eval_suite_is_well_formed` | PASS | grader types, prompt.md keys, Skill graders name an existing skill, one of core/trigger/domain, `bash -n` on scaffolds, ≥3 positive / ≥2 negative trigger cases |
| AC-12 | tests/plugin.rs `borrow_fix_scaffold_fails_with_three_error_codes`, `new_dependency_scaffold_compiles` | PASS | `cargo check --offline` in a temp dir |
| AC-13 | README.md: "Install" (cargo install --path, marketplace add, plugin install, auto-update), "Tools" (nextest, cargo-deny, cargo-audit, cargo-machete, miri with install commands), "Model tiers" (`models set`), "Components" (hooks doc read on 2026-11-04), "Evals" (wrapper usage, plugin-evals doc date, cost, spike result), "Domains" | PASS | text-only criterion; checked by reading README |

Behaviour from PRD sections without an AC test before this run, now covered by `tools/forge-tools/tests/v_core.rs`:
- §4 hooks wiring and fail-open: `hooks_wire_each_event_and_fail_open_without_forge_tools` (each event calls the right
  subcommand, Edit/Write matchers, timeouts; with `forge-tools` absent from PATH each command exits 0 silently).
- §5 subagent table: `agents_match_prd_table` (exact model and tools per agent).
- §3 root manifest `exclude`: `root_manifest_excludes_evals_and_fixtures`.
- §6 error-code reference: `error_codes_reference_covers_every_prd_code`.
- §7.1 scaffold rules: `scaffolds_are_self_contained_isolated_and_committed` (script copied alone to another dir still
  works, every fixture `Cargo.toml` has an empty `[workspace]`, one commit, clean tree).
- §7.1/§7.2 build proof: `borrow_fix_proves_build_through_state_json` (regex grader on `target/forge/state.json`
  requires `"G3-test": "PASS"` and `"green": true`; prompt sets `append_system_prompt`).
- §7.3 wrapper: `plugin_eval_script_passes_fixed_flags`.

All seven pass on the current code (the code was already correct); each fails if the checked file is broken in the
way its name describes (verified by reasoning about each assertion, not by mutating repository files).

Test history: since the v-tools report (89cff85) the only test change is the new tests/plugin.rs (e2a2d91 and
8c16174 touched evals and scripts only). No `#[ignore]`, no `#[allow]`/`#[expect]` without `reason`, no edited
fixtures, no deleted assertions. This run deleted, ignored or changed no existing test.

## Claims checked in skill and agent text
| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-forge/SKILL.md (Recommended lints) | clippy lints `unwrap_used`, `expect_used`, `dbg_macro`, `todo` exist | `cargo clippy --explain <lint>` with rustc 1.98.0 toolchain | kept |
| skills/rust-forge/SKILL.md (Recommended lints) | rustc lint `unsafe_code`; `[workspace.lints]` + `[lints] workspace = true` | Cargo reference, "workspaces → lints" (doc.rust-lang.org/cargo) | kept |
| skills/rust-forge/SKILL.md, references/llm-antipatterns.md | `#[expect(..., reason = "...")]` / `#[allow(..., reason = ...)]` | Rust reference, lint attributes (`expect` stable since 1.81) | kept |
| skills/rust-forge/SKILL.md | `cargo check ... --message-format=json \| forge-tools diag` | tools/forge-tools/src/diag.rs, tests/diag.rs | kept |
| skills/rust-forge/SKILL.md, references/crate-policy.md | crate-check NOT_FOUND / STALE (18 months) / UNKNOWN / OK meanings | tools/forge-tools/src/crate_check.rs:12,154 | kept |
| skills/rust-forge/SKILL.md (state.json) | fields `level`, `gates`, `head`, `timestamp`, `green`, `last_green`, `stop_blocks`; Stop lets through after 3 blocks | tools/forge-tools/src/gates.rs:325-331, src/hook.rs:145-180 | kept |
| references/crate-policy.md | `cargo add` with `--features`, `--dev`, `--build`; git deps need `# forge: allow-git` | `cargo add --help` (cargo 1.98); src/hook.rs:25 | kept |
| references/testing.md | `cargo test --doc` is G4; G3 uses nextest when installed; `cargo test --timings` | src/gates.rs:119-140; `cargo test --help` lists `--timings` | kept |
| references/error-codes.md | titles of E0382, E0499, E0502, E0505, E0597, E0106, E0277, E0308, E0599, E0425; `rustc --explain <code>` | `rustc --explain <code>` (rustc 1.98.0) | kept |
| agents/*.md | model and tools per PRD §5 table | PRD §5; tests/v_core.rs `agents_match_prd_table` | kept |
| hooks/hooks.json | PreToolUse/PostToolUse/Stop command hooks with `matcher`, `timeout` | https://code.claude.com/docs/en/hooks (as recorded in v-tools, fetched 2026-11-05) | kept |
| scripts/plugin-eval.sh | flags `--trust-plugin --scaffold --threshold --no-publish --max-cost-usd --model --judge-model --allow-tools` | PRD §7.3 and README "Evals" (owner's spike 2026-11-05, plugin-evals doc read 2026-11-05) | kept |

No `rust-domain-*` skill exists in this stage, so there is no `metadata.forge-verified` to check.

## Fixes made
- No defects found in code, skill, agent or eval files.
- Added `tools/forge-tools/tests/v_core.rs` (7 tests) for PRD behaviour that had no test (§3, §4, §5, §6, §7.1–§7.3,
  listed above).

## Remaining risks
- Skill and agent advice quality (when to delegate, ownership strategies) is not provable by tests; only the eval suite
  measures it, and it needs a paid `claude plugin eval` run.
- Eval graders are checked for shape only. `core-new-dependency/no-hand-edited-version` matches any Edit of
  `Cargo.toml` whose input contains `N.N`, so an Edit that only touches features but whose `old_string` includes the
  package version would count as a violation (false fail, not a false pass).
- PreToolUse matches `Edit|Write` while PostToolUse also matches `MultiEdit`; if a session still offers a MultiEdit
  tool, a manifest edit through it would bypass the pre-edit dependency check. Not verified against the current tool
  list.
- The README cost and spike figures are the owner's recorded observations and cannot be re-checked offline.
- The task asked to commit the report with the fixes; the harness forbids commits in this session, so the changes are
  left uncommitted.
