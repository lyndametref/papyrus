---
id: TASK-6
title: Serve offline browser access for requirement content
status: Implemented
assignee:
  - '@orchestrator'
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-03T20:37:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-5
references:
  - docs/requirements/business/REQ-06.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/specifications/browser-requirements.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html
  - docs/specifications/ui/browser-requirements-ui.md
ordinal: 6000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Users need browser-based access without installing a separate desktop interface or requiring internet connectivity. This first browser increment covers the requirement content implemented so far - business-requirement and non-functional-requirement groups and requirements - and will expand to other record types as those features arrive.

Deliver `papyrus browser [--host <host>] [--port <port>] [--no-open]` exactly as documented in `papyrus(1)`: a project-local, offline HTTP interface that lists active requirement groups and requirements, shows a record, and edits its documented fields with save-then-reread persistence. The implementation contract, rationale, and full detail are in `docs/specifications/browser-requirements.md`; the behavior summary below is sufficient to implement and verify this ticket without reading any other document.

Behavior contract:

- Command: `papyrus browser [--host <host>] [--port <port>] [--no-open]`, no positionals, no subcommands. Host, port, and open behavior resolve per setting with precedence command line > environment (`PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, `PAPYRUS_BROWSER_OPEN`) > configuration (`browser_host`, `browser_port`, `browser_open`) > defaults `127.0.0.1` / `6420` / `true`. A non-loopback host is allowed and prints a warning. `--no-open` and `browser_open: false` suppress opening; a missing or failing platform opener is a warning only.
- Startup: bind first, then print one flushed line `Serving http://<host>:<port>/` on stdout (or `{"status":"serving","url":...,"host":...,"port":...,"open":...}` with `--json`), then optionally open the browser, then serve until interrupted. A bind failure exits 1; invalid port exits 2; invalid host or invalid configuration exits 4.
- Server: blocking HTTP/1.1 over `std::net::TcpListener` inside the existing crate, with no new dependencies, no async runtime, no TLS, and no outbound network access. One request per connection with `Connection: close`; at most 16 concurrent connections; request line 8192 bytes, target 2048 bytes, headers 16384 bytes and 64 fields, body 1 MiB, 5-second read and write timeouts. Methods `GET`, `HEAD`, `POST`; other methods 405. Status codes: 400, 403, 404, 405, 408, 411, 413, 414, 415, 431, 501, 505 as specified.
- Routes: `GET /` (list, optional `?type=req|nfr`, default `req`), `GET /doc/{id}` (view), `GET /doc/{id}/edit` (form), `POST /doc/{id}` (form-encoded `version`, `title`, `story`, `description`; 303 on success), `GET /api/requirements` (optional `?type=req|nfr`, default `req`), `GET /api/doc/{id}`, `POST /api/doc/{id}` (JSON), `GET /assets/app.css`, `GET /assets/app.js`, and `HEAD` on every `GET` route.
- Navigation (approved UI): every page shares one shell - header, persistent left sidebar `nav` "Document types", main, footer. The sidebar lists Business requirements and Non-functional requirements as links and selects exactly one category at a time; the default is Business requirements (`/` equals `/?type=req`). There is no combined `all` view; any `type` value other than `req` or `nfr` is 400. The list page renders only the selected category's section, with group rows (ID and title as one link, member count, nested member links in ID order), a trailing "Other requirements" subsection for unresolvable group references, and explicit empty states. Record pages navigate by breadcrumb (`Requirements` linking to the record's category, then the owning group when resolvable, then the ID); the record view's only action is Edit and it has no "Back to requirements" action; a group view also lists its member requirements (ID and title links, ID order). Error pages keep a single recovery link to the list.
- Records: active `req` and `nfr` directories from the effective configuration; groups and member requirements only; ascending ID order; a group with no members, an empty document type, and an empty project each show an explicit empty state. Deprecated records are not listed and return 404 by URL. No create, deprecate, restore, delete, or move operations.
- Editing: title for all four record forms, user story for business-requirement forms only, description for all forms. ID, record form, document type, and group membership are read-only and never accepted from the request. Unknown and editor-added `## ` sections are preserved. Writes are atomic, permission-preserving, LF-normalized, and leave no temporary file. A stale `version` token returns 409 and writes nothing; invalid field values return 422 and write nothing.
- Security: every dynamic HTML value is escaped; JSON is built with `serde_json`; record IDs resolve by exact match against the active directory listing and are never concatenated into a path; every read and write passes `Context::ensure_within`; a symlink escaping the project directory returns 404 and exposes nothing; `Host` and `Origin` validation is the CSRF and DNS-rebinding mitigation; HTML responses carry a restrictive CSP plus `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Cache-Control: no-store`; errors never leak absolute paths or internal detail.
- Assets and support: HTML templates, CSS, and JS embedded in the binary with `include_str!`; no CDN, external font, image, or remote URL; every page, link, form, and error state works with JavaScript disabled; only features supported by Firefox 100+, Edge 100+, and Safari 100+; semantic landmarks, labelled controls, associated errors, visible focus, and 4.5:1 contrast. The selected sidebar entry carries `aria-current="page"` plus a non-colour cue (bold and underline); at 320 CSS px the sidebar collapses behind a toggle button labelled "Document types" with `aria-expanded` and `aria-controls`, and the type links stay visible and usable when JavaScript is disabled.
- Configuration: `browser_host` (string, default `127.0.0.1`), `browser_port` (integer 1-65535, default `6420`), `browser_open` (boolean, default `true`), validated on load, listed in `config list` and `config list --json`, and readable and writable through `config get/set/unset`.

