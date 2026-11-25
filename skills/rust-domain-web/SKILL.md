---
name: rust-domain-web
description: Guides building and changing Rust web services and HTTP APIs that use axum on tokio and tower - routing, typed extractors, shared application state without Arc<Mutex<_>> by default, error types that map to HTTP responses, middleware as tower layers, sqlx with migrations and offline query data from cargo sqlx prepare, graceful shutdown, tracing, and integration tests that call the router directly without binding a port. Use it whenever a Rust project depends on axum, or the user asks to add an endpoint, handler, route, middleware, database query or migration to a Rust service, even if they only say "the API", "the server" or "the backend". Complements the core rust-forge skill, which still owns the working loop and gates.
metadata:
  forge-verified: axum=0.8.9
---

# rust-domain-web

Domain guidance for axum services. The core `rust-forge` skill owns the working loop, the gates, the crate policy
and compiler-error strategies; follow it and use this skill only for what is specific to web services.

## 1. Pin the version first

Run `forge-tools domains` and read the `web` line: `crate=axum version=<x>`. Read docs.rs and the official examples
(the axum repository `examples/`, the sqlx README and sqlx-cli README) for exactly that version, and for the tokio,
tower, tower-http and sqlx versions in `Cargo.lock`, before writing code. axum and sqlx reshape extractors, path
syntax, state handling and feature names between minor versions, and version drift is the main source of wrong web
code. If the output says `version=unknown`, the lock file is missing; take the requirement from `Cargo.toml` and
resolve it before relying on details.

The names below were checked against axum 0.8.9, tokio 1.54.1, tower 0.5.3, tower-http 0.7.1, sqlx 0.9.1 and
tracing 0.1.44. For any other version, confirm each name in that version's docs before using it.

## 2. Structure

- Build the router in a function (`fn app(state: AppState) -> Router`) that `main` and the tests both call. Tests
  then exercise the same routes, layers and state as production, and `main` only binds and serves.
- Keep handlers thin: extract, call a function that does the work, map the result to a response. Logic outside
  handlers can be unit-tested without HTTP.
- Group routes per resource and combine them with `Router::merge` or `Router::nest`; one file per resource keeps
  the route table readable.
- In axum 0.8 path captures are written `/notes/{id}` (and `{*rest}` for a wildcard); the older `/:id` form is
  rejected. Check the syntax in the docs for the pinned version.

## 3. Extractors

- Use typed extractors (`State`, `Path<T>`, `Query<T>`, `Json<T>`) instead of parsing the request by hand: axum
  rejects bad input with a proper status before your code runs, and the signature documents the endpoint.
- A body-consuming extractor (`Json`, `String`, `Bytes`, `Form`) must be the last argument, because the body can
  be read only once; the compiler error for a wrong order is hard to read, so check the order first.
- Deserialize into dedicated request types with `serde::Deserialize` and serialize dedicated response types; do not
  expose database rows directly, so the schema can change without breaking the API.
- When the default rejection response is not what clients need, wrap the extractor or map its rejection into your
  error type (section 5) rather than accepting raw input.

## 4. Shared state

- Put shared resources in one `#[derive(Clone)] struct AppState` and attach it with `Router::with_state`; handlers
  take `State<AppState>`. A pool such as `sqlx::SqlitePool` is already a cheap, cloneable handle, so it needs no
  `Arc` and no lock.
- Do not reach for `Arc<Mutex<_>>` by default: a lock held across an `.await` serializes requests and can deadlock.
  Prefer a database, a channel to a task that owns the data, or an atomic. If a mutable in-memory map is really
  needed, hold the lock only for the synchronous update and never across `.await`.
- When a handler needs only part of the state, implement `FromRef` for that substate so the handler declares only
  what it uses.
- Read configuration once at start-up into the state; handlers should not read environment variables.

## 5. Errors that map to responses

- Define one error enum for the service (with `thiserror`, per the core skill) and implement `IntoResponse` for it,
  mapping each variant to a status code and a small JSON body. Handlers return `Result<T, AppError>` and use `?`.
