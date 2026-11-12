---
id: TASK-87
title: >-
  Design review: browser configuration view, single record view layout, and
  lifecycle configuration
status: To Do
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - design-review
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-86
references:
  - docs/specifications/ui/mockups/README.md
  - docs/requirements/business/REQ-06.008.md
  - docs/requirements/business/REQ-06.012.md
  - docs/requirements/business/REQ-06.003.md
priority: high
type: task
ordinal: 39142.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.008 adds an editable configuration page and REQ-06.012 restyles every single record view, and no existing mockup package covers either. The workflow requires an approved mockup before implementation, so without this review the browser tasks would invent layout and the approval gate would be skipped. This task produces the clickable package the user approves.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A new numbered mockup package exists under docs/specifications/ui/mockups/ with a review landing and clickable prototype covering the configuration view, the single record view layout, the lifecycle page per-artifact description and read-only configuration, non-functional requirement grouping, the shared header branding, dependency and reference links, and the glossary landing page.
- [ ] #2 The package has a matching UI description document under docs/specifications/ui/ and is registered in the mockup registry with its status.
- [ ] #3 The package is submitted for explicit user approval, and the registry records approved before the browser implementation tasks start.
- [ ] #4 The approved design covers every REQ-06.003 and REQ-06.006 to REQ-06.012 surface listed above and reuses the existing wireframe styles.
<!-- AC:END -->
