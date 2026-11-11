# Feature Specification — Browser Access for Use Cases, Test Cases, and Test Reports

**Ticket:** TASK-81
**Milestone:** m-15 (Use Cases and Test Documentation)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0013`](ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html) — `approved` on 2026-11-11 (UTC) (TASK-79). UI description: [`ui/browser-use-cases-test-cases-and-test-reports-ui.md`](ui/browser-use-cases-test-cases-and-test-reports-ui.md). Extends the approved baseline MOCK-0001…MOCK-0012.

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), TASK-15 (decision records), TASK-23 (glossary terms), TASK-24 (UI specifications), TASK-40 (UI mockups), TASK-25 (business rules), TASK-26 (audit records), TASK-57 (technical specifications), TASK-66/TASK-67 (front-matter metadata projection), and TASK-72 (managed artifacts) so it also serves three documentation stores — use cases, test cases, and test reports — each with a title, a description, and an optional applicability, plus the general front-matter metadata (`id`, `title`, `created_date`, `updated_date`, `labels`, `references`). The transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused unchanged; this increment adds three documentation categories (`use-case`, `test-case`, `test-report`), their record models, list/view/edit screens, API endpoints, and tests. It implements the browser-visibility part of REQ-05.012, REQ-05.013, REQ-05.014 and REQ-05.007 under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for these three types. Deprecated records of all three types are hidden from lists and unreachable by URL, consistent with every other document type (REQ-05.007, BR-018/BR-019/BR-020).

Use cases, test cases, and test reports are Markdown text records with the fixed template `# <ID> <title>`, then `## Description`, then, only when applicability is provided, `## Applicability`. The browser edits the title, the description, and the applicability, and manages the front-matter `labels` and `references` lists, and must preserve every other byte/section of the file. Whole-file regeneration of a record (for example by reusing the CLI's `render_record` serializer) is forbidden because it destroys editor-added and unknown sections.

## Scope

### In scope

- Extending the persistent sidebar with three entries — "Use Cases", "Test Cases", and "Test Reports" — placed immediately after "Technical Specifications", and the `?type=` category selector to `use-case`, `test-case`, and `test-report`, without changing the list-page structure (baseline spec section 4.6).
- Listing active records of each type, rendering only the selected category, with the first paragraph of the description as a row preview and a metadata line (created date, updated date, labels, references).
- Viewing a record's title, description, applicability, and front-matter metadata, with the description and applicability rendered as body text.
- Editing `title` (required, single-line), `description` (required, multi-line textarea), `applicability` (optional, multi-line textarea), and the front-matter `labels` and `references` lists (tags with add/remove) through forms matching MOCK-0013, with the baseline optimistic-concurrency and atomic-write contract (ADR-0006).
- A field-preserving write contract that changes only the title heading, the `## Description` and `## Applicability` sections, and the front-matter `title`/`labels`/`references` keys, and leaves every other section, line, and byte of the file content-identical.
- A read/write JSON API over the same records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated records of all three types are not listed and are not reachable (active only).
- Front-matter metadata projection (id, title, created_date, updated_date, labels, references) on list, view, and edit, reusing the shared `RecordView` projection (TASK-67).
- Unit and integration tests extending `tests/browser_cli.rs` for the three new categories.

### Out of scope

