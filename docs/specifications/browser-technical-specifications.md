# Feature Specification — Browser Access for Technical Specifications

**Ticket:** TASK-57
**Milestone:** m-11 (Technical Specifications)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0009`](ui/mockups/MOCK-0009-browser-technical-specifications-review.html) — approved 2026-11-07 (TASK-58); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html), [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html), [`MOCK-0005`](ui/mockups/MOCK-0005-browser-ui-specifications-review.html), [`MOCK-0006`](ui/mockups/MOCK-0006-browser-ui-mockups-review.html), [`MOCK-0007`](ui/mockups/MOCK-0007-business-rules-review.html), and [`MOCK-0008`](ui/mockups/MOCK-0008-audit-records-review.html). UI description: [`ui/browser-technical-specifications-ui.md`](ui/browser-technical-specifications-ui.md).

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), TASK-15 (decision records), TASK-23 (glossary terms), TASK-24 (UI specifications), TASK-40 (UI mockups), TASK-25 (business rules), and TASK-26 (audit records) so it also serves the technical-specifications store: active technical specifications with a title and a description. The transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`tech-spec`), its record model, list/view/edit screens, API endpoints, and tests.

It implements the browser-visibility part of REQ-05.009 and REQ-05.007 under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for technical specifications. Deprecated technical specifications are hidden from lists and unreachable by URL, consistent with deprecated requirements, glossary terms, UI specifications, UI mockups, business rules, and audit records (REQ-05.007, BR-017).

Technical specifications are Markdown text records with a fixed template (`# <ID> <title>`, then `## Description`). The browser edits only the title and the description and must preserve every other byte/section of the file. Whole-file regeneration of a record (for example by reusing the CLI's `render_record` serializer) is forbidden because it destroys editor-added and unknown sections.

## Scope

### In scope

- Extending the persistent sidebar with a "Technical Specifications" entry (immediately after "Audit Records") and the `?type=` category selector to `tech-spec`, without changing the list-page structure (baseline spec section 4.6).
- Listing active technical specifications, rendering only the technical-specifications section, with the first paragraph of the description as a row preview.
- Viewing a technical specification's title and description, with the description rendered as body text without a "Description" heading.
- Editing `title` (required, single-line) and `description` (required, multi-line textarea) through a form matching MOCK-0009, with the baseline optimistic-concurrency and atomic-write contract (ADR-0006).
- A field-preserving write contract that changes only the title heading and the `## Description` section and leaves every other section, line, and byte of the file content-identical.
- A read/write JSON API over the same technical specifications.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated technical specifications are not listed and are not reachable (active only).
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, deprecating, restoring, or deleting technical specifications from the browser; changing record IDs or deprecation state. These remain CLI-only (REQ-05.009, BR-017).
- Cross-type navigation beyond the baseline sidebar, and the final completeness check — TASK-19.
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Structured references to non-functional requirements or tasks; such references remain literal text in the title or description (BR-017 defines no reference fields).
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-59.
- Fixing the CLI's own whole-file rewrite behavior; this ticket fixes only the browser path.
- Changing the default technical-specification directory (`technical/`) or any configuration key.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts `tech-spec` in addition to the existing values `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `business-rule`, and `audit-record`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists the document types in this order: Business requirements, Business Rules, Audit Records, Technical Specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. When `document_types.tech-spec.enabled` is `false`, the Technical Specifications sidebar entry is omitted and `?type=tech-spec` renders the category empty state rather than an error (baseline section 4.4).
5. The list page renders only the selected category; the other categories are not rendered at all.
6. A missing or unreadable technical-specifications directory is not a server error: the category renders its empty state and its API returns an empty array.

### 2. Record sources and IDs

1. Technical specifications are read from `<doc_directory>/<document_types.tech-spec.directory>/` (active folder only; default `docs/technical/`). The directory, prefix, and enabled flag are resolved from the effective configuration and are never hardcoded.
2. Only regular files directly inside the active folder whose name ends in `.md` and whose file stem matches `<prefix>-<digits>` (default prefix `TECH`) are records. The `deprecated/` subfolder, files whose stem does not match, and non-`.md` files are ignored. There is no recursive traversal.
3. The record ID is the file stem, matched exactly and case-sensitively (for example `TECH-001`). The ID is never derived from file content and is never accepted from the request.
4. ID resolution is by exact match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to twelve categories).
5. A technical specification whose file is not valid UTF-8 is skipped by the list and API index and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten (baseline section 7.13).
6. Record form, category, and file path are derived from the matched directory and stem, never from the request.

### 3. Record model

1. The browser record has two editable values: `title` and `description`; the ID is read-only.
2. `title` is read from the text of the first line `# <ID> <title>` (the heading). When the heading contains only the ID, the title is the record ID.
3. `description` is read from the body of the `## Description` section. When the section is absent or its body is empty, the value is empty/`null`; the view then shows `(none)`, and the edit form requires a non-empty value before saving.
4. Unknown and editor-added sections, the preamble, and any content not covered by the two fields are preserved on save and are never displayed as editable fields.
5. The record form is `tech-spec`; the record-form label is `Technical specification`; the category label is `Technical Specifications`.

