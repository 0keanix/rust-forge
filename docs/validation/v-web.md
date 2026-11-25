# Validation: v-web

Target: docs/prd/05-web.md · Date: 2026-11-25 · HEAD before fixes: 72149ed

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (67 passed: 65 before this run, plus 2 new in `tests/v_web.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | gates above (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) | PASS | Run before and after the fix |
| D-2 | `tests/plugin.rs::skills_follow_spec`, `tests/plugin.rs::no_placeholders` | PASS | Iterates every `skills/*` dir: name, description 1–1024, ≤150 lines (SKILL.md now 124), `metadata.forge-verified` exact semver for the domain's key crate `axum`, `references/` named in SKILL.md (none exist); unchanged by stage 05 and by this run |
| D-3 | `tests/domain_web.rs::domain_web_scaffold_is_detected_as_web_with_verified_version` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`; asserts `web`, `skill=rust-domain-web`, `crate=axum`, `version=<forge-verified axum>` |
| D-4 | `tests/plugin.rs::eval_suite_is_well_formed`, `tests/plugin.rs::host_buildable_domain_cases_grade_the_build` | PASS | domain-web has three graders with valid types, the Skill grader names an existing skill, plus the `gates-pass` grader on `target/forge/state.json` |
| D-5 | README.md "Evals" table, row `domain-web`; checked by `tests/v_web.rs::domain_web_skill_and_readme_follow_prd` | PASS | Row has the exact command, score 1.00 and cost $0.45. This validation did not re-run the case (paid run outside `cargo test`) |

PRD section behaviour outside the D-list, now covered by `tests/v_web.rs`:
- "Eval case": prompt body word for word, tags `[domain, web]`, `tool_used` Skill grader for `rust-domain-web`, `llm` rubric containing the PRD's expected-behaviour sentence plus `IntoResponse`, migration, `.sqlx`, `oneshot`, port (`domain_web_case_matches_prd`).
- "Deliver": scaffold writes `web-notes-api` with an empty `[workspace]`, whose `Cargo.toml` declares axum at the forge-verified version and whose `Cargo.lock` pins it (`domain_web_case_matches_prd`); SKILL.md ≤150 lines, exact `axum=x.y.z`, README Domains line (`domain_web_skill_and_readme_follow_prd`).
- "Rules for this skill": description length, first sentence names axum, tokio and tower, mentions `rust-forge`; first body section runs `forge-tools domains`, points to docs.rs and says why (version drift); no TODO/TBD/`[insert` (`domain_web_skill_and_readme_follow_prd`).
- "Skill content": routing (`Router::`), extractors, `AppState` / `Arc<Mutex<_>>`, `IntoResponse`, tower layers, `cargo sqlx prepare`, migrations, `with_graceful_shutdown`, tracing, `oneshot`, tests without binding a port (`domain_web_skill_and_readme_follow_prd`).
- Regression for the `.sqlx` fix below: with SKILL.md from 72149ed, `domain_web_skill_and_readme_follow_prd` fails at `v_web.rs:156`; with the current text it passes.

Weakened tests: `git diff 316f25a..72149ed -- tools evals` shows only the added `tests/domain_web.rs`, the new `evals/domain-web/` case and the scaffold dependency change in 72149ed; no `#[ignore]`, no removed assertions, no `#[allow]`, no edits to existing fixtures or tests.

Skill files: `skills/rust-domain-web/` has only SKILL.md, no `references/`; no agent files touched by PRD 05.

Eval graders: a run that does nothing fails the `llm` grader (no handlers, migration or tests); `skill-fired` needs a real Skill call; `gates-pass` alone would pass on the untouched scaffold, which is why it is not the only grader.

## Claims checked in skill and agent text
Sources: crates.io API (`max_stable_version`, version `features`) on 2026-11-25; docs.rs pages for axum 0.8.9, sqlx 0.9.1, tower 0.5.3, tower-http 0.7.1, tokio 1.54.1, tracing 0.1.44; sqlx-cli README at tag v0.9.1 on GitHub.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-domain-web/SKILL.md | `forge-verified: axum=0.8.9`; text checked against tokio 1.54.1, tower 0.5.3, tower-http 0.7.1, sqlx 0.9.1, tracing 0.1.44 (newest stable) | crates.io API `max_stable_version` for each crate | kept |
| skills/rust-domain-web/SKILL.md | Path captures `/{id}`, wildcard `{*rest}`; `/:id` is rejected in 0.8 | docs.rs axum 0.8.9 `Router::route` (Wildcards), `Router::without_v07_checks` ("Adding such routes without calling this method first will panic") | kept |
| skills/rust-domain-web/SKILL.md | `Router::merge`, `Router::nest`, `Router::with_state`, `Router::layer`, `Router::route_layer` | docs.rs axum 0.8.9 `struct.Router.html` | kept |
| skills/rust-domain-web/SKILL.md | `route_layer` runs only for matched routes, so unknown paths return 404 not 401 | docs.rs axum 0.8.9 `Router::route_layer` example (`GET /not-found` with invalid token → 404) | kept |
| skills/rust-domain-web/SKILL.md | Layers wrap previously added routes; the last added runs first | docs.rs axum 0.8.9 `axum::middleware` § Ordering ("executed from bottom to top") | kept |
| skills/rust-domain-web/SKILL.md | `State`, `Path`, `Query`, `Json` extractors; `FromRef` for substates; body extractor last | docs.rs axum 0.8.9 `extract::State`, `extract::FromRef`, `extract` module docs | kept |
| skills/rust-domain-web/SKILL.md | `axum::middleware::from_fn` | docs.rs axum 0.8.9 `middleware/fn.from_fn.html` | kept |
| skills/rust-domain-web/SKILL.md | `axum::serve(listener, app).with_graceful_shutdown(signal)` | docs.rs axum 0.8.9 `fn.serve.html`, `serve::Serve::with_graceful_shutdown` | kept |
| skills/rust-domain-web/SKILL.md | `tokio::signal::ctrl_c()` | docs.rs tokio 1.54.1 `signal/fn.ctrl_c.html` | kept |
| skills/rust-domain-web/SKILL.md | `axum::body::Body`; `axum::body::to_bytes` with a size limit | docs.rs axum 0.8.9 `pub async fn to_bytes(body: Body, limit: usize)` | kept |
| skills/rust-domain-web/SKILL.md | `tower::ServiceExt::oneshot` needs the `util` feature | docs.rs tower 0.5.3 `ServiceExt` ("Available on crate feature util only"); crates.io tower 0.5.3 features | kept |
| skills/rust-domain-web/SKILL.md | `tower_http::trace::TraceLayer` | docs.rs tower-http 0.7.1 `trace/struct.TraceLayer.html`; feature `trace` on crates.io | kept |
| skills/rust-domain-web/SKILL.md | `#[tracing::instrument]` with `skip(...)`; `tracing::info!` with fields | docs.rs tracing 0.1.44 `attr.instrument.html` | kept |
| skills/rust-domain-web/SKILL.md | `sqlx::migrate!("./migrations").run(&pool)`; files embedded in the binary | docs.rs sqlx 0.9.1 `macro.migrate.html` (default `./migrations`, relative to the dir with Cargo.toml) | kept |
| skills/rust-domain-web/SKILL.md | sqlx refuses changed applied migrations (checksums) | docs.rs sqlx 0.9.1 `migrate::MigrateError::VersionMismatch` | kept |
| skills/rust-domain-web/SKILL.md | `sqlx migrate add <name>` | sqlx-cli README v0.9.1 line 57 | kept |
| skills/rust-domain-web/SKILL.md | `query!`/`query_as!` offline via `cargo sqlx prepare` with `DATABASE_URL`, commit `.sqlx` | docs.rs sqlx 0.9.1 `macro.query.html` (offline mode) | kept |
| skills/rust-domain-web/SKILL.md | `.sqlx` is written "at the workspace root" | sqlx-cli README v0.9.1 lines 128–133: written in the current directory; `--workspace` for one at the workspace root | fixed |
| skills/rust-domain-web/SKILL.md | Unchecked `sqlx::query_as::<_, T>(...)` with `.bind` | docs.rs sqlx 0.9.1 `fn.query_as.html` | kept |
| skills/rust-domain-web/SKILL.md | `sqlx::SqlitePool` is a cloneable handle; `sqlx::Error::RowNotFound` | docs.rs sqlx 0.9.1 `type.SqlitePool.html`, `enum.Error.html` | kept |
| skills/rust-domain-web/SKILL.md | `sqlite::memory:` in-memory URL | docs.rs sqlx 0.9.1 `sqlite::SqliteConnectOptions` URL table | kept |
| skills/rust-domain-web/SKILL.md | sqlx features `sqlite`, `runtime-tokio`, `macros`, `migrate` (names in scaffold) | crates.io sqlx 0.9.1 `features` | kept |

## Fixes made
- SKILL.md §7 said `cargo sqlx prepare` output goes "at the workspace root"; sqlx-cli 0.9.1 writes `.sqlx` in the current directory, and only `--workspace` gives one at the workspace root. Text corrected; covered by `tests/v_web.rs::domain_web_skill_and_readme_follow_prd` (fails on the old text).
- Added `tests/v_web.rs` (2 tests) for PRD 05 section behaviour not covered by D-1..D-4 and for the D-5 README row.

## Remaining risks
- Advice quality (state without locks, error mapping, layer order choices) is judged only by the `llm` grader in a paid eval run, not by `cargo test`.
- The D-5 score comes from a single run; it was not repeated here.
- The `gates-pass` grader depends on the Stop hook resolving the dependencies not pinned in the fixture `Cargo.lock` (only axum is pinned) with network access in the eval sandbox.
