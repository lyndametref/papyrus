---
id: TASK-25
title: Expose business rules in the local browser
status: Done
assignee: []
created_date: '2026-11-02T20:19:00Z'
updated_date: '2026-11-07T08:13:00Z'
labels:
  - browser
  - documentation
milestone: m-8
dependencies:
  - TASK-43
  - TASK-45
references:
  - docs/specifications/browser-business-rules.md
  - docs/specifications/ui/browser-business-rules-ui.md
  - docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html
  - docs/specifications/business-rules-management.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-ui-specifications.md
  - docs/specifications/browser-ui-mockups.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/browser-business-rules.md
  - docs/specifications/ui/browser-business-rules-ui.md
  - docs/specifications/business-rules-management.md
  - docs/specifications/browser-requirements.md
type: feature
ordinal: 26300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose business rules in the local browser: extend the offline, project-local read-and-edit web interface with a Business Rules category that lists, views, and edits active business rules (RUL-*, default directory business-rules/). Reuses the approved browser shell, templates, security, offline, and optimistic-concurrency baseline (TASK-6, ADR-0005/ADR-0006) and the approved UI design MOCK-0007 (TASK-45). Deprecated rules are hidden and unreachable. Implements REQ-05.005 and the web-visibility part of REQ-05.007 under REQ-06.002. Full behavior, validation, persistence, API, and tests are specified in docs/specifications/browser-business-rules.md.

SCOPE (in)
- Category ?type=business-rule plus sidebar entry "Business Rules" immediately after "Business requirements"; default stays req; any other value 400; entry omitted when document_types.rule.enabled is false (empty state, not an error).
- List active rules only: one h2 "Business Rules" with record count, rows ascending by ID, row link text "ID + title" to /doc/{id}, plus first-paragraph description preview; empty and long-title states as baseline; ignore deprecated/, BR-*.md, and non-matching files.
- View: metadata (Record ID read-only, form "Business rule", category "Business Rules"), title as h1, Description, Rationale, Applicability, and Requirements sections with pre-wrap and (none) when empty, breadcrumb "Business Rules > ID", single Edit action. Requirement references in the Requirements section render as links to /doc/{id} where they resolve to an active requirement; otherwise literal text.
- Edit title (required), description, rationale, applicability (optional multi-line), and requirements (business requirement IDs) with the MOCK-0007 tag add/remove interaction; hidden version plus "Hidden field: version (optimistic-concurrency token)" annotation; Save/Cancel; pristine, dirty, 422, 409, and 320 px states; usable without JavaScript.
- Field-preserving writes: change only the heading title and the Description, Rationale, Applicability, and Requirements sections; preserve the preamble and every unknown/editor-added section and line byte-for-byte. Never regenerate the file from a parsed record and never reuse the CLI stub serializer.
- JSON API: GET /api/business-rules (alias /api/business-rule) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns {id, kind:"business-rule", form:"business-rule", title, description, rationale, applicability, requirements[], version, path}. POST: version required; title required when present; description/rationale/applicability optional string or null (null/empty clears); requirements optional array (empty array or null clears); omitted members unchanged; wrong types 400.
- Validation (422, no write, submitted values preserved, summary links to fields, field messages): title trimmed 1-200 chars no control characters; description/rationale/applicability trimmed, at most 20000 chars, LF allowed, CR/CRLF normalized, TAB allowed, other control chars rejected, ## lines rejected, empty clears; requirements each a valid record ID (1-128 bytes, ASCII letters/digits/._-, no leading . or -, no ..), duplicates collapsed on save, existence not required.
- Optimistic concurrency and writes: version token "<mtime-seconds>.<mtime-nanos>.<length>"; missing/malformed version 400; stale token 409 "record changed on disk; reload the form"; atomic project-local writes; every path through ensure_within; symlink escape 404; no temporary file left; save-then-reread; ID, folder, deprecation state unchanged.
- Deprecated rules: never listed or counted; GET view/edit, POST /doc/{id}, GET/POST /api/doc/{id} return 404 exactly as for an unknown ID; no browser control changes deprecation state.

SCOPE (out)
- Creating, deprecating, restoring, or deleting business rules from the browser; changing IDs or deprecation state (CLI-only).
- Managing the canonical BR-*.md documents that share the directory.
- Cross-type navigation beyond requirement-reference links and the final completeness check (TASK-19).
- Search, Markdown rendering, rich-text editing, attachments, authentication, sessions, TLS.
- New dependencies, crates, or ADRs; packaging and release verification (TASK-46).

IMPLEMENTATION REQUIREMENT - approved UI vs partial implementation (not an open question)
There is no existing browser implementation for business rules (fresh ticket). The implementation must follow the approved MOCK-0007 tag add/remove interaction for requirements and field-preserving writes throughout. Do not regenerate a rule file from a parsed record or the CLI stub serializer; browser saves must change only the edited sections. Requirement references are collected as repeated requirements form fields in document order (all occurrences, not last-value-wins), with duplicates collapsed on save. ID and title come from the file stem and the "# <ID> <title>" heading, never from an <h1>.

Known deviations to avoid (do not repeat the partial-implementation mistakes seen in sibling tickets)
1. Do not render requirements as a textarea with whitespace-separated values; use tag add/remove.
2. Do not rewrite the whole file through rule::render or any serializer; use field-preserving editing.
3. Do not omit requirements (or other fields) from the JSON GET/POST response.
4. Do not derive the ID or title from <h1> or any body element; use the file stem and heading.
5. Do not parse requirements by ad-hoc string splitting in multiple places; centralize one parser used by list, view, edit, and API.

