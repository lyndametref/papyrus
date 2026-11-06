---
id: TASK-23
title: Expose glossary terms in the local browser
status: Done
assignee: []
created_date: '2026-11-02T20:19:00Z'
updated_date: '2026-11-06T15:21:00Z'
labels:
  - browser
  - documentation
milestone: m-5
dependencies:
  - TASK-30
  - TASK-32
  - TASK-15
references:
  - README.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/specifications/browser-glossary-terms.md
  - docs/specifications/ui/browser-glossary-terms-ui.md
documentation:
  - docs/specifications/browser-glossary-terms.md
  - docs/specifications/ui/browser-glossary-terms-ui.md
type: feature
ordinal: 23300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the offline, project-local browser (TASK-6 baseline, ADR-0005/ADR-0006) to serve active glossary terms from the glossary store, with read and edit support, under the same transport, asset, rendering, security, and configuration baseline.

Scope:
- Categories: extend the persistent sidebar with a Glossary entry (after Decisions) and the ?type= selector to glossary (default req; any other value 400).
- List/view/edit active glossary terms; deprecated terms hidden and unreachable (404).
- Glossary editable fields: term (required) and definition (optional, multi-line). ID, record form, and category read-only.
- JSON API endpoints for glossary (/api/glossary) and generic /api/doc/{id}.
- Reuse baseline escaping, ID resolution, boundary checks, Host/Origin validation, request limits, a11y.

Out of scope: create/deprecate/restore/delete from browser; other record types; search in browser; new deps/ADRs; packaging (TASK-33); cross-type navigation and final completeness check (TASK-19).

Spec: docs/specifications/browser-glossary-terms.md
UI: MOCK-0004 (approved) extends MOCK-0001/MOCK-0002/MOCK-0003.

Acceptance criteria mapping: AC-1..AC-10 in the spec; test scenarios T-23a..T-23j in spec section 10. Investigation notes (storage location, route shape, reused components, new additions) are in spec section "Investigation notes (AC #3)".
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The local browser displays and edits active glossary terms in addition to all previously supported requirement, backlog, and decision content.
- [ ] #2 Category selection supports type=glossary (default req); any other value, including all, returns 400.
- [ ] #3 Deprecated glossary terms are not listed and return 404 by URL.
- [ ] #4 Glossary view shows ID, term, and definition.
- [ ] #5 An edit round-trip persists and is visible through both the browser and the CLI.
- [ ] #6 Field validation returns 422 and does not write for empty term, term over 200 characters, control characters, definition over 20000 characters, and lines starting with ## .
- [ ] #7 A stale version token returns 409 and does not write.
- [ ] #8 Writes are atomic, project-local, leave no temporary file, and symlink escapes return 404.
- [ ] #9 Browser operations remain offline-capable and use no paths outside the project directory except during init.
- [ ] #10 The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Unit and integration tests added/updated and pass.
- [ ] #2 cargo build, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
- [ ] #3 Feature spec and UI description updated with MOCK-0004 reference.
- [ ] #4 MOCK-0004 explicitly approved by the user before implementation (approved 2026-11-06).
- [ ] #5 No new dependencies, crates, or ADRs introduced.
- [ ] #6 Explicit test scenarios T-23a-T-23j (spec section 10) implemented and passing.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation pointers (refinement only; no code):

Storage and records:
- Active terms: <doc_directory>/<document_types.glossary.directory>/ (default docs/glossary/); deprecated in its deprecated/ subfolder; files <ID>.md.
- File format: "# <ID> <term>" heading then "## Definition" section. Parsed with parse_doc/section_value.
- Add Category::Glossary and Form::Glossary variants in records.rs; extend active_entry/active_dir, load path, build_index, resolve_id.
- load_glossary_record reads active folder only, honors document_types.glossary and enabled flag, skips deprecated/.
- apply_glossary_edit serializer (known section: ## Definition) preserving unknown sections.

Rendering:
- sidebar: add Glossary entry after Decisions.
- category_name/short/heading mappings for Glossary.
- backlog_list_section: add Glossary row meta (none beyond ID + term + definition preview first paragraph).
- view: metadata block (ID, form "Glossary term", category "Glossary"), term as h1, definition pre-wrap.
- edit: GlossaryEditValues (term, definition) and GlossaryFieldErrors; render edit_page_glossary.
- breadcrumb/edit_breadcrumb: Glossary label.

HTTP/routing:
- category_from_query: accept "glossary".
- edit_page: match Category::Glossary.
- form_post: match Category::Glossary (term via title validation, definition via validate_multiline).
- record_json: Glossary shape.
- /api/glossary endpoint; include Glossary in api_all? follow decisions pattern.

Tests:
- Extend tests/browser_cli.rs with T-23a..T-23j.

Security/boundaries unchanged: Context::ensure_within, Host/Origin, timeouts, limits.
<!-- SECTION:NOTES:END -->
