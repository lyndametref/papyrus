# UI Description — Papyrus Local Browser (Backlog Records)

**Ticket:** TASK-11
**Specification:** [`../browser-tasks-milestones-and-releases.md`](../browser-tasks-milestones-and-releases.md) — sections 3–9 and UI design handoff
**Mockup package:** [`mockups/MOCK-0002-browser-backlog-records-review.html`](mockups/MOCK-0002-browser-backlog-records-review.html) — one self-contained review package (review landing, design deck, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-04 (TASK-11).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to active task, milestone, and release records, reusing the approved MOCK-0001 shell, revised per user feedback: a single combined backlog view, kanban layout, framed tags, bullet-list acceptance criteria, tag-based dependency editing, and a searchable milestone dropdown.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=task\|milestone\|release` | Backlog (combined) | `backlog`, `list-empty-project`, `list-empty-type`, `list-long` (plus baseline `list-business`, `list-nonfunctional`) |
| `GET /doc/{id}` | Record view | `view-task`, `view-milestone`, `view-release`, `view-task-unresolved`, `view-task-deps` |
| `GET /doc/{id}/edit` | Edit form | `edit-task`, `edit-milestone`, `edit-release`, `edit-task-dirty`, `edit-task-errors`, `edit-task-conflict`, `view-after-save` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-320`, `responsive-edit-320` |

## 4. Backlog view

- Single view named "Backlog" combining tasks, milestones, and releases.
- Releases are shown as separators between milestones.
- Milestones are expandable/collapsible; by default only the first milestone that has a non-archived task is expanded.
- Tasks are organised as a kanban board with one column per task status.
- A task's status is changed by dragging its card into another status column (drag-and-drop); each card carries a drag handle and each column is a drop target, with a helper note stating the interaction.
- Each task card shows ID + title link, assignee, and framed-tag labels.
- Empty states and long-title wrapping as in MOCK-0001.

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

List: sidebar links, task card links (including drag handle for status change), milestone expand/collapse. View: breadcrumb links, Edit, milestone/dependency links. Edit: inputs, textareas, select, search boxes, tag remove/add buttons, Save, Cancel, hidden version. Error: recovery link.

## 9. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0002 | scenario | [`mockups/MOCK-0002-browser-backlog-records-review.html`](mockups/MOCK-0002-browser-backlog-records-review.html) | Review landing, design deck, 26-view web prototype |

## 10. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0002 | `approved` — explicitly approved by the user on 2026-11-04 (TASK-11). |
