# Feature Specification — Browser Access for Audit Records

**Ticket:** TASK-26
**Milestone:** m-9 (Audit Records)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0008`](ui/mockups/MOCK-0008-audit-records-review.html) — approved 2026-11-07 (TASK-49); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html), [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html), [`MOCK-0005`](ui/mockups/MOCK-0005-browser-ui-specifications-review.html), [`MOCK-0006`](ui/mockups/MOCK-0006-browser-ui-mockups-review.html), and [`MOCK-0007`](ui/mockups/MOCK-0007-business-rules-review.html). UI description: [`ui/browser-audit-records-ui.md`](ui/browser-audit-records-ui.md).

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), TASK-15 (decision records), TASK-23 (glossary terms), TASK-24 (UI specifications), TASK-40 (UI mockups), and TASK-25 (business rules) so it also serves the audit-records store: active audit records (ID, title, audit type, and content). The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`audit-record`), its record model, list/view/edit screens, API endpoints, and tests. It implements the browser-visibility part of REQ-05.006/REQ-05.007 for audit records under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for audit records. Deprecated audit records are hidden from lists and unreachable by URL, consistent with deprecated requirements, glossary terms, UI specifications, UI mockups, and business rules.

Audit records are Markdown text records with a fixed section template (`# <ID> <title>`, then `## Audit Type`, then `## Content`). The browser edits only the three documented fields and must preserve every other byte/section of the file. Whole-file regeneration of a record (for example by reusing the CLI's stub serializer) is forbidden because it destroys editor-added and unknown sections.

## Scope

### In scope

- Extending the persistent sidebar with an "Audit Records" entry (immediately after "Business Rules") and the `?type=` category selector to `audit-record`, without changing the list-page structure (baseline spec section 4.6).
- Listing active audit records, rendering only the audit-records section, with the first paragraph of the content as a row preview.
- Viewing an audit record's title, audit type, and content, with the audit type shown at the beginning of the view (badge next to the title/ID plus an "Audit type" row in the metadata block) and the content rendered as body text without a "Content" heading.
- Editing `title`, `audit_type` (select: security/licence/architecture, default security), and `content` (optional, multi-line textarea) through a form matching MOCK-0008, with the baseline optimistic-concurrency and atomic-write contract (ADR-0006).
- A field-preserving write contract that changes only the title heading and the `## Audit Type` and `## Content` sections and leaves every other section, line, and byte of the file content-identical.
- A read/write JSON API over the same audit records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated audit records are not listed and are not reachable (active only).
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, deprecating, restoring, or deleting audit records from the browser; changing record IDs, deprecation state, or status. These remain CLI-only (REQ-05.006, BR-014).
- Cross-type navigation beyond the baseline sidebar, and the final completeness check — TASK-19.
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-50.
- Fixing the CLI's own whole-file rewrite behavior; this ticket fixes only the browser path.
- Validating audit types against a configured allow-list or supporting non-built-in (custom) audit types in the edit select; the browser edit control is limited to the three built-in types per MOCK-0008.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts `audit-record` in addition to the existing values `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, and `business-rule`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists the document types in this order: Business requirements, Business Rules, Audit Records, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. When `document_types.audit.enabled` is `false`, the Audit Records sidebar entry is omitted and `?type=audit-record` renders the category empty state rather than an error (baseline section 4.4).
5. The list page renders only the selected category; the other categories are not rendered at all.
6. A missing or unreadable audit-records directory is not a server error: the category renders its empty state and its API returns an empty array.

### 2. Record sources and IDs

1. Audit records are read from `<doc_directory>/<document_types.audit.directory>/` (active folder only; default `audit-records/`). The directory, prefix, and enabled flag are resolved from the effective configuration and are never hardcoded.
2. Only regular files directly inside the active folder whose name ends in `.md` and whose file stem matches `<prefix>-<digits>` (default prefix `AUD`) are records. The `deprecated/` subfolder, files whose stem does not match, and non-`.md` files are ignored. There is no recursive traversal.
3. The record ID is the file stem, matched exactly and case-sensitively (for example `AUD-001`). The ID is never derived from file content and is never accepted from the request.
4. ID resolution is by exact match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to eleven categories).
5. An audit record whose file is not valid UTF-8 is skipped by the list and API index and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten (baseline section 7.13).
6. Record form, category, and file path are derived from the matched directory and stem, never from the request.

### 3. Record model

1. The browser record has three editable values: `title`, `audit_type`, and `content`; the ID is read-only.
2. `title` is read from the text of the first line `# <ID> <title>` (the heading). When the heading contains only the ID, the title is the record ID.
3. `audit_type` is read from the body of the `## Audit Type` section. When the section is absent or its body is empty, the value defaults to `security`.
4. `content` is read from the body of the `## Content` section. When the section is absent or its body is empty, the value is empty/`null`.
5. Unknown and editor-added sections, the preamble, and any content not covered by the three fields are preserved on save and are never displayed as editable fields.
6. The record form is `audit-record` and the category is `Audit Records`.

