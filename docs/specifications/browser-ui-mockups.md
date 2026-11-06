# Feature Specification — Browser Access for UI Mockups

**Ticket:** TASK-40
**Milestone:** m-7 (UI Mockups)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0006`](ui/mockups/MOCK-0006-browser-ui-mockups-review.html) — approved 2026-11-06 (TASK-41); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html), [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html), and [`MOCK-0005`](ui/mockups/MOCK-0005-browser-ui-specifications-review.html). UI description: [`ui/browser-ui-mockups-ui.md`](ui/browser-ui-mockups-ui.md).

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), TASK-15 (decision records), TASK-23 (glossary terms), and TASK-24 (UI specifications) so it also serves the UI-mockup store: active UI mockups. The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`ui-mockup`), its record model, list/view/edit screens, API endpoints, and tests. It implements REQ-05.008 and the web-visibility part of REQ-05.007 for the browser surface under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for UI mockups. Deprecated mockups are hidden from lists and unreachable by URL, exactly as deprecated UI specifications and glossary terms.

UI mockups differ from the sibling document types in one material way: the persisted record is a self-contained HTML review package authored by `@ui-designer`, not a Markdown document with sections. The browser edits only the mockup metadata (title, UI specification ids, references) and must preserve every other byte of the package. Whole-file regeneration of a mockup (for example by reusing the CLI's stub serializer) is forbidden because it destroys the design artifact.

## Scope

### In scope

- Extending the persistent sidebar with a "UI Mockups" entry (after "UI Specifications") and the `?type=` category selector to `ui-mockup`, without changing the list-page structure (baseline spec section 4.6).
- Listing active UI mockups, rendering only the UI-mockup section.
- Viewing a UI mockup's metadata (ID, title, UI specifications, references) through the browser's own templates; the stored HTML package is never served or embedded.
- Editing `title`, `specs`, and `refs` through a tag add/remove form matching MOCK-0006, with the baseline optimistic-concurrency and atomic-write contract (ADR-0006).
- A field-preserving write contract that changes only the mockup metadata lines and leaves the rest of the HTML package byte-identical.
- A read/write JSON API over the same mockup records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated UI mockups are not listed and are not reachable (active only).
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, deprecating, restoring, superseding, or deleting mockups from the browser; changing record IDs, status, task, or supersession. These remain CLI-only (`ui-mockup`, TASK-38).
- Rendering the mockup's HTML package in the browser (no raw-file route, no iframe/embed, no preview of the review package). The browser shows metadata only.
- Editing the mockup's design content (styles, scripts, body markup, slide text).
- Managing the mockups registry `docs/specifications/ui/mockups/README.md` from the browser.
- Browsing business rules, audit records, deployment records, or technical specifications — later tickets.
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-42.
- Fixing the CLI's own whole-file rewrite behavior (see Open question OQ-3); this ticket fixes only the browser path.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts `ui-mockup` in addition to the existing values `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, and `ui-spec`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists the document types in this order: Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications, UI Mockups. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. When `document_types.ui-mockup.enabled` is `false`, the UI Mockups sidebar entry is omitted and `?type=ui-mockup` renders the category empty state rather than an error (baseline section 4.4).
5. The list page renders only the selected category; the other categories are not rendered at all.
6. A missing or unreadable mockup directory is not a server error: the category renders its empty state and its API returns an empty array.

### 2. Record sources and IDs

1. Mockups are read from `<doc_directory>/<document_types.ui-mockup.directory>/` (active folder only; default `docs/specifications/ui/mockups/`). The directory, prefix, and enabled flag are resolved from the effective configuration and are never hardcoded.
2. Only regular files directly inside the active folder whose name ends in `.html` and whose file stem matches `<prefix>-<digits>` (default prefix `MOCK`) are records. The `deprecated/` subfolder, `README.md`, `wireframe.css`, `index.html`, and every non-matching file are ignored. There is no recursive traversal.
3. The record ID is the file stem, matched exactly and case-sensitively (for example `MOCK-006`). The ID is never derived from file content and is never accepted from the request.
4. ID resolution is by exact match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to nine categories).
5. A mockup whose file is not valid UTF-8 is skipped by the list and API index and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten (baseline section 7.13).
6. Record form, category, and file path are derived from the matched directory and stem, never from the request.

