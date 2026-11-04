# Validation run: {{STAGE}}

The previous loki run implemented `{{TARGET_PRD}}`. This run checks that work independently, fixes only real
defects, and writes an audit report. You did not write that code; do not trust its own claims or its tests by
default.

## Do

1. Read `{{TARGET_PRD}}` in full. For any later PRD, also read sections 6, 7, 8 and 10 of `docs/prd/00-core.md`.
2. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and
   `cargo test --workspace`. Record the results.
3. For every acceptance criterion in the target PRD, find the test that proves it and read that test. A criterion
   counts as covered only if the test would fail when the behaviour is broken. If a test is missing or too weak,
   add or strengthen it; it must fail before your fix and pass after.
4. Look for tests that were weakened or bypassed in the implementation commits since the previous validation report
   (`git log`, `git diff` on test files): `#[ignore]`, deleted or loosened assertions, `#[allow]` without a reason,
   fixtures edited to make a test pass. Restore what was weakened.
5. Check every skill and agent file the target PRD touches:
   - frontmatter rules from the target PRD and AC-10 of the core PRD;
   - every file under `references/` is named in its SKILL.md with the condition for reading it;
   - line budgets; no `TODO`, `TBD`, `[insert`;
   - every concrete crate, API name, attribute, CLI flag or code example: verify it on docs.rs, the crate's
     official docs or its repository for the version the text states (or the newest version if none is stated).
     Fix or delete any claim you cannot verify. A wrong API example in a skill is worse than none;
   - `metadata.forge-verified` names the versions the text was actually checked against.
6. Check every eval case the target PRD adds or changes: graders test the behaviour the PRD asks for, and none would
   pass on a run that did nothing (for example a positive case whose only grader is `max: 0`). If the PRD asked for
   a single run of the case, confirm the README row exists.
7. Fix defects with the smallest change. Do not add features, options or refactors beyond the target PRD.
8. Write `docs/validation/{{STAGE}}.md` and commit it with the fixes.

## Report format (`docs/validation/{{STAGE}}.md`)

```markdown
# Validation: {{STAGE}}

Target: {{TARGET_PRD}} · Date: <YYYY-MM-DD> · HEAD before fixes: <sha>

## Gates
| Gate | Result |
|---|---|
| fmt | PASS/FAIL |
| clippy -D warnings | PASS/FAIL |
| cargo test | PASS/FAIL (<n> passed) |

## Acceptance criteria
| AC | Test proving it | Status | Note |
|---|---|---|---|

## Claims checked in skill and agent text
| File | Claim | Source checked | Verdict (kept/fixed/removed) |
|---|---|---|---|

## Fixes made
- <what, why, which test now covers it>

## Remaining risks
- <anything not provable by tests, e.g. advice quality, platform-only behaviour>
```

## Acceptance criteria

- V-1 fmt, clippy with `-D warnings` and `cargo test --workspace` pass after this run.
- V-2 `docs/validation/{{STAGE}}.md` exists in the format above, every acceptance criterion of the target PRD has a
  row with status PASS and a named test (or, for README/text-only criteria, the file and section checked).
- V-3 Every claim row has a source; no row says "not checked".
- V-4 No test was deleted or ignored by this run.
