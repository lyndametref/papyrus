---
id: TASK-70
title: Remove legacy record serializers and enforce the no-drift rule
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - front-matter
  - refactor
  - no-drift
milestone: m-12
dependencies:
  - TASK-67
  - TASK-69
references:
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.001.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: chore
ordinal: 30080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Remove the legacy per-module parsers/writers and the browser re-render path (ADR-0009 Stage 5), and enforce the no-drift rule so one implementation owns parsing, identity, and writing.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Legacy per-module parsers, renderers, and writers are removed; browser apply_*_edit functions are replaced by shared patch operations.
- [ ] #2 No record path uses fs::write; no format opening delimiter exists outside src/record; no ID formatting exists outside record::identity.
- [ ] #3 Every record type reads and writes through the shared module, and existing unit tests pass with their behavior unchanged.
- [ ] #4 Search reads whole persisted record text, including front matter, and the search specification states that metadata values are searchable.
- [ ] #5 Temporary dual implementations are gone and the no-drift checks are part of the test suite.
- [ ] #6 The completion grep checks from ADR-0009 decision 7 return no matches.
<!-- AC:END -->
