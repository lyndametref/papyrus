# Feature Specification — Browser Access for Business Rules

**Ticket:** TASK-25
**Milestone:** m-8 (Business Rules)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0007`](ui/mockups/MOCK-0007-business-rules-review.html) — approved 2026-11-07 (TASK-45); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html), [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html), [`MOCK-0005`](ui/mockups/MOCK-0005-browser-ui-specifications-review.html), and [`MOCK-0006`](ui/mockups/MOCK-0006-browser-ui-mockups-review.html). UI description: [`ui/browser-business-rules-ui.md`](ui/browser-business-rules-ui.md).

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), TASK-15 (decision records), TASK-23 (glossary terms), TASK-24 (UI specifications), and TASK-40 (UI mockups) so it also serves the business-rules store: active business rules (ID, title, description, rationale, applicability, and references to business requirements). The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`business-rule`), its record model, list/view/edit screens, API endpoints, and tests. It implements REQ-05.005 and the web-visibility part of REQ-05.007 for the browser surface under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for business rules. Deprecated business rules are hidden from lists and unreachable by URL, exactly as deprecated requirements, glossary terms, UI specifications, and UI mockups.

Business rules are Markdown text records with a fixed section template (`## Description`, `## Rationale`, `## Applicability`, `## Requirements`). The browser edits only the five documented fields and must preserve every other byte/section of the file. Whole-file regeneration of a rule (for example by reusing the CLI's stub serializer) is forbidden because it destroys editor-added and unknown sections.

## Scope

### In scope

- Extending the persistent sidebar with a "Business Rules" entry (immediately after "Business requirements") and the `?type=` category selector to `business-rule`, without changing the list-page structure (baseline spec section 4.6).
- Listing active business rules, rendering only the business-rules section, with the first paragraph of the description as a row preview.
- Viewing a business rule's title, description, rationale, applicability, and referenced requirement IDs, with requirement references rendered as links to the corresponding requirement browser view where they resolve.
- Editing `title`, `description`, `rationale`, `applicability`, and `requirements` (business requirement IDs) through a form matching MOCK-0007, with the baseline optimistic-concurrency and atomic-write contract (ADR-0006).
- A field-preserving write contract that changes only the edited sections and leaves every other section, line, and byte of the file content-identical.
- A read/write JSON API over the same business-rule records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated business rules are not listed and are not reachable (active only).
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, deprecating, restoring, or deleting business rules from the browser; changing record IDs, deprecation state, or status. These remain CLI-only (REQ-05.005, BR-013).
- Managing the canonical `BR-*.md` business-rule documents that share the same directory; they are not records and are never read or written by the browser.
- Cross-type navigation beyond rendering requirement references as links, and the final completeness check — TASK-19.
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-46.
- Fixing the CLI's own whole-file rewrite behavior; this ticket fixes only the browser path.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts `business-rule` in addition to the existing values `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, and `ui-mockup`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists the document types in this order: Business requirements, Business Rules, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. When `document_types.rule.enabled` is `false`, the Business Rules sidebar entry is omitted and `?type=business-rule` renders the category empty state rather than an error (baseline section 4.4).
5. The list page renders only the selected category; the other categories are not rendered at all.
6. A missing or unreadable business-rules directory is not a server error: the category renders its empty state and its API returns an empty array.

### 2. Record sources and IDs

1. Business rules are read from `<doc_directory>/<document_types.rule.directory>/` (active folder only; default `business-rules/`). The directory, prefix, and enabled flag are resolved from the effective configuration and are never hardcoded.
2. Only regular files directly inside the active folder whose name ends in `.md` and whose file stem matches `<prefix>-<digits>` (default prefix `RUL`) are records. The `deprecated/` subfolder, files whose stem does not match (including the canonical `BR-*.md` documents), and non-`.md` files are ignored. There is no recursive traversal.
3. The record ID is the file stem, matched exactly and case-sensitively (for example `RUL-001`). The ID is never derived from file content and is never accepted from the request.
4. ID resolution is by exact match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to ten categories).
5. A business rule whose file is not valid UTF-8 is skipped by the list and API index and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten (baseline section 7.13).
6. Record form, category, and file path are derived from the matched directory and stem, never from the request.

### 3. Record model