### 4. List page

1. For the `audit-record` category the list page renders one section (`h2`) titled "Audit Records", showing the record count, followed by one row per record in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Each row also shows the first paragraph of the record's content beneath the primary link; records with empty content show no preview. The preview is plain text (first paragraph only, not the full content).
4. Empty state and long-title wrapping follow the baseline (section 10.4).
5. Deprecated audit records are never listed or counted.

### 5. Record view

1. The view page shows:
   - metadata block: Record ID (read-only, monospace), Record form (`Audit record`), Category (`Audit Records`);
   - the audit type at the beginning of the view, as a badge next to the record ID/title and as an "Audit type" row in the metadata block (security/licence/architecture, or the stored value for a record created with a custom type);
   - the title as the page title (`h1`);
   - the content rendered as body text with `white-space: pre-wrap`, or `(none)` when empty, without a "Content" heading.
2. The breadcrumb is `Audit Records` (a link to `/?type=audit-record`), then the record ID as the current, non-linked crumb.
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.

### 6. Edit form

1. Editable fields:
   - `title` (required, single-line);
   - `audit_type` (required select among security, licence, architecture; defaults to security);
   - `content` (optional, multi-line textarea).
2. ID, record form, and category are displayed as read-only metadata; they are never accepted from the request.
3. Breadcrumb on the edit page: `Audit Records › {id} › Edit`; `Edit` is the current, non-linked crumb.
4. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present exactly as in the baseline (section 4.23).
5. The form shows field help text, the pristine state, a dirty state with the "Unsaved changes" indicator, a validation-error state (422), a conflict state (409), and a 320 CSS-pixel responsive layout, matching MOCK-0008.
6. The form submits `application/x-www-form-urlencoded` with `version`, `title`, `audit_type`, and `content` fields. Repeated text fields use last-value-wins.
7. No create, deprecate, restore, delete, or status control is present.

### 7. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `audit_type` is required in the form (the select defaults to security, so a value is always submitted); it must be one of `security`, `licence`, or `architecture`. An empty or unknown value is rejected.
3. `content` follows baseline section 7.3/7.4: trimmed; at most 20000 Unicode scalar values; LF allowed; CR and CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section; a line whose first two characters are `## ` is rejected.
4. Validation failures return `422`, re-render the edit form with the submitted values preserved, a form-level summary whose entries link to the offending field, and field-level messages using the same text. Nothing is written.
5. Field messages:
   - `Title is required.`
   - `Title must be at most 200 characters.`
   - `Title cannot contain control characters.`
   - `Audit type is required.`
   - `Audit type must be one of security, licence, or architecture.`
   - `Content must be at most 20000 characters.`
   - `Content cannot contain control characters.`
   - `Content contains a line starting with ## .`
6. JSON validation failures use the same message text in the baseline JSON error object (`422`).

### 8. Persistence and editing contract

1. The file is read as UTF-8 text. On save, the file is rewritten atomically (baseline section 7.10) with only these substitutions:
   1. the first line is replaced with `# <ID> <title>`;
   2. the `## Audit Type` and `## Content` sections keep their original positions; a non-empty value replaces that section's body, an empty value removes the section, and a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end of the file. The `## Audit Type` value is always non-empty (one of the three built-in types), so that section is always written;
   3. the `## Content` section body is written as the normalized content text; when the content is empty the section is removed;
   4. every other `## ` section (including editor-added and unknown sections) is written back content-identically, in its original order and position;
   5. the preamble (lines between the heading and the first section) is preserved content-identically;
   6. line endings are normalized to LF and the file ends with a single trailing newline.
2. Every other content of the file is preserved content-identically: doctype, front matter, unknown sections, and any text outside the three edited fields. The browser never regenerates the file from a parsed record and never reuses the CLI's stub serializer (`render_record`).
3. The ID, file name, directory, deprecation state, and any read-only metadata never change on a browser save.
4. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing or malformed `version` → `400`; stale token → `409` with the message "record changed on disk; reload the form" and no write; matching token proceeds.
5. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`; nothing outside the project is read or written.
6. After a successful write the server keeps no cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET` (save-then-reread).
7. No temporary file is left behind after a failed or rejected request.

### 9. Deprecated audit records

1. Deprecated audit records live in the active directory's `deprecated/` subfolder. That folder is never scanned for records, never listed, never counted, and never resolved.
2. A request for a deprecated record's ID returns `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`, exactly as for an unknown ID.
3. No browser control changes deprecation state; deprecation and restoration remain CLI-only (REQ-05.007, BR-014).

