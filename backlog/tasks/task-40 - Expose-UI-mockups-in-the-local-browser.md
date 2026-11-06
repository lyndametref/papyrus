---
id: TASK-40
title: Expose UI mockups in the local browser
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T20:56:00Z'
labels:
  - browser
  - documentation
milestone: m-7
dependencies:
  - TASK-38
  - TASK-41
  - TASK-15
references:
  - docs/specifications/browser-ui-mockups.md
  - docs/specifications/ui/mockups/MOCK-0006-browser-ui-mockups-review.html
  - docs/specifications/ui/browser-ui-mockups-ui.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
priority: medium
type: feature
ordinal: 25200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the offline, project-local local browser with a UI Mockups category that lists, views, and edits active UI mockups (prefix MOCK, default directory docs/specifications/ui/mockups/). Reuses the approved browser shell, templates, security, offline, and optimistic-concurrency baseline (TASK-6, ADR-0005/ADR-0006) and the approved UI design MOCK-0006 (TASK-41). Deprecated mockups are hidden and unreachable. Implements REQ-05.008 and the web-visibility part of REQ-05.007 under REQ-06.002.

SCOPE (in)
- Category ?type=ui-mockup plus sidebar entry "UI Mockups" after "UI Specifications"; default stays req; any other value 400; entry omitted when document_types.ui-mockup.enabled is false (empty state, not an error).
- List active mockups only: one h2 "UI Mockups" with record count, rows ascending by ID, row link text "ID + title" to /doc/{id}; ignore deprecated/, README.md, wireframe.css, and non-matching files; empty and long-title states as baseline.
- View: metadata (Record ID read-only, form "UI mockup", category "UI Mockups"), title h1, "UI specifications" and "References" sections with pre-wrap and (none) when empty, breadcrumb "UI Mockups > ID", single Edit action. The stored HTML package is never served, inlined, iframed, or embedded.
- Edit title (required), specs (required, one or more UI specification ids), refs (optional) with the MOCK-0006 tag add/remove interaction; hidden version plus "Hidden field: version (optimistic-concurrency token)" annotation; Save/Cancel; pristine, dirty, 422, 409, and 320 px states; usable without JavaScript.
- Field-preserving writes: change only the <title> text, a stub <h1> that starts with the record ID, the specs/refs HTML comment lines, and visible metadata lines that already exist. Preserve every other byte of the review package (styles, scripts, slides, body markup). Never regenerate the file from a parsed record and never reuse the CLI stub serializer.
- JSON API: GET /api/ui-mockups (aliases /api/ui-mockup, /api/mockups) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns the full record {id, kind:"ui-mockup", form:"ui-mockup", title, specs[], refs[], task, status, superseded_by, version, path}. POST: version required; title optional-but-required-when-present; specs optional non-empty array (null is 422); refs optional array (empty array or null clears); omitted members unchanged; wrong types 400.
- Validation (422, no write, submitted values preserved, summary links, field messages): title trimmed 1-200 chars no control characters; specs at least one valid record ID (1-128 bytes, ASCII letters/digits/._-, no leading . or -, no ..), no duplicates, existence not required; refs each 1-200 chars no control characters, may contain / and ., no duplicates.
- Optimistic concurrency and writes: version token "<mtime-seconds>.<mtime-nanos>.<length>"; missing/malformed version 400; stale token 409 "record changed on disk; reload the form"; atomic project-local writes; every path through ensure_within; symlink escape 404; no temporary file left; save-then-reread; ID, folder, status unchanged.
- Deprecated mockups: never listed or counted; GET view/edit, POST, and both API methods return 404 exactly as for an unknown ID.
- Tests: unit tests for metadata extraction, field-preserving serialization, validation, and JSON shapes; integration tests in tests/browser_cli.rs for list, view, edit round-trip, preservation of a review package, repeated form fields, validation, conflict, deprecated visibility, JSON API, atomicity, and offline asset scan.

SCOPE (out)
- Creating, deprecating, restoring, superseding, or deleting mockups from the browser; changing IDs, status, task, or supersession (CLI-only).
- Editing the mockup's design content or rendering/previewing the package; managing the mockups registry README.
- Search, Markdown rendering, rich-text editing, attachments, authentication, sessions, TLS.
- New dependencies, crates, or ADRs; packaging and release verification (TASK-42).
- Fixing the CLI's own whole-file rewrite (flagged as follow-up OQ-3 in the specification).

