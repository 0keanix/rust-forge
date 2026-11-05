---
tags: [domain, cli]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a small notes CLI on clap. Add an export subcommand that writes all notes as JSON or CSV, with the default format taken from a config file in the user config dir and overridable by a flag. Cover it with tests that run the binary.
