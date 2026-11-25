---
type: llm
focus: trace
---

The task was to add POST /notes and GET /notes/{id} to a small axum service, backed by SQLite through sqlx with a
migration, with offline-ready query data and router tests that do not bind a port.

Expected behaviour: adds handlers with typed extractors and an error type that maps to status codes, a sqlx
migration and offline query data, and tests via the router as a tower service; gates pass.

PASS if the handlers use typed extractors (`State`, `Path`, `Json` or equivalent), errors go through a type that
implements `IntoResponse` and maps to status codes (for example 404 for a missing note), a migration file is added
and applied through sqlx, the query data is kept offline-ready (a committed `.sqlx` directory from
`cargo sqlx prepare`, or only queries that need no live database at compile time), and the tests drive the `Router`
as a tower service (for example with `oneshot`) without binding a TCP port.
FAIL if handlers parse raw bodies or paths by hand, errors are returned as bare strings or panics, there is no
migration, compile-time-checked queries need a live database with no offline data, or any test binds a port.
Whether the code builds and the gates pass is graded separately; do not judge it here.