IMPLEMENTATION REQUIREMENT - approved UI vs partial implementation (not an open question)
The approved UI description docs/specifications/ui/browser-ui-mockups-ui.md and MOCK-0006 require specs and refs to use the tag add/remove interaction. The uncommitted partial implementation on branch task/40 renders them as textareas with whitespace-separated values. Replace that with the tag interaction: one tag per value with a "Remove <value>" control, an add input plus Add button, repeated specs/refs form fields collected in document order (all occurrences, not last-value-wins), and a no-JS fallback where each tag is a text input (clearing it removes the value) plus a trailing empty input with the same name (typing adds a value). Chip layout and states follow MOCK-0006.

Known deviations in the partial implementation to correct (do not treat as design choices)
1. specs/refs textareas instead of tag add/remove.
2. apply_ui_mockup_edit regenerates the whole file through ui_mockup::render_record, which replaces a review package with the minimal stub; replace with field-preserving editing.
3. JSON GET/POST for a mockup omits specs, refs, task, status, and superseded_by.
4. ID and title are taken from the first <h1>, which misreads review packages (for example "TASK-41 - Design review..."); the ID must be the file stem and the title must come from <title>.
5. specs/refs are read by ad-hoc string splitting in render and mod; centralize one parser used by list, view, edit, and API.

ASSUMPTIONS
- The persisted mockup is a design artifact authored by @ui-designer; browser edits must be field-preserving.
- The HTML comment block is the machine-readable metadata source; review packages without it show empty specs/refs and receive the comment block on first save; visible metadata lines are never inserted into such packages.
- Spec id existence is not required, matching ui-mockup create/edit; duplicates are rejected because the tag fields model a set (stricter than the CLI).
- task/status/superseded_by are exposed read-only in JSON but not displayed in the browser (MOCK-0006 OQ-3).
- Real-browser verification in Firefox/Edge/Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with TASK-15/TASK-23/TASK-24.
- No new dependencies, crates, or ADRs.

DEPENDENCIES
- TASK-15 (browser baseline) - completed.
- TASK-38 (ui-mockup CLI record model and lifecycle) - completed.
- TASK-41 (approved UI design, MOCK-0006) - completed.
- Blocks TASK-42 (UI-mockups milestone packaging).

