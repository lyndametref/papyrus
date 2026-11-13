# UI Description — Papyrus Local Browser (Backlog Records)

**Ticket:** TASK-11
**Specification:** [`../browser-tasks-milestones-and-releases.md`](../browser-tasks-milestones-and-releases.md) — sections 3–9 and UI design handoff
**Mockup package:** [`mockups/MOCK-0002-browser-backlog-records-review.html`](mockups/MOCK-0002-browser-backlog-records-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-12 (UTC) (TASK-102). Revision (per-milestone kanban board, REQ-06.013) approved; TASK-103 implementation may proceed.

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to active task, milestone, and release records, reusing the approved MOCK-0001 shell, revised per user feedback: a single combined backlog view, kanban layout, framed tags, bullet-list acceptance criteria, tag-based dependency editing, and a searchable milestone dropdown.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=task\|milestone\|release` | Backlog (combined) | `backlog`, `backlog-status-failed`, `list-empty-project`, `list-empty-type`, `list-long` (plus baseline `list-business`, `list-nonfunctional`) |
| `GET /doc/{id}` | Record view | `view-task`, `view-milestone`, `view-release`, `view-task-unresolved`, `view-task-deps` |
| `GET /doc/{id}/edit` | Edit form | `edit-task`, `edit-milestone`, `edit-release`, `edit-task-dirty`, `edit-task-errors`, `edit-task-conflict`, `view-after-save` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-320`, `responsive-edit-320` |

## 4. Backlog view

- Single view named "Backlog" combining tasks, milestones, and releases.
- Milestones are expandable and collapsible sections (existing behavior, retained); by default only the first milestone that has a non-archived task is expanded.
- Expanding a milestone renders that milestone's tasks as a kanban board: one column per configured task status (REQ-01.002), laid out horizontally within the milestone, in configured status order, with empty-status columns still visible.
- The board is nested inside the milestone section; milestones remain the grouping unit of the view.
- Milestones assigned to a release appear under that release; milestones assigned to no release are grouped together before the first release. Each release boundary is a bold separator that names the release and does not rely on colour alone. Release ordering follows ascending release ID; milestones within a release follow ascending milestone ID; the no-release group precedes all releases.
- Tasks that belong to no milestone appear in their own expandable section, placed after all milestone sections (including all release groups); cards in the section are ordered by ascending task ID and each shows the task ID and title and links to the task record view.
- Cards are ordered by ascending task ID within a column and show at least the task ID and title, linking to the task record view (`/doc/{id}`). Resolved presentation details (TASK-102): cards also show assignee and labels (as framed tags) beneath the ID+title link; columns are equal-width thirds (one per configured status) within a milestone; the release separator is bold with a 2 px bottom border and names the release; the non-drag status control is a compact select dropdown on the card listing all configured statuses with the current status selected; by default only the first milestone that has a non-archived task is expanded (others collapsed); empty columns show the placeholder text "No tasks in this status."
- **Refined under TASK-103.** The status select is paired with a visible `Change status` submit button and sits inside a plain `POST /doc/{id}` form, so the control works with JavaScript disabled and from the keyboard. The script does not auto-submit on select change: a status change is submitted by the button or by dropping the card. Drag-and-drop is script-only enhancement; without the script the card and its form remain fully usable. The board renders an alert with `role="alert"` above the groups when a status change cannot be persisted, and the card stays in its stored column. A task whose stored status is not configured appears in a trailing `Unconfigured status` column. Tasks whose milestone is absent or unresolved appear in an expandable `Tasks without a milestone` section after all milestone sections. After a successful change the board returns with the card's milestone expanded.
- A task's status is changed by dragging its card into another status column, or with an equivalent status control on the card that does not require dragging. Either action persists the status immediately to the task record through the existing browser write contract (ADR-0006 version token, ADR-0013 single write path); the board is not a staging area with a separate save step. The change alters only the status value and preserves every other field of the task. A change that cannot be persisted leaves the card in its original column and reports the failure to the user; after a successful change the value is still shown after the view is reloaded.
- Archived tasks and archived milestones are not shown on the board.
- Empty states and long-title wrapping as in MOCK-0001.
- Presentation details resolved by the MOCK-0002 revision (TASK-102) and recorded above: card contents beyond the task ID and title (assignee + labels); column widths (equal thirds); separator styling (bold, 2 px bottom border, names the release); non-drag status control shape (compact select dropdown); default expansion state (first milestone with tasks expanded, others collapsed); empty-column placeholder text ("No tasks in this status.").

## 5. Record views

- Task: metadata block (ID, form, category, status, milestone link-or-text, assignee, labels as framed tags, dependencies links both directions (depends on / depended on by), acceptance criteria as bullet list, comments), description, breadcrumb `Backlog › ID`, single Edit.
- Milestone: metadata (ID, form, category, release), description, breadcrumb.
- Release: metadata, description, breadcrumb.

## 6. Edit forms

- Task: title, description, status, milestone (searchable dropdown), assignee, labels (add/remove tag workflow with search box, same as dependencies), dependencies (tag-adding workflow with search), acceptance criteria (add/remove lines); milestone/release fields as applicable.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Data-entry fields use the full width of the middle part.
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/archived, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, task card links, milestone expand/collapse, drag handle for status change, per-card status select (non-drag). View: breadcrumb links, Edit, milestone/dependency links. Edit: inputs, textareas, select, search boxes, tag remove/add buttons, Save, Cancel, hidden version. Error: recovery link.

## 9. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0002 | scenario | [`mockups/MOCK-0002-browser-backlog-records-review.html`](mockups/MOCK-0002-browser-backlog-records-review.html) | Review landing, 27-view web prototype (kanban board revision) |

## 10. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0002 | `approved` — explicitly approved by the user on 2026-11-12 (UTC) (TASK-102). Revision (per-milestone kanban board, REQ-06.013) approved; baseline approval 2026-11-04 (TASK-11) superseded by this revision. |

## Traceability

| Artifact | Reference |
| --- | --- |
| Requirement | REQ-06.013; REQ-06.002; REQ-06.003; REQ-01.002; REQ-02.001; REQ-02.002 |
| ADR | ADR-0006; ADR-0013 |
| Ticket | TASK-11; TASK-101; TASK-102 (mockup revision); TASK-103 (implementation) |
| UI mockups | MOCK-0002 (approved 2026-11-12 UTC under TASK-102) |
