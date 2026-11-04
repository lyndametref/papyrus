---
id: TASK-40
title: Expose UI mockups in the local browser
status: To Do
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-04T19:22:00Z'
labels:
  - browser
  - documentation
milestone: m-7
dependencies:
  - TASK-38
  - TASK-41
  - TASK-15
priority: medium
type: feature
ordinal: 25200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement list/view/edit for UI mockups in the local browser, reusing document views.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Browser lists mockups with ID and title and excludes deprecated per visibility rule.
- [ ] #2 Selecting a mockup displays content and allows editing; changes persist.
- [ ] #3 Interface works offline, sanitizes input, no paths outside project; renders in supported browsers.
<!-- AC:END -->
