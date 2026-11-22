---
id: TASK-273
title: "Implement the request-scoped record snapshot for resolution"
created_date: '2026-11-20T17:43:07Z'
updated_date: '2026-11-22T20:45:27Z'
labels: []
references:
  - ADR-037
  - TECH-043
  - NFR-007.002
  - TECH-072
  - TECH-040
  - NFR-007.001
  - TC-178
  - TC-179
status: Done
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-272
ordinal: 30
documentation:
  - src/browser/records.rs
  - src/browser/mod.rs
  - tests/browser_cli.rs
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
  - docs/specifications/technical/TECH-072.md
type: implementation
---

## Description
## Context
`src/browser/records.rs::resolve_id` scans every enabled descriptor directory once per looked-up ID and re-parses the match. `load_record` calls `resolve_title`, which calls `resolve_id` for milestone, release, and superseded-by; `view_page`, `edit_page`, and the API projections call `resolve_id` for dependencies, references, and related records. A page with N references therefore performs N directory scans and N parses. The MS-032 baseline at 1,000 records records view-grouped-member at 23,431 ms p50 and 23,950 ms p95 (baseline-only, 20 samples), and `docs/benchmarks/MS-032-baseline.md` attributes the cost to per-reference directory rescans and re-parses.

## Scope
Build one request-scoped record snapshot: a map from record ID to `records::Record` covering the enabled descriptors' active directories. It is owned by the request, built lazily on first ID resolution, and dropped with the response. The first lookup lists each enabled descriptor directory once (file names only, no parse); an ID lookup is one map read and at most one file parse, and each resolved record is memoised for the rest of the request. Thread the snapshot through title and reference resolution, list, view, and edit rendering in `src/browser/mod.rs` and `src/browser/records.rs`.

The snapshot is display-only. `form_post`, `api_post`, and `save_ops` still read the live record through `record::store::read_document` or `read_document_normalized`, compute the version token from the live bytes, and write through `record::store::write_atomic`. Archived records and disabled types stay invisible because only enabled descriptors' active directories are indexed. An ID that is not in the snapshot returns the same `None` or unresolved outcome that `resolve_id` returns today for an unknown or archived ID; no path falls back to a directory rescan.

The task requirement to fail with a clear diagnostic applies to an internal snapshot inconsistency, such as a record claimed by the directory listing that cannot be read or parsed at resolution time; it is not a change to the unknown, archived, or disabled-ID outcome. Those IDs keep the current unresolved-reference or not-found behaviour fixed by ADR-037 decision 25, and the diagnostic stays on the existing error and diagnostic path rather than becoming a new page, status, or client-visible message.

## Assumptions
- TASK-274 later moves the configuration and registry source to the serving snapshot. This task builds the request snapshot from the configuration and registry available at request start and must not store it in `ServerState`.
- The snapshot is per request, not per connection and not shared between requests.

## Definition of done
All acceptance criteria pass; targeted tests, the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; evidence from the listing and parse counts and the write contract is recorded on this task; no user-visible behaviour changes beyond latency.
## Context
`src/browser/records.rs::resolve_id` scans every enabled descriptor directory once per looked-up ID and re-parses the match. `load_record` calls `resolve_title`, which calls `resolve_id` for milestone, release, and superseded-by; `view_page`, `edit_page`, and the API projections call `resolve_id` for dependencies, references, and related records. A page with N references therefore performs N directory scans and N parses. The MS-032 baseline at 1,000 records records view-grouped-member at 23,431 ms p50 and 23,950 ms p95 (baseline-only, 20 samples), and `docs/benchmarks/MS-032-baseline.md` attributes the cost to per-reference directory rescans and re-parses.

## Scope
Build one request-scoped record snapshot: a map from record ID to `records::Record` covering the enabled descriptors' active directories. It is owned by the request, built lazily on first ID resolution, and dropped with the response. The first lookup lists each enabled descriptor directory once (file names only, no parse); an ID lookup is one map read and at most one file parse, and each resolved record is memoised for the rest of the request. Thread the snapshot through title and reference resolution, list, view, and edit rendering in `src/browser/mod.rs` and `src/browser/records.rs`.

The snapshot is display-only. `form_post`, `api_post`, and `save_ops` still read the live record through `record::store::read_document` or `read_document_normalized`, compute the version token from the live bytes, and write through `record::store::write_atomic`. Archived records and disabled types stay invisible because only enabled descriptors' active directories are indexed. An ID that is not in the snapshot returns the same `None` or unresolved outcome that `resolve_id` returns today for an unknown or archived ID; no path falls back to a directory rescan. A record claimed by the directory listing but unreadable at parse time keeps the existing error and diagnostic behaviour.