1. The browser record has five editable values: `title`, `description`, `rationale`, `applicability`, and `requirements` (ordered list of business requirement IDs); the ID is read-only.
2. `title` is read from the text of the first line `# <ID> <title>` (the heading). When the heading contains only the ID, the title is the record ID.
3. `description`, `rationale`, and `applicability` are read from the bodies of the `## Description`, `## Rationale`, and `## Applicability` sections respectively. When a section is absent or its body is empty, the value is empty/`null`.
4. `requirements` is read from the `## Requirements` section, whose body consists of one bullet `- <id>` per referenced business requirement, in stored order. When the section is absent, the value is an empty list.
5. Unknown and editor-added sections, the preamble, and any content not covered by the five fields are preserved on save and are never displayed as editable fields.
6. The record form is `business-rule` and the category is `Business Rules`.

### 4. List page

1. For the `business-rule` category the list page renders one section (`h2`) titled "Business Rules", showing the record count, followed by one row per rule in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Each row also shows the first paragraph of the rule's description beneath the primary link; rules with an empty description show no preview. The preview is plain text (first paragraph only, not the full description).
4. Empty state and long-title wrapping follow the baseline (section 10.4).
5. Deprecated business rules are never listed or counted.

### 5. Record view

1. The view page shows:
   - metadata block: Record ID (read-only, monospace), Record form (`Business rule`), Category (`Business Rules`);
   - the title as the page title (`h1`);
   - `Description`, `Rationale`, `Applicability`, and `Requirements` sections, each rendered with `white-space: pre-wrap`, or `(none)` when empty.
2. Requirement references in the `Requirements` section are rendered as links to the corresponding requirement browser view (`/doc/{id}`) when the ID resolves to an active business or non-functional requirement; otherwise they are rendered as literal text. This is the only place in the browser where a requirement reference is rendered as a link.
3. The breadcrumb is `Business Rules` (a link to `/?type=business-rule`), then the record ID as the current, non-linked crumb.
4. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.

### 6. Edit form

1. Editable fields:
   - `title` (required, single-line);
   - `description` (optional, multi-line textarea);
   - `rationale` (optional, multi-line textarea);
   - `applicability` (optional, multi-line textarea);
   - `requirements` (optional, zero or more business requirement IDs, tag add/remove interaction per MOCK-0007/MOCK-0002).
2. ID, record form, and category are displayed as read-only metadata; they are never accepted from the request.
3. Breadcrumb on the edit page: `Business Rules › {id} › Edit`; `Edit` is the current, non-linked crumb.
4. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present exactly as in the baseline (section 4.23).
5. The form shows field help text, the pristine state, a dirty state with the "Unsaved changes" indicator, a validation-error state (422), a conflict state (409), and a 320 CSS-pixel responsive layout, matching MOCK-0007.
6. **Tag field interaction (MOCK-0007, MOCK-0002 pattern).** `requirements` is edited as tags:
   1. each current value is shown as one tag in a tag box, with a remove control whose accessible name is `Remove <value>`;
   2. an add line below the tag box contains a text input and an `Add` button; activating `Add` appends the trimmed input value as a new tag and clears the add input; an empty or whitespace-only value is not added;
   3. the remove control removes that tag;
   4. tag order is the stored order; adding appends at the end;
   5. the interaction is implemented with progressive enhancement only: the same markup must be operable without JavaScript. Each tag is therefore a text input carrying its value with `name="requirements"`; the trailing add input carries the same name and is empty. Without JavaScript, clearing a tag input removes that value and typing in the add input adds one value. With JavaScript, the remove and `Add` buttons operate on the same inputs and the tag box keeps the approved chip layout.
   6. the add input is not a search field and does not resolve or validate values while typing.
7. The form submits `application/x-www-form-urlencoded` with `version`, `title`, `description`, `rationale`, `applicability`, and repeated `requirements` fields (one occurrence per tag, in display order). All `requirements` occurrences are collected in document order and empty occurrences are dropped; this overrides the baseline "last value wins" rule for this field only (baseline section 5.5). Repeated text fields use last-value-wins.
8. No create, deprecate, restore, delete, or status control is present.

