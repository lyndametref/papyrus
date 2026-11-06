# UI Description — Papyrus Local Browser (UI Mockups)

**Ticket:** TASK-41
**Specification:** [`../ui-mockups-management.md`](../ui-mockups-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10; browser pattern from [`../browser-ui-specifications.md`](../browser-ui-specifications.md)
**Mockup package:** [`mockups/MOCK-0006-browser-ui-mockups-review.html`](mockups/MOCK-0006-browser-ui-mockups-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-06 (UTC).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to UI mockups, reusing the approved MOCK-0001…MOCK-0005 shell. Adds a seventh sidebar entry "UI Mockups" and views for listing, viewing, and editing UI mockups. Deprecated mockups are excluded from lists and navigation (REQ-05.007, REQ-05.008). A mockup record carries: id (`MOCK-###`), title, specs (list of UI specification ids), refs.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle. Prior entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications) are non-clickable in this extension package; only UI Mockups has generated views.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=ui-mockup` | UI mockups list | `list-mockup`, `list-mockup-empty`, `list-mockup-long` |
| `GET /doc/{id}` | UI mockup view | `view-mockup`, `view-mockup-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-mockup`, `edit-mockup-dirty`, `edit-mockup-errors`, `edit-mockup-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-mockup-320`, `responsive-edit-mockup-320` |

## 4. UI mockups list

- One section (`h2`) titled "UI Mockups", showing the record count, followed by one row per mockup in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Empty state and long-title wrapping as in MOCK-0001.
- Deprecated mockups are never listed.

## 5. UI mockup view

- Metadata block: ID (read-only), record form (`UI mockup`), category (`UI Mockups`).
- Title is the page title (`h1`); the persisted visible lines are rendered in sections with `white-space: pre-wrap`: UI specifications, References.
- Breadcrumb: `UI Mockups › {id}` (UI Mockups links to `/?type=ui-mockup`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `title` (required), `specs` (required, list of UI specification ids), `refs` (optional, list of document references). ID is read-only; deprecated/approved state is managed from the command line, not this form.
- Title: trimmed, 1–200 Unicode scalar values, no control characters.
- `specs` and `refs` use the tag add/remove interaction (tag box with remove buttons and an add input), per the MOCK-0002 list-field pattern.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty ("Unsaved changes" indicator), validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown, 404 deprecated (sub-line notes deprecated mockups are hidden), 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit. Edit: title input, specs tag box (add/remove), refs tag box (add/remove), Add buttons, Save, Cancel, hidden version. Error: recovery link.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): UI Mockups entry is placed after UI Specifications (end of the document-type list).
- OQ-2 (editable fields): title, specs, and refs are editable; ID is read-only.
- OQ-3 (list): no status/task/superseded indications are shown.
- OQ-4 (list field interaction): specs and refs use the MOCK-0002 tag add/remove pattern.
- OQ-5 (example data): record ids in the prototype (MOCK-0007, MOCK-0009, UI-001…UI-003) are illustrative wireframe examples, not registry records.

## 10. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0006 | scenario | [`mockups/MOCK-0006-browser-ui-mockups-review.html`](mockups/MOCK-0006-browser-ui-mockups-review.html) | Review landing, UI mockups views (17 views) |

## 11. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0006 | `approved` — explicitly approved by the user on 2026-11-06 (UTC). |