### 3. Record model and metadata source

1. The browser record has three editable values: `title`, `specs` (ordered list of UI specification ids), and `refs` (ordered list of document references); the ID is read-only.
2. `title` is read from the text of the document's `<title>` element:
   1. when the text contains an em dash (`—`, U+2014) or the character reference `&mdash;`, the title is the text after the first occurrence, trimmed;
   2. otherwise, when the text starts with the record ID followed by whitespace, the title is the remainder, trimmed;
   3. otherwise the title is the whole `<title>` text, trimmed;
   4. when the `<title>` element is absent or its text is empty, the title is the record ID.
3. Standard HTML character references in the extracted title (`&amp;`, `&lt;`, `&gt;`, `&quot;`, `&#39;`, `&mdash;`) are decoded for display. No other HTML parsing is performed.
4. `specs` and `refs` are read from the HTML comment lines `<!-- specs: ... -->` and `<!-- refs: ... -->` when present. The comment value is split on Unicode whitespace, each token trimmed, empty tokens dropped, and the order preserved. When the comment line is absent the value is an empty list.
5. Visible lines (`<p>UI specifications: ...</p>`, `<p>References: ...</p>`) are never used as the metadata source; they are updated only when already present (section 8).
6. `task`, `status`, and `superseded_by` comment lines are read-only for the browser, are not displayed on any browser page, and are never written by the browser. The approved UI shows no status, task, or supersession indication (MOCK-0006 OQ-3).
7. The mockups registry `README.md` is never read as record metadata and is never written by the browser.

### 4. List page

1. For the `ui-mockup` category the list page renders one section (`h2`) titled "UI Mockups", showing the record count, followed by one row per mockup in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Rows show no status, task, supersession, or metadata preview (MOCK-0006 OQ-3).
4. Empty state and long-title wrapping follow the baseline (section 10.4).
5. Deprecated mockups are never listed or counted.

### 5. Record view

1. The view page shows:
   - metadata block: Record ID (read-only, monospace), Record form (`UI mockup`), Category (`UI Mockups`);
   - the title as the page title (`h1`);
   - a "UI specifications" section and a "References" section, each rendering its values as text with `white-space: pre-wrap`, values separated by a single space, and `(none)` when the list is empty.
2. The breadcrumb is `UI Mockups` (a link to `/?type=ui-mockup`), then the record ID as the current, non-linked crumb.
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.
4. The stored HTML package is never served, inlined, iframed, or embedded on any browser page. Only escaped metadata values are rendered, so the package's scripts, styles, and markup cannot execute in the browser interface.

### 6. Edit form

1. Editable fields:
   - `title` (required, single-line);
   - `specs` (required, one or more UI specification ids);
   - `refs` (optional, zero or more document references).
2. ID and record form are displayed as read-only metadata; category is shown as read-only context; task, status, and supersession are not shown. None of these values is accepted from the request.
3. Breadcrumb on the edit page: `UI Mockups › {id} › Edit`; `Edit` is the current, non-linked crumb.
4. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present exactly as in the baseline (section 4.23).
5. The form shows field help text, the pristine state, a dirty state with the "Unsaved changes" indicator, a validation-error state (422), a conflict state (409), and a 320 CSS-pixel responsive layout, matching MOCK-0006.
6. **Tag field interaction (MOCK-0006, MOCK-0002 pattern).** `specs` and `refs` are edited as tags:
   1. each current value is shown as one tag in a tag box, with a remove control whose accessible name is `Remove <value>`;
   2. an add line below the tag box contains a text input and an `Add` button; activating `Add` appends the trimmed input value as a new tag and clears the add input; an empty or whitespace-only value is not added;
   3. the remove control removes that tag;
   4. tag order is the stored order; adding appends at the end;
   5. the interaction is implemented with progressive enhancement only: the same markup must be operable without JavaScript. Each tag is therefore a text input carrying its value with `name="specs"` or `name="refs"`; the trailing add input carries the same name and is empty. Without JavaScript, clearing a tag input removes that value and typing in the add input adds one value. With JavaScript, the remove and `Add` buttons operate on the same inputs and the tag box keeps the approved chip layout.
   6. the add input is not a search field and does not resolve or validate values while typing.