UI mockups: MOCK-0001 (`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`) was approved by the user on 2026-11-03 (UTC) and is the authoritative layout reference for the list/sidebar, record view, edit form, error, and 320 px states; the layout description is `docs/specifications/ui/browser-requirements-ui.md`.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The browser command serves the local web interface over the project's documentation store for the requirement record types implemented in this increment (business-requirement and non-functional-requirement groups and requirements) and opens it by default unless disabled; REQ-06.002's backlog-store coverage is delivered by TASK-11.
- [ ] #2 With no internet connection, users can view and edit all currently implemented business and non-functional requirement content in the browser, and saved changes remain available.
- [ ] #3 The interface renders in Firefox, Microsoft Edge, and Safari version 100 or later; the default bind address and port and non-loopback warning match the documented behavior.
- [ ] #4 Browser-facing input is sanitized and the local interface does not read or write outside the project directory, except for init.
- [ ] #5 `papyrus browser --help` shows exactly `papyrus browser [--host <host>] [--port <port>] [--no-open]`; the command accepts no positional arguments and has no subcommands.
- [ ] #6 Effective host, port, and open behavior resolve per setting with precedence command line > environment (`PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, `PAPYRUS_BROWSER_OPEN`) > configuration (`browser_host`, `browser_port`, `browser_open`) > defaults 127.0.0.1 / 6420 / true.
- [ ] #7 An invalid port (0, negative, greater than 65535, non-numeric) exits 2; an invalid host or invalid configuration value exits 4; a port already in use exits 1 with a bind message; the server keeps serving until interrupted.
- [ ] #8 `--no-open` and `browser_open: false` suppress opening the browser; a missing or failing platform opener produces a warning only and the server keeps serving.
- [ ] #9 The list page has a persistent left "Document types" sidebar listing Business requirements and Non-functional requirements, shows exactly one category at a time (Business requirements by default via `/` or `/?type=req`, `/?type=nfr` for non-functional, and 400 for any other value including `all`), lists each group with its member requirements in ascending ID order, and renders explicit empty states for a group without members, an empty document type, and an empty project.
- [ ] #10 A record view shows ID, record form, document type, owning group, user story (business-requirement forms only), and description, with line breaks preserved, no Markdown rendering, and requirement references shown as literal text; a group view additionally lists that group's member requirements (ID and title as links, ascending ID order); the view's only action is Edit, breadcrumbs and the sidebar are the navigation, and the view has no "Back to requirements" action while error pages keep their single recovery link to the list.
- [ ] #11 Editing persists title and description for all four record forms and user story for business-requirement forms only; ID, record form, document type, and group membership are read-only and never accepted from the request.
- [ ] #12 A save writes atomically, preserves unknown and editor-added `## ` sections, normalizes line endings to LF, and the following view reads the new content from disk; `papyrus req view` or `papyrus nfreq view` shows the same content.
- [ ] #13 Deprecated records are absent from lists and return 404 for direct URL access, with no view or edit affordance.
- [ ] #14 The server enforces the documented limits and returns 400, 403, 404, 405, 408, 411, 413, 414, 415, 431, 501, and 505 for the specified conditions, serves one request per connection with `Connection: close`, and uses no TLS.
- [ ] #15 Every dynamic value in HTML is escaped; JSON is produced with serde_json; record IDs resolve by exact match against the active directory listing and are never concatenated into a path; a symlink escaping the project directory returns 404 and exposes no outside content.
- [ ] #16 `browser_host`, `browser_port`, and `browser_open` appear in `config list`, `config get`, `config set`, and `config unset`, validate as specified (port 1-65535), and are included in `config list --json`.
- [ ] #17 Served HTML, CSS, and JS reference no external origin, font, or script; all assets are embedded in the binary; every page, link, form, and error state works with JavaScript disabled.
- [ ] #18 Unit and integration tests cover the specified behavior, including spawning the binary and speaking HTTP over `TcpStream`, an edit round-trip, offline asset checks, boundary and symlink rejection, port conflict, non-loopback warning, `--no-open`, HTML-escaping and XSS, and the JSON API; build, tests, clippy with warnings denied, and formatting pass with the existing dependency set unchanged.
- [ ] #19 The approved UI mockup MOCK-0001 (`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`, approved 2026-11-03 UTC) is recorded in the specification and in this ticket, and the implemented list/sidebar, record view, edit form, error, and 320 px states match it.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 `docs/specifications/browser-requirements.md` is the implemented contract; any deviation is recorded in the specification before the code changes.
- [ ] #2 `Cargo.toml` dependency list is unchanged: no web framework, async runtime, templating engine, Markdown renderer, or second crate added.
- [ ] #3 `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass.
- [ ] #4 Unit and integration tests map to checks T-01 to T-30 in the specification, and the mapping is recorded in the implementation notes.
- [ ] #5 No file under `docs/requirements/` or `docs/business-rules/` is modified.
- [ ] #6 No reads or writes occur outside the project directory; rejected requests leave no temporary file and no partial write.
- [ ] #7 UI mockups are approved by the user before UI implementation begins, and the MOCK ids are recorded in the specification and this ticket.
- [ ] #8 All ticket changes are made through the backlog CLI; no backlog file is edited directly.
- [ ] #9 The implementation notes state how each acceptance criterion was verified.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read `docs/specifications/browser-requirements.md` and the existing `config`, `context`, `requirement`, `sanitize`, `search`, `error`, and `lib` modules; confirm the boundary and exit-code conventions.
2. Add `browser_host`, `browser_port`, and `browser_open` to `Config` with defaults, validation, flattened output, and `config list/get/set/unset` coverage; add unit tests.
3. Add the `browser` subcommand and option resolution (CLI > environment > configuration > defaults), host validation, loopback detection, warning, startup output, bind failure handling, and the platform opener with warning-only failure.
4. Implement the blocking HTTP/1.1 server: accept loop, thread-per-connection with the concurrency cap, request limits and timeouts, route table, status mapping, response headers, and request logging.
5. Implement record indexing over the active `req` and `nfr` directories: stem matching, ID resolution by directory listing, boundary checks, deprecation exclusion, and UTF-8 handling.
6. Implement the record parser and serializer that preserves unknown sections, plus field validation and the version token; implement atomic writes with permission preservation and temporary-file cleanup.
7. Implement HTML rendering with escaped placeholders, the four templates, and the embedded CSS and JS assets; apply the CSP and security headers.
8. Implement the JSON API endpoints and error objects.
9. Add integration tests: HTTP client over `TcpStream`, list/view/edit round-trip, JSON API, status codes and limits, XSS, symlink, port conflict, non-loopback warning, `--no-open` and opener behavior, offline asset scan.
10. Run build, tests, clippy, and formatting; record the verification mapping in the implementation notes and hand off to quality review.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Quality-review follow-up (2026-11-03, UTC)

