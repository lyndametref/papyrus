# Feature Specification — Browser Access for Glossary Terms

**Ticket:** TASK-23
**Milestone:** m-5 (Search and Browser for Glossary)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html) — approved 2026-11-06 (TASK-32); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), and [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html). See [UI design handoff](#ui-design-handoff).

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements), TASK-11 (backlog records), and TASK-15 (decision records) so it also serves the glossary store: active glossary terms. The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`glossary`), its record model, list/view/edit screens, API endpoints, and tests. It implements REQ-05.003 and REQ-05.007 for the browser surface under REQ-06.002, and advances NFR-01.002 (display and edit all Papyrus content) for glossary terms. Deprecated terms are hidden from lists and unreachable by URL, mirroring archived decisions and deprecated requirements.

## Scope

### In scope

- Extending the persistent sidebar with a Glossary entry (after Decisions) and the `?type=` category selector to `glossary`, without changing the list-page structure (baseline spec section 4.6).
- Listing active glossary terms, rendering only the glossary section.
- Viewing a glossary term, including its term and definition.
- Editing a glossary term through the same optimistic-concurrency and atomic-write contract as requirement and decision records (ADR-0006).
- A read/write JSON API over the same glossary records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Deprecated glossary terms are not listed and are not reachable (active only), mirroring deprecated requirements.
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, deprecating, restoring, or deleting glossary terms from the browser; changing record IDs. These remain CLI-only (REQ-05.003, BR-011).
- Managing glossary references from other records in the browser.
- Browsing UI specifications, business rules, audit records, deployment records, or technical specifications — later tickets (TASK-24, TASK-25, TASK-26, TASK-57).
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-33.
- Cross-type navigation and the final completeness check — TASK-19.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts seven values: `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists five entries in this order: Business requirements, Non-functional requirements, Backlog, Decisions, Glossary. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. The list page renders only the selected category; the other categories are not rendered at all.
5. A missing or unreadable glossary directory is not a server error: the category renders its empty state and its API returns an empty array (consistent with the baseline and with `glossary list`).
6. Deprecated glossary terms are never listed, counted, or reachable by URL, and return `404` exactly as for an unknown ID.

### 2. Record sources and IDs

1. Glossary terms are read from `<doc_directory>/<document_types.glossary.directory>/` (active folder only; default `docs/glossary/`). The directory is resolved from the effective configuration; prefixes and directories are never hardcoded.
2. The `deprecated/` subfolder of that directory is never listed or reachable. Only files directly inside the active folder whose `.md` file stem is a valid record ID are records. There is no recursive traversal.
3. Record IDs are the file stems (for example `GLO-001` by default prefix). The document type and record form are derived from the directory that matched, never from the request.
4. ID resolution is by exact, case-sensitive match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to seven categories).
5. Record ID, category, term, and deprecation state that are not editable fields are never accepted from the request.
6. A disabled glossary document type (`document_types.glossary.enabled == false`) contributes no records and no error (BR-016).

### 3. List page

1. For the `glossary` category the list page renders one section (`h2`) titled "Glossary", showing the record count, followed by one row per term in ascending ID order.
2. Each row's primary link text is `ID + term` and links to `/doc/{id}`.
3. Each row also shows the first paragraph of the term's definition beneath the primary link; terms with an empty definition show no preview. The preview is plain text (first paragraph only, not the full definition).
4. Empty state and long-title wrapping as in the baseline (section 10.4).
5. Deprecated terms are never listed.

### 4. Record view

1. Glossary view shows:
   - ID (read-only), record form (`Glossary term`), category (`Glossary`);
   - Term as the page title (`h1`);
   - Definition in a section with `white-space: pre-wrap` (or `(none)`).
2. Breadcrumb on the view page: `Glossary` (a link to `/?type=glossary`), then the record ID as the current, non-linked crumb.
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.
4. The term text is displayed as literal text, never auto-linked.

### 5. Edit form

1. Editable fields:
   - `term` (required);
   - `definition` (optional multi-line).
2. ID, record form, and category are displayed as read-only text and are never accepted from the request.
3. Form fields and media types follow the baseline section 5.5: `version` (required), `title` (required, mapped to term), plus the glossary-specific `definition` field. Unknown form fields are ignored; repeated fields use last-value-wins; percent-decoding follows baseline rules.
4. Breadcrumb on the edit page: `Glossary › {id} › Edit`; `Edit` is the current, non-linked crumb.
5. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present, exactly as in the baseline (section 4.23).
6. Field help text is shown where applicable.
7. Dirty state, validation-error state, conflict state (409), and success behavior (redirect to view, save-then-reread) follow the baseline sections 4.23 and 7.12.

### 6. Validation

1. `term` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `definition` follows baseline section 7.3/7.4: trimmed; at most 20000 Unicode scalar values; LF allowed; CR/CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section; a line whose first two characters are `## ` is rejected.
3. Form-level error summary links each entry to its field and names the actual field error, using the same text as that field's message (baseline section 5.5/7.5).
4. Validation failures return `422` and never modify the file.

