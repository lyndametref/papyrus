# Feature Specification — Design Review: Revise the Backlog Kanban Board Mockup (TASK-102)

**Ticket:** TASK-102
**Milestone:** m-16 (Release 1.0)
**Status:** Refined (execution-ready)
**Requirement:** REQ-06.013
**UI mockups:** [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html) (approved baseline) — revised in place and re-approval required under this ticket. No new MOCK id is allocated.
**UI description:** [`ui/browser-backlog-records-ui.md`](ui/browser-backlog-records-ui.md) (section 4, Backlog view; section 9, MOCK register; section 10, approval status)
**Related specification:** [`browser-spec-revisions-task-101.md`](browser-spec-revisions-task-101.md) (kanban board contract); [`browser-tasks-milestones-and-releases.md`](browser-tasks-milestones-and-releases.md) (section 3, List page)

## Purpose

Revise the approved MOCK-0002 backlog review package so that it represents the per-milestone kanban board required by REQ-06.013, instead of the earlier flat backlog list. The revised package must be explicitly approved by the user and recorded as `approved` in the mockup registry before TASK-103 implementation starts. UI mockups may not be implemented as final UI while unapproved.

This ticket is UI design work only. It does not change requirements, business rules, specifications, or implementation code; the kanban board contract is already specified by REQ-06.013, TASK-101, `browser-tasks-milestones-and-releases.md` section 3, and `browser-backlog-records-ui.md` section 4. The feature specification for the revision handoff is this document.

## Scope

### In scope

- Revising MOCK-0002 in place (no renumbering, no new MOCK id) to a per-milestone kanban board mockup.
- Covering, in the revised package: expandable milestones containing a kanban board with one column per configured status; visible empty columns; milestone grouping under each release with a bold named separator; a tasks-without-milestone section after the milestones; ID plus title cards linking to the record view; the card status control that changes status without dragging; the drag-and-drop interaction; the state after a failed status persistence (card in its original column, failure reported).
- Keeping the existing shell, record views, and edit flows unchanged except where the board changes them.
- Covering the 320 px layout and a pointer-free status control.
- Updating the UI description (`browser-backlog-records-ui.md`) and the mockup registry entry for MOCK-0002 to record the revision, its new views, and its status.
- Submitting the revised package for explicit user approval and recording `approved` only after explicit approval.

### Out of scope

- Implementing the board or the status change (TASK-103).
- Changing requirements (`docs/requirements/`) or business rules (`docs/business-rules/`).
- Changing the browser specification corpus (delivered by TASK-101).
- Allocating a new MOCK id (the revision reuses MOCK-0002).
- Approving the mockup on behalf of the user (explicit user approval is required).

## Surfaces the revised package must cover (MOCK-0002)

1. **Shared shell** (reused from the approved baseline): header, sidebar with the Backlog entry, `main`, footer, one `h1` per page, 320 px sidebar toggle.
2. **Backlog board** (`/?type=task`, Backlog entry): milestones as expandable/collapsible sections; expanded milestone renders its tasks as a kanban board (one column per configured status, configured order, empty columns visible); milestones grouped by release with a bold named separator (no-release group before the first release); a tasks-without-milestone section after all milestones; cards showing ID + title linking to the record view (`/doc/{id}`); drag-and-drop status change; equivalent non-drag (pointer-free) status control; failed-persistence state (card in original column, failure reported); 320 px layout.
3. **Milestone list** (`/?type=milestone`): rows with ID + title link and release; empty state (unchanged).
4. **Release list** (`/?type=release`): rows with ID + title link; empty state (unchanged).
5. **Task view** (`/doc/{id}`): metadata block, description, breadcrumb, single Edit (unchanged).
6. **Milestone view** and **Release view** (unchanged).
7. **Edit forms**: task, milestone, release; pristine, dirty, validation-error, conflict, and 320 px states (unchanged).
8. **Error pages** (unchanged).
9. **Responsive 320 px** list and edit states (unchanged, plus board at 320 px).

## Presentation details resolved by this revision (design decisions)

The following details are not fixed by REQ-06.013 and are resolved by this revision and recorded in the UI description:

- Card contents beyond the task ID and title.
- Column widths.
- Separator styling and weight beyond "bold" and "names the release".
- The shape of the non-drag status control (select, buttons, menu).
- Default expansion state of milestones.
- Empty-column placeholder text.

## Assumptions

- Assumption: The kanban board contract is already specified by REQ-06.013 and TASK-101; this ticket only designs the UI for it.
- Assumption: MOCK-0002 is revised in place (no renumbering) per the global sequential-id rule and the TASK-101 handoff; no new MOCK id is allocated.
- Assumption: The mockup stays `draft` until the user explicitly approves it; only then may the registry be updated to `approved` and TASK-103 implementation begin.
- Assumption: Existing MOCK ids are not renumbered; the revision of MOCK-0002 does not supersede or deprecate other mockups.
- Assumption: The shell, record views, and edit flows are reused unchanged except where the board changes them.
- Assumption: No new dependencies, crates, ADRs, or NFRs are introduced.

## Open questions

- **OQ-1 (recommended default: board replaces the flat task list)** — Whether any flat task listing should remain reachable (for example `?type=task` rendering rows) alongside the board. Recommended default: the board replaces the flat task list in the combined Backlog view. Subject to product confirmation during this review.
- **OQ-2 (recommended default: status change sends the full current record with status changed)** — Whether the board's status change is submitted as a form POST carrying the unchanged title and the new status, or via the JSON API. Either way only the status value changes. Subject to implementation choice in TASK-103.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06.013; REQ-06.002; REQ-06.003; REQ-01.002; REQ-02.001; REQ-02.002; NFR-01.002; NFR-05.001 |
| Business rule | BR-004; BR-005; BR-016 |
| ADR | ADR-0005; ADR-0006; ADR-0010; ADR-0012; ADR-0013 |
| Ticket | TASK-101 (dependency); TASK-102 (this ticket); TASK-103 (downstream implementation) |
| Specification | `docs/specifications/browser-tasks-milestones-and-releases.md` (section 3); `docs/specifications/ui/browser-backlog-records-ui.md` (section 4); `docs/specifications/browser-spec-revisions-task-101.md` |
| UI mockups | MOCK-0002 (approved baseline; revision and re-approval required under this ticket) |
