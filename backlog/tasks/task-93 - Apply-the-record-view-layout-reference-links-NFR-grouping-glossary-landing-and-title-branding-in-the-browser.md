---
id: TASK-93
title: >-
  Apply the record view layout, reference links, NFR grouping, glossary landing,
  and title branding in the browser
status: To Do
assignee: []
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-86
  - TASK-87
  - TASK-88
references:
  - docs/requirements/business/REQ-06.006.md
  - docs/requirements/business/REQ-06.009.md
  - docs/requirements/business/REQ-06.010.md
  - docs/requirements/business/REQ-06.011.md
  - docs/requirements/business/REQ-06.012.md
  - docs/requirements/business/REQ-01.005.md
priority: high
type: feature
ordinal: 39100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.006 and REQ-06.009 to REQ-06.012 define presentation rules the browser does not implement: non-functional requirements sit in a catch-all section, record metadata is spread across the page, references are plain text, the glossary landing page shows dates and references, and the header emphasizes Papyrus instead of the project. These changes share the rendering layer and the approved mockup, so they are delivered together.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every single record view shows a summary card with only the record ID, created date, and updated date, plus the owning group for a sub-requirement.
- [ ] #2 Labels render as a taglist under the title, and dependencies and references render at the end as native disclosure sections that appear only when the record has at least one value and work with JavaScript disabled.
- [ ] #3 Task dependencies, a task milestone, references values, and a sub-requirement owning group render as links to the referenced record; an unresolved value renders as plain text without an error; links are local and work offline.
- [ ] #4 Non-functional atomic requirements are listed under their owning group with the group title, a missing group falls back to the group number, an empty group still renders, and no Other requirements catch-all remains.
- [ ] #5 The glossary landing page omits created and updated dates and references and shows labels as tags, while glossary detail views follow the record view layout.
- [ ] #6 The shared header renders the configured project name in large bold type and the Papyrus mention smaller on every page, leaving the P mark unchanged.
- [ ] #7 Routes, links, form values, and JSON carry Papyrus IDs only.
- [ ] #8 Tests or fixtures cover the group derivation, unresolved references, disclosure sections, and header rendering.
<!-- AC:END -->