### 7. Persistence and editing contract

1. On save, the file is read as UTF-8 text; the first line is replaced with `# <ID> <term>`; the `## Definition` section keeps its original position; a non-empty value replaces that section's body; an empty value removes the section; a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end.
2. Known section order: `## Definition`. (Glossary records have no other known sections.)
3. Unknown and editor-added sections are preserved content-identically, in their original order and position (baseline section 7.7).
4. A glossary term with no `## Definition` section is displayed and edited as empty; saving never adds that section unless a value is supplied.
5. Writes are atomic and project-local, following baseline section 7.10.
6. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`.
7. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing/malformed `version` → `400`; stale token → `409` with message "record changed on disk; reload the form"; matching token proceeds.
8. After a successful write the server does not keep a cached copy; the redirect target re-reads the file (save-then-reread).
9. Editing a record never changes its ID, its directory, or its deprecated state.

### 8. JSON API

1. `GET /api/glossary` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id","title"}` (title is the term).
2. `GET /api/doc/{id}` and `POST /api/doc/{id}` work for glossary IDs in addition to requirement, backlog, and decision IDs, returning the full record shape: `{"id","kind":"glossary","form":"glossary","title","definition","version","path"}`.
3. `POST /api/doc/{id}` accepts a JSON object with `version` (required), `title` (required, mapped to term), and the editable fields for that record type. Omitted fields are left unchanged; an explicit `null` clears an optional field. Unknown members are ignored; a non-object body or a member of the wrong type receives `400`.
4. Error responses follow baseline section 5.3 (HTML error page or JSON error object with `code`/`message`).

### 9. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, no inline scripts/styles.
2. Templates may be extended with glossary-specific placeholders; unknown placeholders are left untouched as in the baseline.
3. The sidebar's Glossary entry and the glossary list/view/edit templates are the only rendering additions.

### 10. Tests

1. Unit tests cover: ID validation for glossary ID shapes, record parsing/serialization for glossary terms, field validation for the new fields, version token, JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-23a: list renders the glossary category, ordered by ID, with the selected sidebar entry marked `aria-current="page"`.
   - T-23b: `?type=glossary` works, default remains `req`, and any other value is `400`.
   - T-23c: view shows ID, term, and definition.
   - T-23d: edit round-trip persists and is visible through both the UI and the CLI.
   - T-23e: field validation returns `422` and does not write (empty term, term over 200 characters, control characters, definition over 20000 characters, `## ` line).
   - T-23f: stale version token returns `409` and does not write.
   - T-23g: deprecated glossary terms are not listed and return `404` by URL.
   - T-23h: writes are atomic and leave no temporary file; symlink escape returns `404`.
   - T-23i: JSON API returns the documented shapes and rejects bad bodies.
   - T-23j: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Baseline artifacts:** MOCK-0001 — approved 2026-11-03 (UTC). MOCK-0002 — approved 2026-11-04 (UTC). MOCK-0003 — approved 2026-11-06 (UTC). All remain authoritative layout references.

**Extension artifact (TASK-23):** MOCK-0004 — produced by `@ui-designer` as a self-contained review package that reuses the approved shell and adds the Glossary view (list, view, edit, empty, error, and 320 px states). MOCK-0001, MOCK-0002, and MOCK-0003 are not modified in place. Its status is `approved` (approved 2026-11-06, TASK-32).

### Screens to be covered by the extension package

1. **Shared shell** (reused): header, sidebar with five entries (Business requirements, Non-functional requirements, Backlog, Decisions, Glossary), `main`, footer, one `h1`.
2. **Glossary list** (`/?type=glossary`): rows with ID + term link and definition preview; empty state; long titles.
3. **Glossary view** (`/doc/{id}`): metadata block (ID, form, category), term as `h1`, definition with `white-space: pre-wrap`, breadcrumb `Glossary › ID`, single `Edit` action.
4. **Edit form**: term, definition, version annotation; pristine, dirty, validation-error, conflict, and 320 px states.
5. **Error pages**: reused from MOCK-0001 (404 unknown/deprecated, 400, 405, 415, 500).

## Investigation notes (AC #3)

The following records where glossary terms appear in the browser, the route shape, and which existing components are reused. These notes are the deliverable of acceptance criterion #3 and are intended to be read by the implementer alongside the spec.

- **Storage location:** active terms in `<doc_directory>/<document_types.glossary.directory>/` (default `docs/glossary/`); deprecated terms in its `deprecated/` subfolder. Files are `<ID>.md` (e.g. `GLO-001.md`).
- **File format:** `# <ID> <term>` heading, then `## Definition` section. Parsed with the same `parse_doc`/`section_value` helpers as requirement records.
- **Browser navigation:** a new sidebar entry "Glossary" is added after "Decisions", linking to `/?type=glossary`. The `?type=` selector gains the `glossary` value (default remains `req`; any other value is `400`).
- **Route shape (unchanged from baseline):**
  - `GET /?type=glossary` — list page.
  - `GET /doc/{id}` — view page.
  - `GET /doc/{id}/edit` — edit form.
  - `POST /doc/{id}` — save.
  - `GET /api/glossary` — summary array.
  - `GET /api/doc/{id}` / `POST /api/doc/{id}` — full record.
