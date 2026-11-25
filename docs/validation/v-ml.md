# Validation: v-ml

Target: docs/prd/04-ml.md · Date: 2026-11-25 · HEAD before fixes: 64c8add

## Gates
| Gate | Result |
|---|---|
| fmt | PASS |
| clippy -D warnings | PASS |
| cargo test | PASS (64 passed: 62 before this run, plus 2 new in `tests/v_ml.rs`) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|
| D-1 | gates above (`cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`) | PASS | Run before and after the new test |
| D-2 | `tests/plugin.rs::skills_follow_spec`, `tests/plugin.rs::no_placeholders` | PASS | Iterates every `skills/*` dir: name, description 1–1024, ≤150 lines, `metadata.forge-verified` with semver per domain crate, `references/` named in SKILL.md; unchanged by stage 04 and by this run |
| D-3 | `tests/domain_ml.rs::domain_ml_scaffold_is_detected_as_ml_with_verified_version` | PASS | Runs `scaffold.sh` in a temp dir, then `forge-tools domains`; asserts `ml`, `skill=rust-domain-ml`, `crate=candle-core`, `version=<forge-verified candle-core>` |
| D-4 | `tests/plugin.rs::eval_suite_is_well_formed`, `tests/plugin.rs::host_buildable_domain_cases_grade_the_build` | PASS | domain-ml has `graders/*.md` with valid frontmatter, plus the `gates-pass` grader on `target/forge/state.json` |
| D-5 | README.md "Evals" table, row `domain-ml`; checked by `tests/v_ml.rs::domain_ml_skill_and_readme_follow_prd` | PASS | Row has the exact command, score 1.00 and cost $0.93. This validation did not re-run the case (paid run outside `cargo test`) |

PRD section behaviour outside the D-list, now covered by `tests/v_ml.rs`:
- "Eval case": prompt body word for word, tags `[domain, ml]`, `tool_used` Skill grader for `rust-domain-ml`, `llm` rubric with the expected-behaviour terms (Metal feature, memory-maps, normalizes, tiny fixture model / recorded tensors, network) (`domain_ml_case_matches_prd`).
- "Deliver": scaffold writes `ml-embed`, whose `Cargo.toml` declares candle-core at the forge-verified version and whose `Cargo.lock` pins it (`domain_ml_case_matches_prd`); SKILL.md ≤150 lines, exact `crate=x.y.z` pairs for candle-core, burn and llama-cpp-2, README Domains line (`domain_ml_skill_and_readme_follow_prd`).
- "Rules for this skill": description length, first sentence names candle, burn and llama.cpp, mentions `rust-forge`; first body section runs `forge-tools domains`, points to docs.rs and says why (version drift); no TODO/TBD/`[insert` (`domain_ml_skill_and_readme_follow_prd`).
- "Skill content": safetensors, GGUF, metal/cuda features, Metal on Apple Silicon, CPU fallback, memory-mapping, tokenizer, batching, tiny models and fixtures, `benches/` (`domain_ml_skill_and_readme_follow_prd`).
- Regression for commit 64c8add: the skill must say burn's default features enable no backend. Mutation check: with SKILL.md from ba3493f, `domain_ml_skill_and_readme_follow_prd` fails at `v_ml.rs:147`; with the current text it passes.

Weakened tests: `git diff a197548..64c8add -- tools evals` shows only the added `tests/domain_ml.rs` and the new `evals/domain-ml/` case; no `#[ignore]`, no removed assertions, no `#[allow]`, no edits to existing fixtures.

Skill files: `skills/rust-domain-ml/` has only SKILL.md (112 lines), no `references/`; no agent files touched by PRD 04.

## Claims checked in skill and agent text
Sources: crate sources of candle-core 0.11.1, candle-nn 0.11.1 and tokenizers 0.23.2 (from the local cargo registry, as published on crates.io); crates.io API (`max_stable_version`, version `features`, `dependencies`) for candle-core, burn, burn-core, burn-nn, llama-cpp-2, llama-cpp-sys-2 and tokenizers on 2026-11-25.

| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|
| skills/rust-domain-ml/SKILL.md | `forge-verified: candle-core=0.11.1,burn=0.22.1,llama-cpp-2=0.1.161` are the newest stable | crates.io API `max_stable_version`: candle-core 0.11.1, burn 0.22.1, llama-cpp-2 0.1.161 | kept |
| skills/rust-domain-ml/SKILL.md | tokenizers 0.23.2 checked | crates.io API `max_stable_version` tokenizers 0.23.2 | kept |
| skills/rust-domain-ml/SKILL.md | candle-core and candle-nn features `metal`, `cuda`, `mkl`, `accelerate`; none default | candle-core 0.11.1 `Cargo.toml` `[features]` (`default = []`, lines 34–57); candle-nn 0.11.1 `Cargo.toml` (`accelerate`, `cuda`, `metal`, `mkl`, `default = []`) | kept |
| skills/rust-domain-ml/SKILL.md | burn features `ndarray`, `cpu`, `wgpu`, `metal`, `cuda`; default enables no backend | crates.io burn 0.22.1 `features`; `default` = std, optim, burn-core/default, …; burn-core 0.22.1 `default` references backends only as optional `burn-cpu?/…`, `burn-wgpu?/…` | kept |
| skills/rust-domain-ml/SKILL.md | llama-cpp-2 features `metal`, `cuda`, `vulkan`; CPU build by default | crates.io llama-cpp-2 0.1.161 `features`: default = openmp, android-shared-stdcxx, common; metal/cuda/vulkan forward to llama-cpp-sys-2 | kept |
| skills/rust-domain-ml/SKILL.md | llama-cpp-2 needs a C/C++ toolchain and cmake at build time | crates.io llama-cpp-sys-2 0.1.161 build dependencies: `cc`, `cmake`, `bindgen` | kept |
| skills/rust-domain-ml/SKILL.md | `Device::metal_if_available(0)` / `cuda_if_available(0)` return `Device::Cpu` when the feature is off, else try the device and may error | candle-core 0.11.1 `src/device.rs:339-353` | kept |
| skills/rust-domain-ml/SKILL.md | `candle_core::utils::metal_is_available()` only reports the feature | candle-core 0.11.1 `src/utils.rs:598` (`cfg!(feature = "metal")`) | kept |
| skills/rust-domain-ml/SKILL.md | dtypes `F16`, `BF16`, `F32` | candle-core 0.11.1 `src/dtype.rs:21-25` | kept |
| skills/rust-domain-ml/SKILL.md | `unsafe { VarBuilder::from_mmaped_safetensors(&paths, dtype, &device) }` | candle-nn 0.11.1 `src/var_builder.rs:642` (`pub unsafe fn`, `paths: &[P], dtype, dev: &Device`) | kept |
| skills/rust-domain-ml/SKILL.md | safe `VarBuilder::from_slice_safetensors`, `VarBuilder::from_tensors` | candle-nn 0.11.1 `src/var_builder.rs:658`, `:622` | kept |
| skills/rust-domain-ml/SKILL.md | `candle_core::safetensors::save` | candle-core 0.11.1 `src/safetensors.rs:421` | kept |
| skills/rust-domain-ml/SKILL.md | candle reads GGUF for its quantized models | candle-core 0.11.1 `src/quantized/gguf_file.rs` | kept |
| skills/rust-domain-ml/SKILL.md | `Tokenizer::from_file`, `with_padding`, `with_truncation`, `encode_batch` | tokenizers 0.23.2 `src/tokenizer/mod.rs:468,661,687,1337` | kept |
| skills/rust-domain-ml/SKILL.md | tokenizers default features pull in C code (`onig`) | tokenizers 0.23.2 `Cargo.toml` `default = ["progressbar", "onig", "esaxx_fast"]` | kept |
| skills/rust-domain-ml/SKILL.md | safetensors loading needs no code execution, unlike pickled checkpoints | safetensors format (header + named raw tensors) as implemented by candle-core 0.11.1 `src/safetensors.rs` | kept |

## Fixes made
- No defects found in the skill, eval case or tests; the burn default-feature error was already fixed in 64c8add before this run.
- Added `tools/forge-tools/tests/v_ml.rs` (2 tests) covering the PRD sections outside D-1..D-5 and the D-5 README row, including a regression check for the burn no-default-backend statement (fails on the ba3493f text).

## Remaining risks
- Advice quality (choice between stacks, batching, dtype guidance) is not provable by tests.
- The `llm` grader's judgement and the single eval run (score 1.00, $0.93) were not re-run here.
- GPU paths (Metal, CUDA) are platform-only; nothing in `cargo test` builds them.
- burn and llama-cpp-2 feature claims were checked against crates.io metadata, not compiled.
