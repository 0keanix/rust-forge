---
tags: [domain, ffi-mobile]
max_turns: 60
timeout_seconds: 1500
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
append_system_prompt: >-
  In this environment Bash commands run in a sandbox where cargo and rustc cannot run. The plugin's hooks run
  outside it: after each edit you get compiler diagnostics, and when you stop the full gates run and report
  failures. Rely on them instead of running cargo yourself.
---

This repo is a workspace with a pure Rust notes core crate and an empty ffi crate that already depends on uniffi. Expose NoteStore (add, list, search) to Kotlin and Swift through the ffi crate, and make it build for aarch64-linux-android and aarch64-apple-ios.
