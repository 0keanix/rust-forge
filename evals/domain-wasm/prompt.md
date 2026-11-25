---
tags: [domain, wasm]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a workspace with a note-filtering crate and an empty Leptos app crate. Build a client-side app with trunk that filters notes as you type, keeping the filter logic in the shared crate with host tests and adding a wasm-bindgen-test for the browser part.