### 7. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `description`, `rationale`, and `applicability` follow baseline section 7.3/7.4: trimmed; at most 20000 Unicode scalar values; LF allowed; CR and CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section; a line whose first two characters are `## ` is rejected.
3. `requirements`:
   1. optional; an empty list is valid;
   2. each value is trimmed;
   3. each value must be a valid record ID per baseline section 7.1 (1–128 bytes; ASCII letters, digits, `.`, `_`, `-`; no leading `.` or `-`; no `..`);
   4. duplicate values are collapsed on save (first occurrence retained), matching `rule edit`, rather than rejected;
   5. a value is not required to resolve to an existing business requirement.
4. Validation failures return `422`, re-render the edit form with the submitted values preserved, a form-level summary whose entries link to the offending field, and field-level messages using the same text. Nothing is written.
5. Field messages:
   - `Title is required.`
   - `Title must be at most 200 characters.`
   - `Title cannot contain control characters.`
   - `Description must be at most 20000 characters.`
   - `Description cannot contain control characters.`
   - `Description contains a line starting with ## .`
   - `Rationale must be at most 20000 characters.`
   - `Rationale cannot contain control characters.`
   - `Rationale contains a line starting with ## .`
   - `Applicability must be at most 20000 characters.`
   - `Applicability cannot contain control characters.`
   - `Applicability contains a line starting with ## .`
   - `Requirement references must be valid record IDs.`
6. JSON validation failures use the same message text in the baseline JSON error object (`422`).

### 8. Persistence and editing contract

1. The file is read as UTF-8 text. On save, the file is rewritten atomically (baseline section 7.10) with only these substitutions:
   1. the first line is replaced with `# <ID> <title>`;
   2. the `## Description`, `## Rationale`, `## Applicability`, and `## Requirements` sections keep their original positions; a non-empty value replaces that section's body, an empty value removes the section, and a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end of the file;
   3. the `## Requirements` section body is written as one bullet `- <id>` per requirement reference, in stored order, with duplicates collapsed; when the list is empty the section is removed;
   4. every other `## ` section (including editor-added and unknown sections) is written back content-identically, in its original order and position;
   5. the preamble (lines between the heading and the first section) is preserved content-identically;
   6. line endings are normalized to LF and the file ends with a single trailing newline.
2. Every other content of the file is preserved content-identically: doctype, front matter, unknown sections, and any text outside the five edited fields. The browser never regenerates the file from a parsed record and never reuses the CLI's stub serializer (`render_record`).
3. The ID, file name, directory, deprecation state, and any read-only metadata never change on a browser save.
4. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing or malformed `version` → `400`; stale token → `409` with the message "record changed on disk; reload the form" and no write; matching token proceeds.
5. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`; nothing outside the project is read or written.
6. After a successful write the server keeps no cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET` (save-then-reread).
7. No temporary file is left behind after a failed or rejected request.

### 9. Deprecated business rules

1. Deprecated business rules live in the active directory's `deprecated/` subfolder. That folder is never scanned for records, never listed, never counted, and never resolved.
2. A request for a deprecated rule's ID returns `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`, exactly as for an unknown ID.
3. No browser control changes deprecation state; deprecation and restoration remain CLI-only (REQ-05.007, BR-013).

### 10. JSON API

1. `GET /api/business-rules` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id": "RUL-001", "title": "Require signed contracts"}`. The alias `/api/business-rule` returns the same response.
2. `GET /api/doc/{id}` for a business-rule ID returns the full record shape:

   ```json
   {
     "id": "RUL-001",
     "kind": "business-rule",
     "form": "business-rule",
     "title": "Require signed contracts",
     "description": "Every contract must be signed...",
     "rationale": "Reduces disputes...",
     "applicability": "Applies to all contracts...",
     "requirements": ["REQ-05.005"],
     "version": "1762000000.123456789.412",
     "path": "business-rules/RUL-001.md"
   }
   ```

   `description`, `rationale`, and `applicability` are strings or `null` (null when the section is absent or empty); `requirements` is an array (empty when absent); `path` is relative to the project directory with `/` separators and never exposes an absolute path.
3. `POST /api/doc/{id}` requires `Content-Type: application/json` and accepts an object with:
   - `version` (required, string);
   - `title` (required when present, string; omitting it leaves the title unchanged);
   - `description` (optional string or null; null or empty clears the section);
   - `rationale` (optional string or null; null or empty clears the section);
   - `applicability` (optional string or null; null or empty clears the section);
   - `requirements` (optional array of strings; when present it replaces the whole list; an empty array or null clears it).