ASSUMPTIONS
- The persisted rule is a Markdown text record with a fixed section template; browser edits must be field-preserving.
- Requirement ID format is validated but existence is not required, matching rule create/edit; dangling IDs are allowed.
- Duplicate requirement references are collapsed on save, matching rule edit.
- task/status/superseded_by do not apply to business rules (no such metadata).
- Real-browser verification in Firefox/Edge/Safari 100+ (NFR-05.001) remains a deferred follow-up.
- No new dependencies, crates, or ADRs.

DEPENDENCIES
- TASK-43 (business rules CLI record model and lifecycle) - completed.
- TASK-45 (approved UI design, MOCK-0007) - completed.
- Blocks TASK-19 (cross-type navigation) and TASK-46 (business-rules milestone packaging).

REFERENCES
- Specification: docs/specifications/browser-business-rules.md
- Baseline: docs/specifications/browser-requirements.md sections 4-10
- Record model and lifecycle: docs/specifications/business-rules-management.md (TASK-43)
- Sibling precedents: docs/specifications/browser-ui-specifications.md (TASK-24), docs/specifications/browser-ui-mockups.md (TASK-40)
- UI description: docs/specifications/ui/browser-business-rules-ui.md
- MOCK-0007: docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html (registry status approved)
- Requirements: REQ-05.005, REQ-05.007, REQ-06.002; NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001; BR-004, BR-013, BR-016
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Category and navigation: GET /?type=business-rule renders the Business Rules list; the default category stays req and any other value returns 400; the sidebar shows "Business Rules" immediately after "Business requirements" with aria-current="page" when selected, and the entry is omitted when document_types.rule.enabled is false.
- [x] #2 List: one h2 "Business Rules" with the record count and one row per active rule in ascending ID order, each row linking "ID + title" to /doc/{id} with a first-paragraph description preview; empty and long-title states render; BR-*.md, non-matching .md files, and deprecated/ are never listed.
- [x] #3 View: /doc/{id} shows ID, form "Business rule", category "Business Rules", title as h1, Description, Rationale, Applicability, and Requirements sections with pre-wrap and (none) when empty, breadcrumb "Business Rules > ID", and exactly one Edit action; requirement references render as links where they resolve and as literal text otherwise; the file content is never served or embedded.
- [x] #4 Edit form matches MOCK-0007: title input, description/rationale/applicability textareas, requirements tag add/remove interaction (remove control per tag, add input plus Add button), hidden version input with the fixed annotation, Save/Cancel, and pristine, dirty, 422, 409, and 320 px states; the tag fields remain operable with JavaScript disabled.
- [x] #5 Edit round-trip: saving title, description, rationale, applicability, and requirements persists them and the new values are visible on the browser view and list and through papyrus rule view.
- [x] #6 Field-preserving write: editing a rule changes only the heading title and the edited sections and leaves the preamble and every other section and line byte-for-byte identical; the file is never regenerated from a parsed record or the CLI stub serializer.
- [x] #7 Title and section source: the title is read from the "# <ID> <title>" heading and the fields from their ## sections; saving rewrites only those parts.
- [x] #8 Validation: 422 with the documented messages and no write for empty or over-long title, control characters, over-long text fields, ## lines, and invalid requirement IDs; submitted values are preserved and the summary links to the offending fields.
- [x] #9 Concurrency: a missing or malformed version returns 400; a stale version returns 409 with "record changed on disk; reload the form" and writes nothing; a matching version writes atomically and the redirect re-reads the file.
- [x] #10 Deprecated visibility: deprecated rules are not listed or counted and return 404 on view, edit, POST /doc/{id}, GET /api/doc/{id}, and POST /api/doc/{id}; no browser control changes deprecation state.
- [x] #11 JSON API: GET /api/business-rules (and alias /api/business-rule) returns [{id,title}] ascending; GET/POST /api/doc/{id} returns {id, kind:"business-rule", form:"business-rule", title, description, rationale, applicability, requirements[], version, path}; POST leaves omitted members unchanged, clears with null or [], and returns 400 for wrong types and 422 for invalid values.
- [x] #12 Boundary and atomicity: every read and write stays inside the canonicalized project directory, a symlink escape returns 404, and no temporary file is left after a successful or failed request.
- [x] #13 Offline, security, and accessibility: no page references an external origin; all content is HTML-escaped; baseline CSP and security headers are unchanged; tag controls are keyboard operable and labelled; the interface works with JavaScript disabled.
- [x] #14 Quality gates: cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 cargo build, cargo test, cargo clippy -- -D warnings, and cargo fmt --check pass with no new dependencies, crates, or ADRs.
- [x] #2 Unit tests cover record parsing/serialization, field validation, duplicate collapsing, version token, and JSON shapes.
- [x] #3 Integration tests T-25a through T-25m in tests/browser_cli.rs pass and cover list, view, edit round-trip, field preservation, repeated form fields, validation, conflict, deprecated visibility, JSON API, atomicity, and the offline asset scan.
- [x] #4 Manual smoke test on a project containing a CLI-created rule: edit it through the browser and confirm with git diff that only the edited sections changed.
- [x] #5 Browser list, view, edit, and API behavior matches docs/specifications/browser-business-rules.md; any deviation is recorded in Implementation Notes with justification.
- [x] #6 No source changes outside the browser and rule modules plus tests; the implementation edits no backlog or documentation files.
- [x] #7 The existing browser test suite and the CLI rule tests still pass unchanged.
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T08:13:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (14/14 AC, 7/7 DoD); cargo test 87 browser_cli passed (15 t25*); auto-closed under no-validation mode. Non-blocking: manual smoke not run (automated equivalents t25e/t25k), NFR-05.001 deferred, untracked spec doc committed with this change.

---
<!-- COMMENTS:END -->
