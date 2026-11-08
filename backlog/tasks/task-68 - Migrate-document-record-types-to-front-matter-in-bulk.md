---
id: TASK-68
title: >-
  Migrate document record types to front matter lazily on first read and first
  write
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T16:09:00Z'
labels:
  - front-matter
  - migration
  - documentation
milestone: m-12
dependencies:
  - TASK-63
  - TASK-65
references:
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/non-functional/NFR-04.001.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
priority: high
type: feature
ordinal: 30060
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement lazy, per-record migration for document record types under REQ-05.011: each managed Markdown record file gains front matter on its first read and on its first write, with no bulk migration command. Migration runs through record::patch, is idempotent, preserves non-record prose and HTML, leaves frozen ADR bodies intact except for the added front-matter block, and reports each converted file.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On first read and first write of a managed document record without front matter, front matter is added and the record thereafter carries front matter.
- [ ] #2 Migration is idempotent: a record that already carries front matter is not re-migrated on subsequent reads or writes.
- [ ] #3 No bulk migration command is required; the corpus converges record by record as records are read or written.
- [ ] #4 Migration is expressed only through record::patch and reports a per-file inventory with path, action, and resulting keys.
- [ ] #5 Frozen ADR bodies gain only the front-matter block; ## Date and ## References stay intact; decision status and type stay in the document core.
- [ ] #6 Non-record prose under docs/specifications, docs/domain-model, docs/business-processes, README files, and all HTML including mockups are not modified.
- [ ] #7 created_date comes from the record date section where present, otherwise file mtime; updated_date is the migration time; best-effort provenance is documented.
- [ ] #8 After migration no managed document record lacks front matter and the per-file report is the review artifact.
<!-- AC:END -->