### 4. List page

1. For the `tech-spec` category the list page renders one section (`h2`) titled "Technical Specifications", showing the record count, followed by one row per record in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Each row also shows the first paragraph of the record's description beneath the primary link; records with empty description show no preview. The preview is plain text (first paragraph only, not the full description).
4. Empty state: "No technical specifications were found in the configured directory. Technical specifications are created from the command line; this browser does not create records. Deprecated technical specifications are hidden." Long-title and unbreakable-token wrapping follow the baseline (section 10.4).
5. Deprecated technical specifications are never listed or counted.

### 5. Record view

1. The view page shows:
   - metadata block: Record ID (read-only, monospace), Record form (`Technical specification`), Category (`Technical Specifications`);
   - the title as the page title (`h1`);
   - the description rendered as body text with `white-space: pre-wrap`, or `(none)` when empty, without a "Description" heading.
2. The breadcrumb is `Technical Specifications` (a link to `/?type=tech-spec`), then the record ID as the current, non-linked crumb.
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.

### 6. Edit form

1. Editable fields:
   - `title` (required, single-line);
   - `description` (required, multi-line textarea).
2. ID, record form, and category are displayed as read-only metadata; they are never accepted from the request.
3. Breadcrumb on the edit page: `Technical Specifications › {id} › Edit`; `Edit` is the current, non-linked crumb. The page heading is `Edit {id}` with the current title shown as read-only context, matching MOCK-0009.
4. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present exactly as in the baseline (section 4.23).
5. The form shows field help text, the pristine state, a dirty state with the "Unsaved changes" indicator, a validation-error state (422), a conflict state (409), and a 320 CSS-pixel responsive layout, matching MOCK-0009.
6. The form submits `application/x-www-form-urlencoded` with `version`, `title`, and `description` fields. Repeated text fields use last-value-wins.
7. No create, deprecate, restore, delete, or status control is present.

### 7. Validation

1. `title` follows baseline section 7.3: trimmed; required; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `description` follows baseline section 7.3/7.4: trimmed; required (a non-empty value after trimming); at most 20000 Unicode scalar values; LF allowed; CR and CRLF normalized to LF; TAB allowed; other control characters rejected; a line whose first two characters are `## ` is rejected. The description is never cleared through the browser.
3. Validation failures return `422`, re-render the edit form with the submitted values preserved, a form-level summary whose entries link to the offending field, and field-level messages using the same text. Nothing is written.
4. Field messages:
   - `Title is required.`
   - `Title must be at most 200 characters.`
   - `Title cannot contain control characters.`
   - `Description is required.`
   - `Description must be at most 20000 characters.`
   - `Description cannot contain control characters.`
   - `Description contains a line starting with ## .`
5. JSON validation failures use the same message text in the baseline JSON error object (`422`).

### 8. Persistence and editing contract

1. The file is read as UTF-8 text. On save, the file is rewritten atomically (baseline section 7.10) with only these substitutions:
   1. the first line is replaced with `# <ID> <title>`;
   2. the `## Description` section keeps its original position when present; a non-empty value replaces that section's body; when the section is absent, it is inserted immediately after the preamble (before the first `## ` section) when a section exists, otherwise appended after the heading, restoring the canonical template order;
   3. every other `## ` section (including editor-added and unknown sections) is written back content-identically, in its original order and position;
   4. the preamble (lines between the heading and the first section) is preserved content-identically;
   5. line endings are normalized to LF and the file ends with a single trailing newline.