- **Reused components:**
  - `records::parse_doc`, `section_value`, `valid_id`, `version_token`, `valid_version_token`, `write_atomic`, `Context::ensure_within`, `resolve_id`, `build_index` (extended).
  - Render: `render_template`, `escape`, `sidebar`, `breadcrumb`/`edit_breadcrumb`, `metadata_block`/`metadata_block_backlog`, list/view/edit templates, `conflict_page`, `error_page`, `change`.
  - Validation: `validate_title` (term), `validate_multiline` (definition).
  - Transport/security: Host/Origin validation, request limits, timeouts, embedded assets.
- **New additions required:**
  - `Category::Glossary` and `Form::Glossary` variants in `records.rs`, with match arms in `http.rs`, `render.rs`, and `mod.rs`.
  - `load_glossary_record` (or extension of `load_record_from_path`) reading the active folder only, honoring `document_types.glossary` and its `enabled` flag, and skipping `deprecated/`.
  - `apply_glossary_edit` serializer preserving unknown sections.
  - Sidebar entry and category-name/short/heading mappings in `render.rs`.
  - Glossary-specific view/edit render functions.
- **Deprecated visibility:** deprecated terms live in `deprecated/` and are never indexed, listed, or resolved; viewing/editing a deprecated ID returns `404`.

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | The browser displays and edits active glossary terms in addition to all previously supported content. |
| AC-2 | Category selection supports `type=glossary` (default `req`); any other value returns `400`. |
| AC-3 | Deprecated glossary terms are not listed and return `404` by URL. |
| AC-4 | Glossary view shows ID, term, and definition. |
| AC-5 | An edit round-trip persists and is visible through both the browser and the CLI. |
| AC-6 | Field validation returns `422` and does not write for empty term, term over 200 characters, control characters, definition over 20000 characters, and `## ` lines. |
| AC-7 | A stale version token returns `409` and does not write. |
| AC-8 | Writes are atomic, project-local, leave no temporary file, and symlink escapes return `404`. |
| AC-9 | Browser operations remain offline-capable and use no paths outside the project directory except during init. |
| AC-10 | The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: Deprecated glossary terms are hidden and unreachable, mirroring deprecated requirements and archived decisions.
- Assumption: The category selector is extended to `glossary` rather than introducing new top-level routes, keeping the list-page structure unchanged (baseline section 4.6).
- Assumption: `term` and `definition` are edited as free-text fields; empty definition clears the section.
- Assumption: A new MOCK id (MOCK-0004) is allocated for the extension rather than modifying the approved MOCK-0001/MOCK-0002/MOCK-0003 in place, per the global sequential-id rule. MOCK-0004 is already approved (TASK-32).
- Assumption: No new dependencies, crates, or ADRs are introduced.
- Assumption: AC #10 (real-browser verification in Firefox, Microsoft Edge, and Safari 100+) and NFR-05.001 are deferred follow-ups, not blocking for this ticket; tracked as a follow-up rather than a release gate, consistent with TASK-15.
- Assumption: The glossary document type is read-only in the browser for deprecation/restoration; creating or deprecating terms remains CLI-only.
- Assumption: The glossary record's `term` maps to the record `title` field, consistent with `glossary list` (which reports ID and term) and the search spec (which uses the term as title).
- Assumption: The definition preview on the list row is the first paragraph (first blank-line-delimited block) of the definition, plain text, truncated by the template/CSS; full definition is shown on the view page.

## Open questions

- **OQ-1 (pending)** — Whether the glossary list should support filtering by a search query in the browser is unspecified; this specification lists all active terms in one view, consistent with other document types. Browser search is out of scope (TASK-19 territory).
- **OQ-2 (pending)** — Whether `term` should be editable to an empty value is unspecified; this specification requires a non-empty term (1–200 characters), consistent with `validate_title` and the CLI's required `--term`.
- **OQ-3 (resolved)** — MOCK-0004 is `approved` and its glossary screen sections are complete; no further UI-design dependency remains.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.002, REQ-05, REQ-05.003, REQ-05.007 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-011 (glossary command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-30, TASK-32 |
| UI mockups | MOCK-0001 (approved baseline); MOCK-0002 (approved); MOCK-0003 (approved); MOCK-0004 (approved) — `docs/specifications/ui/mockups/MOCK-0004-browser-glossary-terms-review.html` |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/browser-tasks-milestones-and-releases.md` (TASK-11), `docs/specifications/browser-decision-records.md` (TASK-15), `docs/specifications/glossary-terms-management.md` (TASK-30), `docs/specifications/search-glossary-terms.md` (TASK-31), `docs/specifications/ui/browser-glossary-terms-ui.md` (UI description, TASK-32) |
| Domain model | `docs/domain-model/business-glossary/glossary-term.md` (if present) |