- Creating, deprecating, restoring, or deleting records from the browser; changing record IDs or deprecation state. These remain CLI-only (REQ-05.012/013/014, BR-018/019/020).
- Cross-type navigation beyond the baseline sidebar, and the final completeness check — TASK-19/TASK-82.
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-82.
- Fixing the CLI's own whole-file rewrite behavior; this ticket fixes only the browser path.
- Changing the default directories (`use-cases/`, `test-cases/`, `test-reports/`) or any configuration key.
- Dedicated test-case-reference or outcome fields in test reports (BR-020 leaves these to the body/`references`).

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts `use-case`, `test-case`, and `test-report` in addition to the existing values `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `business-rule`, `audit-record`, and `tech-spec`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists the document types in this order: Business requirements, Business Rules, Audit Records, Technical Specifications, Use Cases, Test Cases, Test Reports, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups, Managed artifacts & lifecycles, Agent skills & workflows. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. When `document_types.use-case.enabled`, `document_types.test-case.enabled`, or `document_types.test-report.enabled` is `false`, the corresponding sidebar entry is omitted and `?type=use-case` / `?type=test-case` / `?type=test-report` renders that category's empty state rather than an error (baseline section 4.4).
5. The list page renders only the selected category; the other categories are not rendered at all.
6. A missing or unreadable directory for any of the three types is not a server error: the category renders its empty state and its API returns an empty array.

### 2. Record sources and IDs

1. Records are read from `<doc_directory>/<document_types.<type>.directory>/` (active folder only; defaults `use-cases/`, `test-cases/`, `test-reports/`). The directory, prefix, and enabled flag are resolved from the effective configuration and are never hardcoded.
2. Only regular files directly inside the active folder whose name ends in `.md` and whose file stem matches `<prefix>-<digits>` (default prefixes `UC`, `TC`, `TR`) are records. The `deprecated/` subfolder, files whose stem does not match, and non-`.md` files are ignored. There is no recursive traversal.
3. The record ID is the file stem, matched exactly and case-sensitively (for example `UC-001`, `TC-001`, `TR-001`). The ID is never derived from file content and is never accepted from the request.
4. ID resolution is by exact match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to the full category set).
5. A record whose file is not valid UTF-8 is skipped by the list and API index and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten (baseline section 7.13).
6. Record form, category, and file path are derived from the matched directory and stem, never from the request.

### 3. Record model

1. The browser record has three editable body values — `title`, `description`, and `applicability` — plus two editable front-matter lists — `labels` and `references`; the ID is read-only.
2. `title` is read from the text of the first line `# <ID> <title>` (the heading). When the heading contains only the ID, the title is the record ID.
3. `description` is read from the body of the `## Description` section. When the section is absent or its body is empty, the value is empty/`null`; the view then shows `(none)`, and the edit form requires a non-empty value before saving.
4. `applicability` is read from the body of the `## Applicability` section. When the section is absent or its body is empty, the value is empty/`null`.
5. Unknown and editor-added sections, the preamble, and any content not covered by the two body fields are preserved on save and are never displayed as editable fields.
6. The front-matter metadata (`id`, `title`, `created_date`, `updated_date`, `labels`, `references`) is projected through the shared `RecordView` (TASK-67): `id` is the storage ID, `title` is the display title, `created_date` and `updated_date` are ISO-8601 UTC strings as stored, and `labels`/`references` are string arrays.
7. The record form labels are `Use case`, `Test case`, and `Test report`; the category labels are `Use Cases`, `Test Cases`, and `Test Reports`.

### 4. List page

1. For each of the three categories the list page renders one section (`h2`) titled with the type label, showing the record count, followed by one row per record in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Each row also shows the first paragraph of the record's description beneath the primary link; records with an empty description show no preview. The preview is plain text (first paragraph only, not the full description).
4. Each row shows a metadata line: created date, updated date, labels, references (front-matter projection).
5. Empty state: "No <type> were found in the configured directory. <Type> are created from the command line; this browser does not create records. Deprecated <type> are hidden." (with `<type>` as `use cases` / `test cases` / `test reports`).
6. Long-title and unbreakable-token wrapping follow the baseline (section 10.4).
7. Deprecated records of all three types are never listed or counted.

### 5. Record view

1. The view page shows:
   - metadata block: Record ID (read-only, monospace), Title, Created date, Updated date, Labels, References (front-matter projection);
   - the title as the page title (`h1`);
   - the description rendered as body text with `white-space: pre-wrap`, or `(none)` when empty, without a "Description" heading;
   - the applicability, when present, rendered as a section headed "Applicability" with `white-space: pre-wrap`.
