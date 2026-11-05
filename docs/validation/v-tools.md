# Validation: v-tools

Target: docs/prd/00-core-tools.md · Date: 2026-11-05 · HEAD before fixes: bcc70caa62e19b81c6b3f005f71610df92ec24d1

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (32 passed; 28 before this run + 4 added) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| T-1 | the three gate commands, run before and after the fixes | PASS | all three green both times |
| T-2 | tests/diag.rs: `dedupes_orders_and_adds_help`, `renders_only_first_three_errors`, `caps_with_omitted_line`, `ignores_non_json_and_other_reasons`, `binary_exit_codes`; tests/validation.rs: `diag_dedupes_same_error_from_lib_and_test_targets` | PASS | the mixed.jsonl duplicates were both `lib` targets, so lib vs test dedupe had no test; added one |
| T-3 | tests/gates.rs: `fmt_failure_stops_before_later_gates`, `clean_fixture_passes_fast`, `missing_tool_is_skipped_not_passed`, `not_a_cargo_project_exits_2` | PASS | asserts exact stdout and the per-gate map in state.json; the SKIPPED test removes cargo-deny/audit/machete from PATH |
| T-4 | tests/crate_check.rs: `ok_for_recent_release`, `stale_for_old_release`, `not_found_on_404`, `unknown_on_transport_error`, `user_agent_names_tool_and_repo` | PASS | recorded responses go through the `HttpClient` trait; no network |
| T-5 | tests/domains.rs: `workspace_with_clap_and_bevy`, `each_domain_from_its_key_crate` | PASS | exact output, including the Bevy version from a hand-written Cargo.lock; all 13 key crates map to the 8 domains |
| T-6 | tests/hook.rs: `manifest_blocks_new_dependency`, `manifest_blocks_version_bump`, `manifest_allows_features_removal_and_other_edits`, `manifest_git_dependency_needs_marker`, `manifest_ignores_other_files` | PASS | covers a target-specific table and the exact deny reason |
| T-7 | tests/hook.rs: `stop_blocks_on_failure_then_lets_through_after_three`, `stop_passes_on_clean_fixture`; tests/validation.rs: `stop_skips_gates_when_nothing_changed_since_green_head`, `stop_runs_full_gates_after_only_a_fast_green` | PASS | the two added tests cover the "changed since last green HEAD" rule; the second one exposed a defect (see Fixes) |
| T-8 | tests/hook.rs: `handlers_silent_outside_cargo_project` | PASS | checks exit code 0, empty stdout and empty stderr for all three events |
| T-9 | tests/models.rs: `set_strong_opus_changes_only_model_lines` | PASS | line-by-line diff: exactly one changed line per strong agent; test-writer is byte-identical |

Behaviour from the PRD sections without an AC test before this run: post-edit runs `rustfmt` and ignores non-`*.rs` files
(now `post_edit_runs_rustfmt_and_ignores_non_rust_files`); stop skips gates when nothing relevant changed (now
`stop_skips_gates_when_nothing_changed_since_green_head`). The post-edit 120 s timeout path (`check deferred`) is
still untested (see Remaining risks).

Test history: the only implementation commit is 473f697. It created every test file. This run found no
`#[ignore]`, no `#[allow]`/`#[expect]` without a `reason`, and no edited fixtures. This run deleted, ignored or
changed no existing test.

## Claims checked in skill and agent text
This PRD adds no skill or agent files (they belong to the next stage). The checked claims are the protocol and API
facts in the code.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| tools/forge-tools/src/hook.rs:1-8 | PreToolUse denies via `hookSpecificOutput.permissionDecision = "deny"` + `permissionDecisionReason` | https://code.claude.com/docs/en/hooks (fetched 2026-11-05) | kept |
| tools/forge-tools/src/hook.rs:1-8 | PostToolUse and Stop block via top-level `decision: "block"` + `reason` | same page, "Top-level decision" section: PostToolUse… Stop, SubagentStop | kept |
| tools/forge-tools/src/hook.rs:1-8 | `systemMessage` warns the user; `hookSpecificOutput.additionalContext` adds context | same page | kept |
| tools/forge-tools/src/crate_check.rs | `/api/v1/crates/<name>` returns `crate.max_stable_version`, `updated_at`, `recent_downloads`, `repository`, and a `versions[]` array with `num`/`created_at` | live GET of https://crates.io/api/v1/crates/serde on 2026-11-05 (all fields present, 316 versions) | kept |
| tools/forge-tools/src/crate_check.rs | User-Agent names the tool and repository, no email | crates.io data-access policy (User-Agent required); value checked by `user_agent_names_tool_and_repo` | kept |
| tools/forge-tools/Cargo.toml | only the allowed dependencies (anyhow, clap derive, serde, serde_json, toml_edit, ureq); the five clippy lints set to deny | PRD §3, file read | kept |

## Fixes made
- `hook stop` counted any green run as "last green", including `gates fast` and a `gates deps` run where every gate
  was SKIPPED. After a green `fast` run with no further changes, Stop let the session end without running the tests
  (G3/G4). Fix in `src/hook.rs` `stop()`: only a `last_green` with `level == "full"` lets Stop skip the gates. Covered by
  `tests/validation.rs::stop_runs_full_gates_after_only_a_fast_green` (failed before the fix because Stop stayed
  silent; passes after).
- Added coverage-only tests (the code was already correct): lib vs test dedupe in diag, post-edit rustfmt and
  non-`.rs` filter, and the Stop skip when nothing changed.

## Remaining risks
- Post-edit 120 s budget / `check deferred` path is not exercised by a test (it would need a deliberately slow build).
- `gates full` G3 nextest branch and `gates unsafe` (miri) only run where those tools are installed; the tests cover the
  `cargo test` fallback and SKIPPED reporting.
- `crate-check` treats a crate with no stable version (or a missing `versions[]` entry) as STALE; if crates.io stops
  embedding `versions` by default, every crate would become STALE. Only the recorded fixtures test this.
- The stop "changed since HEAD" check relies on `git`; outside a git repo it always runs the gates (safe default, untested).
- The task asked to commit the report with the fixes; the harness forbids commits in this session, so the changes are left uncommitted.
