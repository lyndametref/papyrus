# UI Description — Papyrus Local Browser (Final Cross-Type Navigation and Inventory)

**Ticket:** TASK-60
**Specification:** [`../browser-requirements.md`](../browser-requirements.md) baseline; [`../browser-tasks-milestones-and-releases.md`](../browser-tasks-milestones-and-releases.md); [`../browser-technical-specifications.md`](../browser-technical-specifications.md); [`../browser-managed-artifacts-and-lifecycles.md`](../browser-managed-artifacts-and-lifecycles.md); [`../browser-agent-skills-ui.md`](../browser-agent-skills-ui.md) (as spec); requirements `REQ-06.002`, `REQ-06.003`, `REQ-06.004`, `REQ-05.010`, `REQ-05.011`; `NFR-01.002`, `NFR-01.003`, `NFR-02.002`, `NFR-04.001`, `NFR-04.002`, `NFR-05.001`; `ADR-0007`, `ADR-0009`
**Mockup package:** [`mockups/MOCK-0012-browser-final-cross-type-review.html`](mockups/MOCK-0012-browser-final-cross-type-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-10 (UTC) (TASK-60).

## 1. Purpose and scope

Layout, states, elements, and interactions for the final cross-type browser: one navigation/inventory that exposes all record types consistently, demonstrating front-matter metadata on a representative subset (task, milestone, release, technical specification), the managed-artifact lifecycle page, and the agent skills page. Reuses the approved MOCK-0001 shell and the approved patterns of MOCK-0010 and MOCK-0011; does not redraw prior per-type CRUD in full. The browser keeps its current editable field set; `created_date` and `updated_date` are read-only. Metadata edits target front matter, not body sections.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries in final order (Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts &amp; lifecycles, Agent skills &amp; workflows), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. The 320 CSS px sidebar collapses behind an accessible toggle. CSP/escaping/accessibility baseline as in MOCK-0001.

Represented types (Technical specifications, Backlog, Managed artifacts &amp; lifecycles, Agent skills &amp; workflows) are clickable; other entries remain non-clickable context, consistent with prior mockups. Deprecated records are not reachable from navigation (OQ-4).

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /` (inventory) | Cross-type inventory hub | `inventory` |
| `GET /?type=backlog` (tasks) | Task list/detail/edit | `list-tasks`, `view-task`, `edit-task` |
| `GET /?type=backlog` (milestones) | Milestone list/detail | `list-milestones`, `view-milestone` |
| `GET /?type=backlog` (releases) | Release list/detail | `list-releases`, `view-release` |
| `GET /doc/{id}` | Technical specification view | `view-tech-spec` |
| `GET /lifecycle` (or `?type=lifecycle`) | Lifecycle page | `lifecycle` |
| `GET /skills` (alias `/?type=skills`) | Agent skills page | `skills` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list, edit, lifecycle, skills | `responsive-list-320`, `responsive-edit-320`, `responsive-lifecycle-320`, `responsive-skills-320` |

## 4. Inventory hub

A single view listing all record types as cards with illustrative counts. Represented types link to their representative view; other types state they are covered by prior mockups (MOCK-0001..MOCK-0009) and are not clickable here. Serves as the navigation entry point (OQ-2).

## 5. Metadata display

- List rows show at least id, title, created_date, updated_date, labels, references, and type-specific keys (e.g. task status, milestone release) in a metadata line beneath the primary link.
- Detail view shows a metadata block (`dl`) with: Record ID, Title, Created date, Updated date, Labels, References, plus type-specific keys (Status, Assignee, Milestone, Dependencies, Documentation for tasks; Release for milestones; none extra for releases/technical specifications except general keys).
- Created date and updated date are rendered read-only in detail and edit (clearly marked "(read-only)"); timestamps are ISO-8601 UTC strings as stored.
- Labels and references are shown as tags in view and as a tag add/remove interaction in edit (reusing MOCK-0002).
- Edit form keeps the current editable field set; timestamps are rendered as disabled inputs. A hidden version field supports optimistic concurrency.

## 6. Task edit states

Pristine edit form with read-only timestamps. (Dirty, validation-error, and conflict states are covered in detail by MOCK-0010; only the baseline edit form is shown here as minimal context.)

## 7. Lifecycle page

- One section (`h2`) per fixed record type: Tasks, Milestones, Releases, Decisions, Business requirements, Non-functional requirements, Glossary terms, UI specifications, Technical specifications, UI mockups, Business rules, Audit records.
- Task section states/transitions reflect configured statuses (sample: To Do &rarr; In Progress &rarr; Done; note they come from configuration).
- Other sections use documented lifecycles (Draft &rarr; Approved &rarr; Deprecated/Restored, etc.).
- Backlog.md docs, drafts, and project configuration files are excluded (stated on the page).
- Each section: state names + transitions + one-line description per state.

## 8. Agent skills page

Reuses MOCK-0011: read-only, configuration-parameterized page with a "Configuration in effect" block and grouped, copyable snippet blocks. Route: `GET /skills` (alias `GET /?type=skills`).

## 9. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown, 404 deprecated, 400, 405, 415, 500, each with a recovery link to the inventory. 320 CSS px: list, edit, lifecycle, and skills views; sidebar toggle first, full-width rows/fields, stacked form actions, annotation legend. Focus order follows document order; every control has a visible label; works without JavaScript.

## 10. Active elements

- Inventory: type cards/links for represented types.
- Sidebar entry links for represented types (active state + `aria-current`).
- List: sidebar entry, row links, Edit.
- View: breadcrumb links, Edit.
- Edit: title input, description textarea, status select, milestone input, label/reference tag add-remove, Save changes, Cancel, hidden version, read-only timestamps.
- Lifecycle: sidebar entry.
- Skills: sidebar entry, Copy buttons.
- 320 px: sidebar toggle.
- Error pages: recovery link.

## 11. Open questions (recommended defaults)

- OQ-1 (scope of views): navigation shell + representative views + cross-type consistency checks, rather than full per-type CRUD (already covered by prior mockups). (Recommended default.)
- OQ-2 (inventory/landing view): yes, a single inventory/landing view listing all record types with counts. (Recommended default.)
- OQ-3 (sidebar ordering): as established in MOCK-0010/0011 (final order listed in section 2). (Recommended default.)
- OQ-4 (deprecated records reachable): no, consistent with prior mockups. (Recommended default.)
- OQ-5 (responsive 320 px coverage): list + edit + lifecycle + skills. (Recommended default.)

## 12. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0012 | [`mockups/MOCK-0012-browser-final-cross-type-review.html`](mockups/MOCK-0012-browser-final-cross-type-review.html) | web | 21 happy-path, recovery, error, and 320 px views | `approved` |

MOCK-0012 depends on the approved shell of MOCK-0001 and on MOCK-0010 (front-matter metadata and lifecycle) and MOCK-0011 (agent skills page); it does not supersede them. MOCK-0001..MOCK-0011 are unchanged.

## 13. Assumptions

- Metadata is read from YAML front matter (REQ-05.010/011, ADR-0007, ADR-0009); the browser keeps its current editable field set and writes edits back to front matter.
- A save refreshes `updated_date`, preserves other front-matter keys and the body, and never copies metadata into the body.
- Example records are illustrative.
- The version token is a hidden form field; no merge UI is drawn.
- Counts in the inventory are illustrative.

## 14. Approval status

| Item | Status |
| --- | --- |
| MOCK-0012 | `approved` &mdash; explicitly approved by the user on 2026-11-10 (UTC) (TASK-60). |
| This UI description | `approved` |
| Registry row | `approved` |