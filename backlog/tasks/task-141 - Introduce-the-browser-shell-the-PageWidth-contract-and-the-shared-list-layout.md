---
id: TASK-141
title: >-
  Introduce the browser shell, the PageWidth contract, and the shared list
  layout
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
labels:
  - browser
  - shell
  - rust
milestone: m-21
dependencies: []
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.015.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/render.rs
  - src/browser/routes.rs
  - src/browser/templates
priority: high
ordinal: 41540
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.016 requires one shared list layout for every artifact family, and ADR-0028 is Accepted. Today there are two list templates (`list.html`, `document_list.html`) and per-family list builders inside a 4.7k-line `render.rs`; the 1040 px cap and the fixed 210 px sidebar column are hard-coded in `app.css`; and there is no page shell module, so width policy has nowhere to live.

**What.** Introduce the shell, the server-side width contract, and the one shared list layout, and prove the layout on the task family before it spreads. This task is the foundation for TASK-142; it must not leave a second list path behind for the family it migrates.

**In scope.**
- New `src/browser/shell.rs`: document skeleton (head, header, nav, `<main>`, footer), the page width class, and the slot for the nav toggle. `view.html`, `edit.html`, `error.html`, `list.html` render inside it.
- New `src/browser/list.rs` with `ListPlan`, `Column`, and `Row` (including `Row::children` for hierarchical grouped families). Fixed column order for every family: ID, Title, Status, family slot(s), Updated, Actions. Family slots come from declared data (descriptor plus presentation table), never from a template or a `match` on slug.
- Shared list markup carries `data-list-layout="shared"`; action placement is the last cell in the order View, Edit, then any existing family action.
- `PageWidth { Capped, Full }` as a route property, `Capped` by default, rendered as `class="app"`; `Full` renders `class="app app--full"`. No client-side width switching. Assigning `Full` to the board and the mockup class is TASK-144.
- Migrate the task list as the reference implementation and replay its fixtures against the shared layout.
- Capture characterization fixtures for **every** family from the pre-change binary into committed test fixtures: header labels, column order, action cells, row ordering, archived exclusion, and grouped rows for requirements and NFRs. TASK-142 replays them.
- Preserve detail, edit, and error pages and their typed field sets; only list views become uniform (REQ-06.016).

**Out of scope.** Migrating the remaining families and deleting `document_list.html` (TASK-142); the nav toggle, `localStorage`, and the CSS width classes (TASK-144); math and diagram rendering (TASK-145, TASK-146); any new dependency, template engine, or client framework.

**Assumptions.** ADR-0028 decisions 1 to 3 and 6 are the contract. The task list is representative because it has the widest action cell and both an empty and a populated state. No visual redesign happens here: the goal is equivalent output through one layout.

**UI gate.** The task intends no visual change. If refinement turns the shell or the page chrome into a visual change, a UI mockup must be approved through `ui-designer` before merge; the refiner records either the approval or a no-mockup waiver in the task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A shell module renders head, header, nav, main, and footer for every page, and list, view, edit, and error templates render inside it.
- [ ] #2 One shared list layout renders the task list with the fixed order ID, Title, Status, family slots, Updated, Actions, and carries the shared-layout marker.
- [ ] #3 Family variation in the task list comes only from declared descriptor and presentation data, with no per-family list template and no slug match.
- [ ] #4 PageWidth is a server-side route property, Capped by default for every route, rendered as app or app app--full, with no client-side width switching.
- [ ] #5 Characterization fixtures for every family are captured from the pre-change binary and committed for TASK-142 to replay.
- [ ] #6 Detail, edit, and error pages keep their typed fields and render inside the new shell.
- [ ] #7 No new dependency, template engine, or client framework is added, and the existing browser tests pass.
- [ ] #8 Any visual change is either covered by an approved mockup or by a recorded no-mockup waiver.
<!-- AC:END -->
