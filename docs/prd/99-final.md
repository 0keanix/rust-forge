# PRD 99: rust-forge — final integration validation

All stages (`00-core-tools.md`, `00-core.md`, `01`–`08`, `09-automation.md`, `10-journal.md` in `docs/prd/`) are
implemented and each has a validation report in `docs/validation/`. This run checks the plugin as a whole, fixes only
real defects, and writes the final report.

## Do

1. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`.
2. Re-check every acceptance criterion of `00-core-tools.md`, `00-core.md`, `09-automation.md` and `10-journal.md`
   against the final tree; later stages may have broken earlier ones.
3. Add these whole-plugin tests if they do not exist:
   - F-1 there is exactly one `skills/rust-domain-<d>` for each domain in the `forge-tools domains` mapping
     (cli, game, ffi-mobile, ml, web, desktop, wasm, embedded), and no other `rust-domain-*` directory;
   - F-2 `evals/` has the 2 core cases, at least 5 trigger cases, and one `domain-<d>` case per domain whose scaffold
     is detected as that domain;
   - F-3 the README "Domains" section lists exactly the 8 domain skills, and the README "Evals" table has a row for
     every case.
4. Read all `docs/validation/*.md`. Any row not PASS, or any listed risk that a test could cover, gets a fix or a
   test now.
5. Check the skill set as a whole: descriptions do not overlap so much that two skills would trigger on the same
   request (a domain skill and the core skill complementing each other is intended); the core skill points to
   domain skills through `forge-tools domains`, not by listing them.
6. Install check: `cargo install --path tools/forge-tools --root target/forge-install` succeeds, and the installed
   binary answers `--version`.
7. Run the full suite once: `scripts/plugin-eval.sh . --ablation none --json target/forge/final-eval.json`. Record
   overall score, cases passed, cost. Any case below threshold gets a fix or an explanation in the report. Then run
   the trigger cases alone (`--tag trigger --ablation none`) and report the positive and negative hit rates.
8. Write `docs/validation/final.md` in the report format of `docs/prd/validate.md`, plus a section
   "Manual steps for the owner": add the repo as a marketplace and install the plugin; enable marketplace auto-update;
   GitHub secrets and the `FORGE_AUTOMERGE` variable from `09-automation.md`; how to stop automerge and how to revert
   an automerged change; Fable fallback via `models set --strong opus`. Commit.

## Acceptance criteria

- FV-1 fmt, clippy with `-D warnings` and `cargo test --workspace` pass.
- FV-2 Tests for F-1, F-2 and F-3 exist and pass.
- FV-3 Every acceptance criterion of the PRDs listed in step 2 has a PASS row with a named test in
  `docs/validation/final.md`.
- FV-4 The install check in step 6 passes.
- FV-5 The report gives the suite score, cost and trigger hit rates from step 7; positive trigger hit rate is at
  least 0.8 and negative false-positive rate at most 0.2, or the report names the descriptions that miss.
- FV-6 No test was deleted or ignored by this run.
