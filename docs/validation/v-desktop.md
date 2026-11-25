# Validation: v-desktop

Target: docs/prd/06-desktop.md · Date: 2026-11-25 · HEAD before fixes: fa4e1f8

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (70 passed: 68 before this run, plus 2 new in `tests/v_desktop.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | gates above (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) | PASS | Run before and after this run's changes |
| D-2 | `tests/plugin.rs::skills_follow_spec`, `tests/plugin.rs::no_placeholders` | PASS | Iterates every `skills/*` dir: name, description 1–1024, ≤150 lines (SKILL.md is 107), exact `metadata.forge-verified` for the key crate `tauri`, `references/` named in SKILL.md (none exist). Not changed by stage 06 or by this run |
| D-3 | `tests/domain_desktop.rs::domain_desktop_scaffold_is_detected_as_desktop_with_verified_version` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`. Asserts `desktop`, `skill=rust-domain-desktop`, `crate=tauri` and `version=<forge-verified tauri>` |
| D-4 | `tests/plugin.rs::eval_suite_is_well_formed`, `tests/plugin.rs::host_buildable_domain_cases_grade_the_build` | PASS | domain-desktop has `skill-fired` (tool_used), `desktop-behaviour` (llm) and `gates-pass` (regex on `target/forge/state.json`) |
| D-5 | README.md "Evals" table, row `domain-desktop`; checked by `tests/v_desktop.rs::domain_desktop_skill_and_readme_follow_prd` | PASS | The row has the exact command, score 1.00 and cost $0.32. This validation did not re-run the case (paid run outside `cargo test`) |

PRD section behaviour outside the D-list, now covered by `tests/v_desktop.rs`:
- "Eval case": prompt body word for word; tags `[domain, desktop]`; `tool_used` Skill grader for `rust-domain-desktop`; the `llm` rubric contains the PRD's expected-behaviour sentence plus `list_notes`, `add_note`, `capabilities/`, "no `tauri` dependency", wildcard and a FAIL clause (`domain_desktop_case_matches_prd`).
- "Deliver": the scaffold writes `desktop-notes`, whose `Cargo.toml` declares tauri at the forge-verified version. `Cargo.lock` pins that version. The core crate has no tauri dependency. The stub has no commands and no capability, so a run that does nothing cannot satisfy the rubric (`domain_desktop_case_matches_prd`). SKILL.md is ≤150 lines with no `references/`, `tauri=x.y.z` is an exact version, and the README Domains line exists (`domain_desktop_skill_and_readme_follow_prd`).
- "Rules for this skill": description length; the first sentence names Tauri and desktop; the description names the `tauri` crate and mentions `rust-forge`; the first body section runs `forge-tools domains`, points to docs.rs and says why (version drift); no TODO/TBD/`[insert` (`domain_desktop_skill_and_readme_follow_prd`).
- "Skill content": `#[tauri::command]`, `generate_handler!`, capabilities, least privilege, `allow-<command>`, `Builder::manage`, `tauri::State`, a core crate with no tauri dependency, platform prerequisites (WebKitGTK, Xcode, WebView2), `tauri::test::mock_builder`, testing without a window (`domain_desktop_skill_and_readme_follow_prd`).
- Mutation check: replacing "A second `invoke_handler` call replaces the first" with a wrong statement makes `domain_desktop_skill_and_readme_follow_prd` fail at `v_desktop.rs:179`. Restoring the text makes it pass.

Weakened tests: `git diff 8eb37e6..fa4e1f8 -- tools evals scripts hooks` shows only the added `tests/domain_desktop.rs` and the new `evals/domain-desktop/` case. There is no `#[ignore]`, no removed assertion, no `#[allow]`, and no edit to an existing fixture or test.

Skill and agent files: `skills/rust-domain-desktop/` has only SKILL.md and no `references/`. PRD 06 touches no agent files.

Eval graders: a run that does nothing fails the `llm` grader (no commands, no capability, no new core tests). `skill-fired` needs a real Skill call. `gates-pass` alone could pass on the untouched scaffold, so it is not the only grader.

## Claims checked in skill and agent text
Sources: crates.io API `max_stable_version` on 2026-11-25 (tauri 2.12.7, tauri-build 2.7.6, tauri-driver 2.1.0). Published crate sources (the same sources docs.rs renders) of tauri 2.12.7, tauri-build 2.7.6, tauri-macros 2.7.6, tauri-codegen 2.7.6 and tauri-utils 2.10.4 (the versions tauri 2.12.7 resolves to).

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-domain-desktop/SKILL.md | `forge-verified: tauri=2.12.7`; text checked against tauri-build 2.7.6 (newest stable) | crates.io API `max_stable_version` for tauri and tauri-build | kept |
| skills/rust-domain-desktop/SKILL.md | `#[tauri::command]`, `tauri::generate_handler![...]` passed to `Builder::invoke_handler` | tauri 2.12.7 `src/app.rs` `Builder::invoke_handler` doc example | kept |
| skills/rust-domain-desktop/SKILL.md | A second `invoke_handler` call replaces the first | tauri 2.12.7 `src/app.rs:1723` (`self.invoke_handler = Box::new(invoke_handler)`) | kept |
| skills/rust-domain-desktop/SKILL.md | Arguments are camelCase by default (`noteId` for `note_id`) | tauri-macros 2.7.6 `src/command/wrapper.rs:51` (`argument_case: ArgumentCase::Camel`), `rename_all` accepts `camelCase`/`snake_case` | kept |
| skills/rust-domain-desktop/SKILL.md | Make a command `async` so it does not block the main thread | tauri-macros 2.7.6 `src/lib.rs:48` ("Runs the command on the async runtime instead of blocking the main thread") | kept |
| skills/rust-domain-desktop/SKILL.md | Capabilities in `src-tauri/capabilities/` with `identifier`, `windows`, `permissions`; JSON or TOML | tauri-utils 2.10.4 `src/acl/capability.rs:117,163,195`; `src/acl/build.rs` permission formats | kept |
| skills/rust-domain-desktop/SKILL.md | tauri-build generates JSON schemas under `gen/schemas/` | tauri-build 2.7.6 `src/lib.rs:17`; tauri-utils `acl/build.rs:50` | kept |
| skills/rust-domain-desktop/SKILL.md | `tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[...]))`, `tauri_build::try_build(attributes)` | tauri-build 2.7.6 `src/lib.rs:479,565,665`, `src/acl.rs:91,104` | kept |
| skills/rust-domain-desktop/SKILL.md | Generates `allow-<command>` and `deny-<command>` | tauri-utils 2.10.4 `acl/build.rs` `autogenerate_command_permissions` (`allow-{slug}` / `deny-{slug}`, `_`→`-`) | kept |
| skills/rust-domain-desktop/SKILL.md | Without an app manifest, app commands are not gated by permissions | tauri 2.12.7 `src/webview/mod.rs` (ACL enforced only if plugin command, `has_app_acl_manifest`, or remote origin) | kept |
| skills/rust-domain-desktop/SKILL.md | `Builder::manage`, `tauri::State<'_, T>`; unmanaged state fails at runtime | tauri 2.12.7 `src/app.rs:2055`; `src/state.rs:61-67` ("state not managed for field ...") | kept |
| skills/rust-domain-desktop/SKILL.md | `tauri.conf.json` or `Tauri.toml`; `build.frontendDist`, `build.devUrl` | tauri-utils 2.10.4 `src/config/parse.rs:47`; `src/config.rs:3852-3868` (`rename_all = "camelCase"`) | kept |
| skills/rust-domain-desktop/SKILL.md | `generate_context!()` needs the config and `icons/icon.png` (RGBA) | tauri-codegen 2.7.6 `src/context.rs:221-240`, `src/image.rs:18` | kept |
| skills/rust-domain-desktop/SKILL.md | `tauri_build::build()` | tauri-build 2.7.6 `src/lib.rs:647` | kept |
| skills/rust-domain-desktop/SKILL.md | Linux needs WebKitGTK 4.1 | tauri 2.12.7 `Cargo.toml:334` (`webkit2gtk` dependency on Linux/BSD targets; webkit2gtk 2.x binds WebKitGTK 4.1); official Tauri 2 prerequisites guide | kept |
| skills/rust-domain-desktop/SKILL.md | macOS needs the Xcode CLT, Windows needs the MSVC build tools and WebView2 | Official Tauri 2 prerequisites guide (v2.tauri.app/start/prerequisites); not provable from crate source | kept |
| skills/rust-domain-desktop/SKILL.md | `test` feature; `tauri::test::mock_builder`, `mock_context`, `get_ipc_response` | tauri 2.12.7 `Cargo.toml:118` (`test = []`), `src/test/mod.rs:109,166,297` | kept |
| skills/rust-domain-desktop/SKILL.md | `tauri-driver` for WebDriver end-to-end tests | crates.io `tauri-driver` 2.1.0 ("Webdriver server for Tauri applications") | kept |
| skills/rust-domain-desktop/SKILL.md | tauri-build checks that every referenced permission exists | tauri-build 2.7.6 `src/acl.rs:397` ("Permission {} not found ...") | kept |
| skills/rust-domain-desktop/SKILL.md | Tauri 1 `allowlist` does not apply to Tauri 2 | tauri-utils 2.10.4 `src/config.rs` (no `allowlist` key; root `Config` has `deny_unknown_fields`, line 4072) | kept |

## Fixes made
- No defects found in the skill text, the eval case or the existing tests.
- Added `tests/v_desktop.rs` (2 tests). They cover the PRD 06 section behaviour that D-1..D-4 do not: prompt, tags, rubric, scaffold shape, description and first-step rules, skill content, and the verified API facts on invoke_handler and argument case. They also cover the D-5 README row.

## Remaining risks
- Advice quality (error type shape, lock scope, capability granularity) is judged only by the `llm` grader in a paid eval run, not by `cargo test`.
- The D-5 score comes from a single macOS run and was not repeated here. On Linux the app crate needs WebKitGTK, so `gates-pass` depends on the host.
- The platform prerequisites for macOS and Windows come from the official guide and cannot be checked from the crate sources.
- The `gates-pass` grader depends on the Stop hook resolving dependencies that the fixture `Cargo.lock` does not pin (only tauri and tauri-build are pinned), with network access in the eval sandbox.