2. The breadcrumb is `<Type plural>` (a link to `/?type=<category>`), then the record ID as the current, non-linked crumb. (`<Type plural>` is `Use Cases`, `Test Cases`, or `Test Reports`.)
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.
4. A deprecated ID renders the matching `404` page instead of the view.

### 6. Edit form

1. Editable fields:
   - `title` (required, single-line);
   - `description` (required, multi-line textarea);
   - `applicability` (optional, multi-line textarea);
   - `labels` (optional, front-matter list, tag add/remove);
   - `references` (optional, front-matter list, tag add/remove).
2. ID, record form, and category are displayed as read-only metadata; they are never accepted from the request.
3. Breadcrumb on the edit page: `<Type plural> › {id} › Edit`; `Edit` is the current, non-linked crumb. The page heading is `Edit {id}` with the current title shown as read-only context, matching MOCK-0013.
4. The front-matter metadata block shows Record ID, Created date, Updated date (read-only inputs), Labels, and References (tags with add input).
5. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present exactly as in the baseline (section 4.23).
6. The form shows field help text, the pristine state, a dirty state with the "Unsaved changes" indicator, a validation-error state (422), a conflict state (409), and a 320 CSS-pixel responsive layout, matching MOCK-0013.
7. The form submits `application/x-www-form-urlencoded` with `version`, `title`, `description`, `applicability`, and repeated `labels` and `references` fields (one occurrence per tag, in display order). Repeated text fields use last-value-wins; tag fields collect every occurrence.
8. No create, deprecate, restore, delete, or status control is present.

### 7. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `description` follows baseline section 7.3/7.4: trimmed; required (a non-empty value after trimming); at most 20000 Unicode scalar values; LF allowed; CR and CRLF normalized to LF; TAB allowed; other control characters rejected; a line whose first two characters are `## ` is rejected.
3. `applicability` follows the same rules as `description` except that it is optional: an empty value clears the `## Applicability` section.
4. `labels`:
   - optional; an empty list is valid;
   - each value is trimmed;
   - each value must be 1–200 Unicode scalar values with no control characters;
   - duplicate values are collapsed (first occurrence retained).
5. `references`:
   - optional; an empty list is valid;
   - each value is trimmed;
   - each value must be at most 200 Unicode scalar values with no control characters and no whitespace;
   - duplicate values are rejected.
6. Validation failures return `422`, re-render the edit form with the submitted values preserved, a form-level summary whose entries link to the offending field, and field-level messages using the same text. Nothing is written.
7. Field messages:
   - `Title is required.`
   - `Title must be at most 200 characters.`
   - `Title cannot contain control characters.`
   - `Description is required.`
   - `Description must be at most 20000 characters.`
   - `Description cannot contain control characters.`
   - `Description contains a line starting with ## .`
   - `Applicability must be at most 20000 characters.`
   - `Applicability cannot contain control characters.`
   - `Applicability contains a line starting with ## .`
   - `Labels must be at most 200 characters.`
   - `Labels cannot contain control characters.`
   - `References must be at most 200 characters.`
   - `References cannot contain control characters.`
   - `References cannot contain whitespace.`
   - `Duplicate reference: <value>.`
8. JSON validation failures use the same message text in the baseline JSON error object (`422`).

### 8. Persistence and editing contract

1. The file is read as UTF-8 text. On save, the file is rewritten atomically (baseline section 7.10) with only these substitutions:
   1. the first line is replaced with `# <ID> <title>`;
   2. the `## Description` section keeps its original position when present; a non-empty value replaces that section's body; when the section is absent it is inserted immediately after the preamble (before the first `## ` section) when a section exists, otherwise appended after the heading, restoring the canonical template order. An empty value is invalid for `description` and is never written;
   3. the `## Applicability` section keeps its original position when present; a non-empty value replaces that section's body; an empty value removes the section; a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end of the file;
   4. every other `## ` section (including editor-added and unknown sections) is written back content-identically, in its original order and position;
   5. the preamble (lines between the heading and the first section) is preserved content-identically;
   6. line endings are normalized to LF and the file ends with a single trailing newline.