2. Every other content of the file is preserved content-identically: doctype, unknown sections, and any text outside the two edited fields. The front matter is the authoritative metadata store: a save PATCHES the front-matter keys it changes (`title`), refreshes `updated_date` (and never changes `created_date`), preserves every other front-matter key with its stored order and quoting, and never copies metadata into the body. The browser never regenerates the file from a parsed record and never reuses the CLI's stub serializer (`render_record`).
3. The ID, file name, directory, and deprecation state never change on a browser save.
4. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing or malformed `version` → `400`; stale token → `409` with the message "record changed on disk; reload the form" and no write; matching token proceeds.
5. The `409` HTML response renders the conflict page from MOCK-0009: "This record changed on disk", "Nothing was saved.", the `409 Conflict` notice, `Reload the form` and `View current record` actions, and a read-only block listing the unsaved title and description. The JSON response uses the baseline error object with the same message.
6. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`; nothing outside the project is read or written.
7. After a successful write the server keeps no cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET` (save-then-reread).
8. No temporary file is left behind after a failed or rejected request.

### 9. Deprecated technical specifications

1. Deprecated technical specifications live in the active directory's `deprecated/` subfolder. That folder is never scanned for records, never listed, never counted, and never resolved.
2. A request for a deprecated technical specification's ID returns `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`, exactly as for an unknown ID.
3. Every HTML `404` response for a technical specification ID renders the technical-specifications `404` page, and that page is identical for a deprecated ID and an unknown ID. It uses the shared error-page layout and shows the `404` status, the heading `Record not found`, the help text "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address.", and exactly one recovery link, "Back to technical specifications", pointing to `/?type=tech-spec`. Because the deprecated and unknown responses are identical in status, heading, help text, and recovery link, the page never reveals whether a hidden ID exists in `deprecated/`.
4. The technical-specifications `404` page applies to a requested ID that matches the configured technical-specification prefix pattern (`<prefix>-<digits>`, default `TECH`) on `GET /doc/{id}`, `GET /doc/{id}/edit`, and `POST /doc/{id}`. JSON `404` responses keep the baseline JSON error object (section 10.6), and an ID that matches no technical-specification pattern keeps the baseline `404` page.
5. No browser control changes deprecation state; deprecation and restoration remain CLI-only (REQ-05.007, BR-017).

### 10. JSON API

1. `GET /api/tech-specs` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id": "TECH-001", "title": "Offline browser architecture"}`. The alias `/api/tech-spec` returns the same response.
2. `GET /api/doc/{id}` for a technical-specification ID returns the full record shape:

   ```json
   {
     "id": "TECH-001",
     "kind": "tech-spec",
     "form": "tech-spec",
     "title": "Offline browser architecture",
     "description": "Defines how the local browser serves project documents...",
     "version": "1762000000.123456789.412",
     "path": "docs/technical/TECH-001.md"
   }
   ```

   `description` is a string or `null` (null when the section is absent or empty); `path` is relative to the project directory with `/` separators and never exposes an absolute path.
3. `POST /api/doc/{id}` requires `Content-Type: application/json` and accepts an object with:
   - `version` (required, string);
   - `title` (required when present, string; omitting it leaves the title unchanged);
   - `description` (required when present, string; omitting it leaves the description unchanged; an explicit `null` or empty string is invalid because the description is required).
4. Omitted members are left unchanged. Unknown members are ignored. A non-object body, a wrong member type, or a non-string array element is `400`. A title or description that fails validation is `422`.
5. A successful `POST` returns `200` with the full record shape of section 10.2, read back from disk after the write.
6. HTML and JSON error responses follow baseline section 5.3.

### 11. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, and no inline scripts/styles.
2. The sidebar's Technical Specifications entry, the technical-specifications list/view/edit/conflict templates, and the technical-specifications `404` page are the only rendering additions; the shared shell and the remaining error pages are reused.
3. Both fields are native `input`/`textarea` controls with a `<label>`; no tag, select, or custom widget is added. The form is keyboard operable and works with JavaScript disabled. Focus is not trapped.

### 12. Tests

