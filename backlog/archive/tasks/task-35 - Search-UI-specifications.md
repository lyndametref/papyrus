---
id: TASK-35
title: Search UI specifications
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T16:20:00Z'
labels:
  - search
  - documentation
milestone: m-6
dependencies:
  - TASK-34
references:
  - docs/specifications/search-ui-specifications.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.007.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
documentation:
  - docs/specifications/search-ui-specifications.md
priority: medium
type: task
ordinal: 24100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend unified search so UI specifications are included by default and are searchable as document records.

UI specifications are stored under the configured document_types.ui-spec directory (default docs/specifications/ui/) and are searched whenever the documented doc type is selected (including the default all-types search). Deprecated UI specifications are hidden; disabled or absent ui-spec document types contribute no results. No new --type value is added; UI specifications report type doc.

Because the ui-spec directory is shared with UI description documents and the mockups/ subfolder, only files whose stem matches the configured record pattern (<prefix>-<digits>, e.g. UI-001) are treated as records, consistent with ui-spec list.

Specification: docs/specifications/search-ui-specifications.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Default search (no --type) includes active UI specifications together with requirement, task, milestone, release, decision, and glossary records.
- [ ] #2 --type doc returns UI specifications; --type task, milestone, release, and decision never return UI specifications.
- [ ] #3 Matching is case-insensitive substring matching over the whole UI specification file text (heading, View, and Description sections).
- [ ] #4 Quoted phrases are matched as one contiguous substring across UI specifications, including internal spaces.
- [ ] #5 Deprecated UI specifications never appear in results; restored specifications appear again.
- [ ] #6 A disabled or absent ui-spec document type contributes no UI specification results and no error.
- [ ] #7 UI specifications are returned with type value doc and title equal to the specification title (text after the ID on the first heading; ID fallback when empty).
- [ ] #8 Results are ordered by record ID ascending across all searched stores.
- [ ] #9 --limit <n> caps the combined result set after ordering.
- [ ] #10 A missing UI-specification directory yields empty success without creating it.
- [ ] #11 A symbolic link escaping the project directory exits 1 without exposing outside content.
- [ ] #12 search creates no files or directories.
- [ ] #13 Build, tests, clippy, and formatting pass with existing dependencies only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover UI specification search, type filters, quoted phrases, limits, deprecation, disabled type, boundary, and no-write behavior.
- [ ] #3 Feature specification docs/specifications/search-ui-specifications.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- UI specifications are document records searched under --type doc; no new --type value is added.
- Deprecated UI specifications are hidden; disabled/absent ui-spec document types contribute no results.
- Record IDs come from file stems; titles come from the first heading (specification title).
- Because specifications/ui/ is shared with UI description documents and mockups/, only files whose stem matches the configured record pattern (<prefix>-<digits>) are candidates, consistent with ui-spec list.
- Unreadable individual files are skipped; unreadable directories or boundary violations fail with exit 1.

Open questions:
- Whether a future man-page revision should add a dedicated ui-spec type filter (default: no).

Implementation notes:
- Likely change is to include the ui-spec document type key in the document keys searched by src/search.rs, reusing collect_type and its boundary/disabled/deprecation behavior, with record-pattern filtering for the shared directory (consistent with ui-spec list).
<!-- SECTION:NOTES:END -->
