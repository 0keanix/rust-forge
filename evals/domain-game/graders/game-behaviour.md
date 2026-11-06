---
type: llm
focus: trace
---

The task was to add stamina to a small Bevy prototype with a moving player: sprinting drains it, it regenerates after
a short idle delay, sprint is disabled at zero, and headless tests cover the drain and regeneration rules.

PASS if the change adds a stamina component and the systems for it inside a plugin with explicit system ordering
(`.chain()`, `.before`/`.after` or system sets), keeps the tunables (drain rate, regeneration rate, idle delay) in a
resource rather than as literals inside systems, and the new tests build a headless `App` (minimal plugins, no
`DefaultPlugins`, no window or rendering) and step it with `update` to check drain and regeneration.
FAIL if the stamina systems have no explicit ordering against the movement system, the tunables are literals inside
systems, the tests add rendering or windowing plugins, or no test steps an `App`.
Whether the code builds and the gates pass is graded separately; do not judge it here.
