---
id: TASK-179
title: "Design UI for removing reference list from document-type browser list views"
created_date: '2026-11-18T19:30:28Z'
updated_date: '2026-11-19T10:54:42Z'
labels: []
references:
  - REQ-06.019
  - RE-006
  - MOCK-0016
status: Done
assignee:
  - ui-designer
milestone: MS-028
dependencies:
  - TASK-178
ordinal: 20
documentation:
  - docs/specifications/ui/mockups/MOCK-021.html
type: ui-design
---

## Description
UI specification and at most one mockup for removing the reference list from document-type browser list views. Only when feature has UI impact. Shared list contract and MOCK-0016.
## Comments
- **ui-designer**: UI design complete for MS-028 and REQ-06.019 (RE-006, ADR-035). UI-019 (verified) specifies that generic ContentModel::Generic list rows omit the reference list, Typed and HtmlMockup families keep it, labels stay for every family, and the single-document view, CLI list and view, JSON API, RecordView, and front matter are unchanged. MOCK-021 (web) shows the Notes list with no reference line, the Note detail with References retained, the Business Rules list with references retained, and the UI Mockups list with references retained. User explicitly approved MOCK-021. No production code changed.