1. Unit tests cover: record parsing for technical specifications (heading, `## Description`, unknown-section preservation), title extraction, field validation (title and description), version token, and JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-57a: list renders the Technical Specifications category, ascending by ID, with the selected sidebar entry marked `aria-current="page"`; `?type=tech-spec` works, the default stays `req`, any other value is `400`, and the entry is omitted when the type is disabled.
   - T-57b: non-matching `.md` files and `deprecated/` are not listed; the count and empty state render; long unbreakable titles do not force horizontal scrolling.
   - T-57c: view shows ID, form, category, title, description with `pre-wrap` and no "Description" heading, `(none)` for an empty description, the breadcrumb, and a single Edit action.
   - T-57d: edit round-trip persists title and description and is visible through the browser and through `papyrus tech-spec view`.
   - T-57e: editing a record with an unknown/editor-added section and a preamble changes only the edited fields; a byte comparison shows every other section and line unchanged.
   - T-57f: validation returns `422` and writes nothing for empty/over-long title, control characters, empty/over-long description, and `## ` lines.
   - T-57g: stale version token returns `409` (including the conflict page/actions) and does not write; missing/malformed version returns `400`.
   - T-57h: deprecated technical specifications are not listed or counted and return `404` on view, edit, POST, and API; the HTML `404` body for a deprecated ID and for an unknown technical-specification ID is identical and contains the help text "Deprecated technical specifications are not listed and cannot be opened. The same page is used for an unknown address." and the single recovery link "Back to technical specifications" to `/?type=tech-spec`; JSON `404` responses keep the JSON error object.
   - T-57i: JSON API returns the documented summary and full shapes, applies the documented POST semantics, and rejects wrong types with `400`/`422`.
   - T-57j: writes are atomic, leave no temporary file, and a symlink escape returns `404`.
   - T-57k: an offline asset scan finds no external origin in list, view, or edit responses.
   - T-57l: the edit form and list render without requiring JavaScript, and the 320 px layout keeps controls labelled and stacked.
   - T-57m: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. Layout only — no branding, colour palette, logo, or visual identity.

**Approved artifact (TASK-58):** [`MOCK-0009`](ui/mockups/MOCK-0009-browser-technical-specifications-review.html) — `approved` in the mockup registry on 2026-11-07 (UTC). One self-contained review package: review landing, technical-specifications list/view/edit prototypes, conflict page, error pages, and 320 px states (17 views). Layout description: [`ui/browser-technical-specifications-ui.md`](ui/browser-technical-specifications-ui.md). MOCK-0001…MOCK-0008 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with ten entries, `main`, footer, one `h1` per page.
2. **Technical specifications list** (`/?type=tech-spec`): one section with record count, rows of `ID + title` plus description preview; empty and long-title states.
3. **Technical specification view** (`/doc/{id}`): metadata block, title `h1`, description body with `pre-wrap` (no "Description" heading), breadcrumb `Technical Specifications › ID`, single `Edit`.
4. **Edit form** (`/doc/{id}/edit`): title input, description textarea, version annotation, Save changes/Cancel; pristine, dirty, 422, 409 conflict, and 320 px states.
5. **Error pages**: technical-specifications `404` page for deprecated and unknown IDs (identical page, technical-specifications help text, single "Back to technical specifications" recovery link), and the reused `400`, `405`, `415`, `500` pages from MOCK-0001.

### Layout decisions resolved by this specification

