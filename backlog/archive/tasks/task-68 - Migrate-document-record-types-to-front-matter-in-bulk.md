---
id: TASK-68
title: >-
  Migrate document record types to front matter lazily on first read and first
  write
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T18:43:00Z'
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
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
documentation:
  - >-
    docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md
priority: high
type: feature
ordinal: 30060
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement lazy, per-record migration for the document record types under REQ-05.011 and REQ-05.010: each managed Markdown record (business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, audit record, decision, release) gains a YAML front-matter block on its first read and on its first write. No bulk migration command is added; the corpus converges record by record as records are read or written. Migration runs only through record::patch and record::store::write_atomic, is idempotent, preserves non-record prose and HTML, leaves frozen ADR bodies intact except for the added front-matter block, and reports each converted file.

Managed record types in scope: release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, audit record. Task and milestone migration is out of scope (TASK-69). UI mockups and other HTML are excluded (REQ-05.008). Non-record prose Markdown is never modified (ADR-0007 decision 15).

Dependencies: TASK-63 (Done), TASK-65 (Done). Relates to TASK-69, TASK-70, TASK-71, TASK-72.

Feature specification: docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On first read and first write of a managed document record without front matter, front matter is added and the record thereafter carries front matter.
- [ ] #2 Migration is idempotent: a record that already carries front matter is not re-migrated on subsequent reads or writes.
- [ ] #3 No bulk migration command is required; the corpus converges record by record as records are read or written.
- [ ] #4 Migration is expressed only through record::patch and reports a per-file inventory with path, action, and resulting keys.
- [ ] #5 Frozen ADR bodies gain only the front-matter block; ## Date and ## References stay intact; decision status and type stay in the document core.
- [ ] #6 Non-record prose under docs/specifications, docs/domain-model, docs/business-processes, README files, and all HTML including mockups are not modified.
- [ ] #7 created_date comes from the record date section where present (decisions: ## Date), otherwise file mtime; updated_date is the migration time; best-effort provenance is documented.
- [ ] #8 After migration no managed document record lacks front matter and the per-file report is the review artifact.
- [ ] #9 Audit records move the retired ## Audit Type value into front-matter type and clear that section; all other document bodies remain byte-identical apart from the added front-matter block.
- [ ] #10 Read-path migration failures (read-only media, missing permission) leave the original file byte-identical and surface the error.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo test passes for the record module, affected command modules, and integration tests.
- [ ] #2 No fs::write call on a record path exists outside src/record/store.
- [ ] #3 No front-matter delimiter formatting exists outside src/record/.
- [ ] #4 Feature specification is updated and linked from the ticket.
- [ ] #5 No UI mockup or HTML file is modified by this ticket.
<!-- DOD:END -->