Addressed: edit-form dirty status; 503 Host validation + request log; browser config error messages naming the key; test coverage gaps (group/NFR round-trip, empty states, HEAD, 422/409 JSON shapes, aria-describedby, T-14 normalized comparison, T-06 no traffic, T-02 explicit fallback); build/tests/clippy/fmt green.

**T-01 to T-30 test mapping**

| Check | Test(s) |
|---|---|
| T-01 | `t01_command_shape` |
| T-02 | `t02_defaults`; `config::tests::browser_defaults` |
| T-03 | `t03_precedence_per_setting`; `browser::tests::precedence_is_per_setting`, `browser::tests::invalid_environment_values_are_validation_errors` |
| T-04 | `t04_open_and_no_open` |
| T-05 | `t05_missing_opener_is_a_warning` |
| T-06 | `t06_non_loopback_warning` (sends no traffic) |
| T-07 | `t07_port_in_use_exits_one` |
| T-08 | `t08_invalid_port_and_host_exit_codes` |
| T-09 | `t09_list_renders_one_category`; `t09_list_empty_project_and_empty_type`; `t09_group_without_members_and_other_requirements` |
| T-10 | `t10_category_parameter` |
| T-11 | `t11_deprecated_records_are_hidden` |
| T-12 | `t12_record_views`; `t09_group_without_members_and_other_requirements` |
| T-13 | `t13_edit_round_trip`; `t13_edit_round_trip_group_and_nonfunctional` |
| T-14 | `t14_unknown_sections_are_preserved` (normalized whole-section comparison) |
| T-15 | `t15_validation_returns_422_without_writing` (summary, `aria-describedby`, preserved values) |
| T-16 | `t16_immutable_fields_are_ignored` |
| T-17 | `t17_stale_version_is_conflict` |
| T-18 | `t18_writes_are_atomic_and_leave_no_temporary_file` |
| T-19 | `t19_symlink_escape_is_404` |
| T-20 | `t20_html_escaping_prevents_script_injection` |
| T-21 | `t21_json_api` (200/400/409/422/404 error object shapes) |
| T-22 | `t22_request_limits_and_status_codes`; `head_matches_get_without_body`; `t22_concurrency_limit_returns_503_and_logs` |
| T-23 | `t23_one_request_per_connection` |
| T-24 | `t24_assets_are_offline_and_embedded` |
| T-25 | `t25_non_utf8_record_is_skipped_and_not_rewritten` |
| T-26 | `t26_config_keys`; `config::tests::browser_validation_messages_name_the_key`, `browser_port_zero_is_invalid`, `browser_host_validation`, `flattened_browser_keys_follow_document_types`, `browser_keys_serialize_to_json` |
| T-27 | `t24_assets_are_offline_and_embedded` |
| T-28 | `t28_accessibility_and_compatibility_basics`; `edit_form_dirty_status_is_static_and_hidden_until_input` |
| T-29 | `cargo build`; `cargo test`; `cargo clippy --all-targets -- -D warnings`; `cargo fmt --check` (all pass; 124 tests) |
| T-30 | `t30_no_writes_outside_project` |

