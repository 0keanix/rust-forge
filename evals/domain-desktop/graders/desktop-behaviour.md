---
type: llm
focus: trace
---

The task was to expose list_notes and add_note as Tauri 2 commands in a workspace with a notes core crate and a
stub Tauri app crate, with capabilities restricted to what those commands need and the logic testable without the
app.

Expected behaviour: adds two commands in the app crate that delegate to the core, a capability file granting only
what is needed, and tests on the core crate; gates pass for the core.

PASS if the app crate gains `list_notes` and `add_note` commands (`#[tauri::command]`, registered in
`generate_handler!`) that only delegate to the core crate through managed state, the logic and validation stay in
the core crate with no `tauri` dependency, a capability file under the app's `capabilities/` directory grants only
the permissions these commands need (for example the generated `allow-list-notes` and `allow-add-note`, or the
minimal set the agent justifies) and no wildcard or broad plugin grants, and tests are added or kept on the core
crate for adding and listing notes, including a rejected input.
FAIL if business logic is written in the commands, the core crate gains a `tauri` dependency, the capability grants
broad or unrelated permissions, no capability is added, or there are no tests on the core.
Whether the code builds and the gates pass is graded separately; do not judge it here.
