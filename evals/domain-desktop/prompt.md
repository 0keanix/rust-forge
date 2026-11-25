---
tags: [domain, desktop]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a workspace with a notes core crate and a stub Tauri 2 app crate. Expose list_notes and add_note as Tauri commands with capabilities restricted to what those commands need, keeping the logic testable without the app.
