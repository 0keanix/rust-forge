---
tags: [domain, game]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a Bevy prototype with a moving player. Add stamina: sprinting drains it, it regenerates after a short idle delay, and sprint is disabled at zero. Add headless tests for the drain and regeneration rules.
