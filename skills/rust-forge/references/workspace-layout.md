# Workspace layout

- Start with one package. Split into a workspace when a part needs its own dependency set, a stable API, separate
  compilation for build time, or a separate binary.
- Use a virtual root manifest (`[workspace]` with `members`) and put members under a directory such as `crates/`.
  Set `resolver` explicitly in a virtual workspace.
- Keep dependency versions in `[workspace.dependencies]` and inherit them in members with `workspace = true`, so
  one version is used everywhere. Do the same for lints with `[workspace.lints]` and `[lints] workspace = true`.
- Keep the dependency direction one-way: a core library crate with no I/O, and thin binary or adapter crates on
  top. A cycle between crates is a design error, not a build problem.
- Put binaries in `src/main.rs` or `src/bin/`, with logic in the library so integration tests can reach it.
- Keep features additive: enabling a feature must never remove API or change behaviour of other code.
- Exclude fixture or example crates that must not join the workspace with `exclude`, or give them their own empty
  `[workspace]` table.
