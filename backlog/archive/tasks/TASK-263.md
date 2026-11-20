---
id: TASK-263
title: "Design the generic document list, view, edit, conflict, and not-found surface"
created_date: '2026-11-20T17:42:31Z'
updated_date: '2026-11-20T18:00:07Z'
labels: []
references:
  - ADR-039
  - ADR-0028
  - ADR-0033
  - REQ-06.014
  - REQ-06.019
  - TECH-068
status: ToDo
assignee:
  - ui-designer
milestone: MS-035
dependencies:
  - TASK-262
ordinal: 20
documentation: []
type: ui-design
---

## Description
Scope: specify one list layout, one view layout, one edit layout, one conflict page, and one not-found page for document types, with role-aware sections for GroupedMarkdown group and member records and the fixed metadata block for SingleHtmlMockup. Presentation facts come from the seed. Reference visibility per type and role is seed-declared. Specify at most one mockup for this milestone. State the semantic invariants the fixture comparison must check: selectors, text, navigation, form fields, and status codes. Do not design a lifecycle page; ADR-0034 keeps it retired.
## Comments
- **planner**: Retired from the executable plan by owner decision: no UI design needed; presentation facts come from the config seed.
