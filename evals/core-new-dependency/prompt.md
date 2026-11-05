---
tags: [core]
max_turns: 40
timeout_seconds: 900
allowed_tools: [Read, Glob, Grep, Skill, Agent, Bash, Edit, Write]
---

I want `Config` to be loadable from a JSON file. Add a `Config::from_json_str` function that parses a JSON string into a `Config` using a proper JSON library, plus a test for it.
