# Feature Specification — Browser Access for Decision Records

**Ticket:** TASK-15
**Milestone:** m-3 (Decision Records)
**Status:** Refined (execution-ready; UI mockup pending approval)
**UI mockups:** [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html) (pending approval) — extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html) and [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html); see [UI design handoff](#ui-design-handoff)

## Purpose

Extend the offline, project-local, read-and-edit web interface delivered by TASK-6 (requirements) and TASK-11 (backlog records) so it also serves the decisions store: active decision records (ADR, BDR, and configured types). The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds one documentation category (`decision`), its record model, list/view/edit screens, API endpoints, and tests. It implements the decisions-store part of REQ-06.002 under the same requirement, and advances NFR-01.002 (display and edit all Papyrus content) for decision records. It implements REQ-04.001 and REQ-04.002 for the browser surface.

## Scope

### In scope

- Extending the persistent sidebar and the `?type=` category selector to cover `decision` in addition to `req`, `nfr`, `task`, `milestone`, and `release`, without changing the list-page structure (baseline spec section 4.6).
- Listing active decision records, rendering only the decision section.
- Viewing a decision record, including its Nygard fields and supersession relationship.
- Editing a decision record through the same optimistic-concurrency and atomic-write contract as requirement records (ADR-0006).
- A read/write JSON API over the same decision records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Archived decisions are not listed and are not reachable (active only), mirroring deprecated requirements.
- Unit and integration tests extending `tests/browser_cli.rs` for the new category.

### Out of scope

- Creating, archiving, restoring, or deleting decisions from the browser; changing record IDs or decision types. These remain CLI-only (REQ-04.002, BR-008).
- Managing requirement references from a decision in the browser — they appear as literal text in the fields, consistent with the CLI (decision-records-management.md).
- Browsing glossary terms, UI specifications, business rules, audit records, deployment records, or specification documents — later tickets (TASK-23 … TASK-27, TASK-57).
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-16.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts six values: `req` (default), `nfr`, `task`, `milestone`, `release`, `decision`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists four entries in this order: Business requirements, Non-functional requirements, Backlog, Decisions. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible.
4. The list page renders only the selected category; the other categories are not rendered at all.
5. A missing or unreadable decisions directory is not a server error: the category renders its empty state and its API returns an empty array (consistent with the baseline and with `decision list`).
6. Archived decisions are never listed, counted, or reachable by URL, and return `404` exactly as for an unknown ID.

### 2. Record sources and IDs

1. Decision records are read from `<decision_directory>/` (active folder only). The directory is resolved from the effective configuration; prefixes and directories are never hardcoded.
2. Only files directly inside that directory whose `.md` file stem is a valid record ID are records. The `archived/` subfolder, `index.html`, and non-`.md` files are ignored. There is no recursive traversal.
3. Record IDs are the file stems (for example `ADR-001`, `BDR-001` by default prefixes). The document type and record form are derived from the directory that matched, never from the request.
4. ID resolution is by exact, case-sensitive match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to six categories).
5. Record ID, category, decision type, and supersession metadata that is not an editable field are never accepted from the request.

### 3. List page

1. For the `decision` category the list page renders one section (`h2`) titled with the category name, showing the record count, followed by one row per record in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Decision rows additionally show, as read-only text in the row: status and decision type.
4. Empty state and long-title wrapping as in the baseline (section 10.4).

### 4. Record view

1. Decision view shows:
   - ID (read-only), record form (`Decision`), category (`Decision`), decision type;
   - Status (proposed, accepted, rejected, deprecated, or `Superseded by <id>`);
   - Decision text (or `(none)`);
   - Context (or `(none)`);
   - Consequences (or `(none)`);
   - Supersession: when superseded, the superseding decision ID and title as a link when it resolves, otherwise the stored ID as text; when not superseded, `(none)`.
2. Breadcrumb on the view page: `Decisions` (a link to `/?type=decision`), then the record ID as the current, non-linked crumb.
3. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.
4. Requirement references and other IDs inside field values are displayed as literal text, never auto-linked, except where this specification explicitly lists a field as a link (superseding decision).

### 5. Edit form

1. Editable fields:
   - `title` (required);
   - `status` (optional select among the valid transitions for the current status; see decision-records-management.md section 6.7);
   - `decision` (optional);
   - `context` (optional);
   - `consequences` (optional);
   - `superseded-by` (optional ID or empty).
