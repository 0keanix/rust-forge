---
type: llm
focus: trace
---

The task was to add an `export` subcommand to a small clap-based notes CLI that writes all notes as JSON or CSV,
with the default format read from a config file in the user config directory and overridable by a flag, covered by
tests that run the binary.

PASS if the new subcommand uses clap derive and the format is a clap value enum, the config file value sits under
the flag (the flag overrides the config file, which overrides the built-in default), bad input such as an unknown
format or an unparseable config file makes the binary exit with a non-zero code, and the new tests use `assert_cmd` to
run the built binary.
FAIL if the format is parsed by hand from a free-form string, the config file is ignored or overrides the flag, bad
input exits with code 0, or no test runs the binary.
Whether the code builds and its tests pass is graded separately from `target/forge/state.json`; do not judge it here.
