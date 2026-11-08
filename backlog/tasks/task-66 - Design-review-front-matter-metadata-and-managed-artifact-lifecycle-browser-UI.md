---
id: TASK-66
title: 'Design review: front-matter metadata and managed-artifact lifecycle browser UI'
status: To Do
assignee: []
created_date: '2026-11-08T15:06:00Z'
labels:
  - design-review
  - ui-design
milestone: m-12
dependencies:
  - TASK-59
references:
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 30092.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design the browser changes for front-matter metadata in existing list/detail/edit views and the new managed-artifact lifecycle page; obtain explicit user approval before implementation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The review package covers list, detail, and edit views for all managed record types, showing at least id, title, created_date, updated_date, labels, references, and type-specific keys.
- [ ] #2 Edit forms keep the current editable field set; timestamps are read-only; metadata edits target front matter, not body sections.
- [ ] #3 A lifecycle page prototype has one section for each fixed record type and shows task states from configured statuses and documented lifecycles for other types.
- [ ] #4 UI descriptions for the changed list/detail/edit views and the lifecycle page reference REQ-06.003 and REQ-06.004.
- [ ] #5 The mockup is registered in the mockup registry and marked approved only after explicit user approval.
- [ ] #6 The review package is self-contained, clickable, and consistent with the approved MOCK-0001 shell and navigation.
<!-- AC:END -->