**Acceptance-criterion verification (#1 to #19)**

- #1: Serves the documentation store for business/non-functional groups and requirements and opens by default unless disabled — `t02_defaults`, `t03_precedence_per_setting`, `t04_open_and_no_open`, `t09_list_renders_one_category`.
- #2: Offline view/edit with persisted changes — `t13_edit_round_trip`, `t13_edit_round_trip_group_and_nonfunctional`, `t24_assets_are_offline_and_embedded`.
- #3: Browser baseline, defaults, non-loopback warning — `t28_accessibility_and_compatibility_basics`, `t02_defaults`, `t06_non_loopback_warning`.
- #4: Input sanitization and project-boundary containment — `t15_validation_returns_422_without_writing`, `t19_symlink_escape_is_404`, `t20_html_escaping_prevents_script_injection`, `t30_no_writes_outside_project`.
- #5: Exact command shape and no positionals/subcommands — `t01_command_shape`.
- #6: CLI > environment > configuration > defaults per setting — `t03_precedence_per_setting`; `browser::tests::precedence_is_per_setting`.
- #7: Exit codes and bind failure — `t08_invalid_port_and_host_exit_codes`, `t07_port_in_use_exits_one`, `t26_config_keys`; every `Server` test keeps the process serving until dropped.
- #8: `--no-open` / `browser_open: false` and opener warning only — `t04_open_and_no_open`, `t05_missing_opener_is_a_warning`.
- #9: One-category sidebar, ID ordering, empty states — `t09_list_renders_one_category`, `t10_category_parameter`, `t09_list_empty_project_and_empty_type`, `t09_group_without_members_and_other_requirements`.
- #10: Record/group view content, member list, no "Back to requirements", breadcrumbs/sidebar — `t12_record_views`, `t09_group_without_members_and_other_requirements`, `t28_accessibility_and_compatibility_basics`.
- #11: Editable fields per form; ID/form/type/group read-only — `t13_edit_round_trip_group_and_nonfunctional`, `t16_immutable_fields_are_ignored`.
- #12: Atomic write, unknown-section preservation, LF, reread/CLI parity — `t18_writes_are_atomic_and_leave_no_temporary_file`, `t14_unknown_sections_are_preserved`, `t13_edit_round_trip`, `t13_edit_round_trip_group_and_nonfunctional`.
- #13: Deprecated records hidden and 404 — `t11_deprecated_records_are_hidden`.
- #14: Limits, statuses, one request per connection, `Connection: close`, no TLS — `t22_request_limits_and_status_codes`, `head_matches_get_without_body`, `t22_concurrency_limit_returns_503_and_logs`, `t23_one_request_per_connection`.
- #15: Escaped HTML, `serde_json`, exact ID resolution, symlink 404 — `t20_html_escaping_prevents_script_injection`, `t21_json_api`, `t19_symlink_escape_is_404`.
- #16: Browser config keys in list/get/set/unset/JSON with validation — `t26_config_keys`; `config::tests::browser_defaults`, `browser_validation_messages_name_the_key`, `browser_keys_serialize_to_json`.
- #17: Embedded assets, no external origin, no-JS operation — `t24_assets_are_offline_and_embedded`; `t28_accessibility_and_compatibility_basics` (labels, links, landmarks, no forbidden CSS/JS).
- #18: Integration/unit coverage incl. spawned binary over `TcpStream`, round-trip, assets, boundary/symlink, port conflict, non-loopback, `--no-open`, XSS, JSON, gates — `t01`–`t30` as mapped above; 124 tests pass; dependency list unchanged.
- #19: MOCK-0001 recorded and implemented states match, including the dirty "Unsaved changes" status — `edit_form_dirty_status_is_static_and_hidden_until_input`, `t09_list_renders_one_category`, `t12_record_views`, `t13_edit_round_trip`, `t15_validation_returns_422_without_writing`, `t17_stale_version_is_conflict`, `t28_accessibility_and_compatibility_basics`.

### Second quality-review follow-up (2026-11-03, UTC)

Addressed: over-capacity 503 is written without reading the request (immediate `503 Service Unavailable` with `Connection: close`, log line `- - 503`, no Host/Origin validation), so the accept loop cannot block; configuration-file errors for `browser_port`, `browser_open`, and `browser_host` now exit 4 with a message naming the key; the desktop shell and 320 px states are aligned with MOCK-0001 (210 px sidebar grid column, 30 px gap, main `border-left`, narrow footer/header text, wrapping long "Other requirements" links); 422 summary entries repeat the field-level message; added duplicate `Content-Length` 400, T-14 unknown-section order, 320 px wrapping, and config-file error coverage.

Updated coverage: T-22 `t22_concurrency_limit_returns_503_and_logs` now holds 16 connections, confirms a silent 17th connection receives 503/`Connection: close`/EOF in under 1 s without sending a request, asserts the `- - 503` log line, and confirms a normal request completes in under 1 s after slots free. T-15 `t15_validation_returns_422_without_writing` now asserts each summary entry equals its field-level message for an empty title, a 201-character title, a control character, and a `## ` line in story and description. Test suite: 126 tests pass.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-02T20:29:00Z

---

Refinement note: implementation is gated on approved UI mockups. The specification records 12 open questions with recommended defaults; OQ-1 (browser environment variables PAPYRUS_BROWSER_HOST/PORT/OPEN, which the man page does not document), OQ-2 (startup output shape), and OQ-6 (group editing included in the browser) are recommended for user confirmation before release. MOCK ids are to be assigned by @ui-designer and then recorded here.

---

author: @ui-designer
created: 2026-11-02T20:48:00Z

---

UI mockups produced for the browser increment (layout-only wireframes, all status `pending` pending explicit user approval):

- MOCK-0001 to MOCK-0007 — requirement list: All (default), Business filter, Non-functional filter, empty project, empty document type, group with no members, many groups and long titles.
- MOCK-0008 to MOCK-0012 — record view: business requirement with story, non-functional requirement without story, business group, non-functional group, unresolved group reference.
- MOCK-0013 to MOCK-0017 — edit form: business pristine, non-functional pristine, dirty, validation errors (422), conflict (409).
- MOCK-0018 — one generic error page layout with 404 unknown, 404 deprecated, 400, 405, 415, 500 variants.
- MOCK-0019 and MOCK-0020 — 320 CSS px list and edit form with accessibility annotations.
- MOCK-0021 to MOCK-0025 — clickable prototypes for the list, view, edit, error, and responsive/accessibility screen sets.

Artifacts: `docs/specifications/ui/mockups/` (SVG plus HTML, shared `wireframe.css`), registry `docs/specifications/ui/mockups/README.md`, description `docs/specifications/ui/browser-requirements-ui.md`. No source code, requirements, or specification files were modified.

Status: all mockups `pending`; explicit user approval is required before UI implementation. The MOCK ids still need to be recorded in `docs/specifications/browser-requirements.md` (UI mockups line, handoff, traceability) by the specification owner; the ui-designer write scope does not include that file.

---

author: @ui-designer
created: 2026-11-03T19:31:00Z

---

UI review package consolidated: **MOCK-0026** — `docs/specifications/ui/mockups/MOCK-0026-browser-requirements-review.html`. One file: review landing (TASK-6, m-1, spec §4–§10, approval status), design deck (12 slides), web prototype (25 views: list 6, record 5, edit 6, errors 6, 320 px 2). Status: **pending** — awaiting explicit user approval; not approved for implementation. Supersedes and replaces MOCK-0001 … MOCK-0025 (files removed, ids retired; registry notes the one dropped state, the old two-section `list-all`). UI description and registry updated.

---

author: @orchestrator
created: 2026-11-03T19:44:00Z

---

Superseded mockup record: MOCK-0001 through MOCK-0025 were replaced by the single self-contained review package MOCK-0026 (docs/specifications/ui/mockups/MOCK-0026-browser-requirements-review.html). The shared wireframe.css was inlined into MOCK-0026 and deleted; no separate SVG/HTML mockups remain. All mockups remain pending explicit user approval.

---

author: @orchestrator
created: 2026-11-03T19:49:00Z

---

Mockup numbering reset per user: the single current review package is MOCK-0001 at docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html. MOCK-0026 and the retired MOCK-0002..MOCK-0025 range no longer exist; the registry lists only MOCK-0001, still pending explicit approval.

---

author: @task-refiner
created: 2026-11-03T19:55:00Z

---

UI alignment (2026-11-03): MOCK-0001 approved by the user on 2026-11-03 (UTC). Specification updated to the approved UI model: persistent left "Document types" sidebar selecting one category at a time (default Business requirements; `/?type=req|nfr`, no `all` view), group view lists its member requirements, record view has no "Back to requirements" action (breadcrumbs plus sidebar), 320 px sidebar collapse behind an accessible toggle, and `aria-current` plus a non-colour cue on the selected entry. UI design handoff, expected artifacts, and traceability now reference the single self-contained package; the per-state SVG / per-screen-set HTML expectation was removed. ACs #9, #10, and #19 updated accordingly; all other ACs, the DoD, milestone, dependencies, and priority are unchanged. New spec OQ-13 (non-blocking, default recorded) covers a disabled document type's sidebar entry, which the approved mockup does not show.

---

author: @task-refiner
created: 2026-11-03T20:15:00Z

---

Traceability fix (2026-11-03): AC #1 now states the increment scope — the documentation store for business-requirement and non-functional-requirement groups and requirements — and records that REQ-06.002's backlog-store coverage is delivered by TASK-11. Specification corrected to the approved mockup: §4.15 edit breadcrumb is `Requirements › ID › Edit` (no owning-group crumb), new §4.23 plus §7.12 and UI handoff state the version token is a hidden input with a visible static dashed note, and Assumptions/Traceability record the REQ-06.002 increment boundary. Other ACs, DoD, milestone, dependencies, and priority unchanged.

---

author: @task-refiner
created: 2026-11-03T20:32:00Z

---

Spec ambiguities resolved after the second quality review (2026-11-03, UTC): §3.2/§2.12 over-capacity 503 is written without reading the request and logs '- - 503' (Host/Origin validation does not apply); §2.6/§2.8 only a spawn failure of the opener is observable and warns, a later non-zero exit is not a Papyrus error; §3.5/§3.6/§3.12 unknown path is 404 for every method and a known path with a disallowed method is 405; §4.10 a requirement without '## Group' appears under 'Other requirements' as 'Group: (none)'; §5.3/§7.5 422 summary entries name the actual field error and repeat the field-level message. Specification only; no source or mockup files changed.

---
<!-- COMMENTS:END -->
