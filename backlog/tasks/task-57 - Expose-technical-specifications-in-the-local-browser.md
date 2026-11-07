---
id: TASK-57
title: Expose technical specifications in the local browser
status: Done
assignee: []
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-07T17:18:00Z'
labels:
  - browser
  - documentation
milestone: m-11
dependencies:
  - TASK-55
  - TASK-58
  - TASK-15
references:
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/domain-model/business-glossary/technical-specification.md
  - docs/specifications/browser-technical-specifications.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/technical-specifications-management.md
  - docs/specifications/search-technical-specifications.md
  - docs/specifications/browser-audit-records.md
  - docs/specifications/browser-business-rules.md
  - docs/specifications/ui/browser-technical-specifications-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html
documentation:
  - docs/specifications/browser-technical-specifications.md
  - docs/specifications/ui/browser-technical-specifications-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html
  - docs/specifications/browser-requirements.md
  - docs/specifications/technical-specifications-management.md
priority: medium
type: feature
ordinal: 29200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
--------------------------------------------------
Expose technical specifications in the local browser: add a Technical Specifications category to the offline read-and-edit web interface, reusing the document views. Active technical specifications (ID, title, description) are listed, viewed, and edited; deprecated ones are hidden and unreachable. The transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused unchanged. Approved UI: MOCK-0009 (TASK-58); UI description: docs/specifications/ui/browser-technical-specifications-ui.md. Full behavior: docs/specifications/browser-technical-specifications.md.

SCOPE (in)
- Category ?type=tech-spec plus sidebar entry "Technical Specifications" immediately after "Audit Records" (before Non-functional requirements); default stays req; any other value 400; entry omitted when document_types.tech-spec.enabled is false (empty state, not an error).
- List active specifications only: one h2 "Technical Specifications" with the record count, rows ascending by ID, link text "ID + title" to /doc/{id}, first paragraph of the description as preview; empty state and long unbreakable-token wrapping; deprecated/, non-matching files, and non-.md files never listed.
- View: metadata (Record ID, form "Technical specification", category "Technical Specifications"), title as h1, description as body text with pre-wrap and no "Description" heading, (none) when the description is empty, breadcrumb "Technical Specifications > ID", exactly one Edit action.
- Edit: title (required, single-line) and description (required, multi-line textarea); hidden version input plus the fixed annotation "Hidden field: version (optimistic-concurrency token)"; Save changes/Cancel; pristine, dirty, 422, 409 conflict, and 320 px states; usable with JavaScript disabled.
- Field-preserving writes: change only the heading title and the ## Description section; preserve the preamble and every unknown/editor-added section and line byte-for-byte; never regenerate the file from a parsed record and never reuse the CLI render_record serializer.
- Validation: title trimmed, 1-200 Unicode scalar values, no control characters; description trimmed, required, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control characters rejected, a line starting with "## " rejected. Messages: "Title is required.", "Title must be at most 200 characters.", "Title cannot contain control characters.", "Description is required.", "Description must be at most 20000 characters.", "Description cannot contain control characters.", "Description contains a line starting with ## .". Validation returns 422, preserves submitted values, and writes nothing.
- Optimistic concurrency and writes: version token "<mtime-seconds>.<mtime-nanos>.<length>"; missing/malformed version 400; stale version 409 with "record changed on disk; reload the form", the MOCK-0009 conflict page and no write; atomic project-local writes; every path through ensure_within; symlink escape 404; no temporary file left; save-then-reread.
- Deprecated specifications: never listed or counted; GET view/edit, POST /doc/{id}, GET/POST /api/doc/{id} return 404 exactly as for an unknown ID; no browser control changes deprecation state.
- JSON API: GET /api/tech-specs (alias /api/tech-spec) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns {id, kind:"tech-spec", form:"tech-spec", title, description, version, path}. POST: version required; title required when present; description required when present (null/empty is invalid); omitted members unchanged; wrong member types 400; invalid values 422.
- Unit tests and integration tests T-57a through T-57m extending tests/browser_cli.rs, covering category/sidebar, list, view, edit round-trip, field preservation, validation, conflict, deprecated visibility, JSON API, atomicity, offline scan, no-JS/320 px structure, and quality gates.

SCOPE (out)
- Creating, deprecating, restoring, or deleting technical specifications from the browser (CLI-only).
- Cross-type navigation beyond the sidebar and final completeness check (TASK-19).
- Search in the browser UI, Markdown rendering, rich-text editing, attachments, uploads.
- Structured NFR/task reference fields; these remain literal text (BR-017 defines none).
- Auth, sessions, cookies, TLS.
- New dependencies, crates, or ADRs; packaging/verification (TASK-59).
- Fixing the CLI's own whole-file rewrite; only the browser path is field-preserving.

RECORD SOURCES AND MODEL
- Active directory <doc_directory>/<document_types.tech-spec.directory>/ (default docs/technical/), prefix default TECH, resolved from config; never hardcoded.
- Only regular *.md files directly in the active folder whose stem matches <prefix>-<digits> are records; deprecated/, non-matching files, and non-.md files ignored; no recursion.
- ID is the exact case-sensitive file stem, never derived from content or accepted from the request.
- title from the first line "# <ID> <title>" (ID alone when no title); description from the ## Description body; absent/empty description is null and displays as (none).
- Unknown sections, preamble, and editor-added content are preserved on save and are not editable fields.
- Form "tech-spec"; labels "Technical specification" and "Technical Specifications".

