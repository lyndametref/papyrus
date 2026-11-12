---
id: TASK-101
title: Revise the backlog browser specifications for the per-milestone kanban board
status: Done
assignee: []
created_date: '2026-11-11T21:15:00Z'
updated_date: '2026-11-12T07:17:00Z'
labels:
  - specification
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-86
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - docs/specifications/ui/mockups/README.md
  - docs/specifications/browser-spec-revisions-task-101.md
documentation:
  - docs/specifications/browser-spec-revisions-task-101.md
priority: high
type: docs
ordinal: 39035
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Revise the browser specification corpus so it states the per-milestone kanban board contract required by REQ-06.013, and name the mockup revision that must be approved before implementation.

Scope (in): revise docs/specifications/browser-tasks-milestones-and-releases.md (section 3 List page) and docs/specifications/ui/browser-backlog-records-ui.md (section 4 Backlog view) so they state:
- Per-milestone kanban board: milestones stay expandable/collapsible; an expanded milestone shows its tasks in one column per configured task status, in configured order, with empty-status columns still visible.
- Release grouping: milestones assigned to a release appear under that release; milestones with no release are grouped before the first release; every release boundary is a bold separator naming the release and not relying on colour.
- Tasks with no milestone: own expandable section after all milestone sections; cards in ascending task ID order showing ID and title and linking to the record view.
- Card order and contents: ascending task ID within a column; at least ID and title, linking to the record view; further card contents are design decisions.
- Status change from the board: drag between columns plus an equivalent non-drag status control on the card; immediate persistence of the status value only through the existing browser write contract (ADR-0006 version token, ADR-0013 single write path); all other task fields preserved; on persistence failure the card stays in its original column and the failure is reported.
- Archived tasks and milestones excluded.
No revised section may still describe releases as separators between a flat milestone list or status change as drag-only. The specification names the MOCK-0002 revision (via TASK-102) that must be approved before implementation.

Scope (out): implementing the board or status change (TASK-103); designing/approving mockups (TASK-102); changing requirements or business rules; changing code; allocating a new MOCK id.

Dependencies: TASK-86 (completed). Downstream: TASK-102 (mockup revision), TASK-103 (implementation).

Assumptions: release/milestone/task ordering as in docs/specifications/browser-spec-revisions-task-101.md; the board reuses the existing write contract with no new endpoint, crate, module, or ADR; card contents beyond ID and title, column widths, separator styling, and the non-drag control shape are design decisions resolved by TASK-102.

Open questions: OQ-1 whether any flat task listing remains reachable alongside the board (recommended: board replaces it); OQ-2 whether status change is submitted as form POST or via the JSON API (implementation choice in TASK-103). Requirements and business rules are unchanged.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The backlog browser specification defines the per-milestone kanban board: each expanded milestone shows its tasks in one column per configured task status, in configured order, and statuses with no tasks keep their column visible.
- [ ] #2 The specification defines release grouping: milestones assigned to a release appear under that release, milestones with no release are grouped before the first release, and every release boundary is a bold separator that names the release and does not rely on colour.
- [ ] #3 The specification defines the section for tasks that belong to no milestone after all milestone sections, cards in ascending task ID order showing ID and title and linking to the record view, and the exclusion of archived tasks and milestones.
- [ ] #4 The specification defines status change from the board: dragging a card between columns and an equivalent status control on the card that needs no dragging, immediate persistence of the status value only through the existing browser write contract (ADR-0006 version token, ADR-0013 single write path), all other task fields preserved, and the card left in its original column with the failure reported when persistence fails.
- [ ] #5 No revised section still describes releases as separators between a flat milestone list, or status change as drag-only, and the specification names the mockup revision that must be approved before implementation.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All target browser specifications (browser-tasks-milestones-and-releases.md, browser-backlog-records-ui.md) revised and internally consistent
- [ ] #2 No revised section describes releases as separators between a flat milestone list or status change as drag-only
- [ ] #3 The MOCK-0002 revision is named and must be approved (TASK-102) before implementation
- [ ] #4 Traceability sections cite REQ-06.013, ADR-0006, and ADR-0013
- [ ] #5 Requirements and business rules files are unchanged
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Revised docs/specifications/browser-tasks-milestones-and-releases.md §3 (List page) to the per-milestone kanban board contract (release grouping, tasks-without-milestone section, card order/contents, status-change via drag + non-drag control, archived exclusion, design-decision list). Revised docs/specifications/ui/browser-backlog-records-ui.md §4 (Backlog view) accordingly. Updated mockups/README.md to note MOCK-0002 requires revision and re-approval under TASK-102. Traceability cites REQ-06.013, ADR-0006, ADR-0013.
<!-- SECTION:NOTES:END -->