2. The front matter is the authoritative metadata store: a save PATCHES the front-matter keys it changes (`title`, `labels`, `references`), refreshes `updated_date` (and never changes `created_date`), preserves every other front-matter key with its stored order and quoting, and never copies metadata into the body. `labels` is written as a YAML list; an empty list is written as `[]`. `references` is written the same way. The browser never regenerates the file from a parsed record and never reuses the CLI's stub serializer (`render_record`).
3. The ID, file name, directory, deprecation state, and any read-only metadata never change on a browser save.
4. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing or malformed `version` → `400`; stale token → `409` with the message "record changed on disk; reload the form" and no write; matching token proceeds.
5. The `409` HTML response renders the conflict page from MOCK-0013: "This record changed on disk", "Nothing was saved.", the `409 Conflict` notice, `Reload the form` and `View current record` actions, and a read-only block listing the unsaved title, description, applicability, labels, and references. The JSON response uses the baseline error object with the same message.
6. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`; nothing outside the project is read or written.
7. After a successful write the server keeps no cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET` (save-then-reread).
8. No temporary file is left behind after a failed or rejected request.

### 9. Deprecated records

1. Deprecated records of all three types live in the active directory's `deprecated/` subfolder. That folder is never scanned for records, never listed, never counted, and never resolved.
2. A request for a deprecated ID returns `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`, exactly as for an unknown ID.
3. Every HTML `404` response for an ID matching the configured prefix pattern (`<prefix>-<digits>`) renders the matching type's `404` page, and that page is identical for a deprecated ID and an unknown ID. It uses the shared error-page layout and shows the `404` status, the heading `Record not found`, the help text "Deprecated <type> are not listed and cannot be opened. The same page is used for an unknown address.", and exactly one recovery link, "Back to <type plural>", pointing to `/?type=<category>`. Because the deprecated and unknown responses are identical in status, heading, help text, and recovery link, the page never reveals whether a hidden ID exists in `deprecated/`.
4. JSON `404` responses keep the baseline JSON error object (section 10.6), and an ID that matches no pattern for any of the three types keeps the baseline `404` page.
5. No browser control changes deprecation state; deprecation and restoration remain CLI-only (REQ-05.007, BR-018/BR-019/BR-020).

### 10. JSON API

1. `GET /api/use-cases` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id": "UC-001", "title": "Sign in"}`. The alias `/api/use-case` returns the same response. `GET /api/test-cases` (alias `/api/test-case`) and `GET /api/test-reports` (alias `/api/test-report`) behave identically for their types.
2. `GET /api/doc/{id}` for a record of one of the three types returns the full record shape:

   ```json
   {
     "id": "UC-001",
     "kind": "use-case",
     "form": "use-case",
     "title": "Sign in",
     "description": "A user signs in with ...",
     "applicability": "Applies to all clients ...",
     "labels": ["auth"],
     "references": ["REQ-05.001"],
     "created_date": "2026-11-10T20:16:00Z",
     "updated_date": "2026-11-11T08:15:00Z",
     "version": "1762000000.123456789.412",
     "path": "use-cases/UC-001.md",
     "metadata": {
       "id": "UC-001",
       "title": "Sign in",
       "created_date": "2026-11-10T20:16:00Z",
       "updated_date": "2026-11-11T08:15:00Z",
       "labels": ["auth"],
       "references": ["REQ-05.001"]
     }
   }
   ```

   `description` and `applicability` are strings or `null` (null when the section is absent or empty); `labels` and `references` are arrays (empty when absent); `path` is relative to the project directory with `/` separators and never exposes an absolute path. `metadata` is the object defined by TASK-67 (REQ-06.004), whose keys are the front-matter keys in canonical order. For `test-case` records `kind`/`form` are `test-case`; for `test-report` records they are `test-report`.