ASSUMPTIONS
- Assumption: the transport/asset/rendering/security/config baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; one category and record type are added only.
- Assumption: the persisted record is Markdown with the fixed heading + ## Description template; browser edits must be field-preserving.
- Assumption: both title and description are required in the browser per MOCK-0009, technical-specifications-management.md, and BR-017; an on-disk record with an empty description is viewable as (none) but must be completed before saving.
- Assumption: the browser description is multi-line and preserves LF; the CLI --description remains single-line and --editor remains the CLI route for multi-line bodies.
- Assumption: browser validation reuses the baseline limits (title 1-200, text 20000) for consistency with other browser document types; MOCK-0009 does not draw length help text.
- Assumption: deprecated specifications are hidden and unreachable, consistent with REQ-05.007/BR-017 and every other document type.
- Assumption: a disabled or missing tech-spec type contributes no records and no error; the sidebar entry is omitted and ?type=tech-spec renders the empty state.
- Assumption: technical specifications expose no structured NFR/task reference fields; such IDs remain literal text.
- Assumption: real-browser verification (NFR-05.001) is a deferred follow-up, consistent with sibling tickets.
- Assumption: no new dependencies, crates, or ADRs.

OPEN QUESTIONS
- OQ-1 (non-blocking, product): whether technical specifications should gain structured NFR/task references in the browser; current default is literal text.
- OQ-2 (non-blocking, validation): CLI spec and BR-017 state no length limit while the browser baseline applies 1-200 / 20000; this ticket adopts the baseline limits, and MOCK-0009 draws no length help text.
- OQ-3 (non-blocking, follow-up): real-browser verification deferred.

DEPENDENCIES
- TASK-15 - Extend browser access to decision records (completed; sibling baseline).
- TASK-55 - Manage technical specifications from the CLI (completed; record model and lifecycle).
- TASK-58 - Design review: technical specifications browser UI mockup (completed; MOCK-0009 approved).
- Blocks TASK-59 (technical specifications milestone packaging) and TASK-19 (cross-type navigation).

IMPLEMENTATION NOTES
- Reuse the existing browser category/form pattern (Category/Form enumeration, active directory resolution, index building, render templates, route dispatch); do not add a new architecture or dependency.
- Add the tech-spec category to the same category parser, sidebar ordering, list/view/edit/conflict handlers, JSON shapes, and error paths used by audit records and business rules.
- Parse the record into the shared Doc/Section model and write with a field-preserving apply function that replaces only the heading and ## Description; do not call the CLI render_record.
- Reuse the shared validators for title and multi-line text, version tokens, atomic write, and ensure_within.
- Extend tests/browser_cli.rs following the T-26a..T-26l test structure.