### 10. JSON API

1. `GET /api/audit-records` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id": "AUD-001", "title": "Audit title"}`. The alias `/api/audit-record` returns the same response.
2. `GET /api/doc/{id}` for an audit-record ID returns the full record shape:

   ```json
   {
     "id": "AUD-001",
     "kind": "audit-record",
     "form": "audit-record",
     "title": "Audit title",
     "audit_type": "security",
     "content": "Audit content text...",
     "version": "1762000000.123456789.412",
     "path": "audit-records/AUD-001.md"
   }
   ```

   `content` is a string or `null` (null when the section is absent or empty); `audit_type` is always a string (defaulting to `security` when absent); `path` is relative to the project directory with `/` separators and never exposes an absolute path.
3. `POST /api/doc/{id}` requires `Content-Type: application/json` and accepts an object with:
   - `version` (required, string);
   - `title` (required when present, string; omitting it leaves the title unchanged);
   - `audit_type` (optional string or null; when present it must be one of security, licence, architecture; null or omitted leaves the audit type unchanged; an empty string is invalid);
   - `content` (optional string or null; null or empty clears the section).
4. Omitted members are left unchanged. Unknown members are ignored. A non-object body, a wrong member type, or a non-string array element is `400`. A title or field value that fails validation is `422`.
5. A successful `POST` returns `200` with the full record shape of section 10.2, read back from disk after the write.
6. HTML and JSON error responses follow baseline section 5.3.

### 11. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, and no inline scripts/styles.
2. The sidebar's Audit Records entry and the audit-records list/view/edit templates are the only rendering additions; the shared shell and error pages are reused.
3. Tag controls are not applicable; the audit type is a `<select>` with three options and a default, keyboard operable by default. Every input has a `<label>` or accessible name. Focus is not trapped.

### 12. Tests

1. Unit tests cover: record parsing/serialization for audit records (heading, two sections, unknown-section preservation), title extraction, field validation (title, audit type, content), version token, and JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-26a: list renders the Audit Records category, ascending by ID, with the selected sidebar entry marked `aria-current="page"`; `?type=audit-record` works and any other value is `400`.
   - T-26b: non-matching `.md` files and `deprecated/` are not listed.
   - T-26c: view shows ID, form, category, title, audit type (badge and metadata row), content, breadcrumb, and a single Edit action; content has no "Content" heading.
   - T-26d: edit round-trip persists all three fields and is visible through the browser and through `papyrus audit view`.
   - T-26e: editing a record with an unknown/editor-added section changes only the edited fields; a byte comparison shows every other section and line unchanged.
   - T-26f: validation returns `422` and writes nothing for empty title, over-long title, control characters, over-long content, `## ` lines, and invalid audit types.
   - T-26g: stale version token returns `409` and does not write; missing/malformed version returns `400`.
   - T-26h: deprecated audit records are not listed and return `404` on view, edit, POST, and API.
   - T-26i: JSON API returns the documented summary and full shapes, applies the documented POST semantics, and rejects wrong types with `400`/`422`.
   - T-26j: writes are atomic, leave no temporary file, and a symlink escape returns `404`.
   - T-26k: an offline asset scan finds no external origin in list, view, or edit responses.
   - T-26l: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Approved artifact (TASK-49):** [`MOCK-0008`](ui/mockups/MOCK-0008-audit-records-review.html) — `approved` in the mockup registry on 2026-11-07 (UTC). One self-contained review package: review landing, audit-records list/view/edit prototypes, error pages, and 320 px states (17 views). Layout description: [`ui/browser-audit-records-ui.md`](ui/browser-audit-records-ui.md). MOCK-0001…MOCK-0007 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with nine entries, `main`, footer, one `h1` per page.
2. **Audit records list** (`/?type=audit-record`): one section with record count, rows of `ID + title` plus content preview; empty and long-title states.
3. **Audit record view** (`/doc/{id}`): metadata block, title `h1`, audit type badge plus "Audit type" row, content body with `pre-wrap` (no "Content" heading), breadcrumb `Audit Records › ID`, single `Edit`.
4. **Edit form** (`/doc/{id}/edit`): title input, audit type select, content textarea, version annotation, Save/Cancel; pristine, dirty, 422, 409, and 320 px states.
5. **Error pages**: 404 unknown, 404 deprecated, 400, 405, 415, 500 (reused from MOCK-0001).

### Layout conflicts resolved by this specification

