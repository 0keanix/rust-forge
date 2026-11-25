---
type: llm
focus: trace
---

The task was to build a client-side Leptos app with trunk that filters notes as you type, in a workspace with a
note-filtering crate and an empty Leptos app crate, keeping the filter logic in the shared crate with host tests
and adding a wasm-bindgen-test for the browser part.

Expected behaviour: keeps filtering in the shared crate with host tests, builds the Leptos CSR app with trunk for
wasm32, gates non-wasm dependencies behind features, and adds a wasm-bindgen-test; host gates pass.

PASS if the filter function lives in the shared `notes-filter` crate with host unit tests (including a
case-insensitive or empty-query case), the app crate only wires a Leptos CSR component (an input bound to a signal
and a list derived through the shared filter) mounted from `main`, the app is set up for `trunk` building for
`wasm32-unknown-unknown` (an `index.html` trunk can build, and the target named in the reply or in config), any
dependency that does not build for wasm is behind a feature or a target-specific dependency table, and a
`wasm-bindgen-test` test configured to run in a browser exercises the browser part.
FAIL if the filter logic is written inside the component, the shared crate gains a Leptos or wasm-only dependency,
no host tests cover the filter, there is no `wasm-bindgen-test`, or the app uses SSR or hydration instead of CSR.
Whether the code builds and the gates pass is graded separately; do not judge it here.
