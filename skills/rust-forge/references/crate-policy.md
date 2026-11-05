# Crate policy

Invented crates and invented APIs are the most common LLM failure in Rust. This policy makes every dependency
decision checkable.

1. **Check first.** Run `forge-tools crate-check <name>` before proposing a crate.
   - `NOT_FOUND`: the crate does not exist. Do not add it; find a real one.
   - `STALE`: newest stable release is older than 18 months. Prefer a maintained alternative unless the crate is
     finished and stable by design; say which and why.
   - `UNKNOWN`: crates.io could not be reached. This is not a pass; retry or ask the user.
   - `OK`: exists and is maintained. Still compare downloads and repository against alternatives.
2. **Add with cargo.** Use `cargo add <name>` (with `--features`, `--dev` or `--build` as needed). Never write a
   dependency or version into `Cargo.toml` by hand; the pre-edit hook blocks it. Feature changes and removals by
   hand are allowed.
3. **Read docs for the locked version.** Before first use of an unfamiliar API, read the docs for the version in
   `Cargo.lock`. Do not rely on remembered signatures.
4. **Prefer fewer, common crates.** Prefer the standard library, then crates already in the tree, then widely used
   crates. Each new crate adds build time, audit surface and supply-chain risk.
5. **Git, path and pre-release dependencies** need a reason. A git dependency is allowed only with the comment
   `# forge: allow-git` on its line. Avoid pre-release versions unless the user asks.
6. **Audit.** After dependency changes, delegate to deps-guardian or run `forge-tools gates deps` (cargo-deny,
   cargo-audit, cargo-machete).