4. Omitted members are left unchanged. Unknown members are ignored. A non-object body, a wrong member type, or a non-string array element is `400`. A title or field value that fails validation is `422`.
5. A successful `POST` returns `200` with the full record shape of section 10.2, read back from disk after the write.
6. HTML and JSON error responses follow baseline section 5.3.

### 11. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, and no inline scripts/styles.
2. Requirement references rendered as links in the view are the only exception to the baseline "requirement references inside any field are displayed as literal text" rule (section 4.19 of the baseline); the link target is derived from a resolved record ID, never from request input.
3. Tag controls are keyboard operable, every input has a `<label>` or accessible name, remove controls are labelled `Remove <value>`, and the add input is labelled. Focus is not trapped.
4. The sidebar's Business Rules entry and the business-rules list/view/edit templates are the only rendering additions; the shared shell and error pages are reused.

### 12. Tests

1. Unit tests cover: record parsing/serialization for business rules (heading, four sections, requirements bullets, unknown-section preservation), title extraction, field validation (title, description, rationale, applicability, requirement IDs), duplicate collapsing, version token, and JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-25a: list renders the Business Rules category, ascending by ID, with the selected sidebar entry marked `aria-current="page"`; `?type=business-rule` works and any other value is `400`.
   - T-25b: `BR-*.md`, non-matching `.md` files, and `deprecated/` are not listed.
   - T-25c: view shows ID, form, category, title, description, rationale, applicability, requirements, breadcrumb, and a single Edit action; requirement links resolve where applicable.
   - T-25d: edit round-trip persists all five fields and is visible through the browser and through `papyrus rule view`.
   - T-25e: editing a rule with an unknown/editor-added section changes only the edited fields; a byte comparison shows every other section and line unchanged.
   - T-25f: repeated `requirements` form fields are all collected in order (not last-value-wins), with duplicates collapsed.
   - T-25g: validation returns `422` and writes nothing for empty title, over-long title, control characters, over-long text fields, `## ` lines, and invalid requirement IDs.
   - T-25h: stale version token returns `409` and does not write; missing/malformed version returns `400`.
   - T-25i: deprecated business rules are not listed and return `404` on view, edit, POST, and API.
   - T-25j: JSON API returns the documented summary and full shapes, applies the documented POST semantics, and rejects wrong types with `400`/`422`.
   - T-25k: writes are atomic, leave no temporary file, and a symlink escape returns `404`.
   - T-25l: an offline asset scan finds no external origin in list, view, or edit responses.
   - T-25m: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Approved artifact (TASK-45):** [`MOCK-0007`](ui/mockups/MOCK-0007-business-rules-review.html) — `approved` in the mockup registry on 2026-11-07 (UTC). One self-contained review package: review landing, business-rules list/view/edit prototypes, error pages, and 320 px states (17 views). Layout description: [`ui/browser-business-rules-ui.md`](ui/browser-business-rules-ui.md). MOCK-0001…MOCK-0006 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with eight entries, `main`, footer, one `h1` per page.
2. **Business rules list** (`/?type=business-rule`): one section with record count, rows of `ID + title` plus description preview; empty and long-title states.
3. **Business rule view** (`/doc/{id}`): metadata block, title `h1`, Description/Rationale/Applicability/Requirements sections with `pre-wrap`, breadcrumb `Business Rules › ID`, single `Edit`.
4. **Edit form** (`/doc/{id}/edit`): title input, description/rationale/applicability textareas, requirements tag box (add/remove), version annotation, Save/Cancel; pristine, dirty, 422, 409, and 320 px states.
5. **Error pages**: 404 unknown, 404 deprecated, 400, 405, 415, 500 (reused from MOCK-0001).

### Layout conflicts resolved by this specification