## Assumptions
- TASK-274 later moves the configuration and registry source to the serving snapshot. This task builds the request snapshot from the configuration and registry available at request start and must not store it in `ServerState`.
- The snapshot is per request, not per connection and not shared between requests.

## Definition of done
All acceptance criteria pass; targeted tests, the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` are clean; evidence from the listing and parse counts and the write contract is recorded on this task; no user-visible behaviour changes beyond latency.
## Acceptance Criteria
- [ ] #1 A request that resolves one or more IDs builds at most one request-scoped record snapshot; the snapshot is owned by the request and is not stored in ServerState, not shared between connections, and dropped when the response is written.
- [ ] #2 On the first ID resolution the snapshot lists each enabled descriptor's active directory exactly once for file names and parses no record during the listing; each distinct resolved record is parsed at most once and memoised for the rest of the request.
- [ ] #3 A view page with N references performs at most one directory listing per enabled type and at most one parse per distinct referenced record, independent of N; a counter or test seam proves the listing and parse counts.
- [ ] #4 Milestone, release, dependency, requirement-reference, group, and superseded-by lookups in list, view, and edit rendering resolve through the request snapshot; the only records::resolve_id calls left in the browser module are the initial address lookup and calls routed through the snapshot.
- [ ] #5 The snapshot is display-only: form_post, api_post, and save_ops read the live record through record::store::read_document or read_document_normalized, compute the version token from the live bytes, and write through record::store::write_atomic; the snapshot supplies neither bytes nor a version token.
- [ ] #6 A stale version token still returns 409 with the live token and writes nothing, and a successful save is proven by a fresh read of the redirect target; these behaviours are unchanged from the pre-task baseline.
- [ ] #7 An ID that is absent from the snapshot returns the same None or unresolved-reference outcome that records::resolve_id returns today for an unknown, archived, or disabled ID; no path falls back to a directory rescan or a second parse, and a record claimed by the listing but unreadable at parse time keeps the existing error and diagnostic behaviour.
- [ ] #8 Existing browser, record, characterization, and no-drift suites pass; CLI output, JSON payloads, record bytes, HTML semantics, status codes, and headers other than the TASK-276 asset cache headers are unchanged; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check are clean.
- [ ] #9 Tests cover the listing and parse counts, the display-only write contract, the stale-token 409, and the no-rescan path; evidence is recorded on this task.
## Comments
- **rust-dev**: TASK-273 implementation evidence (rust-dev). Snapshot API: RecordSnapshot::{new, resolve, resolve_live, resolve_title, listing_count, parse_count, is_built}; build_index_with_snapshot shares the per-request parse cache; load_record_with(RecordTitles) keeps the public load_record contract. Call sites converted: inventory_page; list_page board and document paths; board cleanup preview, submit, status submit, and failure re-render; view_page; edit_page; document_form_post; api_requirements; api_doc_list; api_get; api_post plus api_post_document and api_post_mockup. No records::resolve_id call remains in browser mod.rs. form_post, api_post, and save_ops use resolve_live plus record::store for bytes and version. Effect (records-382 smoke, release LTO, 1 warm-up + 3 measured, baseline-only): inventory server p50 1764 ms and p95 1777 ms vs recorded 11776 and 11971 ms; view-grouped-member 749 and 749 ms vs recorded 23431 and 23950 ms. Not an NFR verdict; TASK-275 and TASK-278 own the final numbers. Checks: cargo test 1359 passed, 0 failed across 69 suites; characterization_replay 17, list_characterization_replay 3, no_drift 11, document_type_no_drift 21; cargo clippy --all-targets -- -D warnings clean; cargo fmt --check clean. Full suite needs env -u EDITOR because host EDITOR=nano breaks unrelated config_editor_json_null_when_unset. Tests: browser records.rs unit tests prove one listing per enabled type, one parse per distinct record, shared index and snapshot parse cache, unknown-ID no-rescan, unreadable-record diagnostic, and live bytes and token after an out-of-band edit. Existing t17 and t18 and API conflict tests cover stale-token 409; browser_cli 214 passed. Files: browser records.rs, browser mod.rs. Branch task TASK-273; no commit per subagent instruction.
- **orchestrator**: Acceptance met: RecordSnapshot per request with lazy listing and memoised parse, threaded through inventory, list, view, edit, form_post and all API routes; live reads retained for writes; no cross-request cache; 1359 tests pass; measured 382-record inventory 11776 to 1764 ms p50 and grouped member view 23950 to 749 ms p95 (baseline-only).
