# UI Description — Papyrus Local Browser (Glossary Terms)

**Ticket:** TASK-32
**Specification:** [`../glossary-terms-management.md`](../glossary-terms-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0004-browser-glossary-terms-review.html`](mockups/MOCK-0004-browser-glossary-terms-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-06 (TASK-32).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to glossary terms, reusing the approved MOCK-0001/MOCK-0002/MOCK-0003 shell. Adds a fifth sidebar entry "Glossary" and views for listing, viewing, and editing glossary terms. Deprecated terms are excluded from lists and navigation (REQ-05.007, BR-011). Glossary terms have two fields: term (required) and definition (optional, multi-line).

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle. Prior entries (Business requirements, Non-functional requirements, Backlog, Decisions) are non-clickable in this extension package; only Glossary has generated views.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=glossary` | Glossary list | `list-glossary`, `list-glossary-empty`, `list-glossary-long` |
| `GET /doc/{id}` | Glossary term view | `view-glossary`, `view-glossary-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-glossary`, `edit-glossary-dirty`, `edit-glossary-errors`, `edit-glossary-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-glossary-320`, `responsive-edit-glossary-320` |

## 4. Glossary list

- One section (`h2`) titled "Glossary", showing the term count, followed by one row per term in ascending ID order.
- Each row's primary link text is `ID + term` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the term's definition (description) beneath the primary link; terms with an empty definition show no preview. The preview is plain text (first paragraph only, not the full definition).
- Empty state and long-title wrapping as in MOCK-0001.
- Deprecated terms are never listed.

## 5. Glossary term view

- Metadata block: ID (read-only), record form (`Glossary term`), category (`Glossary`).
- Term is the page title (`h1`); definition shown in a section with `white-space: pre-wrap`.
- Breadcrumb: `Glossary › {id}` (Glossary links to `/?type=glossary`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `term` (required, mapped to title) and `definition` (optional, multi-line textarea).
- Term: trimmed, 1–200 Unicode scalar values, no control characters.
- Definition: trimmed, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control characters rejected; empty clears; a line whose first two characters are `## ` is rejected.
- ID, record form, and category are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/deprecated, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit. Edit: term input, definition textarea, Save, Cancel, hidden version. Error: recovery link.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): Glossary entry is placed after Decisions (end of the document-type list).
- OQ-2 (definition field): multi-line textarea, maximum 20000 characters.
- OQ-3 (definition rendering): `white-space: pre-wrap` with `overflow-wrap: break-word`.

## 10. MOCK register

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0004 | scenario | [`mockups/MOCK-0004-browser-glossary-terms-review.html`](mockups/MOCK-0004-browser-glossary-terms-review.html) | Review landing, glossary views (17 views) |

## 11. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0004 | `approved` — explicitly approved by the user on 2026-11-06 (TASK-32). |