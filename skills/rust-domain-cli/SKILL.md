---
name: rust-domain-cli
description: Guides building and changing Rust command-line tools on clap - argument parsing with clap derive (subcommands, global flags, value enums), config layering of defaults, config file, environment and flags, anyhow and thiserror error handling, exit codes, human and --json output, and binary tests with assert_cmd and predicates. Use it whenever a Rust project depends on clap or the user asks to add a subcommand, flag, option, config file, output format or CLI test to a Rust binary, even if they only say "the CLI" or "the tool". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: clap=4.6.7
---

# rust-domain-cli

Domain guidance for Rust CLIs. The core `rust-forge` skill owns the working loop, the gates, the crate policy and
compiler-error strategies; follow it and use this skill only for what is specific to command-line tools.

## 1. Pin the version first

Run `forge-tools domains` and read the `cli` line: `crate=clap version=<x>`. Read docs.rs and the clap guides
(the derive tutorial and reference under `clap::_derive`) for exactly that version, and do the same for
`assert_cmd` and `predicates` from `Cargo.lock` before writing tests. clap's attribute names and features shift
between minor releases, and version drift is the main source of wrong CLI code. If the output says
`version=unknown`, the lock file is missing; take the requirement from `Cargo.toml` and resolve it before relying
on details.

The API names below were checked against clap 4.6.7, assert_cmd 2.2.2 and predicates 3.1.4. If the project uses
other versions, confirm each name in that version's docs before you use it.

## 2. Arguments with clap derive

- Model the interface as types: a top-level struct with `#[derive(Parser)]`, an enum of subcommands with
  `#[derive(Subcommand)]` held in a field marked `#[command(subcommand)]`, and shared option groups as
  `#[derive(Args)]` structs pulled in with `#[command(flatten)]`. Types keep parsing, help text and dispatch in
  one place, so a new subcommand is one enum variant plus one match arm.
- Give each subcommand its own args struct when it has more than one or two options; it keeps the dispatch `match`
  readable and lets you unit-test the handler with a constructed struct.
- Flags that apply to every subcommand (verbosity, `--json`, `--config`) belong on the top-level struct with
  `#[arg(global = true)]`, so users can place them before or after the subcommand.
- Closed sets of choices (output format, color mode) are enums with `#[derive(ValueEnum)]`. clap then validates the
  value, lists the choices in `--help` and rejects typos with a usage error instead of your code inventing one.
  Use the same enum for the value read from config so the two sources cannot disagree.
- Make an option `Option<T>` when its default comes from config: a plain default on the flag would always win and
  hide the config value (see §3).
- Doc comments on fields and variants become help text; write them for the user, not the reader of the code.
- Add one test that calls `Command::debug_assert` on the derived command (via `CommandFactory::command()`); it
  catches conflicting names and bad attribute combinations that otherwise surface only at runtime.

## 3. Config layering

Resolve every setting in a fixed order, lowest first: built-in defaults, config file, environment, flags. Users
expect the most specific source to win, and a fixed order makes behaviour explainable in `--help`.

- Locate the config file in the user config directory for the platform (the `dirs` or `directories` crate, after
  `forge-tools crate-check`), and allow `--config <path>` to override the location. Tests need that override so
  they never read the real home directory.
- A missing config file is normal: fall back to defaults silently. A config file that exists but does not parse
  is a user error: report the path and the parse error, and exit non-zero rather than silently ignoring it.
- Deserialize the file into a struct whose fields are all optional, then merge: `flag.or(env).or(file)` per field,
  then apply the default. Keep the merge in one function that returns a fully resolved settings struct, and test
  that function directly for each precedence case.
- Environment variables through clap need its `env` cargo feature; check that it is enabled before using
  `#[arg(env = ...)]`, or read the variables yourself in the merge step.
- If the project already uses a layering crate, follow its conventions instead of adding a second mechanism.

## 4. Errors and exit codes

- In the binary, return `anyhow::Result<()>` from the top-level logic and add `.context(...)` at each fallible
  boundary (file paths, parse steps) so the final message tells the user what failed and where.
- In library crates or modules other code calls, define error enums with `thiserror`, so callers can match on
  cases; convert to `anyhow` only at the binary edge.
- Exit codes are part of the interface scripts depend on: 0 for success, 2 for usage errors (clap already exits
  with 2 when parsing fails), 1 for runtime failures, and distinct documented codes only where callers need to
  tell cases apart. Return `std::process::ExitCode` from `main` or map error kinds to codes in one place; do not
  call `std::process::exit` deep in the code, because it skips destructors and buffered output.
- Validate input that clap cannot (unknown note ids, invalid ranges, conflicting config) before doing any work,
  and fail with a non-zero code and a message on stderr.

## 5. Output for humans and scripts

- Write results to stdout and diagnostics, progress and errors to stderr, so `tool cmd | jq` works and errors
  stay visible.
- Offer `--json` (or a `--format` value enum) for machine output. Emit one well-formed document built with
  `serde_json` from a serializable type; never assemble JSON or CSV by string concatenation, because escaping
  breaks on the first comma or quote in user data. For CSV use the `csv` crate after `crate-check`.
- Keep the human format free to change and the machine format stable; treat a change to JSON field names as a
  breaking change.
- Disable color and progress output when stdout is not a terminal or when `NO_COLOR` is set.
- Handle a closed pipe (`tool | head`) without a panic: write through a locked, buffered stdout and treat a
  broken-pipe error as a normal exit.

## 6. Testing the binary

- Put end-to-end tests in `tests/` and drive the built binary with `assert_cmd`: `Command::cargo_bin("<bin>")`,
  then `.assert()` and check `.success()`, `.failure()`, `.code(...)` and `.stdout(...)`/`.stderr(...)` with
  `predicates` (for example `predicates::str::contains`). Running the real binary tests parsing, config
  resolution, exit codes and output together, which is what users see.
- Isolate every test: a fresh temp directory for data and config, `--config` pointing into it, and environment
  variables the tool reads set or removed explicitly. A test that reads the developer's real config passes on one
  machine and fails on another.
- Cover each precedence case (default, file only, flag over file), each output format (parse the JSON in the test
  rather than comparing whole strings), and at least one bad-input case per subcommand asserting the exit code
  and a stderr message.
- Keep pure logic (merging, formatting) in library functions with unit tests; reserve binary tests for behaviour
  that crosses the process boundary, since each one costs a process spawn.
- Add `assert_cmd` and `predicates` as dev-dependencies through the core crate policy (`crate-check`, then
  `cargo add --dev`).