3. `POST /api/doc/{id}` requires `Content-Type: application/json` and accepts an object with:
   - `version` (required, string);
   - `title` (required when present, string; omitting it leaves the title unchanged);
   - `description` (optional string or null; null or empty is invalid because the description is required);
   - `applicability` (optional string or null; null or empty clears the section);
   - `labels` (optional array of strings or null; null clears the list);
   - `references` (optional array of strings or null; null clears the list).
4. Omitted members are left unchanged. Unknown members are ignored. A non-object body, a wrong member type, or a non-string array element is `400`. A title or field value that fails validation is `422`.
5. A successful `POST` returns `200` with the full record shape of section 10.2, read back from disk after the write.
6. HTML and JSON error responses follow baseline section 5.3.

### 11. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, and no inline scripts/styles.
2. The sidebar's three new entries, the three list/view/edit templates, the three `404` pages, and the front-matter metadata rows are the only rendering additions; the shared shell and the remaining error pages are reused.
3. Labels and references are rendered as tags in the view and edited with the tag add/remove interaction (reusing the MOCK-0002 pattern). Both fields are keyboard operable; every input has a visible label or accessible name; remove controls are labelled `Remove <value>`; the add input is labelled. Focus is not trapped.
4. Both body fields are native `input`/`textarea` controls with a `<label>`; no tag, select, or custom widget is added for them. The form is keyboard operable and works with JavaScript disabled. Focus is not trapped.
5. The form is fully usable without JavaScript: tag fields submit one occurrence per value, with the trailing add input empty, so the no-JavaScript fallback preserves the approved chip layout.

### 12. Tests

1. Unit tests cover: record parsing for the three types (heading, `## Description`, `## Applicability`, unknown-section preservation), title extraction, field validation (title, description, applicability, labels, references), duplicate handling for labels (collapse) and references (reject), version token, JSON shapes, and front-matter metadata projection.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the three new categories, including at least:
   - T-81a: list renders each category, ascending by ID, with the selected sidebar entry marked `aria-current="page"`; `?type=use-case` / `?type=test-case` / `?type=test-report` work, the default stays `req`, any other value is `400`, and each entry is omitted when its type is disabled.
   - T-81b: non-matching `.md` files and `deprecated/` are not listed; the count and empty state render; long unbreakable titles do not force horizontal scrolling.
   - T-81c: view shows ID, Title, Created date, Updated date, Labels, References, title `h1`, description with `pre-wrap` and no "Description" heading, `(none)` for an empty description, applicability section when present, breadcrumb, and a single Edit action.
   - T-81d: edit round-trip persists title, description, applicability, labels, and references and is visible through the browser and through `papyrus use-case view` / `papyrus test-case view` / `papyrus test-report view`.
   - T-81e: editing a record with an unknown/editor-added section and a preamble changes only the edited fields; a byte comparison shows every other section and line unchanged.
   - T-81f: validation returns `422` and writes nothing for empty/over-long title, control characters, empty/over-long description, `## ` lines, empty applicability-clearance conflicts, invalid labels, invalid references, and duplicate references.
   - T-81g: stale version token returns `409` (including the conflict page/actions) and does not write; missing/malformed version returns `400`.
   - T-81h: deprecated records of all three types are not listed or counted and return `404` on view, edit, POST, and API; the HTML `404` body for a deprecated ID and for an unknown ID of the same type is identical and contains the help text "Deprecated <type> are not listed and cannot be opened. The same page is used for an unknown address." and the single recovery link "Back to <type plural>" to `/?type=<category>`; JSON `404` responses keep the JSON error object.
   - T-81i: JSON API returns the documented summary and full shapes (including `metadata`), applies the documented POST semantics, and rejects wrong types with `400`/`422`.
   - T-81j: writes are atomic, leave no temporary file, and a symlink escape returns `404`.
   - T-81k: an offline asset scan finds no external origin in list, view, or edit responses.
   - T-81l: the edit form and list render without requiring JavaScript, and the 320 px layout keeps controls labelled and stacked.
   - T-81m: front-matter metadata projection matches the CLI views for all three types (labels, references, created_date, updated_date).
   - T-81n: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Approved artifact (TASK-79):** [`MOCK-0013`](ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html) — `approved` in the mockup registry on 2026-11-11 (UTC). One self-contained review package: review landing, use-case/test-case/test-report list/view/edit prototypes, error pages, and 320 px states (39 views). Layout description: [`ui/browser-use-cases-test-cases-and-test-reports-ui.md`](ui/browser-use-cases-test-cases-and-test-reports-ui.md). MOCK-0001…MOCK-0012 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with fifteen entries, `main`, footer, one `h1` per page.