1. The UI description (MOCK-0008) shows the audit type in the card at the beginning of the view (badge next to the title/ID plus an "Audit type" row in the metadata block) and removes the "Content" heading, whereas an earlier reading might treat audit type as a standalone section and content as a headed section. This specification adopts the approved revision: audit type in the card at the beginning, content as body text without a "Content" heading.
2. The CLI template (audit-records-management.md) treats `title`, `audit type`, and `content` as required on create, but the browser allows content to be cleared on edit. This specification adopts the UI description's "empty clears" behavior; a cleared content removes the `## Content` section, which the CLI handles as an empty/absent content on subsequent reads.

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `?type=audit-record` renders the category, the default stays `req`, other values are `400`, and the sidebar entry is ordered after Business Rules with `aria-current="page"` and omitted when the type is disabled. |
| AC-2 | The list shows active records only, ascending by ID, with `ID + title` links, content preview, count, empty state, and no non-record or deprecated files. |
| AC-3 | The view shows ID, form, category, title, audit type (badge and metadata row), content (no "Content" heading), breadcrumb, and one Edit action. |
| AC-4 | The edit form matches MOCK-0008: title, audit type select, content textarea, hidden version plus annotation, Save/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript. |
| AC-5 | An edit round-trip persists all three fields and is visible in the browser and through `papyrus audit view`. |
| AC-6 | Editing a record preserves every section and line not being edited; no whole-file regeneration occurs. |
| AC-7 | Title extraction and update follow section 3 and section 8 (heading); unknown sections preserved. |
| AC-8 | Validation returns `422` with the documented messages and writes nothing (title, audit type, content). |
| AC-9 | A missing/malformed version is `400`; a stale version is `409` with the documented message and no write. |
| AC-10 | Deprecated audit records are hidden and return `404` on view, edit, POST, and API. |
| AC-11 | The JSON API returns the documented summary/full shapes, supports the documented POST semantics, and rejects wrong types. |
| AC-12 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-13 | The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable controls. |
| AC-14 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: An audit record's persisted file is a Markdown text record with a fixed section template; browser edits must be field-preserving and must never regenerate the file from a parsed model, so that editor-added and unknown sections survive.
- Assumption: The `## Audit Type` section body is the machine-readable source of the audit type; records created before the section exists default to `security` on read.
- Assumption: The audit type is restricted in the browser edit form to the three built-in values (security, licence, architecture) per approved MOCK-0008. Records created via the CLI with a non-built-in (custom) audit type are viewable (the badge shows the stored value) but cannot be edited in the browser without changing the type to one of the three built-in values.
- Assumption: Duplicate or missing audit type values are handled by defaulting to `security` on read; the edit form always submits one of the three built-in values.
- Assumption: Content may be cleared on edit, removing the `## Content` section, consistent with the UI description and sibling browser increments.
- Assumption: The tag-field progressive-enhancement pattern is not applicable; the audit type uses a native `<select>`, which is operable without JavaScript.
- Assumption: The JSON API exposes the three editable fields; audit records have no status, task, or supersession metadata, unlike decision records, and no requirement references, unlike business rules.
- Assumption: Real-browser verification in Firefox, Edge, and Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with TASK-15/TASK-23/TASK-24/TASK-40/TASK-25.
- Assumption: No new dependencies, crates, or ADRs are introduced.
- Assumption: The canonical business-rule-style `BR-*.md` documents are not applicable; only `<prefix>-<digits>.md` files are records.

## Open questions

- **OQ-1 (non-blocking, product)** — Whether the browser should support editing non-built-in (custom) audit types created via the CLI. This specification restricts the edit select to the three built-in values per MOCK-0008. Supporting custom types would change the edit control (a text input or an extended select) and validation.
- **OQ-2 (non-blocking, layout)** — Whether the audit type badge should use a distinct visual treatment. This specification follows MOCK-0008; the mockup is the layout authority.
- **OQ-3 (non-blocking, follow-up)** — Real-browser verification in Firefox, Edge, and Safari 100+ is deferred, consistent with sibling tickets.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.006, REQ-05.007, REQ-06, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-014 (audit command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-40, TASK-25 (sibling), TASK-47 (record model), TASK-49 (UI design), TASK-26 |
| UI mockups | MOCK-0001…MOCK-0007 (approved baseline); MOCK-0008 (approved) — `docs/specifications/ui/mockups/MOCK-0008-audit-records-review.html` |
| UI description | `docs/specifications/ui/browser-audit-records-ui.md` (TASK-49) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/audit-records-management.md` (TASK-47 record model and lifecycle), `docs/specifications/browser-business-rules.md` (TASK-25 sibling increment), `docs/specifications/browser-ui-specifications.md` (TASK-24 sibling increment), `docs/specifications/browser-ui-mockups.md` (TASK-40 sibling increment) |
| Domain model | `docs/domain-model/business-glossary/audit-record.md` (if present) |