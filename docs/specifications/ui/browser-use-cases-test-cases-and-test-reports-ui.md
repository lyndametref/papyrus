# UI Description — Papyrus Local Browser (Use Cases, Test Cases, and Test Reports)

**Ticket:** TASK-79
**Specification:** [`../use-cases-test-cases-and-test-reports-management.md`](../use-cases-test-cases-and-test-reports-management.md); baseline [`../browser-requirements.md`](../browser-requirements.md) sections 4–10
**Mockup package:** [`mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html`](mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-11 (UTC) (TASK-79).
**Approval note:** Created by @ui-designer on 2026-11-11 (UTC) for TASK-79, reusing the approved MOCK-0001 shell. MOCK-0013 is `approved` — explicitly approved by the user on 2026-11-11 (UTC) (TASK-79).

## 1. Purpose and scope

Layout, states, elements, and interactions for extending the local browser to use cases, test cases, and test reports, reusing the approved MOCK-0001 shell. Adds sidebar entries "Use Cases", "Test Cases", and "Test Reports" and views for listing, viewing, and editing records of all three types. The browser is read-and-edit only: creating, deprecating, and restoring stay on the command line (BR-018/BR-019/BR-020). Deprecated records of all three types are excluded from lists, counts, and navigation and are not reachable by URL (REQ-05.007, BR-018/BR-019/BR-020).

All three types share the same field set: title (required) and description (required, multi-line), plus an optional applicability (multi-line). ID, record form, and category are read-only. Front-matter metadata (id, title, created_date, updated_date, labels, references) is projected on list/detail/edit views; created_date and updated_date are read-only in edit; metadata edits target front matter, not body sections.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types" with entries (Business requirements, Business Rules, Audit Records, Technical Specifications, Use Cases, Test Cases, Test Reports, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups, Managed artifacts &amp; lifecycles, Agent skills &amp; workflows), `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. The 320 px sidebar collapses behind an accessible toggle.

Prior entries (all except Use Cases, Test Cases, Test Reports) are non-clickable in this extension package; only the three new types have generated views.

The review file links the shared wireframe stylesheet (`wireframe.css`) and loads no external script, font, image, or network resource.

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /?type=use-case` | Use cases list | `list-use-cases`, `list-use-cases-empty`, `list-use-cases-long` |
| `GET /doc/{id}` | Use case view | `view-use-case`, `view-use-case-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-use-case`, `edit-use-case-dirty`, `edit-use-case-errors`, `edit-use-case-conflict` |
| `GET /?type=test-case` | Test cases list | `list-test-cases`, `list-test-cases-empty`, `list-test-cases-long` |
| `GET /doc/{id}` | Test case view | `view-test-case`, `view-test-case-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-test-case`, `edit-test-case-dirty`, `edit-test-case-errors`, `edit-test-case-conflict` |
| `GET /?type=test-report` | Test reports list | `list-test-reports`, `list-test-reports-empty`, `list-test-reports-long` |
| `GET /doc/{id}` | Test report view | `view-test-report`, `view-test-report-after-save` |
| `GET /doc/{id}/edit` | Edit form | `edit-test-report`, `edit-test-report-dirty`, `edit-test-report-errors`, `edit-test-report-conflict` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive list and edit form | `responsive-list-use-cases-320`, `responsive-edit-use-cases-320`, `responsive-list-test-cases-320`, `responsive-edit-test-cases-320`, `responsive-list-test-reports-320`, `responsive-edit-test-reports-320` |

## 4. List views

- One section (`h2`) titled with the type label, showing the record count, followed by one row per record in ascending ID order.
- Each row's primary link text is `ID + title` and links to `/doc/{id}`.
- Each row also shows the first paragraph of the description beneath the primary link; records with an empty description show no preview. The preview is plain text (first paragraph only).
- Each row shows a metadata line: created date, updated date, labels, references (front-matter projection).
- Empty state: "No <type> were found in the configured directory. <Type> are created from the command line; this browser does not create records. Deprecated <type> are hidden." Long-title and unbreakable-token wrapping follow the baseline.
- Deprecated records are never listed or counted.

## 5. Record view

- Metadata block: ID (read-only, monospace), Title, Created date, Updated date, Labels, References (front-matter projection).
- Title is the page title (`h1`); the description is rendered as body text with `white-space: pre-wrap`, preserving line breaks and blank lines. No additional field heading is drawn.
- Applicability, when present, is rendered as a section with `white-space: pre-wrap`.
- Breadcrumb: `<Type plural> › {id}` (type links to `/?type=<category>`); ID is the current, non-linked crumb.
- Single `Edit` action. No "Back to ..." action.
- A deprecated ID renders the 404 page instead of the view.

## 6. Edit form

- Editable fields: `title` (required, single-line), `description` (required, multi-line textarea), `applicability` (optional, multi-line textarea).
- Title: trimmed; empty values and control characters are rejected.
- Description: required; multi-line, LF-preserving; empty values are rejected.
- Applicability: optional; multi-line; when cleared the `## Applicability` section is removed.
- ID, record form, and category are read-only.
- Front-matter metadata block shows Record ID, Created date, Updated date (read-only inputs), Labels, References (tags with add input).
- Hidden `version` input plus dashed annotation "Hidden field: version (optimistic-concurrency token)".
- Pristine, dirty ("Unsaved changes"), validation-error (422), conflict (409), and 320 px states.
- Validation error state: error summary (`role="alert"`) listing "Title is required." and "Description is required."; both messages link to their fields; typed values are kept.
- Conflict state: 409 notice, "Reload the form" and "View current record" actions, and a read-only block listing the unsaved values.
- Save changes → record view; Cancel → record view without saving.

## 7. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown, 404 deprecated, 400, 405, 415, 500, each with a single recovery link back to the use cases list. The 404 deprecated page is visually identical to 404 unknown; its help text states that deprecated records are not listed and cannot be opened.

320 px list and edit form: sidebar toggle first, full-width rows/fields, stacked form actions, and an annotation legend drawn inside the narrow frame. Focus order follows document order; every control has a visible label; the designed interface works without JavaScript.

## 8. Active elements

List: sidebar entries (Use Cases, Test Cases, Test Reports), row links. View: breadcrumb links, Edit. Edit: title input, description textarea, applicability textarea, Save changes, Cancel, label/reference add inputs, hidden version. Error: recovery link. 320 px: sidebar toggle.

## 9. Open questions (resolved defaults)

- OQ-1 (sidebar ordering): Use Cases, Test Cases, Test Reports are placed immediately after Technical Specifications; all other entries keep their order.
- OQ-2 (category values): `type=use-case`, `type=test-case`, `type=test-report` select the corresponding category; any other unknown value remains a 400 per the baseline.
- OQ-3 (list preview): list rows show the first paragraph of the description beneath ID + title, consistent with MOCK-0009.
- OQ-4 (field rules): only rules stated in `use-cases-test-cases-and-test-reports-management.md` and BR-018/BR-019/BR-020 are drawn; title and description are trimmed, required, and reject control characters; applicability is optional multi-line.
- OQ-5 (metadata projection): all three types demonstrate front-matter metadata projection (id, title, created_date, updated_date, labels, references) in list/detail/edit, since this ticket is specifically about them.

## 10. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0013 | [`mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html`](mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html) | web | 39 happy-path, recovery, error, and 320 px views | `approved` |

MOCK-0013 depends on the approved shell of MOCK-0001 and does not supersede any earlier mockup. MOCK-0001..MOCK-0012 are unchanged.

## 11. Assumptions

- Use cases, test cases, and test reports are stored as `<ID>.md` with `# <ID> <title>`, `## Description`, and optional `## Applicability`; the browser reads and writes title, description, and applicability.
- Browsing, viewing, and editing reuse the TASK-6 transport, rendering, security, and configuration baseline; no new ADR, dependency, or implementation technology is introduced.
- Deprecated records are excluded exactly like other document types: not listed, not counted, not reachable, not editable.
- Example records (UC-001, UC-002, UC-009, TC-001, TC-002, TC-009, TR-001, TR-002, TR-009) are illustrative.
- The version token is a hidden form field; no merge UI is drawn.
- Applicability is editable in the browser as a multi-line textarea, consistent with description.

## 12. Approval status

| Item | Status |
| --- | --- |
| MOCK-0013 | `approved` — explicitly approved by the user on 2026-11-11 (UTC) (TASK-79) |
| This UI description | `approved` — 2026-11-11 (UTC), TASK-79 |
| Registry row | `approved` |

MOCK-0013 is explicitly approved by the user on 2026-11-11 (UTC) (TASK-79). TASK-81 may proceed to implementation.