2. Decision type, ID, and category are displayed as read-only text and are never accepted from the request.
3. The form may supply either a `status` value or a `superseded-by` value, not both. Supplying both is a validation error (`Cannot set both status and superseded-by.`).
4. When `superseded-by` is non-empty, the status is set to `Superseded by <id>` (Nygard convention) and any `status` selection is ignored; if both are supplied, the request is rejected with `422`.
5. Form fields and media types follow the baseline section 5.5: `version` (required), `title` (required), plus the decision-specific fields above. Unknown form fields are ignored; repeated fields use last-value-wins; percent-decoding follows baseline rules.
6. Breadcrumb on the edit page: `Decisions › {id} › Edit`; `Edit` is the current, non-linked crumb.
7. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present, exactly as in the baseline (section 4.23).
8. Field help text is shown where applicable.
9. Dirty state, validation-error state, conflict state (409), and success behavior (redirect to view, save-then-reread) follow the baseline sections 4.23 and 7.12.

### 6. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `decision`, `context`, and `consequences` follow baseline section 7.3/7.4: trimmed; at most 20000 Unicode scalar values; LF allowed; CR/CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section; a line whose first two characters are `## ` is rejected.
3. `status`, when supplied, must be one of `proposed`, `accepted`, `rejected`, `deprecated`, `superseded`, and the transition must be valid per decision-records-management.md section 6.7. An unknown value or invalid transition is a validation error.
4. `superseded-by`, when non-empty, must resolve to an active or archived decision of the same decision type; an unknown value is a validation error (`Decision not found.`); a type mismatch is a validation error (`Decision type mismatch.`); self-reference is a validation error (`A decision cannot supersede itself.`).
5. Form-level error summary links each entry to its field and names the actual field error, using the same text as that field's message (baseline section 5.5/7.5).
6. Validation failures return `422` and never modify the file.

### 7. Persistence and editing contract

1. On save, the file is read as UTF-8 text; the first line is replaced with `# <ID> <title>`; each known section keeps its original position; a non-empty value replaces that section's body; an empty value removes the section; a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end.
2. Known section order: `## Status`, `## Decision`, `## Context`, `## Consequences`. The `## Status` section is written as the status value, or `Superseded by <id>` when superseded.
3. Unknown and editor-added sections are preserved content-identically, in their original order and position (baseline section 7.7).
4. A decision record with no `## Decision`/`## Context`/`## Consequences` section is displayed and edited as empty; saving never adds that section unless a value is supplied.
5. Writes are atomic and project-local, following baseline section 7.10.
6. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`.
7. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing/malformed `version` → `400`; stale token → `409` with message "record changed on disk; reload the form"; matching token proceeds.
8. After a successful write the server does not keep a cached copy; the redirect target re-reads the file (save-then-reread).
9. Editing a record never changes its ID, its directory, its decision type, or its archived state.

### 8. JSON API

1. `GET /api/decisions` returns `200 application/json` with an array of summary objects ordered by ID ascending: `{"id","title","status","type"}`.
2. `GET /api/doc/{id}` and `POST /api/doc/{id}` work for decision IDs in addition to requirement and backlog IDs, returning the full record shape: `{"id","kind":"decision","form":"decision","title","type","status","decision","context","consequences","superseded_by":{"id","title"}|null,"version","path"}`.
3. `POST /api/doc/{id}` accepts a JSON object with `version` (required), `title` (required), and the editable fields for that record type. Omitted fields are left unchanged; an explicit `null` clears an optional field. Unknown members are ignored; a non-object body or a member of the wrong type receives `400`.
4. Error responses follow baseline section 5.3 (HTML error page or JSON error object with `code`/`message`).

### 9. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, no inline scripts/styles.
2. Templates may be extended with decision-specific placeholders; unknown placeholders are left untouched as in the baseline.
3. The sidebar's Decisions entry and the decision list/view/edit templates are the only rendering additions.

### 10. Tests

1. Unit tests cover: ID validation for decision ID shapes, record parsing/serialization for decisions, field validation for the new fields, version token, JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new category, including at least:
   - T-15a: list renders the decision category, ordered by ID, with the selected sidebar entry marked `aria-current="page"`.
   - T-15b: `?type=decision` works, default remains `req`, and any other value is `400`.
   - T-15c: view shows all decision-specific fields; superseding-decision link resolves.
   - T-15d: edit round-trip persists and is visible through both the UI and the CLI.
   - T-15e: field validation returns `422` and does not write (empty title, invalid status transition, unknown superseding decision, type mismatch, self-supersession, both status and superseded-by, `## ` line).
   - T-15f: stale version token returns `409` and does not write.
   - T-15g: archived decisions are not listed and return `404` by URL.
   - T-15h: writes are atomic and leave no temporary file; symlink escape returns `404`.
   - T-15i: JSON API returns the documented shapes and rejects bad bodies.
   - T-15j: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Baseline artifacts:** MOCK-0001 — approved 2026-11-03 (UTC). MOCK-0002 — approved 2026-11-04 (UTC). Both remain authoritative layout references.