1. The UI description (MOCK-0007) renders requirement references as links to requirement views where available, whereas the baseline (section 4.19) displays requirement references as literal text. This specification adopts the UI description's behavior for the `Requirements` section of the business-rule view only; all other requirement references in the browser remain literal text.
2. The UI description says duplicate requirement references are collapsed. This specification adopts collapsing rather than rejection, matching `rule edit`. If the implementation prefers rejection, validation messages and tests change accordingly (see Open question OQ-2).

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `?type=business-rule` renders the category, the default stays `req`, other values are `400`, and the sidebar entry is ordered after Business requirements with `aria-current="page"` and omitted when the type is disabled. |
| AC-2 | The list shows active rules only, ascending by ID, with `ID + title` links, description preview, count, empty state, and no non-record or deprecated files. |
| AC-3 | The view shows ID, form, category, title, description, rationale, applicability, requirements, breadcrumb, and one Edit action, and renders requirement references as links where they resolve. |
| AC-4 | The edit form matches MOCK-0007: title, description, rationale, applicability, requirements tag add/remove, hidden version plus annotation, Save/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript. |
| AC-5 | An edit round-trip persists all five fields and is visible in the browser and through `papyrus rule view`. |
| AC-6 | Editing a rule preserves every section and line not being edited; no whole-file regeneration occurs. |
| AC-7 | Title extraction and update follow section 3 and section 8 (heading, em dash not applicable; unknown sections preserved). |
| AC-8 | Validation returns `422` with the documented messages and writes nothing (title, text fields, requirement IDs). |
| AC-9 | A missing/malformed version is `400`; a stale version is `409` with the documented message and no write. |
| AC-10 | Deprecated business rules are hidden and return `404` on view, edit, POST, and API. |
| AC-11 | The JSON API returns the documented summary/full shapes, supports the documented POST semantics, and rejects wrong types. |
| AC-12 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-13 | The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable tag controls. |
| AC-14 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: A business rule's persisted file is a Markdown text record with a fixed section template; browser edits must be field-preserving and must never regenerate the file from a parsed model, so that editor-added and unknown sections survive.
- Assumption: The `## Requirements` section (bullets) is the machine-readable source of requirement references. Rules created before a reference is added display an empty list and receive the section on first save.
- Assumption: Requirement ID format is validated but existence is not required, matching `rule create`/`edit`; dangling IDs are allowed so a rule can reference a requirement before it is registered.
- Assumption: Duplicate requirement references are collapsed on save, matching `rule edit`; this is looser than the CLI's input handling but produces the same stored result.
- Assumption: The tag field's no-JavaScript fallback uses one text input per value plus a trailing empty input with the same field name; this preserves the approved chip layout with JavaScript and keeps the form usable without it.
- Assumption: The JSON API exposes the five editable fields; business rules have no status, task, or supersession metadata, unlike decision records.
- Assumption: Real-browser verification in Firefox, Edge, and Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with TASK-15/TASK-23/TASK-24/TASK-40.
- Assumption: No new dependencies, crates, or ADRs are introduced.
- Assumption: The canonical `BR-*.md` documents sharing the directory are not records and are never touched by the browser; only `<prefix>-<digits>.md` files are.

## Open questions

- **OQ-1 (non-blocking, product)** — Whether `requirements` should be required to resolve to an existing active business requirement. This specification does not require resolution, for parity with the CLI. Tightening it later changes validation only.
- **OQ-2 (non-blocking, product)** — Whether duplicate requirement references should be rejected instead of collapsed. This specification collapses them to match `rule edit` and the UI description. If rejection is preferred, the tag interaction and validation messages change (a `Duplicate requirement reference: <value>.` message is added).
- **OQ-3 (non-blocking, layout)** — Whether requirement references in the view should be rendered as links (adopted here per MOCK-0007 OQ-2) or as literal text (baseline section 4.19). This specification adopts links; the baseline text-display rule applies to all other requirement references in the browser.
- **OQ-4 (non-blocking, naming)** — The JSON `kind`/`form` value is `business-rule` to match the category selector. If the implementation prefers the CLI type key `rule`, the value changes in one place and tests follow.
- **OQ-5 (non-blocking, follow-up)** — Real-browser verification in Firefox, Edge, and Safari 100+ is deferred, consistent with sibling tickets.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.005, REQ-05.007, REQ-06, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-013 (rule command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-40, TASK-43 (record model), TASK-45 (UI design), TASK-25 |
| UI mockups | MOCK-0001…MOCK-0006 (approved baseline); MOCK-0007 (approved) — `docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html` |
| UI description | `docs/specifications/ui/browser-business-rules-ui.md` (TASK-45) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/business-rules-management.md` (TASK-43 record model and lifecycle), `docs/specifications/browser-ui-specifications.md` (TASK-24 sibling increment), `docs/specifications/browser-ui-mockups.md` (TASK-40 sibling increment) |
| Domain model | `docs/domain-model/business-glossary/business-rule.md` (if present) |