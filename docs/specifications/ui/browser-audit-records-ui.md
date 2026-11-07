# UI Description — Papyrus Local Browser (Audit Records)

**Ticket:** TASK-49
**Specification:** [`../audit-records-management.md`](../audit-records-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0008-audit-records-review.html`](mockups/MOCK-0008-audit-records-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-07 (UTC).
**Approval note:** MOCK-0008 explicitly re-approved by the user on 2026-11-07 (UTC) (TASK-49), covering both revisions: audit type shown in the card at the beginning of the view (badge next to the title/ID plus an "Audit type" row in the metadata block); Content heading removed. Reuses the approved shell.

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to audit records, reusing the approved MOCK-0001/MOCK-0002/MOCK-0003/MOCK-0004/MOCK-0005/MOCK-0006/MOCK-0007 shell. Adds a sidebar entry "Audit Records" and views for listing, viewing, and editing audit records. Deprecated audit records are excluded from lists and navigation (REQ-05.006, REQ-05.007). Audit records have three fields: title (required), audit type (select: security/licence/architecture, default security), and content (optional, multi-line). ID, record form, and category are read-only.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Business Rules, Audit Records, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle. Prior entries (Business requirements, Business Rules, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups) are non-clickable in this extension package; only Audit Records has generated views.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=audit-record` | Audit records list | `list-audit-records`, `list-audit-records-empty`, `list-audit-records-long` |
| `GET /doc/{id}` | Audit record view | `view-audit-record`, `view-audit-record-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-audit-record`, `edit-audit-record-dirty`, `edit-audit-record-errors`, `edit-audit-record-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-audit-records-320`, `responsive-edit-audit-records-320` |

## 4. Audit records list

- One section (`h2`) titled "Audit Records", showing the record count, followed by one row per record in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the record's content beneath the primary link; records with empty content show no preview. The preview is plain text (first paragraph only, not the full content).
- Empty state and long-title wrapping as in MOCK-0001.
- Deprecated audit records are never listed.

## 5. Audit record view

- Metadata block: ID (read-only), record form (`Audit record`), category (`Audit Records`).
- Title is the page title (`h1`); the audit type is shown at the beginning of the view in the metadata card — as a badge next to the record ID/title and as an "Audit type" row in the metadata block (security/licence/architecture). The content is rendered as body text with `white-space: pre-wrap` (or `(none)` when empty), without a "Content" heading.
- Breadcrumb: `Audit Records › {id}` (Audit Records links to `/?type=audit-record`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `title` (required), `audit_type` (select: security/licence/architecture, default security), `content` (optional, multi-line textarea).
- Title: trimmed, 1–200 Unicode scalar values, no control characters.
- Audit type: required, one of security, licence, architecture; defaults to security.
- Content: trimmed, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control characters rejected; empty clears.
- ID, record form, and category are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/deprecated, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit. Edit: title input, audit type select, content textarea, Save, Cancel, hidden version. Error: recovery link.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): Audit Records entry is placed immediately after Business Rules (business-domain grouping); prior entries otherwise unchanged.
- OQ-2 (audit type rendering): resolved — the audit type is shown in the card at the beginning of the view (badge next to the title/ID plus an "Audit type" row in the metadata block), not as a standalone section/chapter; in the edit form it remains a select control defaulting to security. (Resolved design decision, 2026-11-07.)
- OQ-3 (editable fields): title, audit type, and content are editable; ID, record form, and category are not.
