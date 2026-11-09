# UI Description — Papyrus Local Browser (Front-Matter Metadata and Managed-Artifact Lifecycle)

**Ticket:** TASK-66
**Specification:** [`../browser-requirements.md`](../browser-requirements.md) baseline; [`../browser-tasks-milestones-and-releases.md`](../browser-tasks-milestones-and-releases.md); [`../browser-technical-specifications.md`](../browser-technical-specifications.md); requirements `REQ-06.003`, `REQ-06.004`, `REQ-05.010`, `REQ-05.011`
**Mockup package:** [`mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html`](mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-09.

## 1. Purpose and scope

Layout, states, elements, and interactions for (A) showing front-matter metadata in the existing list/detail/edit views for managed record types, and (B) a new managed-artifact lifecycle page. Reuses the approved MOCK-0001 shell and prior per-type mockups (MOCK-0002..MOCK-0009). The browser keeps its current editable field set; `created_date` and `updated_date` are read-only. Metadata edits target front matter, not body sections. A save writes the new value into front matter, preserves the other front-matter fields and the rest of the file, refreshes `updated_date`, and never copies metadata into the body.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Business Rules, Audit Records, Technical Specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups, Managed artifacts &amp; lifecycles), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. The 320 px sidebar collapses behind an accessible toggle.

Prior entries keep their existing pattern: non-clickable except the type being demonstrated. The new entry "Managed artifacts &amp; lifecycles" is clickable and opens the lifecycle page.

The review file inlines the complete wireframe stylesheet and loads no external stylesheet, script, font, image, or network resource.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=backlog` (tasks) | Task list/detail/edit | `list-tasks`, `view-task`, `view-task-after-save`, `edit-task`, `edit-task-dirty`, `edit-task-errors`, `edit-task-conflict` |
| `GET /?type=backlog` (milestones) | Milestone list/detail | `list-milestones`, `view-milestone` |
| `GET /?type=backlog` (releases) | Release list/detail | `list-releases`, `view-release` |
| `GET /doc/{id}` | Document record view | `view-tech-spec` |
| `GET /lifecycle` (or `?type=lifecycle`) | Lifecycle page | `lifecycle` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-320`, `responsive-edit-320` |

## 4. Metadata display

- List rows show at least id, title, created_date, updated_date, labels, references, and type-specific keys (e.g. task status, milestone release) in a metadata line beneath the primary link.
- Detail view shows a metadata block (`dl`) with: Record ID, Title, Created date, Updated date, Labels, References, plus type-specific keys (Status, Assignee, Milestone, Dependencies, Documentation, Type, Ordinal for tasks; Release for milestones; none extra for releases/technical specifications except general keys).
- Created date and updated date are rendered read-only in detail and edit (clearly marked "(read-only)"); timestamps are ISO-8601 UTC strings as stored.
- Labels and references are shown as tags in view and as a tag add/remove interaction in edit (reusing MOCK-0002).
- Edit form keeps the current editable field set; timestamps are rendered as disabled inputs. A hidden version field supports optimistic concurrency.

## 5. Task edit states

Pristine, dirty ("Unsaved changes"), validation-error (422) with error summary linking to fields, conflict (409) with reload/view actions and a read-only block of unsaved values. 320 px variant collapses the sidebar and stacks form actions.

## 6. Lifecycle page

- One section (`h2`) per fixed record type: Tasks, Milestones, Releases, Decisions, Business requirements, Non-functional requirements, Glossary terms, UI specifications, Technical specifications, UI mockups, Business rules, Audit records.
- Task section states/transitions reflect configured statuses (sample: To Do &rarr; In Progress &rarr; Done; note they come from configuration).
- Other sections use documented lifecycles (Draft &rarr; Approved &rarr; Deprecated/Restored, etc.).
- Backlog.md docs, drafts, and project configuration files are excluded (stated on the page).
- Each section: state names + transitions + one-line description per state (OQ-6).

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown, 404 deprecated, 400, 405, 415, 500, each with a recovery link. 320 px list and edit form: sidebar toggle first, full-width rows/fields, stacked form actions, annotation legend. Focus order follows document order; every control has a visible label; works without JavaScript.

## 8. Active elements

List: sidebar entry, row links. View: breadcrumb links, Edit. Edit: title input, description textarea, status select, milestone input, label/reference tag add-remove, Save changes, Cancel, hidden version. Lifecycle: sidebar entry. 320 px: sidebar toggle.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar label/placement): "Managed artifacts &amp; lifecycles" placed at the end of the document-type list. (Recommended default; pending user confirmation.)
- OQ-2 (route): `lifecycle` (or `?type=lifecycle`) as a distinct route. (Recommended default.)
- OQ-3 (scope): metadata demonstrated on a representative subset (task, milestone, release, technical specification); the metadata block applies to every managed record type. (Main scope question.)
- OQ-4 (timestamp format): ISO-8601 UTC string as stored, read-only, possibly with human-readable secondary.
- OQ-5 (labels/references): tags in view; tag add/remove interaction in edit (reuse MOCK-0002).
- OQ-6 (lifecycle depth): state names + transitions + one-line description per state.

## 10. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0010 | [`mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html`](mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html) | web | 21 happy-path, recovery, error, and 320 px views | `approved` |

MOCK-0010 depends on the approved shell of MOCK-0001 and the per-type mockups MOCK-0002..MOCK-0009; it does not supersede them. MOCK-0001..MOCK-0009 are unchanged.

## 11. Assumptions

- Metadata is read from YAML front matter (REQ-05.010/011); the browser keeps its current editable field set and writes edits back to front matter.
- A save refreshes `updated_date`, preserves other front-matter keys and the body, and never copies metadata into the body.
- Example records are illustrative.
- The version token is a hidden form field; no merge UI is drawn.

## 12. Approval status

| Item | Status |
| --- | --- |
| MOCK-0010 | `approved` &mdash; explicitly approved by the user on 2026-11-09. |
| This UI description | `approved` |
| Registry row | `approved` |

Approval received from the user on 2026-11-09. MOCK-0010 is marked approved per TASK-66 AC #5.