7. The form submits `application/x-www-form-urlencoded` with `version`, `title`, and repeated `specs` and `refs` fields (one occurrence per tag, in display order). All occurrences are collected in document order and empty occurrences are dropped; this overrides the baseline "last value wins" rule for these two fields only (baseline section 5.5).
8. No create, deprecate, restore, supersede, delete, or status control is present.

### 7. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `specs`:
   1. at least one non-empty value is required;
   2. each value is trimmed, must be a valid record ID per baseline section 7.1 (1–128 bytes; ASCII letters, digits, `.`, `_`, `-`; no leading `.` or `-`; no `..`), and is stored exactly as entered after trimming;
   3. duplicate values are rejected;
   4. a value is not required to resolve to an existing UI specification, matching `ui-mockup create`/`edit` (see OQ-2).
3. `refs`:
   1. optional; an empty list is valid;
   2. each value is trimmed, 1–200 Unicode scalar values, contains no control characters, and contains no whitespace (the stored comment format is space-separated, so a value with whitespace could not round-trip);
   3. values may contain `/`, `.`, `-`, and `_` (for example `docs/specifications/ui-mockups-management.md`); they are stored as text and are never resolved as filesystem paths or links;
   4. duplicate values are rejected.
4. Validation failures return `422`, re-render the edit form with the submitted values preserved, a form-level summary whose entries link to the offending field, and field-level messages using the same text. Nothing is written.
5. Field messages:
   - `Title is required.`
   - `Title must be at most 200 characters.`
   - `Title cannot contain control characters.`
   - `At least one UI specification id is required.`
   - `UI specification ids must be valid record IDs.`
   - `Duplicate UI specification id: <value>.`
   - `References must be at most 200 characters.`
   - `References cannot contain control characters.`
   - `References cannot contain whitespace.`
   - `Duplicate reference: <value>.`
6. JSON validation failures use the same message text in the baseline JSON error object (`422`).

### 8. Persistence and editing contract

1. The file is read as UTF-8 text. On save, the file is rewritten atomically (baseline section 7.10) with only these substitutions:
   1. the `<title>` element text becomes `<ID> <separator> <title>`, where `<separator>` is `&mdash;` when the original used `&mdash;`, otherwise `—`, with one space on each side; the title is HTML-escaped in text context;
   2. an `<h1>` element whose text starts with `<ID>` followed by whitespace is rewritten as `<ID> <title>`, preserving the element's attributes; every other `<h1>` (for example the review package's landing heading) is left untouched;
   3. the `<!-- specs: ... -->` and `<!-- refs: ... -->` comment lines are replaced with the new space-separated values;
   4. a visible `<p>UI specifications: ...</p>` or `<p>References: ...</p>` line is rewritten only when that line already exists;
   5. when the `specs` or `refs` comment line is absent, a comment block containing both lines is inserted immediately after the opening `<body>` tag (or appended at the end of the file when no `<body>` tag exists). Visible metadata lines are never inserted into a record that does not already have them.
