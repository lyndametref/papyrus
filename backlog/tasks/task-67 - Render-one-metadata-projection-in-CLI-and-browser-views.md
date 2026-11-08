---
id: TASK-67
title: Render one metadata projection in CLI and browser views
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - front-matter
  - cli
  - browser
  - metadata
milestone: m-12
dependencies:
  - TASK-63
  - TASK-64
  - TASK-65
  - TASK-66
references:
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-01.005.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
priority: high
type: feature
ordinal: 30050
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver one metadata projection for CLI text/JSON and browser list/detail, and route browser saves through the shared front-matter writer per REQ-06.004.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 One RecordView projection feeds CLI list/view plain text, CLI --json, browser list/detail JSON, and browser HTML.
- [ ] #2 CLI and browser show identical metadata values for the same record: display id, title, created_date, updated_date, labels, references, and the type-specific keys.
- [ ] #3 --json adds an ordered metadata object; the top-level identity remains the display ID and any storage ID appears only inside metadata or a labelled diagnostic.
- [ ] #4 Browser editable fields are unchanged and created_date and updated_date are read-only.
- [ ] #5 A browser save patches front matter, preserves other front-matter keys and the rest of the file, refreshes updated_date, and never copies metadata into the body.
- [ ] #6 Tests assert CLI/browser parity, front-matter round trip on save, read-only timestamps, and absence of storage IDs in identity positions.
- [ ] #7 The browser specification pages that currently claim front-matter preservation are corrected as a prerequisite by business-analyst/tech-writer.
<!-- AC:END -->
