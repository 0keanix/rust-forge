---
description: Run rust-forge cargo quality gates (fast, full, deps or unsafe)
argument-hint: "[fast|full|deps|unsafe]"
allowed-tools: Bash(forge-tools gates:*)
---

Run `forge-tools gates <level>` in the project root, where `<level>` is "$ARGUMENTS" or `fast` when that is empty,
and show its output unchanged.
If a gate failed, explain the first failure briefly and propose the next step; SKIPPED gates are not passes.
