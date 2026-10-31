---
id: TASK-8
title: Manage milestones and releases from the CLI
status: To Do
assignee: []
created_date: '2026-10-31T22:25:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-7
references:
  - docs/requirements/business/REQ-03.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Project maintainers need to define delivery milestones and group them into releases before tasks can be assigned to those delivery containers.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can create, list, view, and edit milestones and releases with the documented fields and IDs.
- [ ] #2 A milestone can belong to zero or one release, and one release can include multiple milestones.
- [ ] #3 Milestone archive and deletion behavior match the documented commands; release status is represented as documented without inferring an undocumented archive command.
- [ ] #4 Milestone and release data uses project-local text files, sanitized input, and no access outside the project directory.
<!-- AC:END -->