REFERENCES
- Specification: docs/specifications/browser-ui-mockups.md
- Baseline: docs/specifications/browser-requirements.md sections 4-10
- Record model and lifecycle: docs/specifications/ui-mockups-management.md (TASK-38)
- Sibling precedent: docs/specifications/browser-ui-specifications.md (TASK-24)
- UI description: docs/specifications/ui/browser-ui-mockups-ui.md
- MOCK-0006: docs/specifications/ui/mockups/MOCK-0006-browser-ui-mockups-review.html (registry status approved)
- Requirements: REQ-05.007, REQ-05.008, REQ-06.002; NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001; BR-004, BR-016
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Category and navigation: GET /?type=ui-mockup renders the UI Mockups list; the default category stays req and any other value returns 400; the sidebar shows "UI Mockups" after "UI Specifications" with aria-current="page" when selected, and the entry is omitted when document_types.ui-mockup.enabled is false.
- [x] #2 List: one h2 "UI Mockups" with the record count and one row per active mockup in ascending ID order, each row linking "ID + title" to /doc/{id}; empty and long-title states render; README.md, wireframe.css, non-matching .html files, and deprecated/ are never listed.
- [x] #3 View: /doc/{id} shows ID, form "UI mockup", category "UI Mockups", title as h1, "UI specifications" and "References" sections with pre-wrap and (none) when empty, breadcrumb "UI Mockups > ID", and exactly one Edit action; the stored HTML package, its scripts, and its markup never appear in the response.
- [x] #4 Edit form matches MOCK-0006: title input, specs and refs tag add/remove interaction (remove control per tag, add input plus Add button), hidden version input with the fixed annotation, Save/Cancel, and pristine, dirty, 422, 409, and 320 px states; the tag fields remain operable with JavaScript disabled (clear a tag input to remove, type in the trailing empty input to add).
- [x] #5 Edit round-trip: saving title, specs, and refs persists them and the new values are visible on the browser view and list and through papyrus ui-mockup view.
- [x] #6 Field-preserving write: editing a real review package changes only the metadata it edits (title text, stub h1 when present, specs/refs comments, existing visible metadata lines) and leaves every other byte identical; the file is never regenerated from a parsed record or the CLI stub serializer.
- [x] #7 Title source: the title is read from <title> after the first em dash or &mdash;, else with a leading record ID removed, else the whole <title> text, else the record ID; saving updates <title> preserving the separator style and rewrites only an <h1> that starts with the record ID.
- [x] #8 Validation: 422 with the documented messages and no write for empty or over-long title, control characters, zero specs, invalid spec id, duplicate spec id, over-long ref, control characters, whitespace, or duplicate refs; submitted values are preserved and the summary links to the offending fields.
- [x] #9 Concurrency: a missing or malformed version returns 400; a stale version returns 409 with "record changed on disk; reload the form" and writes nothing; a matching version writes atomically and the redirect re-reads the file.
- [x] #10 Deprecated visibility: deprecated mockups are not listed or counted and return 404 on view, edit, POST /doc/{id}, GET /api/doc/{id}, and POST /api/doc/{id}; no browser control changes deprecation or supersession state.
- [x] #11 JSON API: GET /api/ui-mockups (and aliases /api/ui-mockup, /api/mockups) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns {id, kind:"ui-mockup", form:"ui-mockup", title, specs[], refs[], task, status, superseded_by, version, path}; POST leaves omitted members unchanged, ignores the read-only task/status/superseded_by members, rejects an empty or null specs with 422, clears refs with [] or null, and returns 400 for wrong types.
- [x] #12 Boundary and atomicity: every read and write stays inside the canonicalized project directory, a symlink escape returns 404, and no temporary file is left after a successful or failed request.
- [x] #13 Offline, security, and accessibility: no page references an external origin; all metadata is HTML-escaped; baseline CSP and security headers are unchanged; tag controls are keyboard operable and labelled.
- [x] #14 Quality gates: cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
- [x] #2 Unit tests cover metadata extraction, field-preserving serialization, validation, and JSON shapes.
- [x] #3 Integration tests T-40a through T-40m in tests/browser_cli.rs pass and cover list, view, edit round-trip, review-package preservation, repeated form fields, validation, conflict, deprecated visibility, JSON API, atomicity, and the offline asset scan.
- [x] #4 Manual smoke test on a project containing both a CLI-created stub and a real review package: edit each through the browser and confirm with git diff that only metadata lines changed for the review package.
- [x] #5 Browser list, view, edit, and API behavior matches docs/specifications/browser-ui-mockups.md; any deviation is recorded in Implementation Notes with justification.
- [x] #6 No source changes outside the browser and ui_mockup modules plus tests; the implementation edits no backlog or documentation files.
- [x] #7 The existing browser test suite and the CLI ui-mockup tests still pass unchanged.
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-06T20:13:00Z

---

Refinement audit of the uncommitted partial implementation on branch task/40 (read-only review; no source changes made). Required behavior is in docs/specifications/browser-ui-mockups.md. Gaps to fix: (1) specs/refs are textareas with whitespace-separated values, but the approved MOCK-0006 requires tag add/remove; (2) records::apply_ui_mockup_edit calls ui_mockup::render_record, which rewrites the whole file and would replace a real review package with the minimal stub; browser saves must be field-preserving; (3) record_json for UiMockup omits specs, refs, task, status, and superseded_by; (4) ui_mockup::parse_record takes ID and title from the first <h1>, which misreads review packages; ID must be the file stem and title must come from <title>; (5) specs/refs are parsed by ad-hoc string splitting in render.rs and mod.rs instead of one shared parser. Also flagged as a follow-up outside this ticket: the CLI ui-mockup edit path has the same whole-file rewrite risk (OQ-3 in the specification).

---

created: 2026-11-06T20:56:00Z

---

Validated 2026-11-06 (UTC): quality-gatekeeper Pass with findings (14/14 AC, 7/7 DoD); cargo test 290 passed; user-approved for Done.

---
<!-- COMMENTS:END -->