2. **Use cases list** (`/?type=use-case`): one section with record count, rows of `ID + title` plus description preview and metadata line; empty and long-title states.
3. **Use case view** (`/doc/{id}`): metadata block, title `h1`, description body with `pre-wrap`, applicability section, breadcrumb `Use Cases › ID`, single `Edit`.
4. **Use case edit form** (`/doc/{id}/edit`): title input, description textarea, applicability textarea, front-matter metadata block (ID, created date, updated date read-only; labels and references tags), hidden version plus annotation; pristine, dirty, 422, 409, and 320 px states.
5. **Test cases list/view/edit** and **Test reports list/view/edit**: same structure as use cases, with their respective labels.
6. **Error pages**: 404 unknown, 404 deprecated, 400, 405, 415, 500 (reused from MOCK-0001, with type-specific help text and recovery links).

### Layout decisions resolved by this specification

1. The sidebar places Use Cases, Test Cases, and Test Reports immediately after Technical Specifications, per the approved MOCK-0013 and its UI description (OQ-1).
2. The category values are `use-case`, `test-case`, and `test-report`; any other value is `400` (OQ-2).
3. List rows show the first paragraph of the description as a preview, consistent with MOCK-0009 (OQ-3).
4. Field rules are limited to those stated in the management specification and BR-018/BR-019/BR-020: title and description are trimmed, required, and reject control characters; applicability is optional multi-line (OQ-4).
5. All three types demonstrate front-matter metadata projection (id, title, created_date, updated_date, labels, references) in list, detail, and edit, because this ticket is specifically about them (OQ-5).

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `?type=use-case`, `?type=test-case`, and `?type=test-report` render their categories, the default stays `req`, other values are `400`, and the three sidebar entries are ordered after Technical Specifications with `aria-current="page"` and omitted when their type is disabled. |
| AC-2 | Each list shows active records only, ascending by ID, with `ID + title` links, description preview, metadata line, count, empty state, and no non-record or deprecated files. |
| AC-3 | Each view shows ID, Title, Created date, Updated date, Labels, References, title `h1`, description (no "Description" heading), applicability section when present, breadcrumb, and one Edit action; an empty description shows `(none)`. |
| AC-4 | Each edit form matches MOCK-0013: title input, description textarea, applicability textarea, front-matter metadata block (ID, created date, updated date read-only; labels and references tags), hidden version plus annotation, Save/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript. |
| AC-5 | An edit round-trip persists title, description, applicability, labels, and references and is visible in the browser and through `papyrus use-case view` / `papyrus test-case view` / `papyrus test-report view`. |
| AC-6 | Editing a record preserves every section and line not being edited; no whole-file regeneration occurs. |
| AC-7 | Title, description, and applicability extraction/update follow sections 3 and 8; unknown sections and preamble preserved. |
| AC-8 | Validation returns `422` with the documented messages and writes nothing (title, description, applicability, labels, references). |
| AC-9 | A missing/malformed version is `400`; a stale version is `409` with the documented message/conflict page and no write. |
| AC-10 | Deprecated records of all three types are hidden from lists and counts and return `404` on view, edit, POST, and API; the HTML `404` page is identical for a deprecated ID and an unknown ID of the same type and renders the type-specific help text plus the single "Back to <type plural>" recovery link to `/?type=<category>`; JSON `404` keeps the JSON error object. |
| AC-11 | The JSON API returns the documented summary/full shapes (including `metadata`), supports the documented POST semantics, and rejects wrong types. |
| AC-12 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-13 | The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable controls. |
| AC-14 | Front-matter metadata projection matches the CLI views for all three types. |
| AC-15 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds three categories and record types only.
- Assumption: A record's persisted file is a Markdown text record with the fixed template `# <ID> <title>` plus `## Description` and an optional `## Applicability`; browser edits are field-preserving and never regenerate the file from a parsed model, so that editor-added and unknown sections survive.
- Assumption: The browser edit form requires both title and description, matching MOCK-0013 and the management specification; an on-disk record with an absent/empty description is viewable as `(none)` but must be completed before it can be saved from the browser.
- Assumption: The browser description and applicability are multi-line and preserve LF; the CLI `--description`/`--applicability` remain single-line and `--editor` remains the CLI route for multi-line bodies. This is consistent with the management specification section 7.
- Assumption: Browser title and description validation reuses the baseline editing limits (title 1–200, multi-line text at most 20000 Unicode scalar values) for consistency with every other browser document type; MOCK-0013 draws no length help text.
- Assumption: Deprecated records of all three types are excluded exactly like other document types: not listed, not counted, not reachable, not editable (REQ-05.007, BR-018/BR-019/BR-020).
- Assumption: The `document_types.<type>.enabled` flag controls the sidebar entry and category for each type, like the other document types; a missing directory is an empty category, not an error.
- Assumption: Labels are collapsed on duplicate, matching the task-label handling; references reject duplicates, matching the UI-mockup reference handling.
- Assumption: Front-matter metadata projection reuses the shared `RecordView` (TASK-67); `created_date` and `updated_date` are read-only in the browser and `updated_date` is refreshed only when a save changes at least one byte.
- Assumption: Real-browser verification in Firefox, Edge, and Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with sibling tickets.
- Assumption: No new dependencies, crates, or ADRs are introduced.

