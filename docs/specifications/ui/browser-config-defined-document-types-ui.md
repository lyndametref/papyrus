# UI Description — Browser Views for Config-Defined Document Types

**Ticket:** TASK-114
**Specification:** [`../browser-config-defined-document-types.md`](../browser-config-defined-document-types.md) (feature specification); [`../browser-requirements.md`](../browser-requirements.md) (baseline shell); [`../../requirements/business/REQ-06.014.md`](../../requirements/business/REQ-06.014.md); [`../../requirements/business/REQ-06.003.md`](../../requirements/business/REQ-06.003.md); ADR-0020, ADR-0017
**Mockup package:** [`mockups/MOCK-0015-browser-config-defined-document-types-review.html`](mockups/MOCK-0015-browser-config-defined-document-types-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-14 (UTC) with the `Status` column header carrying no `(read-only)` qualifier; the read-only nature is stated in prose.

## 1. Purpose and scope

Layout, elements, states, and interactions for the generic browser views of a document type that exists only in configuration. The example type is slug `note`, label **Notes**, prefix `NOTE`, folder `docs/notes/`, with configured statuses `active`, `review`, `approved` in that order.

The package covers the list view with its status filter (default, filtered, empty), the detail view, the edit view with validation and stale-version states, the lifecycle page delta for a config-defined enabled type and a disabled type, and the grouped sidebar treatment. Built-in typed pages use the shared list contract (`browser-shell-ui.md` §3) and appear only as navigation context.

## 2. Shared shell

Every page keeps the approved shell: header (product name, project name, static offline/bind note), persistent left sidebar "Document types", `main`, footer, one `h1` per page. The selected sidebar entry carries a bold label plus the current-page indication.

The sidebar is grouped by artifact category, in this order. Group labels are small uppercase headings; the entries below each label keep their Title Case labels and their relative order inside the group.

| Group | Entries, in order |
| --- | --- |
| Backlog | Backlog |
| Decisions | Decisions |
| Documents | Business requirements, Business Rules, Audit Records, Technical Specifications, Use Cases, Test Cases, Test Reports, Non-functional requirements, Glossary, UI Specifications, UI Mockups, then every enabled config-defined document type |
| Project | Configuration, Managed artifacts &amp; lifecycles, Agent skills &amp; workflows |

- Every entry of the previous flat sidebar is kept; no entry is renamed, removed, or reordered within its group.
- Backlog and Decisions are single-entry groups because each is one unified page.
- Config-defined types appear in the Documents group after the built-in document types, in configured order. The example **Notes** comes after `UI Mockups` and before the Project group.
- A disabled type has no sidebar entry. The disabled example is **Deploy logs** (`enabled: false`, prefix `DEPLOY`, folder `docs/deploy-logs/`); it appears only as a lifecycle section.

## 3. Screen map

| Route | Screen | Prototype view |
| --- | --- | --- |
| `/?type=note` | Notes list, all records | `list-notes` |
| `/?type=note&status=review` | Notes list, filtered | `list-notes-review` |
| `/?type=note` | Notes list, empty | `list-notes-empty` |
| `/doc/NOTE-001` | Note detail | `view-note` |
| `/doc/NOTE-001/edit` | Note edit | `edit-note` |
| `POST /doc/NOTE-001/edit` | Edit validation errors | `edit-note-errors` |
| `POST /doc/NOTE-001/edit` | Stale-version conflict | `edit-note-conflict` |
| `/lifecycle` | Lifecycle page delta | `lifecycle` |
| — | Sidebar treatment | `nav-notes` |

## 4. Notes list

- Page title `Notes`; subtitle names the configured folder and prefix. Count `N notes`.
- The list reads only the active directory. Archived records are never listed, and an archived address and an unknown address return the same 404 page.
- Status filter above the list: label `Status`, a single-select control, an `Apply filter` action, and a `Show all` reset link.
- The select lists only the filterable statuses in configured order — `active`, `review`, `approved`. `archived` is never offered. A type with no custom statuses offers only `active`.
- The select has no `All`/`All records`/empty-value option. When no filter is active it shows a disabled, default-selected placeholder option `Status`, so an untouched submit sends no `status` parameter at all. `Show all` is the only reset and returns the list without a `status` parameter.
- With no `status` parameter the list shows every record in the active directory, including records whose stored status is a custom status. The absence of `status` is the no-filter state; there is no sentinel status value.
- A filtered request (`status=<value>`) lists only records whose stored front-matter value equals the selected value exactly. The active filter is shown as a badge; the count becomes the filtered count (`2 of 4 records`).
- A value outside the offered set (including `archived`) is rejected, not applied as an unfiltered or guessed match. An explicit empty `status=` is rejected with `400`.
- Table columns, in order: `ID`, `Title`, `Status`, family slot(s) (none for a config-defined type), `Created`, `Updated`, `Actions`. Status is the structural Status column, carries no qualifier, and is read-only, as the note below the list states. The Title cell links to the detail view and carries labels/references. Status is rendered as stored, including custom statuses.
- `Created` and `Updated` render the record's front-matter `created_date` and `updated_date` in the shared `YYYY-MM-DD` format.
- A note states that status is never editable in the browser, that archived records are never listed, and that an archived address and an unknown address return the same 404 page.
- Empty state: the filter remains visible with the disabled `Status` placeholder selected; the shared empty message and help line from `browser-shell-ui.md` §3.6 render, and the count is `0 notes`.

## 5. Note detail

- Breadcrumb `Notes › NOTE-001`; `Notes` links to the list.
- Summary card at the beginning shows only `ID`, `Created`, `Updated`.
- Title is the page `h1`; a single `Edit` action is aligned with it.
- `Status` is displayed read-only, followed by a `read-only` qualifier.
- Labels render as a tag list under the status line.
- The description is rendered as body text preserving line breaks.
- References are drawn at the end of the page as a native `<details>` disclosure section with summary `References (N)`, shown only when the record has at least one reference. Unresolved reference targets render as plain text; reference links resolve to the referenced record's view when one exists.
- A note restates that status is display-only and that archived records are not reachable: an archived address and an unknown address return the same 404 page.

## 6. Note edit

- Breadcrumb `Notes › NOTE-001 › Edit`; heading `Edit NOTE-001`; subtitle is the record title.
- Read-only timestamps `Created` and `Updated` are drawn as disabled fields with a note that they are refreshed once per save; they are not editable.
- Editable fields: `Title` (required, 1-200 characters, single line, trimmed), `Description` (optional, multi-line, line breaks preserved), `Labels` (repeatable rows, each with `Remove`, plus `Add label`), `References` (repeatable rows, each with `Remove`, plus `Add reference`).
- **No status field is drawn.** No control, hidden input, or save path writes status. A note explains that status is lifecycle- and configuration-owned and that a save carrying it would be rejected as an unknown field.
- Hidden `version` token is drawn only as an annotation, not as a user-visible field.
- Actions: `Save changes` (primary) and `Cancel` (returns to the detail view without saving).
- Validation-error state: an alert summary titled "There is a problem" lists `Title is required.` as a link to its field; the title field is marked invalid; typed label and reference values are kept; nothing is saved. Description is not marked required and produces no validation error when empty.
- Conflict state (stale version token): heading "This note changed on disk", `409 Conflict` notice, actions `Reload the form` and `View current record`, and a read-only block with the unsaved values. No merge is offered.

## 7. Lifecycle page delta

- Three category groups with headings in order: `Backlog`, `Decisions`, `Documents`; one section per declared type.
- Backlog: Tasks, Milestones, Releases. Decisions: Decision records. Documents: every declared document type, enabled and disabled.
- Every document section shows the main storage lifecycle `active → archived`, any declared custom statuses as front-matter values that do not change location, and read-only `enabled`, ID prefix, and folder.
- Built-in document sections keep their curated description; a config-defined type has no curated text and uses the shared document description.
- **Notes** section (enabled): description of the shared document lifecycle; main `active → archived`; custom statuses `review → approved` in configured order, front-matter only; `enabled: true`; prefix `NOTE`; folder `docs/notes/`. Highlights that only `active`/`archived` change location, that `archived` is no longer listed or reachable, and that an archived address and an unknown address return the same 404 page.
- **Deploy logs** section (disabled example): shows `enabled: false`; main `active → archived`; no custom statuses; prefix `DEPLOY`; folder `docs/deploy-logs/`. States that it has no sidebar entry, list, detail, edit, or API response and its records are neither listed nor reachable.
- All values on the page are display-only; changes are made through Configuration. The page links to Configuration for changes.

## 8. Sidebar treatment view

- Draws the full grouped sidebar so the reviewer can see the insertion point: group labels `Backlog`, `Decisions`, `Documents`, `Project`, with every existing entry kept in Title Case and in its existing order inside the group; `Notes` sits in `Documents` after `UI Mockups`; `Configuration`, `Managed artifacts & lifecycles`, and `Agent skills & workflows` sit in `Project`.
- Main content explains the three rules: entries unchanged and grouped by category, enabled config-defined type gets one entry with its derived label, disabled type has no entry.
- Actions: open the Notes list, open the lifecycle page.

## 9. Active elements

- List: sidebar `Notes` entry, status select, `Apply filter`, `Show all`, row title links.
- Detail: breadcrumb `Notes` link, `Edit`, `References (N)` disclosure summary, reference links when resolvable.
- Edit: title input, description textarea (optional), label inputs with `Remove` and `Add label`, reference inputs with `Remove` and `Add reference`, `Save changes`, `Cancel`, hidden version token.
- Validation: error-summary link to the title field, `Save changes`, `Cancel`.
- Conflict: `Reload the form`, `View current record`.
- Lifecycle: sidebar `Notes` and `Managed artifacts & lifecycles` entries, `Configuration` link for changes.
- Sidebar view: `Notes`, lifecycle link.

## 10. Open questions and assumptions

- OQ-1 (resolved default): the sidebar is grouped into `Backlog`, `Decisions`, `Documents`, and `Project`; the `Notes` entry sits in `Documents` after `UI Mockups`; `Configuration`, `Managed artifacts & lifecycles`, and `Agent skills & workflows` sit in `Project`. All earlier entries are kept with unchanged labels and relative order.
- OQ-2 (resolved default): status filtering is drawn as a single-select control plus `Apply filter`; no client-side behaviour is implied and the mockup uses review navigation only.
- OQ-3 (resolved default): status values are shown with neutral badges; no colour or visual distinction between statuses is designed.
- Assumption (revised 2026-11-14): the select has no `All`/empty-value option; an untouched submit omits `status` entirely, `Show all` is the only reset, and an explicit empty `status=` is rejected with `400`.
- Assumption: reference links are plain text when the target record has no view in this prototype package.
- Assumption: example records, IDs, dates, and the disabled `Deploy logs` type are illustrative.
- The package is web-only; no mobile or tablet platform is designed.

## 11. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0015 | [`mockups/MOCK-0015-browser-config-defined-document-types-review.html`](mockups/MOCK-0015-browser-config-defined-document-types-review.html) | web | 9 views: list, filtered, empty, detail, edit, validation, conflict, lifecycle, sidebar | `approved` (2026-11-14) |

MOCK-0015 depends on the approved shell of MOCK-0001, the generic document patterns of MOCK-0009 and MOCK-0013, and the lifecycle/layout patterns of MOCK-0010 and MOCK-0014. It does not revise or supersede any earlier mockup.

Revision 2026-11-14: sidebar grouped into `Backlog`, `Decisions`, `Documents`, `Project`; list columns reordered to `Status | ID | Title | Created | Updated` with a new `Created` column; archived records never appear and the list/filter copy states that archived and unknown addresses return the same 404 page.

Revision 2026-11-14 (approved): the list `Status` column header no longer carries a `(read-only)` qualifier; the read-only nature is stated in prose only. Approved by the user on 2026-11-14 (UTC).

Revision 2026-11-14 (filter-control correction, status `approved` retained): the status filter no longer offers `All`/`All records`. The select lists only the filterable statuses plus a disabled, default-selected `Status` placeholder; an untouched submit sends no `status`, `Show all` is the only reset, and an explicit empty `status=` is rejected with `400`. Applied to the default, filtered, and empty list screens and to §4 above.

## 12. Approval status

| Item | Status |
| --- | --- |
| MOCK-0015 | `approved` — explicit user approval 2026-11-14 (UTC) |
| This UI description | `approved` — 2026-11-14 (UTC) |
| Registry row | `approved` |

Any material revision requires explicit user approval before implementation. Status changes to `approved` only after the user explicitly approves this package.
