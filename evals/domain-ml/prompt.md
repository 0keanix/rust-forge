---
tags: [domain, ml]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a CLI that should embed text. Add an embed command that loads a small sentence-embedding model from safetensors with candle, uses Metal on macOS and falls back to CPU, and returns normalized vectors. Tests must not download a model.