REFERENCES FOR CONTEXT/RATIONALE
- Feature spec: docs/specifications/browser-technical-specifications.md
- Baseline: docs/specifications/browser-requirements.md sections 4-10
- Record model: docs/specifications/technical-specifications-management.md (TASK-55)
- Sibling precedents: docs/specifications/browser-audit-records.md (TASK-26), docs/specifications/browser-business-rules.md (TASK-25)
- Search: docs/specifications/search-technical-specifications.md (TASK-56)
- UI description: docs/specifications/ui/browser-technical-specifications-ui.md
- MOCK-0009: docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html (registry approved)
- Requirements: REQ-05.009, REQ-05.007, REQ-06.002; NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001; BR-004, BR-016, BR-017
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Category and navigation: GET /?type=tech-spec renders the Technical Specifications list; the default category stays req and any other value returns 400; the sidebar shows "Technical Specifications" immediately after "Audit Records" (before Non-functional requirements) with aria-current="page" when selected, and the entry is omitted when document_types.tech-spec.enabled is false.
- [x] #2 List: one h2 "Technical Specifications" with the record count and one row per active specification in ascending ID order, each row linking "ID + title" to /doc/{id} with a first-paragraph description preview; empty and long-title states render; non-matching .md files, non-.md files, and deprecated/ are never listed.
- [x] #3 View: /doc/{id} shows ID, form "Technical specification", category "Technical Specifications", title as h1, description as body text with pre-wrap and no "Description" heading, (none) when empty, breadcrumb "Technical Specifications > ID", and exactly one Edit action; the file content is never served or embedded.
- [x] #4 Edit form matches MOCK-0009: title input, description textarea, hidden version input with the fixed annotation, Save changes/Cancel, and pristine, dirty, 422, 409 conflict, and 320 px states; the form remains operable with JavaScript disabled.
- [x] #5 Edit round-trip: saving title and description persists them and the new values are visible on the browser view and list and through papyrus tech-spec view.
- [x] #6 Field-preserving write: editing a specification changes only the heading title and the ## Description section and leaves the preamble and every other section and line byte-for-byte identical; the file is never regenerated from a parsed record or the CLI render_record serializer.
- [x] #7 Title and description source: the title is read from the "# <ID> <title>" heading and the description from the ## Description section; saving rewrites only those parts; an absent ## Description is inserted immediately after the preamble (or after the heading when no section exists).
- [x] #8 Validation: 422 with the documented messages and no write for empty or over-long title, control characters, empty or over-long description, and ## lines; submitted values are preserved and the summary links to the offending fields.
- [x] #9 Concurrency: a missing or malformed version returns 400; a stale version returns 409 with "record changed on disk; reload the form", the MOCK-0009 conflict page/actions, and writes nothing; a matching version writes atomically and the redirect re-reads the file.
- [x] #10 Deprecated and unknown technical-specification 404: deprecated specifications are not listed or counted; GET /doc/{id}, GET /doc/{id}/edit, POST /doc/{id}, GET /api/doc/{id}, and POST /api/doc/{id} return 404. Every HTML 404 for a technical-specification ID (deprecated or unknown) renders the identical technical-specifications 404 page: the help text "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address." and exactly one recovery link "Back to technical specifications" to /?type=tech-spec; JSON 404 responses keep the baseline JSON error object; no browser control changes deprecation state.
- [x] #11 JSON API: GET /api/tech-specs (and alias /api/tech-spec) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns {id, kind:"tech-spec", form:"tech-spec", title, description, version, path}; POST leaves omitted members unchanged, rejects null/empty description, returns 400 for wrong types, and 422 for invalid values.
- [x] #12 Boundary and atomicity: every read and write stays inside the canonicalized project directory, a symlink escape returns 404, and no temporary file is left after a successful or failed request.
- [x] #13 Offline, security, and accessibility: no page references an external origin; all content is HTML-escaped; baseline CSP and security headers are unchanged; controls are keyboard operable and labelled; the interface works with JavaScript disabled.
- [x] #14 Quality gates: cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Implementation matches docs/specifications/browser-technical-specifications.md and the approved MOCK-0009; any deviation is recorded in Implementation Notes with justification.
- [x] #2 Unit tests cover record parsing (heading, ## Description, unknown-section preservation), title extraction, title/description validation, version token, and JSON shapes.
- [x] #3 Integration tests T-57a through T-57m in tests/browser_cli.rs pass and cover category/sidebar, list, view, edit round-trip, field preservation, validation, conflict, deprecated visibility, JSON API, atomicity, the offline asset scan, and no-JS/320 px structure.
- [x] #4 Manual smoke test on a project containing a CLI-created technical specification: edit it through the browser and confirm with git diff that only the heading title and the ## Description section changed.
- [x] #5 cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
- [x] #6 No source changes outside the browser and tech_spec modules plus tests; the implementation edits no backlog or documentation files.
- [x] #7 The existing browser test suite and the CLI tech-spec tests still pass unchanged.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add the technical-specifications category and record form to the existing browser category/form model, configuration-key mapping, and sidebar ordering.
2. Implement the active-directory scan and exact-stem ID resolution using the shared record index and ensure_within.
3. Parse the heading title and ## Description into the shared document model; add a field-preserving apply function for title and description only.
4. Add list, view, edit, and conflict templates matching MOCK-0009; reuse the shared shell, error-page layout, CSS, and optional JS.
5. Wire the category route, list/view/edit/conflict handlers, and the JSON API summary/full/POST endpoints with the documented shapes.
6. Reuse the shared title/multiline validators, version-token checks, atomic write, and 404 handling for deprecated and unknown IDs.
7. Render the technical-specifications 404 page for deprecated and unknown technical-specification IDs, with the technical-specifications help text and the single "Back to technical specifications" recovery link.
8. Extend unit tests and tests/browser_cli.rs with T-57a through T-57m.
9. Verify cargo build, test, clippy, and fmt; run the manual smoke test and confirm field-preserving writes with git diff.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
### Quality-review fixes (2026-11-07)

- Manual smoke test: CLI-created `TECH-001` plus editor-added `## Editor Notes` and `## Unknown` sections, edited through the browser (title and multi-line description). The follow-up view/list and `papyrus tech-spec view TECH-001` showed the new values. A `git diff --no-index` between the pre-edit copy and the saved file shows only the `# TECH-001 ...` heading and the `## Description` body changed; the preamble and both unknown sections are byte-identical.
- MOCK-0009 deviations remaining after the fixes: none requiring further change. Approved deviations: the unified technical-specifications 404 page uses the spec-mandated `Record not found` heading instead of MOCK-0009's deprecated-screen `Record not available`, and the 320 px edit form reuses the fuller desktop help text from MOCK-0009.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: task-refiner
created: 2026-11-07T17:01:00Z

---

Refinement alignment after quality review: AC10 and docs/specifications/browser-technical-specifications.md section 9 now require the technical-specifications 404 page (identical for deprecated and unknown tech-spec IDs) to render the help text "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address." and the single "Back to technical specifications" link to /?type=tech-spec. T-57h and AC-10 mapping updated; no other AC changed.

---
<!-- COMMENTS:END -->
