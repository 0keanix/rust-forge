# Validation: v-cli

Target: docs/prd/01-cli.md · Date: 2026-11-06 · HEAD before fixes: 7d8de3783e9140b57330025f67fd29838db883b9

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (52 passed; 49 before this run, +3 in `tests/v_cli.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | the three gates above, `cargo test --workspace` | PASS | Run before and after this validation. |
| D-2 | `plugin::skills_follow_spec`, `plugin::no_placeholders` | PASS | Generic over `skills/*`: name = dir, description 1–1024 chars, `references/` named in SKILL.md, ≤150 lines for `rust-domain-*`, `metadata.forge-verified` exact versions of key crates only. rust-domain-cli has 102 lines and no `references/`. |
| D-3 | `domain_cli::domain_cli_scaffold_is_detected_as_cli` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`; asserts `cli`, `skill=rust-domain-cli`, `crate=clap`. Would fail if the scaffold dropped clap or detection broke. |
| D-4 | `plugin::eval_suite_is_well_formed` | PASS | Checks keys, single kind tag, `bash -n scaffold.sh`, grader types, Skill grader names an existing skill. PRD-specific content is checked by `v_cli::domain_cli_case_matches_prd` (added). |
| D-5 | README.md, "Evals" table row `domain-cli`; `v_cli::domain_cli_skill_and_readme_follow_prd` | PASS | Row has the exact command, score 1.00 and cost $0.38. The score itself is not reproducible in `cargo test`. |

Behaviour specified outside the AC list:

| PRD section | Test | Status |
|---|---|---|
| Deliver: README "Domains" line | `v_cli::domain_cli_skill_and_readme_follow_prd` | PASS (added) |
| Eval case: prompt body verbatim, tags `[domain, cli]`, `tool_used` Skill `rust-domain-cli` without `max`, `llm` rubric from expected behaviour, scaffold is cli-notes | `v_cli::domain_cli_case_matches_prd` | PASS (added) |
| Rules: description first sentence names clap, mentions core `rust-forge`; first body step is `forge-tools domains` + docs.rs | `v_cli::domain_cli_skill_and_readme_follow_prd` | PASS (added) |
| Out-of-scope forge-tools change 2c58cea (G4 skipped without a lib target) | `gates::bin_only_crate_skips_doctests_and_stays_green` + `v_cli::lib_crate_still_runs_doctests` | PASS (added the second one) |

## Claims checked in skill and agent text
Sources: crate sources of the exact versions in `~/.cargo/registry/src` (same content as docs.rs for those versions) and `cargo search` on 2026-11-06, which reports clap 4.6.7, assert_cmd 2.2.2, predicates 3.1.4 as newest.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-domain-cli/SKILL.md | `metadata.forge-verified: clap=4.6.7` is the newest stable | `cargo search clap` → 4.6.7 | kept |
| skills/rust-domain-cli/SKILL.md | checked against assert_cmd 2.2.2, predicates 3.1.4 | `cargo search` → both newest | kept |
| skills/rust-domain-cli/SKILL.md | `#[derive(Parser)]`, `#[derive(Subcommand)]`, `#[derive(Args)]`, `#[derive(ValueEnum)]` | clap_builder-4.6.7/src/derive.rs (traits `Parser` l.29, `Subcommand` l.262, `Args` l.227, `ValueEnum` l.293); clap re-exports derives with `derive` feature | kept |
| skills/rust-domain-cli/SKILL.md | `#[command(subcommand)]`, `#[command(flatten)]` | clap_derive-4.6.7/src/attr.rs (`"flatten"` l.122, subcommand magic attr) | kept |
| skills/rust-domain-cli/SKILL.md | `#[arg(global = true)]` | clap_builder-4.6.7/src/builder/arg.rs `Arg::global` l.917 (derive maps `arg(x = v)` to builder methods) | kept |
| skills/rust-domain-cli/SKILL.md | `#[arg(env = ...)]` needs clap's `env` cargo feature | clap-4.6.7/Cargo.toml `env = ["clap_builder/env"]` l.128; clap_derive attr `"env"` l.121 | kept |
| skills/rust-domain-cli/SKILL.md | `Command::debug_assert` via `CommandFactory::command()` | clap_builder-4.6.7/src/builder/command.rs l.620; derive.rs `CommandFactory::command` l.120 | kept |
| skills/rust-domain-cli/SKILL.md | clap exits with 2 on parse errors | clap_builder-4.6.7/src/util/mod.rs `USAGE_CODE: i32 = 2` | kept |
| skills/rust-domain-cli/SKILL.md | doc comments become help text | clap derive reference (`verbatim_doc_comment` attr in clap_derive-4.6.7/src/attr.rs; doc comments → about/help) | kept |
| skills/rust-domain-cli/SKILL.md | `std::process::ExitCode` from `main`; `std::process::exit` skips destructors | std docs (`process::exit`: "no destructors on the current stack or any other thread's stack will be run") | kept |
| skills/rust-domain-cli/SKILL.md | anyhow `.context(...)` | anyhow-1.0.100/src/lib.rs `Context::context` l.622 | kept |
| skills/rust-domain-cli/SKILL.md | assert_cmd `Command::cargo_bin("<bin>")`, `.assert()`, `.success()`, `.failure()`, `.code(...)`, `.stdout(...)`, `.stderr(...)` | assert_cmd-2.2.2/src/cmd.rs l.70 (not deprecated in 2.2.2), src/assert.rs l.161/194/277/379/479 | kept |
| skills/rust-domain-cli/SKILL.md | `predicates::str::contains` | predicates-3.1.4/src/str/basics.rs l.283, re-exported by src/str/mod.rs l.14 | kept |
| skills/rust-domain-cli/SKILL.md | `dirs`/`directories`, `csv`, `serde_json` crates named without API details, gated through `crate-check` | principle only, no API claims; crates exist on crates.io (scaffold depends on `dirs`, `csv`, `serde_json`) | kept |
| skills/rust-domain-cli/SKILL.md | no `TODO`/`TBD`/`[insert`, 102 lines ≤150, no `references/` | `plugin::no_placeholders`, `plugin::skills_follow_spec` | kept |

## Fixes made
- No defect found in the skill, the eval case or the README; no skill text changed.
- Added `tools/forge-tools/tests/v_cli.rs`:
  - `lib_crate_still_runs_doctests`: a library crate with a failing doctest must report `FAIL G4-doctest`. Before it,
    nothing caught G4 being skipped for every crate after 2c58cea; verified by forcing the skip in `gates.rs`
    (test failed), then restoring the source.
  - `domain_cli_case_matches_prd`: prompt body verbatim, tags, Skill grader targets `rust-domain-cli` with no `max`,
    `llm` rubric covers clap derive, value enum, flag over config, non-zero exit, assert_cmd.
  - `domain_cli_skill_and_readme_follow_prd`: description trigger and core-skill mention, first step
    `forge-tools domains` + docs.rs, README "Domains" line and "Evals" row.
- No test was weakened or ignored between 93fc5c8 (v-core) and HEAD: the diff on test files only adds
  `domain_cli.rs` and one test in `gates.rs`.

## Remaining risks
- Advice quality (layering order, exit-code conventions, broken-pipe handling) is not provable by tests.
- The D-5 score (1.00, $0.38) comes from one eval run with an `llm` judge; it is recorded, not reproduced.
- 2c58cea changed forge-tools behaviour although PRD 01 forbids that in this stage; it was an owner commit with a
  regression test and is now covered from both sides.
- With cargo-nextest installed, G3 fails on a crate with no tests ("no tests to run"); outside PRD 01, noted only.
- This run does not commit: the harness forbids it; the owner commits on acceptance.
