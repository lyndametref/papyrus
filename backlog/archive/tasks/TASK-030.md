---
id: TASK-030
title: Search glossary terms
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T13:01:00Z'
labels:
  - search
  - documentation
milestone: MS-006
dependencies:
  - TASK-029
references:
  - docs/specifications/search-glossary-terms.md
  - docs/requirements/business/REQ-006.001.md
  - docs/requirements/business/REQ-005.003.md
  - docs/requirements/business/REQ-005.007.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
documentation:
priority: medium
type: task
ordinal: 23100
---


## Description
Extend unified search so glossary terms are included by default and are searchable as document records.

Glossary terms are stored under the configured document_types.glossary directory (default docs/glossary/) and are searched whenever the documented `doc` type is selected (including the default all-types search). Deprecated glossary terms are hidden; disabled glossary document types contribute no results. No new `--type` value is added; glossary terms report type `doc`.

Specification: docs/specifications/search-glossary-terms.md

## Acceptance Criteria
- [ ] Default search includes active glossary terms together with requirement, task, milestone, release, and decision records.
- [ ] --type doc returns glossary terms; --type task, milestone, release, and decision never return glossary terms.
- [ ] Matching is case-insensitive substring matching over the whole glossary term file text.
- [ ] Quoted phrases are matched as one contiguous substring across glossary terms.
- [ ] Deprecated glossary terms never appear; restored terms appear again.
- [ ] A disabled or absent glossary document type contributes no glossary results and no error.
- [ ] Glossary terms are returned with type value doc and title equal to the term.
- [ ] Results are ordered by record ID ascending across all searched stores.
- [ ] --limit <n> caps the combined result set after ordering.
- [ ] A missing glossary directory yields empty success without creating it.
- [ ] A symbolic link escaping the project directory exits 1 without exposing outside content.
- [ ] search creates no files or directories.
- [ ] Build, tests, clippy, and formatting pass with existing dependencies only.

## Definition of Done
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover glossary search, type filters, quoted phrases, limits, deprecation, disabled type, boundary, and no-write behavior.
- [ ] #3 Feature specification docs/specifications/search-glossary-terms.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.


## Notes
Assumptions:
- Glossary terms are document records and are searched under --type doc; no new --type value is added.
- Deprecated glossary terms are hidden; disabled/absent glossary document types contribute no results.
- Record IDs come from file stems; titles come from the first heading (term text).
- Unreadable individual files are skipped; unreadable directories or boundary violations fail with exit 1.

Open questions:
- Whether a future man-page revision should add a dedicated glossary type filter (default: no).

Implementation notes:
- Likely change is to include the glossary document type key in the document keys searched by src/search.rs, reusing collect_type and its boundary/disabled/deprecation behavior.
