---
description: Show the last rust-forge gate results and detected domains
allowed-tools: Bash(cat:*), Bash(forge-tools domains:*)
---

Read `target/forge/state.json` in the project root and run `forge-tools domains`.
Report the gate level, each gate's result, the git HEAD the state was recorded at and its timestamp, and the
detected domains with the skill to load. If `state.json` does not exist, say that no gates have run yet.