2. Every other byte of the file is preserved content-identically, in its original order and position: doctype, `<head>` children, inline `<style>`, scripts, the review package's slides, prototypes, and body markup. The browser never regenerates the file from a parsed record and never reuses the CLI's stub serializer (`render_record`).
3. The `<!-- mockup-id -->`, `<!-- status -->`, `<!-- task -->`, and `<!-- superseded-by -->` comments and their visible lines are never written or moved by the browser.
4. The ID, file name, directory, deprecation state, and status never change on a browser save.
5. A record without a `<title>` element is displayed with the record ID as its title. Saving never creates a `<title>` element; when one exists it is rewritten as in section 8.1.
6. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing or malformed `version` → `400`; stale token → `409` with the message "record changed on disk; reload the form" and no write; matching token proceeds.
7. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`; nothing outside the project is read or written.
8. After a successful write the server keeps no cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET` (save-then-reread).
9. No temporary file is left behind after a failed or rejected request.

### 9. Deprecated and superseded mockups

1. Deprecated mockups live in the active directory's `deprecated/` subfolder. That folder is never scanned for records, never listed, never counted, and never resolved.
2. A request for a deprecated mockup's ID returns `404` on `GET /doc/{id}`, `GET /doc/{id}/edit`, `POST /doc/{id}`, `GET /api/doc/{id}`, and `POST /api/doc/{id}`, exactly as for an unknown ID.
3. A mockup that is superseded is deprecated by definition (TASK-38 section 7) and is therefore also hidden and unreachable.
4. No browser control changes deprecation or supersession state; those remain CLI-only (REQ-05.007).

### 10. JSON API

1. `GET /api/ui-mockups` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id": "MOCK-006", "title": "Browser UI mockups review package"}`. The aliases `/api/ui-mockup` and `/api/mockups` return the same response.
2. `GET /api/doc/{id}` for a mockup ID returns the full record shape:

   ```json
   {
     "id": "MOCK-006",
     "kind": "ui-mockup",
     "form": "ui-mockup",
     "title": "Browser UI mockups review package",
     "specs": ["UI-002", "UI-003"],
     "refs": ["docs/specifications/ui-mockups-management.md"],
     "task": "TASK-41",
     "status": "approved",
     "superseded_by": null,
     "version": "1762000000.123456789.412",
     "path": "docs/specifications/ui/mockups/MOCK-006-browser-ui-mockups-review.html"
   }
   ```

   `specs` and `refs` are arrays (empty when absent); `task` and `status` are strings (empty when absent); `superseded_by` is a string or `null`; `path` is relative to the project directory with `/` separators and never exposes an absolute path.
3. `POST /api/doc/{id}` requires `Content-Type: application/json` and accepts an object with:
   - `version` (required, string);
   - `title` (required when present, string; omitting it leaves the title unchanged);
   - `specs` (optional array of strings; when present it must contain at least one value and replaces the whole list; an explicit `null` is `422` because the field is required);
   - `refs` (optional array of strings; when present it replaces the whole list; an empty array or `null` clears it).
4. Omitted members are left unchanged. The read-only members `task`, `status`, and `superseded_by` are ignored when sent and are never changed by the browser. Unknown members are ignored. A non-object body, a wrong member type, or a non-string array element is `400`. A title that fails validation is `422`.
5. A successful `POST` returns `200` with the full record shape of section 10.2, read back from disk after the write.
6. HTML and JSON error responses follow baseline section 5.3.

### 11. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, and no inline scripts/styles.
2. The stored mockup HTML is untrusted data. It is never served as a page, never inlined into a response, never parsed as HTML for rendering, and never used to build a path. Only the extracted metadata values are escaped and rendered.
3. Tag controls are keyboard operable, every input has a `<label>` or accessible name, remove controls are labelled `Remove <value>`, and the add input is labelled. Focus is not trapped.
4. The sidebar's UI Mockups entry and the mockup list/view/edit templates are the only rendering additions; the shared shell and error pages are reused.

### 12. Tests

