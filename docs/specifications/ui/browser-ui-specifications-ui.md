# UI Description — Papyrus Local Browser (UI Specifications)

**Ticket:** TASK-36
**Specification:** [`../ui-specifications-management.md`](../ui-specifications-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0005-browser-ui-specifications-review.html`](mockups/MOCK-0005-browser-ui-specifications-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-06 (UTC).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to UI specifications, reusing the approved MOCK-0001/MOCK-0002/MOCK-0003/MOCK-0004 shell. Adds a sixth sidebar entry "UI Specifications" and views for listing, viewing, and editing UI specifications. Deprecated specifications are excluded from lists and navigation (REQ-05.007, BR-012). UI specifications have three fields: title (required, from heading), view (required), and description (optional, multi-line).

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle. Prior entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary) are non-clickable in this extension package; only UI Specifications has generated views.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=ui-spec` | UI specifications list | `list-ui-spec`, `list-ui-spec-empty`, `list-ui-spec-long` |
| `GET /doc/{id}` | UI specification view | `view-ui-spec`, `view-ui-spec-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-ui-spec`, `edit-ui-spec-dirty`, `edit-ui-spec-errors`, `edit-ui-spec-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-ui-spec-320`, `responsive-edit-ui-spec-320` |

## 4. UI specifications list

- One section (`h2`) titled "UI Specifications", showing the record count, followed by one row per specification in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the specification's description beneath the primary link; specifications with an empty description show no preview. The preview is plain text (first paragraph only, not the full description).
- Empty state and long-title wrapping as in MOCK-0001.
- Deprecated specifications are never listed.

## 5. UI specification view

- Metadata block: ID (read-only), record form (`UI specification`), category (`UI Specifications`).
- Title is the page title (`h1`); View and Description shown in sections with `white-space: pre-wrap` (or `(none)`).
- Breadcrumb: `UI Specifications › {id}` (UI Specifications links to `/?type=ui-spec`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `title` (required, mapped to heading), `view` (required), `description` (optional, multi-line textarea).
- Title: trimmed, 1–200 Unicode scalar values, no control characters. View: single-line, same limits. Description: trimmed, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control characters rejected; empty clears; a line whose first two characters are `## ` is rejected.
- ID, record form, and category are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/deprecated, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit. Edit: title input, view input, description textarea, Save, Cancel, hidden version. Error: recovery link.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): UI Specifications entry is placed after Glossary (end of the document-type list).
- OQ-2 (editable fields): title, view, and description are editable; ID is not.
- OQ-3 (description rendering): `white-space: pre-wrap` with `overflow-wrap: break-word`.

## 10. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0005 | scenario | [`mockups/MOCK-0005-browser-ui-specifications-review.html`](mockups/MOCK-0005-browser-ui-specifications-review.html) | Review landing, UI specifications views (17 views) |

## 11. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0005 | `approved` — explicitly approved by the user on 2026-11-06 (UTC). |
