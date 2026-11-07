# UI Description — Papyrus Local Browser (Business Rules)

**Ticket:** TASK-45
**Specification:** [`../business-rules-management.md`](../business-rules-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0007-business-rules-review.html`](mockups/MOCK-0007-business-rules-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-07 (UTC).
**Approval note:** MOCK-0007 explicitly approved by the user on 2026-11-07 (UTC) (TASK-45). Reuses the approved shell; no design changes.

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to business rules, reusing the approved MOCK-0001/MOCK-0002/MOCK-0003/MOCK-0004/MOCK-0005/MOCK-0006 shell. Adds a sidebar entry "Business Rules" and views for listing, viewing, and editing business rules. Deprecated business rules are excluded from lists and navigation (REQ-05.005, REQ-05.007, BR-013). Business rules have five fields: title (required), description (optional, multi-line), rationale (optional, multi-line), applicability (optional, multi-line), and zero or more references to business requirements (requirement IDs).

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Business Rules, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. 320 px sidebar collapses behind an accessible toggle. Prior entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups) are non-clickable in this extension package; only Business Rules has generated views.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=business-rule` | Business rules list | `list-business-rules`, `list-business-rules-empty`, `list-business-rules-long` |
| `GET /doc/{id}` | Business rule view | `view-business-rule`, `view-business-rule-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-business-rule`, `edit-business-rule-dirty`, `edit-business-rule-errors`, `edit-business-rule-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-business-rules-320`, `responsive-edit-business-rules-320` |

## 4. Business rules list

- One section (`h2`) titled "Business Rules", showing the record count, followed by one row per rule in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the rule's description beneath the primary link; rules with an empty description show no preview. The preview is plain text (first paragraph only, not the full description).
- Empty state and long-title wrapping as in MOCK-0001.
- Deprecated business rules are never listed.

## 5. Business rule view

- Metadata block: ID (read-only), record form (`Business rule`), category (`Business Rules`).
- Title is the page title (`h1`); sections Description, Rationale, Applicability, and Requirements (requirement references) shown with `white-space: pre-wrap` (or `(none)` when empty).
- Requirement references are rendered as their IDs (e.g. `REQ-05.005`); in the full browser they link to the corresponding requirement view where available. In this package they are shown as literal IDs because requirement views are out of scope for this review package.
- Breadcrumb: `Business Rules › {id}` (Business Rules links to `/?type=business-rule`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.

## 6. Edit form

- Editable fields: `title` (required), `description` (optional, multi-line textarea), `rationale` (optional, multi-line textarea), `applicability` (optional, multi-line textarea), `requirements` (list of business requirement IDs, tag add/remove interaction per MOCK-0002).
- Title: trimmed, 1–200 Unicode scalar values, no control characters.
- Description/rationale/applicability: trimmed, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control characters rejected; empty clears.
- Requirement references: added one at a time via an input + Add button and removed via per-tag remove buttons (MOCK-0002 list-field pattern); duplicates collapsed.
- ID, record form, and category are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty, validation-error (422), conflict (409), and 320 px states.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown/deprecated, 400, 405, 415, 500. 320 px list and edit form with sidebar toggle and annotation legend.

## 8. Active elements

List: sidebar links, row links. View: breadcrumb links, Edit. Edit: title input, description/rationale/applicability textareas, requirement tag add input + Add button + per-tag remove buttons, Save, Cancel, hidden version. Error: recovery link.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): Business Rules entry is placed immediately after Business requirements (business-domain grouping); prior entries otherwise unchanged.
- OQ-2 (requirement references rendering): rendered as links to requirement views where available; in this review package shown as literal IDs because requirement views are out of scope.
- OQ-3 (editable fields): title, description, rationale, applicability, and requirements are editable; ID, record form, and category are not. Rationale and applicability are editable only via the full form (no dedicated CLI options), consistent with BR-013.