1. The sidebar places Technical Specifications immediately after Audit Records, before Non-functional requirements, per the approved MOCK-0009 and its UI description.
2. The view renders the description as body text without a heading, per MOCK-0009; there is no standalone "Description" section heading.
3. The edit form labels both fields as required and uses "Save changes" / "Cancel", per MOCK-0009.
4. The technical-specifications `404` page renders the same help text and the same recovery link for a deprecated ID and an unknown ID. MOCK-0009's separate unknown and deprecated error screens are unified into one page so the response cannot reveal whether a hidden ID exists in `deprecated/`.

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `?type=tech-spec` renders the category, the default stays `req`, other values are `400`, and the sidebar entry is ordered after Audit Records with `aria-current="page"` and omitted when the type is disabled. |
| AC-2 | The list shows active records only, ascending by ID, with `ID + title` links, description preview, count, empty state, and no non-record or deprecated files. |
| AC-3 | The view shows ID, form, category, title, description (no "Description" heading), breadcrumb, and one Edit action; an empty description shows `(none)`. |
| AC-4 | The edit form matches MOCK-0009: title input, description textarea, hidden version plus annotation, Save changes/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript. |
| AC-5 | An edit round-trip persists title and description and is visible in the browser and through `papyrus tech-spec view`. |
| AC-6 | Editing a record preserves every section and line not being edited; no whole-file regeneration occurs. |
| AC-7 | Title and description extraction/update follow sections 3 and 8; unknown sections and preamble preserved. |
| AC-8 | Validation returns `422` with the documented messages and writes nothing (title and description). |
| AC-9 | A missing/malformed version is `400`; a stale version is `409` with the documented message/conflict page and no write. |
| AC-10 | Deprecated and unknown technical-specification IDs are hidden from lists and counts and return `404` on view, edit, POST, and API; the HTML `404` page is identical for both and renders the technical-specifications help text plus the single "Back to technical specifications" recovery link to `/?type=tech-spec`; JSON `404` keeps the JSON error object. |
| AC-11 | The JSON API returns the documented summary/full shapes, supports the documented POST semantics, and rejects wrong types. |
| AC-12 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-13 | The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable controls. |
| AC-14 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: A technical specification's persisted file is a Markdown text record with the fixed template `# <ID> <title>` plus `## Description`; browser edits are field-preserving and never regenerate the file from a parsed model, so that editor-added and unknown sections survive.
- Assumption: The browser edit form requires both title and description, matching MOCK-0009, `technical-specifications-management.md`, and BR-017; an on-disk record with an absent/empty description is viewable as `(none)` but must be completed before it can be saved from the browser.
- Assumption: The browser description is multi-line and preserves LF; the CLI `--description` remains single-line and `--editor` remains the CLI route for multi-line bodies. This is consistent with `technical-specifications-management.md` section 7.3.
- Assumption: Browser title and description validation reuses the baseline editing limits (title 1–200, multi-line text at most 20000 Unicode scalar values) for consistency with every other browser document type; MOCK-0009's help text does not draw these limits. See OQ-2.
- Assumption: Deprecated technical specifications are excluded exactly like other document types: not listed, not counted, not reachable, not editable (REQ-05.007, BR-017).
- Assumption: The `document_types.tech-spec.enabled` flag controls the sidebar entry and category, like the other document types; a missing directory is an empty category, not an error.
- Assumption: Technical specifications expose no structured NFR/task reference fields in the browser; such IDs remain literal text in the title or description (BR-017 defines no reference fields).
- Assumption: Real-browser verification in Firefox, Edge, and Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with sibling tickets.
- Assumption: No new dependencies, crates, or ADRs are introduced.

## Open questions

- **OQ-1 (non-blocking, product)** — Whether technical specifications should gain structured references to non-functional requirements or tasks in the browser. REQ-05.009 says they "may be referenced" but BR-017 defines no reference fields; this specification treats them as literal text.
- **OQ-2 (non-blocking, validation)** — `technical-specifications-management.md` and BR-017 state no length limit, while the browser baseline applies title 1–200 and multi-line text 20000 limits. This specification adopts the baseline limits for consistency with the other browser document types; MOCK-0009 draws no length help text. If the product prefers no browser length limit, only the documented messages and tests for the two length rules change.
- **OQ-3 (non-blocking, follow-up)** — Real-browser verification in Firefox, Edge, and Safari 100+ is deferred, consistent with sibling tickets.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.009, REQ-05.007, REQ-06, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-016 (configuration keys and precedence), BR-017 (tech-spec command behavior) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-40, TASK-25 (sibling), TASK-26 (sibling), TASK-55 (record model), TASK-58 (UI design), TASK-57 |
| UI mockups | MOCK-0001…MOCK-0008 (approved baseline); MOCK-0009 (approved) — `docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html` |
| UI description | `docs/specifications/ui/browser-technical-specifications-ui.md` (TASK-58) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/technical-specifications-management.md` (TASK-55 record model and lifecycle), `docs/specifications/search-technical-specifications.md` (TASK-56), `docs/specifications/browser-audit-records.md` (TASK-26 sibling increment), `docs/specifications/browser-business-rules.md` (TASK-25 sibling increment) |
| Domain model | `docs/domain-model/business-glossary/technical-specification.md` |