**Extension artifact (TASK-15):** MOCK-0003 — produced by `@ui-designer` as a self-contained review package that reuses the approved shell and adds the Decisions view (list, view, edit, empty, error, and 320 px states). MOCK-0001 and MOCK-0002 are not modified in place. Its status is `pending` (created 2026-11-05, TASK-15; awaiting explicit user approval).

### Screens to be covered by the extension package

1. **Shared shell** (reused): header, sidebar with four entries (Business requirements, Non-functional requirements, Backlog, Decisions), `main`, footer, one `h1`.
2. **Decision list** (`/?type=decision`): rows with ID + title link, status, decision type; empty state; long titles.
3. **Decision view** (`/doc/{id}`): metadata block (ID, form, category, type, status), decision text, context, consequences, supersession, breadcrumb `Decisions › ID`, single `Edit` action.
4. **Edit form**: title, status, decision, context, consequences, superseded-by, version annotation; pristine, dirty, validation-error, conflict, and 320 px states.
5. **Error pages**: reused from MOCK-0001 (404 unknown/archived, 400, 405, 415, 500).

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | The browser displays and edits active decision records in addition to requirement and backlog content. |
| AC-2 | Browser operations remain offline-capable, use project-local text-file persistence, sanitize input, and access no paths outside the project directory except during init. |
| AC-3 | The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later. |
| AC-4 | Archived decisions are not listed and return `404` by URL. |
| AC-5 | Optimistic concurrency and atomic writes hold for the new record type. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds one category and record type only.
- Assumption: Archived decisions are hidden and unreachable, mirroring deprecated requirements.
- Assumption: The category selector is extended to `decision` rather than introducing new top-level routes, keeping the list-page structure unchanged (baseline section 4.6).
- Assumption: `decision`, `context`, and `consequences` are edited as free-text fields; empty clears the section.
- Assumption: The configured decision statuses and the documented transition graph govern the `status` field in the browser.
- Assumption: A new MOCK id (MOCK-0003) is allocated for the extension rather than modifying the approved MOCK-0001/MOCK-0002 in place, per the global sequential-id rule.
- Assumption: No new dependencies, crates, or ADRs are introduced.
- Assumption: AC #3 (real-browser verification in Firefox, Microsoft Edge, and Safari 100+) and NFR-05.001 are deferred follow-ups, not blocking for this ticket; tracked as a follow-up rather than a release gate.
- Assumption: The decision type is read-only in the browser; creating decisions or changing their type remains CLI-only.
- Assumption: Supersession in the browser follows the CLI rule: `--superseded-by` cannot be combined with `--status`; the browser enforces the same mutual exclusion as a validation error.

## Open questions

- **OQ-1 (pending)** — Whether the decision type should be filterable in the browser list (e.g. `?type=decision&decision-type=adr`) is unspecified; this specification lists all active decisions of all types in one view, with the type shown per row. If filtering is desired, the query parameter and empty-state copy change.
- **OQ-2 (pending)** — Whether `status` should be editable to the bare value `superseded` in addition to using `superseded-by` is unspecified; this specification permits `superseded` as a valid status value without recording a superseding ID, consistent with decision-records-management.md OQ-1.
- **OQ-3 (pending)** — MOCK-0003 is `pending` and its decision-record screen sections must be completed by `@ui-designer` and approved by the user before UI implementation. Until then, the mockup is not implementation-ready for the UI portions.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.002, REQ-04, REQ-04.001, REQ-04.002 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-008 (decision command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-13, TASK-15 |
| Increment boundary | REQ-06.002 covers both stores; the documentation-store part (requirements) was delivered by TASK-6/TASK-7, the backlog-store part by TASK-11, the decisions-store part by this ticket. |
| UI mockups | MOCK-0001 (approved baseline); MOCK-0002 (approved); MOCK-0003 (pending) — `docs/specifications/ui/mockups/MOCK-0003-browser-decision-records-review.html` |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/browser-tasks-milestones-and-releases.md` (TASK-11), `docs/specifications/decision-records-management.md` (TASK-13) |
| Related specification | `docs/specifications/ui/browser-decision-records-ui.md` (UI description) |