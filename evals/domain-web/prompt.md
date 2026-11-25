---
tags: [domain, web]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is an axum service. Add POST /notes and GET /notes/{id} backed by SQLite through sqlx with a migration, keep the query data offline-ready, and add router tests that do not bind a port.
