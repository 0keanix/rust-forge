# Validation: v-ffi-mobile

Target: docs/prd/03-ffi-mobile.md · Date: 2026-11-06 · HEAD before fixes: f43b440

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (61 passed: 59 before this run, plus 2 new in `tests/v_ffi_mobile.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | gates above (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) | PASS | Run before and after the fixes |
| D-2 | `tests/plugin.rs::skills_follow_spec`, `tests/plugin.rs::no_placeholders` | PASS | Iterates every `skills/*` dir: name, description 1–1024, ≤150 lines, `metadata.forge-verified` with semver per domain crate, `references/` named in SKILL.md; unchanged by stage 03 and by this run |
| D-3 | `tests/domain_ffi_mobile.rs::domain_ffi_mobile_scaffold_is_detected_as_ffi_mobile` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`; asserts `ffi-mobile`, `skill=rust-domain-ffi-mobile`, `crate=uniffi`, `version=<forge-verified uniffi>` |
| D-4 | `tests/plugin.rs::eval_suite_is_well_formed`, `tests/plugin.rs::host_buildable_domain_cases_grade_the_build` | PASS | domain-ffi-mobile has `graders/*.md` with valid frontmatter, plus the `gates-pass` grader |
| D-5 | README.md "Evals" table, row `domain-ffi-mobile`; checked by `tests/v_ffi_mobile.rs::domain_ffi_mobile_skill_and_readme_follow_prd` | PASS | Row has the exact command, score 1.00 and cost $0.39. This validation did not re-run the case (paid run outside `cargo test`) |

PRD section behaviour outside the D-list, now covered by `tests/v_ffi_mobile.rs`:
- "Eval case": prompt body word for word, tags `[domain, ffi-mobile]`, `tool_used` Skill grader for `rust-domain-ffi-mobile`, `llm` rubric containing the expected-behaviour terms (`domain_ffi_mobile_case_matches_prd`).
- "Deliver": scaffold writes the notes workspace; `notes-ffi/Cargo.toml` declares uniffi at the forge-verified version, `Cargo.lock` pins it, the core crate has no uniffi and has add/list/search plus tests (`domain_ffi_mobile_case_matches_prd`); SKILL.md ≤150 lines, README Domains line (`domain_ffi_mobile_skill_and_readme_follow_prd`).
- "Rules for this skill": description length, first sentence names uniffi and cbindgen, mentions `rust-forge`; first body section runs `forge-tools domains`, points to docs.rs and says why (version drift); no TODO/TBD/`[insert` (`domain_ffi_mobile_skill_and_readme_follow_prd`).
- "Skill content": proc-macro vs UDL, cbindgen, cargo-ndk, XCFramework, pure core crate with its own tests, error and ownership sections, host vs device/simulator (`domain_ffi_mobile_skill_and_readme_follow_prd`).

Weakened tests: `git diff 87bfe54..f43b440 -- tools` shows only the added `tests/domain_ffi_mobile.rs`; no `#[ignore]`, no removed assertions, no `#[allow]`, no fixture edits.

## Claims checked in skill and agent text
Sources: crate sources of uniffi, uniffi_macros, uniffi_core (all 0.32.2) and cargo-ndk 4.1.2, downloaded from crates.io; crates.io API `max_stable_version` for uniffi = 0.32.2 on 2026-11-06.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-domain-ffi-mobile/SKILL.md | `forge-verified: uniffi=0.32.2` is the newest stable | crates.io API, uniffi versions list | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `uniffi::setup_scaffolding!()`, `#[uniffi::export]` | uniffi_macros 0.32.2 `src/lib.rs` (`setup_scaffolding`, `export`), re-exported by `uniffi/src/lib.rs:7` | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | derives `uniffi::Record`, `Enum`, `Object`, `Error` | uniffi_macros 0.32.2 `src/lib.rs:117-138` `proc_macro_derive` | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | UDL: build script with `build` feature, `uniffi::include_scaffolding!` | uniffi 0.32.2 `src/lib.rs:24-25` (`generate_scaffolding` under `build`), uniffi_macros `include_scaffolding` | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `uniffi::uniffi_bindgen_main()` behind the `cli` feature | uniffi 0.32.2 `src/lib.rs:8-30`, `src/cli/mod.rs:8` | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `generate --library <lib> --language kotlin\|swift --out-dir <dir>` | uniffi 0.32.2 `src/cli/uniffi_bindgen.rs:61-102`: `--library` is "Deprecated … UniFFI will auto-detect this"; `--language`, `--out-dir` exist | fixed (now `generate <lib> …`, notes `--library` is deprecated) |
| skills/rust-domain-ffi-mobile/SKILL.md | exported objects live behind `Arc` | uniffi_macros 0.32.2 `src/object.rs:151,193` (lower/lift `Arc<Self>`) | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `cargo ndk -t arm64-v8a -o <dir> build --release`, one `-t` per ABI | cargo-ndk 4.1.2 README.md:66 | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | cargo-ndk finds the NDK through `ANDROID_NDK_HOME` | cargo-ndk 4.1.2 `src/cli/mod.rs:121,465-472`, README.md:78 | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `#[unsafe(no_mangle)]` in edition 2024, `#[no_mangle]` before | doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html (shows `#[unsafe(no_mangle)]`) | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | iOS targets `aarch64-apple-ios`, `aarch64-apple-ios-sim`, `x86_64-apple-ios` | doc.rust-lang.org/rustc/platform-support.html (lists all three targets) | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `lipo` merges simulator slices; `xcodebuild -create-xcframework`; a fat library cannot hold device and simulator arm64 | developer.apple.com/documentation/xcode/creating-a-multi-platform-binary-framework-bundle (`xcodebuild -create-xcframework`; "Avoid using tools such as lipo to combine architecture slices built for iOS and iOS Simulator") | kept |
| skills/rust-domain-ffi-mobile/SKILL.md | `crate-type = ["lib", "cdylib", "staticlib"]` | doc.rust-lang.org/cargo/reference/cargo-targets.html (`lib`, `cdylib`, `staticlib` crate types) | kept |

## Fixes made
- SKILL.md §3: the bindgen command used `--library`, which uniffi 0.32.2 deprecates (library mode is auto-detected). Changed to `generate <lib> --language … --out-dir …` with a note. Covered by `tests/v_ffi_mobile.rs::domain_ffi_mobile_skill_and_readme_follow_prd`, which failed on the old text and passes now.
- Added `tests/v_ffi_mobile.rs` (2 tests) for the PRD section behaviour listed above.

## Remaining risks
- Android and iOS builds (cargo-ndk, XCFramework) and the generated Kotlin/Swift were not run; they need the NDK and macOS with Xcode. The skill states this.
- The `llm` grader and the quality of the advice are judged only by the single eval run recorded in README (score 1.00); not re-run here.
- The cbindgen section gives procedures only (no cbindgen API); its version was not pinned in `forge-verified`, as the PRD requires pairs only for crates with version-specific guidance.
