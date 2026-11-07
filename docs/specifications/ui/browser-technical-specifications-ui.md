# UI Description — Papyrus Local Browser (Technical Specifications)

**Ticket:** TASK-58
**Specification:** [`../technical-specifications-management.md`](../technical-specifications-management.md); [`../search-technical-specifications.md`](../search-technical-specifications.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0009-browser-technical-specifications-review.html`](mockups/MOCK-0009-browser-technical-specifications-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-07 (UTC) (TASK-58).
**Approval note:** Created by @ui-designer on 2026-11-07 (UTC) for TASK-58, reusing the approved MOCK-0001/MOCK-0008 shell. MOCK-0009 was explicitly approved by the user on 2026-11-07 (UTC) for TASK-58.

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to technical specifications, reusing the approved MOCK-0001/MOCK-0002/MOCK-0003/MOCK-0004/MOCK-0005/MOCK-0006/MOCK-0007/MOCK-0008 shell. Adds a sidebar entry "Technical Specifications" and views for listing, viewing, and editing technical specifications. The browser is read-and-edit only: creating, deprecating, and restoring stay on the command line (BR-017). Deprecated technical specifications are excluded from lists, counts, and navigation and are not reachable by URL (REQ-05.007, BR-017).

Technical specifications have two fields: title (required) and description (required, multi-line). ID, record form, and category are read-only.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Business Rules, Audit Records, Technical Specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. The 320 px sidebar collapses behind an accessible toggle.

Prior entries (Business requirements, Business Rules, Audit Records, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups) are non-clickable in this extension package; only Technical Specifications has generated views.

The review file inlines the complete wireframe stylesheet and loads no external stylesheet, script, font, image, or network resource.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=tech-spec` | Technical specifications list | `list-technical-specifications`, `list-technical-specifications-empty`, `list-technical-specifications-long` |
| `GET /doc/{id}` | Technical specification view | `view-technical-specification`, `view-technical-specification-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-technical-specification`, `edit-technical-specification-dirty`, `edit-technical-specification-errors`, `edit-technical-specification-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-technical-specifications-320`, `responsive-edit-technical-specifications-320` |

## 4. Technical specifications list

- One section (`h2`) titled "Technical Specifications", showing the record count, followed by one row per record in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the description beneath the primary link; records with an empty description show no preview. The preview is plain text (first paragraph only, not the full description).
- Empty state and long-token wrapping as in MOCK-0001/MOCK-0008: an unbreakable title token wraps inside the row without horizontal scrolling.
- Deprecated technical specifications are never listed or counted.

## 5. Technical specification view

- Metadata block: ID (read-only), record form (`Technical specification`), category (`Technical Specifications`).
- Title is the page title (`h1`); the description is rendered as body text with `white-space: pre-wrap`, preserving line breaks and blank lines. No additional field heading is drawn.
- Breadcrumb: `Technical Specifications › {id}` (Technical Specifications links to `/?type=tech-spec`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.
- A deprecated ID renders the 404 page instead of the view (see section 7).

## 6. Edit form

- Editable fields: `title` (required, single-line) and `description` (required, multi-line textarea rendered with `white-space: pre-wrap`).
- Title: trimmed; empty values and control characters are rejected; no length rule beyond the documented sources is drawn.
- Description: required; multi-line, LF-preserving; line breaks are kept when saved; empty values are rejected.
- ID, record form, and category are read-only.
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty ("Unsaved changes"), validation-error (422), conflict (409), and 320 px states.
- Validation error state: error summary (`role="alert"`) listing "Title is required." and "Description is required."; both messages link to their fields; typed values are kept.
- Conflict state: 409 notice, "Reload the form" and "View current record" actions, and a read-only block listing the unsaved values.
- Save changes → record view; Cancel → record view without saving.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001/MOCK-0008: 404 unknown, 404 deprecated, 400, 405, 415, 500, each with a single recovery link back to the technical specifications list. The 404 deprecated page is visually identical to 404 unknown; its help text states that deprecated technical specifications are not listed and cannot be opened, and that the same page is used for an unknown address.

320 px list and edit form: sidebar toggle first, full-width rows/fields, stacked form actions, and an annotation legend drawn inside the narrow frame. Focus order follows document order; every control has a visible label; the designed interface works without JavaScript.

## 8. Active elements

List: sidebar entry, row links. View: breadcrumb links, Edit. Edit: title input, description textarea, Save changes, Cancel, hidden version. Error: recovery link. 320 px: sidebar toggle.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): Technical Specifications is placed immediately after Audit Records; all other entries keep their MOCK-0008 order.
- OQ-2 (category value): `type=tech-spec` selects the technical specifications category; any other unknown value remains a 400 per the baseline.
- OQ-3 (list preview): list rows show the first paragraph of the description beneath ID + title, consistent with MOCK-0008.
- OQ-4 (field rules): only rules stated in `technical-specifications-management.md` and BR-017 are drawn; no length limits are invented. Title and description are trimmed, required, and reject control characters.
- OQ-5 (references): technical specifications may mention NFR or task IDs as literal text in title or description; the browser exposes no structured reference fields.

## 10. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0009 | [`mockups/MOCK-0009-browser-technical-specifications-review.html`](mockups/MOCK-0009-browser-technical-specifications-review.html) | web | 17 happy-path, recovery, error, and 320 px views | `approved` |

MOCK-0009 depends on the approved shell of MOCK-0001 and MOCK-0008 and does not supersede any earlier mockup. MOCK-0001..MOCK-0008 are unchanged.

## 11. Assumptions

- Technical specifications are stored as `<ID>.md` with `# <ID> <title>` and `## Description`; the browser reads and writes only title and description.
- Browsing, viewing, and editing reuse the TASK-6 transport, rendering, security, and configuration baseline; no new ADR, dependency, or implementation technology is introduced.
- Deprecated records are excluded exactly like other document types: not listed, not counted, not reachable, not editable.
- Example records (TECH-001, TECH-002, TECH-003, TECH-009) are illustrative.
- The version token is a hidden form field; no merge UI is drawn.

## 12. Approval status

| Item | Status |
| --- | --- |
| MOCK-0009 | `approved` — explicitly approved by the user on 2026-11-07 (UTC) (TASK-58) |
| This UI description | `approved` — 2026-11-07 (UTC), TASK-58 |
| Registry row | `approved` |

Approval was explicit from the user on 2026-11-07 (UTC) for TASK-58. TASK-57 may implement against MOCK-0009. Any later material revision of MOCK-0009 requires a new explicit user approval and status back to `draft` until then.
