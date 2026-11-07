---
id: TASK-56
title: Search technical specifications
status: Done
assignee: []
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-07T11:13:00Z'
labels:
  - search
  - documentation
milestone: m-11
dependencies:
  - TASK-55
references:
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/search-technical-specifications.md
priority: medium
type: task
ordinal: 29100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend unified search to include technical specifications.

Technical specifications are document records stored under the configured document_types.tech-spec directory (default docs/technical/). They are searched whenever doc is selected, including the default all-types search. Only active records matching <prefix>-<digits>.md (for example TECH-001.md) are searched; the deprecated/ subfolder and non-record files are never scanned. A disabled or absent tech-spec type contributes no results and no error. Deprecated technical specifications are hidden; restored specifications reappear.

Matching, ordering, output shapes, exit codes, boundary enforcement, and the read-only guarantee are reused unchanged from search-requirements.md. The type filter value remains doc; no dedicated --type tech-spec value is added.

Assumptions:
- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004); no new ADR or dependency is required.
- Assumption: deprecated technical specifications are hidden from search, consistent with other document types and BR-017/REQ-05.007.
- Assumption: the searchable text is the whole persisted record text, so editor-added body sections are searchable.
- Assumption: text output uses doc as the type value, not the tech-spec type or prefix.
- Assumption: record IDs come from file stems, so configured prefixes are honored without hardcoding TECH.
- Assumption: technical specifications live in the configured document_types.tech-spec directory (default technical/), gated by document_types like other document records.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: search performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only.

Open questions:
- Whether a future man-page revision should add a dedicated tech-spec type filter (default: no).

Implementation notes:
- Likely change is to include the tech-spec document type key in the document keys searched by src/search.rs, reusing collect_type and its boundary/disabled/deprecation behavior, with record-pattern filtering for the shared directory (consistent with tech-spec list).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Search returns technical specifications by default and supports type filters.
- [x] #2 Quoted phrases and limits work across supported types.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover active technical specifications in default and --type doc search, exclusion of other type filters, quoted phrases, --limit, deprecated exclusion, disabled type, missing directory, symlink escape, and read-only behavior.
- [ ] #3 Feature specification docs/specifications/search-technical-specifications.md reviewed and aligned.
- [ ] #4 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #5 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- Rust 1.99.1 and single-crate layout retained (ADR-0001, ADR-0004); no new ADR or dependency.
- Deprecated technical specifications hidden from search.
- Searchable text is whole persisted record text.
- text output uses doc as type value.
- Record IDs come from file stems.
- tech-spec lives in configured document_types.tech-spec directory.
- Unreadable files skipped; unreadable dir or boundary violation exits 1.
- search performs no writes.
- No UI mockups (CLI-only).

Open questions:
- Whether a future man-page revision should add a dedicated tech-spec type filter (default: no).

Implementation notes:
- Add "tech-spec" to DOC_KEYS in src/search.rs.
- Reuse collect_type boundary/disabled/deprecation behavior.
- Filter file stems by configured prefix + digits, consistent with tech-spec list.
- Add integration tests in tests/search_cli.rs following audit search tests.
<!-- SECTION:NOTES:END -->
