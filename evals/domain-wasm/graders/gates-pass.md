---
type: regex
target:
  source: file
  path: target/forge/state.json
pattern: '(?=[\s\S]*"G3-test":\s*"PASS")(?=[\s\S]*"green":\s*true)'
---
