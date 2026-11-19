---
id: TECH-062
title: "Browser Specification Revisions for the Per-Milestone Kanban Board (TASK-101)"
created_date: '2026-11-16T17:46:52Z'
updated_date: '2026-11-19T18:07:29Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - BR-004
  - BR-005
  - BR-016
  - MOCK-0002
  - NFR-01.002
  - NFR-01.003
  - NFR-05.001
  - REQ-01.002
  - REQ-02.001
  - REQ-02.002
  - REQ-06.002
  - REQ-06.003
  - REQ-06.013
  - REQ-06.020
  - REQ-06.021
  - REQ-06.022
  - RE-008
  - ADR-0022
  - ADR-0032
  - TASK-205
  - TASK-101
  - TASK-102
  - TASK-103
  - TASK-11
  - TASK-86
status: draft
---
# TECH-062 Browser Specification Revisions for the Per-Milestone Kanban Board (TASK-101)

## Description
**Ticket:** TASK-101
**Milestone:** m-16 (Release 1.0)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html) (approved baseline) — revision required and re-approval required under TASK-102 before implementation. No new MOCK id is allocated by this ticket.

## Purpose

Revise the browser specification corpus so that it states the new backlog view contract introduced by REQ-06.013: milestones remain expandable and collapsible, an expanded milestone shows its tasks as a kanban board with one column per configured task status, milestones are grouped by release with a bold named separator, tasks without a milestone get their own section after all milestones, cards are ordered by ascending task ID and link to the task record view, and a task's status is changed either by dragging its card or with an equivalent non-drag control, persisting immediately through the existing browser write contract. The revisions also remove any wording that still describes releases as separators between a flat milestone list or status change as drag-only.

This ticket is documentation/specification work only. It does not change requirements, business rules, implementation code, or UI mockups. The mockup revision is delivered by TASK-102; the implementation is delivered by TASK-103.

## Scope

### In scope

- Revising `docs/specifications/browser-tasks-milestones-and-releases.md` so its list-page section states the per-milestone kanban board, release grouping, the tasks-without-milestone section, card order and contents, and the status-change contract.
- Revising `docs/specifications/ui/browser-backlog-records-ui.md` section 4 (Backlog view) so it states the same contract and no longer describes releases as separators between a flat milestone list or status change as drag-only.
- Recording which mockup package requires revision (MOCK-0002, via TASK-102) and that it must be approved before implementation.
- Updating traceability sections in the revised specifications to cite REQ-06.013, ADR-0006, and ADR-0013.

### Out of scope

- Implementing the board or the status change (TASK-103).
- Designing or approving UI mockups (TASK-102, `@ui-designer`).
- Changing requirements (`docs/requirements/`) or business rules (`docs/business-rules/`).
- Changing CLI, server, or template implementation.
- Creating, deprecating, restoring, or renaming records.
- Allocating a new MOCK id; the revision reuses MOCK-0002.

## Behavior

### 1. Board layout (AC #1)

1. The Backlog view keeps milestones as expandable and collapsible sections (existing behavior, retained).
2. Expanding a milestone renders that milestone's tasks as a kanban board: one column per configured task status (REQ-01.002), laid out horizontally within the milestone.
3. Columns appear in configured status order exactly; no reordering, no omission.
4. A status that has no tasks for that milestone keeps its column visible (empty column with an explicit empty state).
5. The board is nested inside the milestone section; milestones remain the grouping unit of the view.

### 2. Release grouping (AC #2)

1. Milestones assigned to a release appear under that release.
2. Milestones assigned to no release are grouped together before the first release.
3. Each release boundary is marked by a bold separator that names the release and does not rely on colour alone.
4. Ordering follows REQ-06.020: milestones within a release and in the no-release group are ordered by the lowest ordinal among their non-archived tasks ascending (milestones with no non-archived task after all others, by milestone ID ascending; ties by milestone ID ascending); releases are ordered by the lowest ordinal among the non-archived tasks in their milestones ascending (releases with no such task after all others, by release ID ascending; ties by release ID ascending). The no-release group precedes all releases.

### 3. Tasks without a milestone (AC #3)

1. Tasks that belong to no milestone appear in their own expandable section.
2. That section is placed after all milestone sections (including all release groups).
3. Cards in the section are ordered by ascending task ID.
4. Each card shows the task ID and title and links to the task record view.

### 4. Card order and contents (AC #3, AC #5)

1. Cards are ordered by ascending task ID within a column.
2. Each card shows at least the task ID and title and links to the task record view (`/doc/{id}`).
3. Card contents beyond the ID and title (assignee, labels, and similar) are design decisions and are not fixed by this specification.
4. Column widths, separator styling, and empty-column presentation details are design decisions.

### 5. Status change from the board (AC #4)

1. A task's status is changed by dragging its card into another status column.
2. Each card also offers an equivalent status control that performs the same change without dragging (pointer-free operable).
3. Either action persists the status immediately to the task record; the board is not a staging area with a separate save step.
4. Persistence uses the existing browser write contract: the version token of ADR-0006 and the single write path of ADR-0013. No second write path, endpoint, or ADR is introduced.
5. The change alters only the status value and preserves every other field of the task.
6. A change that cannot be persisted leaves the card in its original column and reports the failure to the user.
7. After a successful change the value is still shown after the view is reloaded (save-then-reread).

### 6. Archived exclusion (AC #3)

