---
name: rust-forge
description: Drives Rust development in any project with a Cargo.toml or .rs files - writing, fixing, testing and reviewing Rust code, resolving compile and borrow-checker errors, running cargo, and choosing or adding crates. Use it whenever the user works on Rust code, pastes a rustc or cargo error, asks about cargo build or test behaviour, or mentions a crate, even if they do not say "Rust" explicitly. Provides the working loop with deterministic quality gates (forge-tools gates), compact compiler diagnostics, the crate-verification policy, delegation to Rust subagents, and loading of the matching rust-domain skill.
---

# rust-forge

Work on Rust in a fixed loop with deterministic gates, because the compiler and cargo are better judges of
correctness than recall. `forge-tools` provides the gates, diagnostics and crate checks; the plugin hooks call it
after edits and at stop.

## Working loop

1. **Domains.** Run `forge-tools domains` in the project root. For each line it prints, load the skill it names
   (`skill=rust-domain-<domain>`) if that skill is available, and use the printed `version=` to pick docs for
   exactly that version. "no domains detected" means only this skill applies.
2. **Plan.** For a new project, a new crate or a change that crosses crate boundaries, delegate to
   crate-architect first. For small changes, state the steps yourself.
3. **Implement in small steps.** After each step run `cargo check --workspace --all-targets`. Keep every step
   compiling; a small broken step is cheap to fix and a large one is not.
4. **Gates fast.** Run `forge-tools gates fast` (fmt, check, clippy with `-D warnings`). Fix every failure.
5. **Tests.** Add or update tests for the behaviour you changed (delegate to test-author for larger sets).
6. **Gates full.** Run `forge-tools gates full` (fast plus tests and doctests). The Stop hook runs it too and
   blocks the stop while it is red.
7. **Review.** For non-trivial changes delegate to reviewer, then address blockers and majors.

Report a SKIPPED gate as skipped, never as passed; tell the user which tool to install.

## Compile errors

Do not read raw cargo output into context. Pipe it through the compressor:

```
cargo check --workspace --all-targets --message-format=json | forge-tools diag
```

The post-edit hook already returns this compact summary after edits to `.rs` files. Fix errors by code using
`references/error-codes.md`: restructure ownership before adding lifetimes or `clone()`.

## Delegation

| Situation | Agent |
|---|---|
| New project or crate, workspace layout, crate choice, feature flags | crate-architect |
| Planned change to implement in steps | implementer |
| Build fails and the fix is not obvious | compile-fixer (pass only the `forge-tools diag` output) |
| Tests to write: unit, integration, doctests, proptest, insta | test-author |
| Change is done and gates are green | reviewer |
| Diff contains `unsafe` or FFI | unsafe-auditor |
| Dependencies added or changed, or before a release | deps-guardian |

**Escalation.** compile-fixer stops after 3 iterations and returns `UNRESOLVED` with the remaining diagnostic.
Then delegate to crate-architect with that diagnostic: repeated borrow-checker failures usually mean the ownership
design is wrong, and another local patch will not fix it.

## Crate policy (summary)

- Add dependencies only with `cargo add`; never type a dependency or version into `Cargo.toml` by hand. The
  pre-edit hook blocks hand edits that add a dependency or change a version.
- Run `forge-tools crate-check <name>` first. `NOT_FOUND` means the crate does not exist: do not add it.
  `STALE` means no stable release for 18 months: look for a maintained alternative. `UNKNOWN` means the check
  could not reach crates.io: it is not a pass.
- Before first use of an unfamiliar API, read the docs for the version recorded in `Cargo.lock`, not the version you
  remember. APIs change between versions and recalled signatures are a common source of errors.

Read `references/crate-policy.md` before choosing between crates or adding a git, path or pre-release dependency.

## Recommended lints

Suggest this block in the root `Cargo.toml` of a new workspace, with `[lints] workspace = true` in each member:

```toml
[workspace.lints.rust]
unsafe_code = "deny"

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = "deny"
dbg_macro = "deny"
todo = "deny"
```

Allow `unwrap_used` and `expect_used` in tests with `#[expect(..., reason = "...")]` or `#[allow(..., reason = "...")]`
at the test module, always with a reason. Crates that need `unsafe` override `unsafe_code` locally and get an
unsafe-auditor review.

## Reading `target/forge/state.json`

`forge-tools gates` writes this file after every run:

- `level` - the level of the last run (`fast`, `full`, `deps`, `unsafe`);
- `gates` - result per gate of the last run: `PASS`, `FAIL` or `SKIPPED`;
- `head` and `timestamp` - git HEAD and time of the last run;
- `green` - whether the last run had no failing gate;
- `last_green` - level, head and timestamp of the last run without failures;
- `stop_blocks` - consecutive Stop-hook blocks per session (after 3 the stop is let through with a warning).

If `last_green.level` is `full` and `last_green.head` equals the current HEAD with no Rust or manifest changes since,
the work is gated green. Full logs are under `target/forge/logs/`.

## References

- `references/error-codes.md` - read when fixing a compile error with one of E0382, E0499, E0502, E0505, E0597,
  E0106, E0277, E0308, E0599, E0425, or before delegating to compile-fixer.
- `references/llm-antipatterns.md` - read before reviewing a change and before finishing any implementation.
- `references/workspace-layout.md` - read when creating a project or crate, or splitting a crate.
- `references/testing.md` - read when writing or changing tests or when tests are slow.
- `references/crate-policy.md` - read before adding, replacing or choosing a dependency.
