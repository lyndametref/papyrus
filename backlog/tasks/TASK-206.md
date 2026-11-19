---
id: TASK-206
title: "Design UI for lowest-ordinal milestone ordering, markers, and cleanup in the Backlog board"
created_date: '2026-11-19T11:02:48Z'
updated_date: '2026-11-19T18:23:55Z'
labels: []
references:
  - REQ-06.020
  - REQ-06.021
  - REQ-06.022
  - REQ-06.013
  - RE-008
  - UI-002
  - MOCK-0002
  - MOCK-0017
status: Done
assignee:
  - ui-designer
milestone: MS-030
dependencies:
  - TASK-205
ordinal: 20
documentation:
  - docs/specifications/ui/mockups/MOCK-022.html
type: ui-design
---

## Description
UI specification and at most one mockup for ordering milestones and releases by the lowest ordinal of their non-archived tasks ascending, lifecycle markers, and cleanup controls in the browser Backlog board. Revises UI-002, which currently specifies ascending milestone and release ID ordering and contradicts the corrected rule; it must document ascending lowest-ordinal ordering plus the markers and cleanup controls. May amend MOCK-0002 or MOCK-0017. MOCK-0016 is not impacted: the browser shell and flat list layout are out of scope for the Backlog board change.
## Comments
- **ui-designer**: UI design completed and mockup approved.
- **orchestrator**: User verified UI-002 revision and approved MOCK-022 mockup. UI design verified.
