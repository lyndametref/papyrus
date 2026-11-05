# UI Description — Papyrus Local Browser (Decision Records)

**Ticket:** TASK-15
**Specification:** [`../browser-decision-records.md`](../browser-decision-records.md) — sections 3–9 and UI design handoff
**Mockup package:** [`mockups/MOCK-0003-browser-decision-records-review.html`](mockups/MOCK-0003-browser-decision-records-review.html) — one self-contained review package (review landing, design deck, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user (created 2026-11-05, TASK-15).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to decision records, reusing the approved MOCK-0001/MOCK-0002 shell. Adds a fourth sidebar entry "Decisions" and views for listing, viewing, and editing decision records (ADR/BDR and configured types), including supersession relationships and Nygard status vocabulary.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog, Decisions), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=decision` | Decision list | `list-decisions`, `list-decisions-empty`, `list-decisions-long` |
| `GET /doc/{id}` | Decision view | `view-decision`, `view-decision-superseded`, `view-decision-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-decision`, `edit-decision-dirty`, `edit-decision-errors`, `edit-decision-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-decisions-320`, `responsive-edit-decisions-320` |

## 4. Decision list

- One section (`h2`) titled with the category name, showing the record count, followed by one row per record in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Decision rows additionally show, as read-only text in the row: status and decision type.
- Empty state and long-title wrapping as in MOCK-0001.
- Archived decisions are not listed.

## 5. Decision view

- Metadata block: ID (read-only), record form (`Decision`), category (`Decision`), decision type, status.
- Decision text, context, consequences (each shown or `(none)`).
- Supersession: when the decision is superseded, show `Superseded by <id>` (link to the superseding decision when it resolves); otherwise `(none)`.
- Requirement references appearing inside field values are displayed as literal text, never auto-linked.
- Breadcrumb: `Decisions › {id}` (Decisions links to `/?type=decision`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `title` (required), `status` (select among valid transitions for the current status), `decision`, `context`, `consequences`, `superseded-by` (optional ID or empty).
- Decision type and ID are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/archived, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit, superseding-decision link. Edit: inputs, textareas, select, Save, Cancel, hidden version. Error: recovery link.

## 9. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0003 | scenario | [`mockups/MOCK-0003-browser-decision-records-review.html`](mockups/MOCK-0003-browser-decision-records-review.html) | Review landing, design deck, decision-record views (19 views) |

## 10. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0003 | `approved` — explicitly approved by the user (created 2026-11-05, TASK-15). |