1. Unit tests cover: metadata extraction from `<title>` (em dash, `&mdash;`, no separator, absent title), `specs`/`refs` comment parsing and ordering, the field-preserving serializer (title, stub `<h1>`, comments, visible lines when present, insertion when absent, all other bytes identical), validation (required specs, invalid id, duplicates, refs length/control characters/whitespace), and JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-40a: list renders the UI Mockups category, ascending by ID, with the selected sidebar entry marked `aria-current="page"`; `?type=ui-mockup` works and any other value is `400`.
   - T-40b: `README.md`, `wireframe.css`, non-matching `.html` files, and `deprecated/` are not listed.
   - T-40c: view shows ID, form, category, title, UI specifications, references, breadcrumb, and a single Edit action; the stored package's markup and scripts do not appear in the response.
   - T-40d: edit round-trip persists title, specs, and refs, visible through the browser and through `papyrus ui-mockup view`.
   - T-40e: editing a real review package (a fixture with inline `<style>`, `<script>`, and body markup) changes only the metadata lines; a byte comparison shows every other byte unchanged.
   - T-40f: repeated `specs`/`refs` form fields are all collected in order (not last-value-wins).
   - T-40g: validation returns `422` and writes nothing for empty title, over-long title, control characters, zero specs, invalid spec id, duplicate spec id, over-long ref, duplicate ref.
   - T-40h: stale version token returns `409` and does not write; missing/malformed version returns `400`.
   - T-40i: deprecated mockups are not listed and return `404` on view, edit, POST, and API.
   - T-40j: JSON API returns the documented summary and full shapes, applies the documented POST semantics, and rejects wrong types with `400`/`422`.
   - T-40k: writes are atomic, leave no temporary file, and a symlink escape returns `404`.
   - T-40l: an offline asset scan finds no external origin in list, view, or edit responses.
   - T-40m: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Approved artifact (TASK-41):** [`MOCK-0006`](ui/mockups/MOCK-0006-browser-ui-mockups-review.html) — `approved` in the mockup registry on 2026-11-06 (UTC). One self-contained review package: review landing, UI-mockup list/view/edit prototypes, error pages, and 320 px states (17 views). Layout description: [`ui/browser-ui-mockups-ui.md`](ui/browser-ui-mockups-ui.md). MOCK-0001…MOCK-0005 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with seven entries, `main`, footer, one `h1` per page.
2. **UI mockups list** (`/?type=ui-mockup`): one section with record count, rows of `ID + title`; empty and long-title states.
3. **UI mockup view** (`/doc/{id}`): metadata block, title `h1`, UI specifications and References sections with `pre-wrap`, breadcrumb `UI Mockups › ID`, single `Edit`.
4. **Edit form** (`/doc/{id}/edit`): title input, specs tag box (add/remove), refs tag box (add/remove), version annotation, Save/Cancel; pristine, dirty, 422, 409, and 320 px states.
5. **Error pages**: 404 unknown, 404 deprecated, 400, 405, 415, 500 (reused from MOCK-0001).

### Layout conflicts resolved by this specification

