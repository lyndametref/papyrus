---
id: TASK-6
title: Serve offline browser access for requirement content
status: Initiated
assignee:
  - '@orchestrator'
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-03T19:55:00Z'
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
- [ ] #1 The browser command serves the local web interface over the project's backlog and documentation stores and opens it by default unless disabled.
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
<!-- COMMENTS:END -->