- Map "not found" from the database (`sqlx::Error::RowNotFound`, or an `Option` that is `None`) to 404, validation
  failures to 400 or 422 and conflicts such as unique-constraint violations to 409; everything else is 500.
- Log the internal cause of a 500 with `tracing` and return a generic message: database errors can leak schema
  details and are no help to a client.

## 6. Middleware with tower layers

- Add cross-cutting behaviour (request tracing, timeouts, CORS, compression, body size limits) as tower layers via
  `Router::layer`, using `tower-http` where it provides the layer, instead of repeating code in handlers.
- Layers wrap the routes added before the `.layer` call, and the last one added runs first on a request. Keep the
  order deliberate, for example tracing outermost so it sees every request, including rejected ones.
- Use `Router::route_layer` for layers such as auth that should run only for matched routes, so unknown paths still
  return 404 rather than 401.
- For small custom logic, prefer `axum::middleware::from_fn` over a hand-written `tower::Service`; write a service
  only when you need its readiness or a reusable layer.

## 7. sqlx: migrations and offline query data

- Put schema changes in a `migrations/` directory as numbered SQL files (sqlx-cli `sqlx migrate add <name>` creates
  them) and apply them at start-up with `sqlx::migrate!("./migrations").run(&pool).await?`. The macro embeds the
  files in the binary, so the deployed service and the tests run the same schema.
- Never edit a migration that has run anywhere; add a new one. sqlx records checksums and refuses changed files.
- The checked macros (`query!`, `query_as!`) verify SQL against a database at compile time. To build without one,
  run `cargo sqlx prepare` (with `DATABASE_URL` set and migrations applied) and commit the generated `.sqlx`
  directory. It is written in the current directory; when several crates of a workspace use the macros, run
  `cargo sqlx prepare --workspace` for one `.sqlx` at the workspace root. Rerun it after every query change, or
  the build will fail in CI and the gates.
- If no database or sqlx-cli is available in the session, write the query with the unchecked functions
  (`sqlx::query_as::<_, T>(...)` with `.bind`) and say so, instead of committing checked macros with no `.sqlx`
  data that cannot compile offline.
- Enable only the sqlx features you use (the database driver, the runtime, `macros`, `migrate`); check their exact
  names in the pinned version's `Cargo.toml` on docs.rs, because they have been renamed between releases.
- For SQLite tests use an in-memory database (`sqlite::memory:`) with a single-connection pool and run the
  migrations on it, so every test starts from a fresh schema without files on disk.

## 8. Graceful shutdown

- Serve with `axum::serve(listener, app)` and chain `.with_graceful_shutdown(signal)`, where the signal future
  completes on `tokio::signal::ctrl_c()` (and SIGTERM on Unix, which container platforms send). In-flight requests
  then finish instead of being cut off.
- Close or drop resources after the server future returns, such as `pool.close().await`, so the database sees a
  clean disconnect.

## 9. tracing

- Initialize `tracing-subscriber` once in `main`, with the filter taken from `RUST_LOG`, never in library code or
  tests that run in parallel.
- Add `tower_http::trace::TraceLayer` to the router for one span per request, and `#[tracing::instrument]` on
  functions that do real work, skipping large or secret arguments (state, bodies, passwords) with `skip(...)`.
- Log with structured fields (`tracing::info!(note_id = id, "created")`) instead of formatted strings, so logs can
  be filtered by field.

## 10. Tests without binding a port

- Test the router as a tower service: build it with `app(state)`, then send requests with
  `tower::ServiceExt::oneshot(router, request)` and assert on the status and body. No port, no network, no flaky
  timing, and tests can run in parallel. `tower` with the `util` feature belongs in `[dev-dependencies]`.
- Build requests with the `http` request builder and an `axum::body::Body`; read the response body with
  `axum::body::to_bytes` and a size limit, then parse it with `serde_json`.
- Give every test its own state (a fresh in-memory database with migrations applied) so tests do not share rows.
- Cover the error mapping too: an unknown id returns 404, malformed JSON is rejected, and a created resource can be
  read back.