1. Archived tasks and archived milestones are not shown on the board.
2. Archived exclusion is consistent with the rest of the browser backlog views.

### 7. Presentation details that remain design decisions

1. Card contents beyond the task ID and title.
2. Column widths.
3. Separator styling and weight beyond "bold" and "names the release".
4. The shape of the non-drag status control (select, buttons, menu).
5. Default expansion state of milestones.
6. Empty-column placeholder text.

These are resolved by TASK-102 (mockup revision) and recorded in the UI description.

## Target specifications

| Specification | Required change |
|---|---|
| `docs/specifications/browser-tasks-milestones-and-releases.md` | Section 3 (List page) revised to the per-milestone kanban board, release grouping, tasks-without-milestone section, card order and contents, and status-change contract; no flat per-category task rows; traceability cites REQ-06.013, ADR-0006, ADR-0013. |
| `docs/specifications/ui/browser-backlog-records-ui.md` | Section 4 (Backlog view) revised to state empty columns, release grouping, no-milestone section, card order and contents, and the non-drag status control; no wording that describes releases as separators between a flat milestone list or status change as drag-only; mockup registry note updated. |
| `docs/specifications/ui/mockups/README.md` | Note that MOCK-0002 requires revision and re-approval (delivered by TASK-102); no renumbering. |

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

- Baseline artifact: MOCK-0002 — `docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html`, approved 2026-11-04 (TASK-11). It currently shows the earlier backlog list and no longer represents the contract above.
- Revision: MOCK-0002 is revised in place (no renumbering) by TASK-102 to the per-milestone kanban board, including visible empty columns, release grouping with a bold named separator, the tasks-without-milestone section, ID-plus-title cards linking to the record view, the drag-and-drop interaction, the non-drag status control, and the failed-persistence state (card in original column, failure reported). The revised package keeps the existing shell, record views, and edit flows except where the board changes them, and covers the 320 px layout and a pointer-free status control.
- The revised MOCK-0002 must be explicitly approved by the user and recorded as `approved` in the mockup registry before TASK-103 implementation starts. No package is approved for implementation until the user explicitly approves it.

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | Per-milestone kanban board: one column per configured status, configured order, empty columns visible. |
| AC-2 | Release grouping: milestones under their release, no-release group before first release, bold named separator not relying on colour. |
| AC-3 | Tasks-without-milestone section after all milestones; cards ascending task ID showing ID and title linking to record view; archived excluded. |
| AC-4 | Status change by drag and by non-drag control; immediate persistence through existing write contract (version token, single write path); only status changes; card stays in original column with failure reported on persistence failure. |
| AC-5 | No revised section describes releases as separators between a flat milestone list or status change as drag-only; the specification names the mockup revision (MOCK-0002 via TASK-102) that must be approved before implementation. |

## Assumptions

- Assumption: Release and milestone ordering follows REQ-06.020 (lowest contained non-archived task ordinal ascending, with no-task records last ordered by ID ascending, and ties by ID ascending); the no-release group precedes all releases. Lifecycle markers (REQ-06.021) and milestone/release cleanup controls (REQ-06.022) are specified in TECH-011 and do not change this board contract.
- Assumption: Task ordering within a column is ascending task ID, consistent with the existing browser list ordering.
- Assumption: The `?type=` category selector and the sidebar Backlog entry remain; the combined Backlog view's task presentation becomes the board. The flat per-category task row list is replaced by the board surface.
- Assumption: The board reuses the existing browser write contract (ADR-0006 version token, ADR-0013 single write path); no new endpoint, crate, module, or ADR is introduced.
- Assumption: Card contents beyond ID and title, column widths, separator styling, the non-drag control shape, default expansion state, and empty-column placeholder text are design decisions resolved by TASK-102.
- Assumption: Archived tasks and milestones are excluded, consistent with the existing browser backlog views.
- Assumption: No new dependencies, crates, or ADRs are introduced by this ticket.

## Open questions

- **OQ-1 (recommended default: board replaces the flat task list)** — Whether any flat task listing should remain reachable (for example `?type=task` rendering rows) alongside the board. Recommended default: the board replaces the flat task list in the combined Backlog view. Subject to product confirmation during TASK-102 review.
- **OQ-2 (recommended default: status change sends the full current record with status changed)** — Whether the board's status change is submitted as a form POST carrying the unchanged title and the new status, or via the JSON API. Either way only the status value changes and all other fields are preserved through the existing write contract. Subject to implementation choice in TASK-103.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06.013; REQ-06.002; REQ-06.003; REQ-01.002; REQ-02.001; REQ-02.002; NFR-01.002; NFR-01.003; NFR-05.001 |
| Business rule | BR-004 (browser command behavior); BR-005 (task command behavior); BR-016 (configuration keys and precedence) |
| ADR | ADR-0005; ADR-0006; ADR-0010; ADR-0012; ADR-0013 |
| Ticket | TASK-86 (dependency); TASK-101; downstream TASK-102 (mockup revision), TASK-103 (implementation) |
| Specification | `docs/specifications/browser-tasks-milestones-and-releases.md`; `docs/specifications/ui/browser-backlog-records-ui.md`; `docs/specifications/browser-requirements.md` (shared shell, shared record view layout, reference links) |
| UI mockups | MOCK-0002 (approved baseline; revision required and re-approval required under TASK-102) |
| Related specification | `docs/specifications/browser-spec-revisions-task-86.md` (prior browser revision pass) |
