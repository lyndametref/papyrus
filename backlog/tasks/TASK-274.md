---
id: TASK-274
title: "Implement the mtime-invalidated config, registry, and route snapshot with fail-closed reload"
created_date: '2026-11-20T17:43:07Z'
updated_date: '2026-11-22T20:17:46Z'
labels: []
references:
  - ADR-037
  - ADR-0016
  - ADR-0020
  - ADR-0005
  - ADR-0006
  - TECH-072
  - ADR-0019
  - NFR-007.001
  - NFR-007.002
  - TC-178
  - TC-179
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-273
ordinal: 40
documentation:
  - src/browser/mod.rs
  - src/commands.rs
  - src/registry.rs
  - src/browser/routes.rs
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
  - docs/specifications/technical/TECH-072.md
type: implementation
---

## Description
## Context
`ServerState` in `src/browser/mod.rs` holds a startup `ArtifactRegistry` and `BrowserRoutes`; `ServerState::config()` calls `commands::load_effective_config` on every use, so configuration is re-read and re-validated per request while the registry and routes never notice an edit. `run()` loads the configuration, derives routes from the registry passed by the CLI, and stores all three in `ServerState`. The MS-032 baseline and ADR-037 identify the per-request load and the never-reloaded registry and routes as the configuration-side defects.

## Scope
Implement the ADR-037 and TECH-072 serving snapshot. One immutable `ServingSnapshot` owns the effective `Config`, the `ArtifactRegistry`, the `BrowserRoutes` table, and the input fingerprint. `ServerState` holds one swap point (a single-writer, multi-reader cell of `Arc<ServingSnapshot>`) and no second copy of any part. A request takes one handle at start and passes it to routing, validation, handlers, and rendering.

The fingerprint is modification time (seconds and nanoseconds) plus byte length, read with `std::fs::metadata` from the selected configuration file only. The registry seed is compiled into the binary and the route table is derived in process, so both share the configuration fingerprint and are invalidated together; there is no independent registry or route fingerprint. This refines, without changing, the task's "config file and registry sources" wording. The check runs once per request before routing. An unchanged file performs only the metadata read, with no load, validation, registry construction, or route derivation.

A reload runs the startup path in order: load and validate the configuration, build the registry, derive the routes. It publishes one fully built snapshot by one atomic swap; there is no per-field update and no partial publication. Reloads are serialized and re-checked under the lock, so concurrent requests observing one change cause one rebuild; a request that cannot take the lock serves the current snapshot without waiting, and a request already holding a handle finishes on it.

A failed reload is fail-closed for serving: the new snapshot is discarded, the last known good snapshot keeps serving with its normal status code and page, and one `warning: ...` line is written to standard error naming the failing input and the error. The failing fingerprint is remembered and not re-reported for the same input state; a changed or fixed file retries the reload, and a new warning is emitted only if the retry also fails. No new HTTP status, header, page, endpoint, JSON member, or exit code is introduced, no configuration error content is sent to the client, and the `--debug` request log is unchanged. Startup has no last-known-good snapshot: the first snapshot must build, with exit 1 for a missing configuration and exit 4 for an invalid one (ADR-0016 decision 3).

Settings read before the listener binds (`browser_host`, `browser_port`, `browser_open`) remain startup-only. A reload changes pages, navigation, lists, and routes, not the bound socket, the concurrency cap, or the `Host` and `Origin` allowlist. The request record snapshot from TASK-273 is built from the request's serving snapshot and discarded with the request; record content is never cached across requests, and writes still read the live record for their bytes and version token.

## Assumptions
- The standard library only: `std::fs::metadata`, `std::sync::RwLock`, and `std::sync::Arc`; no new crate, file watcher, background thread, timer, index file, or serialization format.
- `commands::load_effective_config` and the existing registry and route constructors remain the only loaders and validators; the reload is the startup path, not a second loader.

## Definition of done
All acceptance criteria pass; targeted tests, the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; invalidation, atomic publication, fail-closed reload, warning de-duplication, and startup behaviour are proven by tests and recorded on this task; no user-visible behaviour change for a valid configuration beyond the TASK-276 asset cache headers.
## Acceptance Criteria
- [ ] One snapshot accessor: ServerState exposes exactly one serving-snapshot accessor; the per-request config call and every browser-side load_effective_config call are removed, and a request takes one immutable handle at start and passes it to routing, validation, handlers, and rendering.
- [ ] Snapshot parts: one ServingSnapshot owns the effective Config, the ArtifactRegistry, the BrowserRoutes table, and the fingerprint; ServerState holds one swap point and no second copy of any part, and no accessor returns one part without the others.
- [ ] Fingerprint: modification time in seconds and nanoseconds plus byte length of the selected configuration file only, read with std::fs::metadata; the registry and route table share that fingerprint and are invalidated only together; a changed, disappeared, or unreadable file attempts one reload, while an unchanged file performs only the metadata read and no load, validation, registry construction, or route derivation.
- [ ] Reload order and atomic publication: a reload loads and validates the configuration, builds the registry, derives the routes, and publishes one fully built snapshot by one atomic swap; a reader never observes a partial snapshot, a request already holding a handle finishes on the old snapshot, and a request starting after the swap gets the new one.
- [ ] Serialization and de-duplication: reloads are serialized, the observing request re-checks the fingerprint under the lock and skips the rebuild when the snapshot is current, concurrent requests observing one change cause one rebuild, and a request that cannot take the reload lock serves the current snapshot without waiting.
- [ ] Fail-closed reload: a load, validation, registry, or route error discards the new snapshot and keeps the last-known-good snapshot serving with its normal status code and page; exactly one warning line is written to standard error naming the failing input and the error; the failing fingerprint is remembered and not re-reported for the same input state; a changed or fixed file retries and warns again only if the retry fails.
- [ ] No new surface: a reload failure adds no HTTP status, header, page, endpoint, JSON member, or exit code, sends no configuration error content to the client, and leaves the debug request log unchanged.
- [ ] Startup behaviour is unchanged: the first snapshot must build; a missing configuration exits 1 and an invalid configuration exits 4; fail-closed serving does not apply before the first snapshot exists.
- [ ] Startup-only settings: browser_host, browser_port, browser_open, the concurrency cap, and the Host and Origin allowlist come from the startup configuration; a reload never rebinds the socket, and edits to those keys take effect only at the next browser start.
- [ ] Request-scoped records: the TASK-273 record snapshot is built from the request's serving snapshot and discarded with the request; record content is not cached across requests, and every write re-reads the live record through record::store for its bytes and version token.
- [ ] No per-record or per-request configuration read, YAML parse, validation, or registry construction remains on the read path; a call-site audit and tests prove that the only configuration access is the once-per-request fingerprint check and the service accessor.
- [ ] Behaviour preservation: for a valid configuration the pages, status codes, JSON payloads, record bytes, and all headers other than the TASK-276 asset cache headers are unchanged; the existing browser, record, configuration, characterization, and no-drift suites pass; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
- [ ] Tests cover invalidation by a configuration edit, the unchanged-fingerprint fast path, atomic generation, a single rebuild under concurrent observers, fail-closed reload with one warning and de-duplication, startup exit codes, and the non-rebind rule; evidence is recorded on this task.