## Open questions

- **OQ-1 (non-blocking, product)** — Whether use cases, test cases, and test reports should gain structured references to tasks or milestones in the browser. REQ-05.012/013/014 leave association to tasks/milestones undecided; this specification treats them as literal text in the general `references` list.
- **OQ-2 (non-blocking, validation)** — The management specification and BR-018/019/020 state no length limit for title/description/applicability, while the browser baseline applies title 1–200 and multi-line text 20000 limits. This specification adopts the baseline limits for consistency with the other browser document types; MOCK-0013 draws no length help text. If the product prefers no browser length limit, only the documented messages and tests for the two length rules change.
- **OQ-3 (non-blocking, follow-up)** — Real-browser verification in Firefox, Edge, and Safari 100+ is deferred, consistent with sibling tickets.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.007, REQ-05.010, REQ-05.012, REQ-05.013, REQ-05.014, REQ-06, REQ-06.002, REQ-06.004 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-016 (configuration keys and precedence), BR-018 (use-case document type), BR-019 (test-case document type), BR-020 (test-report document type) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0007, ADR-0009 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-25, TASK-26, TASK-40, TASK-57, TASK-58, TASK-66, TASK-67, TASK-72, TASK-78, TASK-79, TASK-81 |
| UI mockups | MOCK-0001…MOCK-0012 (approved baseline); MOCK-0013 (approved) — `docs/specifications/ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html` |
| UI description | `docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md` (TASK-79) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/use-cases-test-cases-and-test-reports-management.md` (TASK-78 record model and lifecycle), `docs/specifications/search-use-cases-test-cases-and-test-reports.md` (TASK-80), `docs/specifications/browser-technical-specifications.md` (TASK-57 sibling increment), `docs/specifications/browser-audit-records.md` (TASK-26 sibling increment), `docs/specifications/browser-business-rules.md` (TASK-25 sibling increment), `docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md` (TASK-67) |
| Domain model | `docs/domain-model/business-glossary/` (use case, test case, test report terms if present) |