1. The approved UI description and MOCK-0006 require `specs` and `refs` to use the tag add/remove interaction. A partial implementation on branch `task/40` renders them as multi-line textareas with whitespace-separated values. The tag interaction is authoritative; the textarea rendering is a deviation the implementation must replace (section 6.6).
2. MOCK-0006 shows tag chips as non-input elements with `type="button"` remove controls. Because the baseline requires the interface to work with JavaScript disabled, the shipped tag value must be a text input styled to the approved chip layout, with the remove/`Add` controls as progressive enhancement (section 6.6.5). Chip appearance, placement, and states follow MOCK-0006.

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `?type=ui-mockup` renders the category, the default stays `req`, other values are `400`, and the sidebar entry is ordered after UI Specifications with `aria-current="page"` and omitted when the type is disabled. |
| AC-2 | The list shows active mockups only, ascending by ID, with `ID + title` links, count, empty state, and no non-record or deprecated files. |
| AC-3 | The view shows ID, form, category, title, UI specifications, references, breadcrumb, and one Edit action, and never serves or embeds the stored package. |
| AC-4 | The edit form matches MOCK-0006: title, specs/refs tag add/remove, hidden version plus annotation, Save/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript. |
| AC-5 | An edit round-trip persists title, specs, and refs and is visible in the browser and through `papyrus ui-mockup view`. |
| AC-6 | Editing a review package changes only the metadata it edits and preserves every other byte; no whole-file regeneration occurs. |
| AC-7 | Title extraction and update follow section 3.2 and section 8.1 (em dash/`&mdash;`, stub `<h1>` only). |
| AC-8 | Validation returns `422` with the documented messages and writes nothing (title, specs, refs rules). |
| AC-9 | A missing/malformed version is `400`; a stale version is `409` with the documented message and no write. |
| AC-10 | Deprecated mockups are hidden and return `404` on view, edit, POST, and API. |
| AC-11 | The JSON API returns the documented summary/full shapes, supports the documented POST semantics, and rejects wrong types. |
| AC-12 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-13 | The interface remains offline-capable with no external resources, escaped metadata, unchanged security headers, and keyboard-operable tag controls. |
| AC-14 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: A mockup record's persisted file is a design artifact authored by `@ui-designer`; therefore browser edits must be field-preserving and must never regenerate the file from a parsed model. This is the main reason the CLI's stub serializer is not reused.
- Assumption: The comment block (`<!-- specs: ... -->`, `<!-- refs: ... -->`) is the machine-readable metadata source. Review packages created before this metadata existed are displayed with empty specs/refs and receive the comment block on first save; visible metadata lines are added only to records that already have them.
- Assumption: Spec id format is validated but existence is not required, matching `ui-mockup create`/`edit`; dangling ids are allowed so a mockup can be written before its specification is registered.
- Assumption: Duplicate `specs`/`refs` values are rejected because the tag fields model a set; this is stricter than the CLI, which stores whatever it is given.
- Assumption: The tag field's no-JavaScript fallback uses one text input per value plus a trailing empty input with the same field name; this preserves the approved chip layout with JavaScript and keeps the form usable without it.
- Assumption: The JSON API exposes `task`, `status`, and `superseded_by` read-only, consistent with sibling record shapes, even though the browser pages do not display them.
- Assumption: Real-browser verification in Firefox, Edge, and Safari 100+ (NFR-05.001) remains a deferred follow-up, consistent with TASK-15/TASK-23/TASK-24.
- Assumption: No new dependencies, crates, or ADRs are introduced.

## Open questions

- **OQ-1 (resolved)** — Whether the browser shows status, task, or supersession for mockups: MOCK-0006 OQ-3 resolves this as "no"; those fields are CLI-only and hidden in the browser.
- **OQ-2 (non-blocking, product)** — Whether `specs` should be required to resolve to an existing active UI specification. This specification does not require resolution, for parity with the CLI. Tightening it later changes validation only.
- **OQ-3 (non-blocking, follow-up)** — The CLI `ui-mockup edit` path rewrites the whole file from `render_record`, which would replace a review package with the minimal stub. That defect is outside TASK-40's write scope (browser only) but is a data-loss risk for TASK-38/TASK-39; it should be tracked as a separate CLI fix.
- **OQ-4 (non-blocking, product)** — Whether refs should support removal of all values in one action (as business rules do). This specification uses the tag remove control per value; a "remove all" control is not shown in MOCK-0006.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.007, REQ-05.008, REQ-06, REQ-06.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-38 (mockup CLI), TASK-40, TASK-41 (UI design), TASK-42 (packaging) |
| UI mockups | MOCK-0001…MOCK-0005 (approved baseline); MOCK-0006 (approved) — `docs/specifications/ui/mockups/MOCK-0006-browser-ui-mockups-review.html` |
| UI description | `docs/specifications/ui/browser-ui-mockups-ui.md` (TASK-41) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/ui-mockups-management.md` (TASK-38 record model and lifecycle), `docs/specifications/browser-ui-specifications.md` (TASK-24 sibling increment), `docs/specifications/search-ui-mockups.md` (TASK-39) |
| Domain model | `docs/specifications/ui/mockups/README.md` (mockup registry; status values pending/approved/deprecated) |
