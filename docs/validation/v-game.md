# Validation: v-game

Target: docs/prd/02-game.md · Date: 2026-11-06 · HEAD before fixes: 9ee9501

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (58 passed: 55 before this run, plus 3 new in `tests/v_game.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | gates above (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) | PASS | Run before and after the new tests |
| D-2 | `tests/plugin.rs::skills_follow_spec`, `tests/plugin.rs::no_placeholders` | PASS | Iterates every `skills/*` dir: name, description 1–1024, ≤150 lines, `metadata.forge-verified` with semver per domain crate, every `references/` file named in SKILL.md; unchanged by this stage |
| D-3 | `tests/domain_game.rs::domain_game_scaffold_is_detected_as_game_with_verified_version` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`; asserts `game`, `skill=rust-domain-game`, `crate=bevy` |
| D-4 | `tests/plugin.rs::eval_suite_is_well_formed`, `tests/plugin.rs::host_buildable_domain_cases_grade_the_build` | PASS | domain-game has `graders/*.md`, valid frontmatter keys, and the `gates-pass` grader required by 00-core §7.1 |
| D-5 | README.md "Evals" table, row `domain-game`; checked by `tests/v_game.rs::domain_game_skill_and_readme_follow_prd` | PASS | Row has the exact command, score 1.00 and cost $0.46. This validation did not re-run the case (it is a paid run outside `cargo test`) |
| D-6 | `tests/domain_game.rs::domain_game_scaffold_is_detected_as_game_with_verified_version` | PASS | Asserts `version=<forge-verified bevy>`; `tests/v_game.rs::scaffold_lock_pins_verified_bevy` also checks that the scaffold's Cargo.lock pins it (mutation to 0.19.0 makes it fail) |

PRD section behaviour outside the D-list, now covered by `tests/v_game.rs`:
- "Eval case": prompt body word for word, tags `[domain, game]`, `tool_used` Skill grader targeting `rust-domain-game` with no `max:` cap, `llm` rubric containing the expected-behaviour terms, scaffold named game-stamina (`domain_game_case_matches_prd`).
- "Deliver": Cargo.lock pins bevy to `metadata.forge-verified`, Cargo.toml declares bevy (`scaffold_lock_pins_verified_bevy`); `references/bevy-testing.md` covers the four required topics and SKILL.md names it with a reading condition; README Domains line (`domain_game_skill_and_readme_follow_prd`).
- "Rules for this skill": the first sentence of the description names Bevy, the description mentions `rust-forge`, the first body section runs `forge-tools domains` and points to docs.rs (`domain_game_skill_and_readme_follow_prd`).
- "Skill content": ECS parts, plugins, ordering, states, `dynamic_linking` restricted to dev builds, assets, headless `MinimalPlugins` testing, migration one minor version at a time with `cargo check` (`domain_game_skill_and_readme_follow_prd`).

## Claims checked in skill and agent text
Source for every row: the bevy 0.20.1 crate sources downloaded from static.crates.io (`bevy`, `bevy_app`, `bevy_ecs`, `bevy_time`, `bevy_input`, `bevy_state`, `bevy_asset`, `bevy_internal`, all 0.20.1). crates.io API: `max_stable_version` = 0.20.1 on 2026-11-06.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| SKILL.md | `metadata.forge-verified: bevy=0.20.1` is the newest stable | crates.io API `crates/bevy` | kept |
| SKILL.md | `MessageWriter::write`, `MessageReader` | bevy_ecs 0.20.1 `src/message/message_writer.rs:74` | kept |
| SKILL.md | `App::add_message` | bevy_app 0.20.1 `src/app.rs:427` | kept |
| SKILL.md | `App::configure_sets`, `#[derive(SystemSet)]`, `.chain()`, `.before`/`.after`, `.run_if` | bevy_app 0.20.1 `src/app.rs:402`; bevy_ecs 0.20.1 `src/schedule/` | kept |
| SKILL.md | `#[derive(States)]`, `App::init_state`, `OnEnter`/`OnExit`, `in_state`, `NextState::set` | bevy_state 0.20.1 `src/app.rs`, `src/state/transitions.rs`, `src/condition.rs`, `src/state/resources.rs` | kept |
| SKILL.md, bevy-testing.md | `StatesPlugin` is not in `MinimalPlugins`; path `bevy::state::app::StatesPlugin` | bevy_internal 0.20.1 `src/default_plugins.rs` (MinimalPlugins = TaskPool, FrameCount, Time, ScheduleRunner), `src/lib.rs:99` (`pub use bevy_state as state`); bevy_state `src/lib.rs:55` (`pub mod app`) | kept |
| SKILL.md | `AssetServer::load` returns a handle immediately; dropping all handles unloads | bevy_asset 0.20.1 `src/server/mod.rs` | kept |
| SKILL.md | `Res<Time>`, `Time::delta_secs()` | bevy_time 0.20.1 `src/time.rs` | kept |
| SKILL.md | feature `dynamic_linking`; `cargo run --features bevy/dynamic_linking` | bevy 0.20.1 `Cargo.toml:2909` | kept |
| SKILL.md | `Startup`, `Update`, `FixedUpdate`, `Commands`, `Query`, `Res`, `ResMut` | bevy_app / bevy_ecs 0.20.1 | kept |
| bevy-testing.md | `MinimalPlugins` contents; `App::update` vs `App::run` | bevy_internal 0.20.1 `src/default_plugins.rs:163`; bevy_app `src/app.rs` | kept |
| bevy-testing.md | `TimeUpdateStrategy::ManualDuration(Duration)` | bevy_time 0.20.1 `src/lib.rs:125` | kept |
| bevy-testing.md | `ButtonInput<KeyCode>` `press`, `release`, `clear` | bevy_input 0.20.1 `src/button_input.rs:252` | kept |
| bevy-testing.md | `World::write_message`, `World::query`, `EntityRef::get`, `world_mut().spawn`, `resource_mut` | bevy_ecs 0.20.1 `src/world/mod.rs:3041` | kept |
| evals/domain-game/scaffold.sh | `Single`, `Camera2d`, `Sprite::from_color`, `ButtonInput<KeyCode>` compile on 0.20.1 | bevy_ecs 0.20.1 `src/system/query.rs` (`Single`); the README Evals row records a green Stop-hook `gates full` on this scaffold with bevy 0.20.1 | kept |

## Fixes made
- No defects found in the skill, the reference, the eval case or forge-tools.
- Added `tools/forge-tools/tests/v_game.rs` (3 tests) for PRD 02 behaviour that had no test: the eval case's prompt, graders and scaffold, the Cargo.lock pin, the skill rules and content topics, the reference topics, and the README lines. A mutation of the lock pin (0.20.1 → 0.19.0) made `scaffold_lock_pins_verified_bevy` fail; restoring it made the test pass.
- Test-weakening check (`git diff 2295629 HEAD` on test files): only additions (`domain_game.rs`, `gates.rs`, `plugin.rs`); no `#[ignore]`, no deleted or loosened assertions, no `#[allow]` without a reason.

## Remaining risks
- How good the advice is (ordering guidance, the TimeUpdateStrategy warm-up note, migration procedure) is checked only by the single eval run, not by tests.
- The scaffold is not compiled in `cargo test` (out of scope per PRD); a compile was shown only by the eval run recorded in the README.
- The `llm` grader is non-deterministic; one run at 1.00 is weak evidence of stable triggering.
