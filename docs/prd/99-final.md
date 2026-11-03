# PRD 99: rust-forge — final integration validation

All stages (`docs/prd/00-core.md` and `01`–`08`) are implemented and each has a validation report in
`docs/validation/`. This run checks the plugin as a whole, fixes only real defects, and writes the final report.

## Do

1. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
   `cargo test --workspace`.
2. Re-check every acceptance criterion of `00-core.md` against the final tree; later stages may have broken
   earlier ones.
3. Add these whole-plugin tests if they do not exist:
   - F-1 there is exactly one `skills/rust-domain-<d>` for each domain in the `forge-tools domains` mapping
     (cli, game, ffi-mobile, ml, web, desktop, wasm, embedded), and no other `rust-domain-*` directory;
   - F-2 `evals/evals.json` has at least 10 evals: the 2 core evals and one per domain, each domain fixture detected
     as its domain;
   - F-3 the README "Domains" section lists exactly the 8 domain skills.
4. Read all `docs/validation/*.md`. Any row not PASS, or any listed risk that a test could cover, gets a fix or a
   test now.
5. Check the skill set as a whole: descriptions do not overlap so much that two skills would trigger on the same
   request (a domain skill and the core skill complementing each other is intended); the core skill points to
   domain skills through `forge-tools domains`, not by listing them.
6. Install check: `cargo install --path tools/forge-tools --root target/forge-install` succeeds, and the installed
   binary answers `--version`.
7. Write `docs/validation/final.md` in the report format of `docs/prd/validate.md`, plus a section
   "Manual checks for the owner": trigger tests per skill description, a real Claude Code session with the plugin
   installed, Fable fallback via `models set --strong opus`. Commit.

## Acceptance criteria

- FV-1 fmt, clippy with `-D warnings` and `cargo test --workspace` pass.
- FV-2 Tests for F-1, F-2 and F-3 exist and pass.
- FV-3 Every acceptance criterion of `00-core.md` has a PASS row with a named test in `docs/validation/final.md`.
- FV-4 The install check in step 6 passes.
- FV-5 No test was deleted or ignored by this run.
