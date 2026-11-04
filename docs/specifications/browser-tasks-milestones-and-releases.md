# Feature Specification — Browser Access for Backlog Records (Tasks, Milestones, Releases)

**Ticket:** TASK-11
**Milestone:** m-2 (Backlog Discovery and Management)
**Status:** Refined (execution-ready; UI mockup approved)
**UI mockups:** [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html) (approved) — extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html); see [UI design handoff](#ui-design-handoff)

## Purpose

Extend the first browser increment (TASK-6, `docs/specifications/browser-requirements.md`) so the same offline, project-local, read-and-edit web interface also serves the backlog store: active task, milestone, and release records. The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds three backlog categories, their record models, list/view/edit screens, API endpoints, and tests. It implements the backlog-store part of REQ-06.002 under the same requirement, and advances NFR-01.002 (display and edit all Papyrus content) for the backlog record types.

## Scope

### In scope

- Extending the persistent sidebar and the `?type=` category selector to cover `task`, `milestone`, and `release` in addition to `req` and `nfr`, without changing the list-page structure (baseline spec section 4.6).
- Listing active task, milestone, and release records, each category rendering only its own section.
- Viewing a task, milestone, or release record, including its backlog-specific fields.
- Editing a task, milestone, or release record through the same optimistic-concurrency and atomic-write contract as requirement records (ADR-0006).
- A read/write JSON API over the same backlog records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- Archived tasks and milestones are not listed and are not reachable (active only), mirroring deprecated requirements.
- Unit and integration tests extending `tests/browser_cli.rs` for the new categories.

### Out of scope

- Creating, archiving, restoring, or deleting tasks, milestones, or releases from the browser; moving tasks between milestones; changing record IDs. These remain CLI-only, consistent with the baseline's no-lifecycle-operations stance (REQ-05.007).
- Browsing decision records, glossary terms, UI specifications, business rules, audit records, deployment records, or specification documents — later tickets (TASK-15, TASK-19, TASK-23 … TASK-27).
- Search inside the browser UI, Markdown rendering, rich-text editing, attachments, or file uploads.
- Authentication, authorization, sessions, cookies, or TLS.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-12.

## Behavior

### 1. Category model

1. The category selector (`?type=`) accepts five values: `req` (default), `nfr`, `task`, `milestone`, `release`. Any other value, including `all`, is `400`.
2. `GET /` and `GET /?type=req` render the same page (baseline rule retained).
3. The sidebar lists three entries in this order: Business requirements, Non-functional requirements, Backlog. The Backlog entry covers task, milestone, and release records as a single unified view (approved MOCK-0002). The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The sidebar remains extensible: a later increment adds a category by adding one entry and one route value.
4. The list page renders only the selected category; the other categories are not rendered at all.
5. A missing or unreadable backlog directory is not a server error: the corresponding category renders its empty state and its API returns an empty array (consistent with the baseline and with `task list`/`milestone list`/`release list`).
6. Archived tasks and milestones are never listed, counted, or reachable by URL, and return `404` exactly as for an unknown ID. Releases have no archived state.

### 2. Record sources and IDs

1. Task records are read from `<backlog_directory>/tasks/` (active folder only). Milestone records from `<backlog_directory>/milestones/`. Release records from `<backlog_directory>/releases/`. Directories are resolved from the effective configuration; prefixes and directories are never hardcoded.
2. Only files directly inside those directories whose `.md` file stem is a valid record ID (baseline section 7.1) are records. Subdirectories (including `archived/`), `index.html`, and non-`.md` files are ignored. There is no recursive traversal.
3. Record IDs are the file stems (`TASK-001`, `MS-001`, `REL-001` by default prefixes). The document type and record form are derived from the directory that matched, never from the request.
4. ID resolution is by exact, case-sensitive match against the active directory listing of the matching category. An ID that matches no active record is `404`; an ID that matches records in more than one category is `500` with a generic message (baseline section 7.1, extended to five categories).
5. Record ID, category, and (for tasks) milestone/release assignment metadata that is not an editable field are never accepted from the request.

### 3. List page

1. For `task`, `milestone`, and `release` categories the list page renders one section (`h2`) titled with the category name, showing the category's record count, followed by one row per record in ascending ID order.
2. Each row's primary link text is `ID + title` and links to `/doc/{id}`.
3. Task rows additionally show, as read-only text in the row: status, milestone ID (if any), and assignee (if any). Milestone rows additionally show the release ID (if any). Release rows show no extra metadata.
4. Empty states: a project with no records of the selected category shows the category's empty state; a category with records but none matching a (future) filter shows the same empty state. (No filtering is offered in this increment; filtering remains CLI-only.)
5. Long titles wrap inside their row (baseline section 10.4).

### 4. Record view

1. Task view shows:
   - ID (read-only), record form (`Task`), category (`Task`), title;
   - Status;
   - Milestone ID and title as a link when the milestone resolves, otherwise the stored value or `(none)`;
   - Assignee (or `(none)`);
   - Labels (list, or an explicit empty state);
   - Dependencies as links whose text is `ID + title` when they resolve, otherwise the stored ID as text;
   - Acceptance criteria (list, or an explicit empty state);
   - Comments (list of `author: text`, or an explicit empty state);
   - Description (or `(none)`).
2. Milestone view shows:
   - ID, record form (`Milestone`), category, title;
   - Release ID and title as a link when it resolves, otherwise the stored value or `(none)`;
   - Description (or `(none)`).
3. Release view shows:
   - ID, record form (`Release`), category, title;
   - Description (or `(none)`).
4. Breadcrumb on the view page: `Tasks` / `Milestones` / `Releases` (a link to `/?type=task|milestone|release`), then the record ID as the current, non-linked crumb. There is no owning-group crumb for backlog records.
5. The view page offers exactly one action, `Edit`. There is no "Back to ..." action; the breadcrumb and sidebar are the navigation.
6. Requirement references and other IDs inside field values are displayed as literal text, never auto-linked, except where this specification explicitly lists a field as a link (milestone, dependencies).

### 5. Edit form

1. Editable fields:
   - Task: `title` (required), `description` (optional), `status` (required, must be one of the configured statuses), `milestone` (optional ID or empty), `assignee` (optional), `labels` (optional, one per line), `dependencies` (optional, one ID per line), `acceptance_criteria` (optional, one per line).
   - Milestone: `title` (required), `description` (optional), `release` (optional ID or empty).
   - Release: `title` (required), `description` (optional).
2. Comments are not editable in the browser; they are displayed read-only on the view page and managed via the CLI. (Assumption: see Open questions.)
3. Record ID, category, and (for tasks) the milestone/release assignment's resolution are displayed as read-only text and are never accepted from the request.
4. Form fields and media types follow the baseline section 5.5: `version` (required), `title` (required), `description`, plus the backlog-specific fields above. Unknown form fields are ignored; repeated fields use last-value-wins; percent-decoding follows baseline rules.
5. Breadcrumb on the edit page: `Tasks › {id} › Edit` (and the milestone/release equivalents); `Edit` is the current, non-linked crumb. The owning milestone/release appears only in the read-only metadata block.
6. The hidden `version` input and the visible, non-interactive dashed annotation `Hidden field: version (optimistic-concurrency token)` are present, exactly as in the baseline (section 4.23).
7. Field help text is shown where applicable (for example "One value per line", "A line starting with `## ` is not allowed").
8. Dirty state, validation-error state, conflict state (409), and success behavior (redirect to view, save-then-reread) follow the baseline sections 4.23 and 7.12.

### 6. Validation

1. `title` follows baseline section 7.3: trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters.
2. `description` follows baseline section 7.3/7.4: trimmed; at most 20000 Unicode scalar values; LF allowed; CR/CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section; a line whose first two characters are `## ` is rejected.
3. `status` (task) must be one of the configured statuses; an unknown value is a validation error (`Status is not a configured status.`).
4. `milestone` (task) must be empty or resolve to an active milestone; an unknown value is a validation error (`Milestone not found.`).
5. `release` (milestone) must be empty or resolve to an existing release; an unknown value is a validation error (`Release not found.`).
6. `labels`, `dependencies`, and `acceptance_criteria` (task) are parsed as one value per line, trimmed; empty lines are ignored. Each value must be 1–200 Unicode scalar values with no control characters. A dependency value must resolve to an active task and must not equal the record's own ID (`Dependency not found.` / `A task cannot depend on itself.`).
7. Form-level error summary links each entry to its field and names the actual field error, using the same text as that field's message (baseline section 5.5/7.5).
8. Validation failures return `422` and never modify the file.

### 7. Persistence and editing contract

1. On save, the file is read as UTF-8 text; the first line is replaced with `# <ID> <title>`; each known section keeps its original position; a non-empty value replaces that section's body; an empty value removes the section; a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end.
2. Known section order:
   - Task: `## Description`, `## Status`, `## Milestone`, `## Assignee`, `## Labels`, `## Dependencies`, `## Acceptance Criteria`, `## Comments`.
   - Milestone: `## Description`, `## Release`.
   - Release: `## Description`.
3. Unknown and editor-added sections are preserved content-identically, in their original order and position (baseline section 7.7).
4. A task record with no `## Milestone` section is displayed and edited as a task without a milestone; saving never adds a `## Milestone` section unless a value is supplied. The same applies to `## Assignee`, `## Labels`, etc.
5. Writes are atomic and project-local, following baseline section 7.10 (temporary file, `sync_all`, permission copy, rename, directory sync, cleanup on failure).
6. Every read and write path passes through `Context::ensure_within` (baseline section 7.11). Symlink escapes are rejected with `404`.
7. Optimistic concurrency follows baseline section 7.12: version token `"<mtime-seconds>.<mtime-nanos>.<length>"`; missing/malformed `version` → `400`; stale token → `409` with message "record changed on disk; reload the form"; matching token proceeds.
8. After a successful write the server does not keep a cached copy; the redirect target re-reads the file (save-then-reread).
9. Editing a record never changes its ID, its directory, or its archived state.

### 8. JSON API

1. `GET /api/tasks`, `GET /api/milestones`, and `GET /api/releases` return `200 application/json` with an array of summary objects ordered by ID ascending:
   - Task: `{"id","title","status","milestone","assignee"}`.
   - Milestone: `{"id","title","release"}`.
   - Release: `{"id","title"}`.
2. `GET /api/doc/{id}` and `POST /api/doc/{id}` work for task, milestone, and release IDs in addition to requirement IDs, returning the full record shape:
   - Task: `{"id","kind":"task","form":"task","title","status","milestone":{"id","title"}|null,"assignee","labels":[],"dependencies":[],"acceptance_criteria":[],"comments":[{"author","text"}],"description","version","path"}`.
   - Milestone: `{"id","kind":"milestone","form":"milestone","title","release":{"id","title"}|null,"description","version","path"}`.
   - Release: `{"id","kind":"release","form":"release","title","description","version","path"}`.
3. `POST /api/doc/{id}` accepts a JSON object with `version` (required), `title` (required), and the editable fields for that record type. Omitted fields are left unchanged; an explicit `null` clears an optional field. Unknown members are ignored; a non-object body or a member of the wrong type receives `400`.
4. Error responses follow baseline section 5.3 (HTML error page or JSON error object with `code`/`message`).

### 9. Assets, rendering, security, and accessibility

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates/assets, `{{name}}` placeholder substitution, HTML escaping, CSP, Host/Origin validation, request limits, timeouts, browser baseline (Firefox/Edge/Safari 100+), accessibility basics, no external resources, no inline scripts/styles.
2. Templates may be extended with backlog-specific placeholders; unknown placeholders are left untouched as in the baseline.
3. The sidebar's Backlog entry and the backlog list/view/edit templates are the only rendering additions.

### 10. Tests

1. Unit tests cover: ID validation for the new ID shapes, record parsing/serialization for task/milestone/release, field validation for the new fields, version token, JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new categories, including at least:
   - T-11a: list renders one backlog category at a time, ordered by ID, with the selected sidebar entry marked `aria-current="page"`.
   - T-11b: `?type=task|milestone|release` works, default remains `req`, and any other value is `400`.
   - T-11c: view shows all backlog-specific fields; milestone/dependency links resolve.
   - T-11d: edit round-trip persists and is visible through both the UI and the CLI.
   - T-11e: field validation returns `422` and does not write (empty title, unknown status, unknown milestone/release, dependency on self, `## ` line).
   - T-11f: stale version token returns `409` and does not write.
   - T-11g: archived tasks/milestones are not listed and return `404` by URL.
   - T-11h: writes are atomic and leave no temporary file; symlink escape returns `404`.
   - T-11i: JSON API returns the documented shapes and rejects bad bodies.
   - T-11j: build, tests, clippy, and formatting pass with existing dependencies only.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Baseline artifact:** MOCK-0001 — [`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`](ui/mockups/MOCK-0001-browser-requirements-review.html), approved 2026-11-03 (UTC). It remains the authoritative layout reference for the requirement categories and the shared shell.

**Extension artifact (TASK-11):** MOCK-0002 — produced by `@ui-designer` as a self-contained review package that reuses the MOCK-0001 shell and adds the unified Backlog view (tasks, milestones, releases) with list, view, edit, empty, error, and 320 px states. MOCK-0001 is not modified in place. Its status is `approved` (explicitly approved by the user on 2026-11-04, TASK-11).

### Screens to be covered by the extension package

1. **Shared shell** (reused from MOCK-0001): header, sidebar with three entries (Business requirements, Non-functional requirements, Backlog), `main`, footer, one `h1`.
2. **Task list** (`/?type=task`): rows with ID + title link, status, milestone, assignee; empty state; long titles.
3. **Milestone list** (`/?type=milestone`): rows with ID + title link, release; empty state.
4. **Release list** (`/?type=release`): rows with ID + title link; empty state.
5. **Task view** (`/doc/{id}`): metadata block (ID, form, category, status, milestone, assignee, labels, dependencies, acceptance criteria, comments), description, breadcrumb `Tasks › ID`, single `Edit` action.
6. **Milestone view**: metadata block (ID, form, category, release), description, breadcrumb.
7. **Release view**: metadata block, description, breadcrumb.
8. **Edit forms**: task (title, description, status, milestone, assignee, labels, dependencies, acceptance criteria, version annotation), milestone (title, description, release), release (title, description); pristine, dirty, validation-error, conflict, and 320 px states.
9. **Error pages**: reused from MOCK-0001 (404 unknown/archived, 400, 405, 415, 500).

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | The browser displays and edits active task, milestone, and release records in addition to requirement content. |
| AC-2 | Browser operations remain offline-capable, use project-local text-file persistence, sanitize input, and access no paths outside the project directory except during init. |
| AC-3 | The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later. |
| AC-4 | Archived tasks/milestones are not listed and return `404` by URL. |
| AC-5 | Optimistic concurrency and atomic writes hold for the new record types. |

## Assumptions

- Assumption: The transport, asset, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused without change; this ticket adds categories and record types only.
- Assumption: "Full lifecycle fields" for tasks means title, description, status, milestone, assignee, labels, dependencies, and acceptance criteria are browser-editable; comments are displayed read-only and managed via the CLI. (User selected full lifecycle fields; comments editing is deferred as an open question.)
- Assumption: Milestone editable fields are title, description, and release assignment; release editable fields are title and description.
- Assumption: Archived tasks/milestones are hidden and unreachable, mirroring deprecated requirements; releases have no archived state.
- Assumption: The category selector is extended to `task`, `milestone`, and `release` rather than introducing new top-level routes, keeping the list-page structure unchanged (baseline section 4.6).
- Assumption: Labels, dependencies, and acceptance criteria are edited as one value per line in a textarea; empty lines are ignored.
- Assumption: The configured statuses are the allowed values for a task's `status` field in the browser.
- Assumption: A new MOCK id (MOCK-0002) is allocated for the extension rather than modifying the approved MOCK-0001 in place, per the global sequential-id rule.
- Assumption: No new dependencies, crates, or ADRs are introduced.
- Assumption: AC #10 (real-browser verification in Firefox, Microsoft Edge, and Safari 100+) and NFR-05.001 are deferred follow-ups, not blocking for this ticket; tracked as a follow-up rather than a release gate.

## Open questions

- **OQ-1 (resolved: comments read-only in the browser)** — Confirmed by user: comments are displayed read-only; comment management stays CLI-only.
- **OQ-2 (recommended default: one value per line for list fields)** — Labels, dependencies, and acceptance criteria are stored as Markdown lists. The browser edits them as one value per line. If a different delimiter or rich editing is desired, the parsing rules change.
- **OQ-3 (resolved: per-line 200-character cap)** — Confirmed by user: labels, dependencies, and acceptance-criteria items are capped at 200 Unicode scalar values per line.
- **OQ-4 (resolved: new MOCK-0002 delivered)** — Confirmed by user: MOCK-0001 stays approved; the extension is delivered as approved MOCK-0002 reusing the approved shell.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.002, REQ-02, REQ-02.001, REQ-02.002, REQ-02.003, REQ-02.004, REQ-03, REQ-03.001, REQ-03.002, REQ-05.001, REQ-05.002, REQ-05.007 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-005 (task command behavior), BR-006 (milestone command behavior), BR-007 (release command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0002, ADR-0003, ADR-0004, ADR-0005, ADR-0006 |
| Ticket | TASK-6 (baseline), TASK-8, TASK-9, TASK-11 |
| Increment boundary | REQ-06.002 covers both stores; the documentation-store part was delivered by TASK-6/TASK-7, the backlog-store part by this ticket. |
| UI mockups | MOCK-0001 (approved baseline); MOCK-0002 (approved) — `docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html` |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/task-management.md` (TASK-9), `docs/specifications/milestone-and-release-management.md` (TASK-8) |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5/TASK-10) |
