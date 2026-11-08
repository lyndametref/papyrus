---
id: TASK-19
title: Complete browser access for all Papyrus content
status: To Do
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-08T15:06:00Z'
labels: []
milestone: m-12
dependencies:
  - TASK-60
  - TASK-23
  - TASK-24
  - TASK-25
  - TASK-26
  - TASK-67
  - TASK-72
references:
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/requirements/business/REQ-06.004.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
ordinal: 30300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Users need one local, offline browser interface that exposes every record type supported by Papyrus, not a separate interface for each document family. Per-type list, view, and edit views are delivered by the dedicated document-type browser tasks (TASK-6, TASK-11, TASK-15, TASK-23 through TASK-26). This task consolidates them into one navigation and content inventory and verifies cross-type consistency; it does not re-implement per-type rendering.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 One browser navigation and content inventory exposes all supported Papyrus content — requirements, tasks, milestones, releases, decisions, glossary terms, UI specifications, business rules, and audit records — using the per-type list/view/edit views delivered by earlier browser increments.
- [ ] #2 Browser changes persist in text files inside the project directory and are visible in search according to documented lifecycle rules.
- [ ] #3 The local browser works without internet access and renders in Firefox, Microsoft Edge, and Safari version 100 or later.
- [ ] #4 Browser input is sanitized and no path outside the project directory is read or written, except during init.
<!-- AC:END -->
