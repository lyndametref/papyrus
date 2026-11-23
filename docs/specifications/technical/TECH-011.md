---
id: TECH-011
title: "Browser Access for Backlog Records (Tasks, Milestones, Releases)"
created_date: '2026-11-16T17:46:44Z'
updated_date: '2026-11-19T21:42:42Z'
labels: []
references:
  - ADR-0001
  - ADR-0003
  - ADR-0004
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - ADR-0022
  - BR-004
  - BR-005
  - BR-006
  - BR-007
  - BR-016
  - MOCK-0001
  - MOCK-0002
  - MOCK-0017
  - NFR-001.002
  - NFR-005.001
  - REQ-001.002
  - REQ-002
  - REQ-002.001
  - REQ-002.002
  - REQ-002.003
  - REQ-002.004
  - REQ-003
  - REQ-003.001
  - REQ-003.002
  - REQ-005.001
  - REQ-005.002
  - REQ-005.007
  - REQ-006
  - REQ-006.002
  - REQ-006.003
  - REQ-006.006
  - REQ-006.007
  - REQ-006.008
  - REQ-006.009
  - REQ-006.010
  - REQ-006.011
  - REQ-006.012
  - REQ-006.013
  - TASK-001
  - TASK-10
  - TASK-101
  - TASK-102
  - TASK-103
  - TASK-11
  - TASK-12
  - TASK-147
  - TASK-15
  - TASK-19
  - TASK-23
  - TASK-27
  - TASK-5
  - TASK-6
  - TASK-7
  - TASK-8
  - TASK-86
  - TASK-9
  - REQ-006.020
  - REQ-006.021
  - REQ-006.022
  - RE-008
  - ADR-0032
  - TASK-205
status: draft
---
# TECH-011 Browser Access for Backlog Records (Tasks, Milestones, Releases)

## Description
> **Revised (TASK-147, 2026-11-15):** section 3 gains the per-milestone **Clean up all** action required by the 2026-11-15 revision of REQ-006.013. This revision supersedes the earlier out-of-scope statement that every archiving operation is CLI-only; creating, restoring, and deleting tasks remain CLI-only, and no project-wide bulk archive is added. The board layout, the board's status change, and archived exclusion are unchanged. The action, its confirmation warning, and its failure report are drawn by [`MOCK-0017`](ui/mockups/MOCK-0017-browser-cleanup-all-board-action-review.html) and recorded in [`ui/browser-backlog-records-ui.md`](ui/browser-backlog-records-ui.md); MOCK-0017 stays `draft` because the approval step was waived for the TASK-147 implementation run, so the visible design awaits owner review.

> **Revised (TASK-205/MS-030, 2026-11-19):** section 3 **Release grouping** item 4 now follows REQ-006.020 (ordering by lowest contained non-archived task ordinal ascending, replacing ascending milestone/release ID ordering); new **Lifecycle markers** subsection per REQ-006.021 (🔘/✅ before titles); new **Milestone and release cleanup controls** subsection per REQ-006.022 (browser-only archive controls with confirmation and partial-failure handling). ADR-0022, ADR-0032, ADR-0006, and ADR-0013 remain authoritative; no new endpoint, write path, or ADR is introduced. Status moved to draft pending verification.
## Purpose

Extend the first browser increment (TASK-6, `docs/specifications/browser-requirements.md`) so the same offline, project-local, read-and-edit web interface also serves the backlog store: active task, milestone, and release records. The command contract, transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged; this increment adds three backlog categories, their record models, list/view/edit screens, API endpoints, and tests. It implements the backlog-store part of REQ-006.002 under the same requirement, and advances NFR-001.002 (display and edit all Papyrus content) for the backlog record types.

## Scope

### In scope

- Extending the persistent sidebar and the `?type=` category selector to cover `task`, `milestone`, and `release` in addition to `req` and `nfr`, without changing the list-page structure (baseline spec section 4.6).
- Listing active task, milestone, and release records, each category rendering only its own section.
- Viewing a task, milestone, or release record, including its backlog-specific fields.
- Editing a task, milestone, or release record through the same optimistic-concurrency and atomic-write contract as requirement records (ADR-0006).
- A read/write JSON API over the same backlog records.
- HTML escaping, ID resolution, boundary checks, Host/Origin validation, request limits, and browser-support/a11y rules reused from the baseline.
- The per-milestone `Clean up all` action of the 2026-11-15 revision of REQ-006.013, its server-rendered confirmation warning, its re-use of the existing task archive entry point, and its per-task failure report.
- Archived tasks and milestones are not listed and are not reachable (active only), mirroring deprecated requirements.
- Unit and integration tests extending `tests/browser_cli.rs` for the new categories.

### Out of scope

- Creating, restoring, or deleting tasks, milestones, or releases from the browser; moving tasks between milestones; changing record IDs; archiving a single task from the record view; and any bulk archive other than the per-milestone `Clean up all` action of REQ-006.013. These remain CLI-only, consistent with the baseline's no-lifecycle-operations stance (REQ-005.007).
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

The combined Backlog view (`/?type=task`, sidebar entry "Backlog") is the per-milestone kanban board required by REQ-006.013 and keeps its card content; only its width class is `Full`. The milestone (`?type=milestone`) and release (`?type=release`) category pages render through the shared list layout in `browser-shell-ui.md` §3: one section per category, one row per record in ascending ID order, the milestone `Release` family slot, and no extra slot for release. The empty states and long-title wrapping follow the shared contract.

**Board layout**

1. Milestones remain expandable and collapsible sections (existing behavior, retained).
2. Expanding a milestone renders that milestone's tasks as a kanban board: one column per configured task status (REQ-001.002), laid out horizontally within the milestone.
3. Columns appear in configured status order exactly; no reordering, no omission.
4. A status that has no tasks for that milestone keeps its column visible (empty column with an explicit empty state).
5. The board is nested inside the milestone section; milestones remain the grouping unit of the view.
6. A task whose stored status is not one of the configured statuses (or is absent) is shown in a trailing `Unconfigured status` column that appears only when at least one such task exists, so no active task disappears from the board.
7. Exactly one milestone is expanded by default: the first milestone in render order that has at least one active task. `GET /?type=task&expand={milestone-id}` expands that milestone instead; the parameter is honoured only for an active milestone ID and is otherwise ignored.

**Release grouping**

1. Milestones assigned to a release appear under that release.
2. Milestones assigned to no release are grouped together before the first release.
3. Each release boundary is marked by a bold separator that names the release and does not rely on colour alone.
4. Ordering of milestones and releases on the board follows REQ-006.020 and is recomputed from current record data on every render: milestones inside a release are ordered by the lowest ordinal among their non-archived tasks ascending; a milestone with no non-archived task is ordered after every milestone that has one, by milestone ID ascending; milestones with equal lowest ordinals are ordered by milestone ID ascending. Milestones in the no-release group follow the same rule. Releases are ordered by the lowest ordinal among the non-archived tasks in their milestones ascending; a release with no such task is ordered after every release that has one, by release ID ascending; releases with equal lowest ordinals are ordered by release ID ascending. The no-release group precedes all releases.
5. A release with no active milestones still renders its separator and an explicit empty state for the group.

**Lifecycle markers (REQ-006.021)**

1. Every displayed open milestone shows the marker `🔘` immediately before its title; every displayed closed milestone shows `✅` immediately before its title.
2. Every displayed planned release shows `🔘` immediately before its title; every displayed released release shows `✅` immediately before its title.
3. The marker is placed immediately before the title text in the milestone summary and in the release heading, in addition to the existing record ID and title, and does not replace status information shown elsewhere.
4. Archived milestones are not displayed and therefore carry no marker.
5. The marker reflects the stored lifecycle state and changes after that state changes and the view is reloaded.

**Milestone and release cleanup controls (REQ-006.022)**

1. Each displayed closed milestone shows a cleanup control immediately after its title.
2. Each displayed released release that contains at least one closed milestone shows a cleanup control immediately after its title.
3. A released release with no closed milestone shows no cleanup control.
4. Activating a milestone cleanup control shows a confirmation that lists the milestone ID that will be archived; confirming archives that milestone using the milestone archive semantics of REQ-003.001 and ADR-0032, removing it from the view; cancelling archives nothing.
5. Activating a release cleanup control shows a confirmation that lists the IDs of the closed milestones in that release that will be archived; confirming archives each listed milestone, one by one, using the same archive semantics; the release itself remains visible and released and is never archived.
6. If archiving one milestone fails, the failure is reported for that milestone, the remaining milestones are still archived, and the failed milestone stays visible.
7. The cleanup controls are browser-only actions: they introduce no CLI command, no JSON API endpoint, and no new write path; archiving reuses the existing task/milestone archive entry point and the single write path of ADR-0013.
8. Milestone and release lifecycle rules are unchanged by these controls.

**Tasks without a milestone**

1. Tasks that belong to no milestone appear in their own expandable section.
2. That section is placed after all milestone sections (including all release groups).
3. Cards in the section are ordered by ascending task ID.
4. Each card shows the task ID and title and links to the task record view.
5. A task whose stored milestone value is absent or does not resolve to an active milestone belongs to this section. The section is omitted when no task belongs to it.

**Card order and contents**

1. Cards are ordered by ascending task ID within a column.
2. Each card shows at least the task ID and title and links to the task record view (`/doc/{id}`).
3. Card contents beyond the ID and title (assignee, labels, and similar) are design decisions and are not fixed by this specification.
4. Column widths, separator styling, and empty-column presentation details are design decisions.

**Status change from the board**

1. A task's status is changed by dragging its card into another status column.
2. Each card also offers an equivalent status control that performs the same change without dragging (pointer-free operable).
3. Either action persists the status immediately to the task record; the board is not a staging area with a separate save step.
4. Persistence uses the existing browser write contract: the version token of ADR-0006 and the single write path of ADR-0013. No second write path, endpoint, or ADR is introduced.
5. The change alters only the status value and preserves every other field of the task.
6. A change that cannot be persisted leaves the card in its original column and reports the failure to the user.
7. After a successful change the value is still shown after the view is reloaded (save-then-reread).
8. The board submits its status change as a status-only form POST to the existing `/doc/{id}` endpoint: the form carries the version token, the new `status`, and a status-only marker. The endpoint applies a status-only patch — the `status` front-matter key when the record carries front matter, otherwise the `## Status` section — and changes nothing else apart from the routine `updated_date` refresh. No new route, endpoint, or write path is added.
9. A successful status change redirects `303` to `/?type=task`, carrying `expand={milestone-id}` when the card belongs to a milestone so that the changed card stays visible. A failed change re-renders the board from the current stored data with the card in its stored column and an alert that names the failure: `409` for a stale version token, `422` for a status that is not configured, and `500` for a write failure.

**Clean up all (per-milestone bulk archive)**

1. The **final configured task status** is the last entry of the configured task `statuses` list (REQ-001.002, REQ-002.003), and it is the board's last column. The `active` comparison is an exact, case-sensitive comparison of that entry's configured string with `active`.
2. The action is offered in the header of the final configured status column of every milestone board when the final configured status differs from `active`. When the final configured status is `active`, no milestone board offers the action.
3. The action is not offered in any other column, not in the trailing `Unconfigured status` column, and not in the `Tasks without a milestone` section, which is not a milestone board. It is offered for a closed milestone exactly as for an open one, because closed milestones still render.
4. The action is offered whenever clause 2 holds, whether or not the column currently contains tasks. When the column is empty, the warning in clause 6 states that no task will be archived and offers no confirm control.
5. Activating the action is a `GET` preview on the board route: `GET /?type=task&expand={milestone}&cleanup={milestone}`. The preview writes nothing, keeps that milestone expanded, and replaces the column's action with the confirmation region of clause 6. Cancel is a plain link back to `/?type=task&expand={milestone}`, so cancelling archives nothing and performs no request that can write.
6. The confirmation region is server-rendered markup on the same page — no script dialog, no inline script, no client-side confirmation — so the action works with JavaScript disabled under the existing CSP. It names the milestone, states how many tasks will be archived, lists the affected task IDs in ascending ID order (each with its title), carries the word `warning`, and contains one form with a confirm submit control named `Archive 1 task` for a single task and `Archive {count} tasks` otherwise, and the cancel link of clause 5.
7. The confirm form posts to the existing board route `/?type=task` (no new path is added) with `cleanup=1` and `milestone={milestone}`. The server re-derives the final configured status from the current configuration, resolves the milestone from the enabled milestone index, and re-reads the task index, so the archived set is the column's contents for that milestone at confirm time. A milestone that is absent, archived, or not enabled is `404`; a request made when the final configured status is `active` is `422`; a missing or empty `milestone` field is `400`.
8. Each task in the set is archived independently through the existing task archive entry point — `papyrus task archive` semantics (REQ-002.004), the archive location and sequence of REQ-005.007 and ADR-0022, and the single write path of ADR-0013. No second write path, no directory move performed outside that path, and no new endpoint for the write. Archiving preserves each task's ID, dependencies, and previous status.
9. A failure for one task is reported for that task and does not abort the remaining archives; the loop continues until every task in the set has been attempted. No rollback is attempted for tasks already archived.
10. When every task is archived, the response is `303` to `/?type=task&expand={milestone}`; the re-read board no longer shows them. When one or more fail, the board is re-rendered from the current stored data with an alert region (`role="alert"`) that states how many tasks were archived out of the total and lists each failed task ID with its failure message; `200` is returned because the request did archive records and only part of it failed. A request whose set is empty redirects `303` and archives nothing.
11. The bulk form carries no per-task version token: archiving is per-record idempotent, a task whose archive destination already exists fails with a per-task message, and the request remains guarded by the baseline Host and Origin validation and the form media-type check. No optimistic-concurrency claim is made for this action, and the status form's version-token contract of the previous section is unchanged.
12. The action archives only tasks of the milestone whose board hosts it; it never archives another milestone's tasks, the without-milestone section, or the project. Archived tasks stay excluded from the board afterwards, consistent with archived exclusion below. No CLI equivalent and no JSON API endpoint exists for this action in this release.
13. No other board behaviour changes: card contents, card order, column order, empty-column text, release grouping, default expansion, and the status change of the previous section are all unaffected.

**Archived exclusion**

1. Archived tasks and archived milestones are not shown on the board.
2. Archived exclusion is consistent with the rest of the browser backlog views.

**Presentation details that remain design decisions**

1. Card contents beyond the task ID and title.
2. Column widths.
3. Separator styling and weight beyond "bold" and "names the release".
4. The shape of the non-drag status control (select, buttons, menu).
5. Default expansion state of milestones.
6. Empty-column placeholder text.
7. Placement of the `Clean up all` action inside the final column header (top or bottom of the header, its label wording beyond `Clean up all`, and whether it is a link or a button).
8. Layout of the confirmation warning above the milestone's columns (ordering of the count, the ID list, and the controls; list bullets versus a comma-separated run).
9. Presentation of the post-action failure list on the board.

Items 7–9 are resolved by MOCK-0017 (`draft`; recorded in [`ui/browser-backlog-records-ui.md`](ui/browser-backlog-records-ui.md) §4a, §8, §9, and §10 and in the mockup registry).

These are resolved by TASK-102 (mockup revision), with the no-JavaScript submission affordance refined under TASK-103, and recorded in the UI description: cards show assignee and labels; columns are equal thirds; the release separator is bold with a 2 px bottom border; the non-drag control is a labelled select plus a `Change status` submit button; the first milestone with active tasks is expanded; an empty column reads `No tasks in this status.`

**Empty states and wrapping**

1. Empty states: a project with no records of the selected category shows the category's empty state; a category with records but none matching a (future) filter shows the same empty state. (No filtering is offered in this increment; filtering remains CLI-only.)
2. Long titles wrap inside their row/card (baseline section 10.4).

### 4. Record view

Record views in this specification follow the shared single record view layout defined in section 4c of `docs/specifications/browser-requirements.md` (summary card with ID, created date, updated date; labels as a taglist under the title; dependencies and references at the end as native disclosure sections), in addition to the type-specific fields below. Record references that resolve to Papyrus records follow the shared reference-link rule defined in section 4b of `docs/specifications/browser-requirements.md`.
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

1. All baseline rules in sections 6, 9, and 10 of `docs/specifications/browser-requirements.md` apply unchanged: embedded templates rendered server-side and every `/assets/*` file served from the static asset table, `{{name}}` placeholder substitution, HTML escaping, CSP, Host/Origin validation, request limits, timeouts, browser baseline (Firefox 115+; NFR-005.001), accessibility basics, no external resources, no server-authored inline scripts or styles (the diagram page injects a scoped `<style>` element client-side under the diagram CSP; see `browser-requirements.md` §9.7).
2. Templates may be extended with backlog-specific placeholders; unknown placeholders are left untouched as in the baseline.
3. The sidebar's Backlog entry and the backlog list/view/edit templates are the only rendering additions.

### 10. Tests

1. Unit tests cover: ID validation for the new ID shapes, record parsing/serialization for task/milestone/release, field validation for the new fields, version token, JSON shapes.
2. Integration tests extend `tests/browser_cli.rs` with scenarios for the new categories, including at least:
   - T-11a: milestone and release lists render through the shared layout (`data-list-layout="shared"`) with the milestone `Release` slot, ordered by ID, with the selected sidebar entry marked `aria-current="page"`; the task route still renders the board.
   - T-11b: `?type=task|milestone|release` works, default remains `req`, and any other value is `400`.
   - T-11c: view shows all backlog-specific fields; milestone/dependency links resolve.
   - T-11d: edit round-trip persists and is visible through both the UI and the CLI.
   - T-11e: field validation returns `422` and does not write (empty title, unknown status, unknown milestone/release, dependency on self, `## ` line).
   - T-11f: stale version token returns `409` and does not write.
   - T-11g: archived tasks/milestones are not listed and return `404` by URL.
   - T-11h: writes are atomic and leave no temporary file; symlink escape returns `404`.
   - T-11i: JSON API returns the documented shapes and rejects bad bodies.
   - T-11j: build, tests, clippy, and formatting pass with existing dependencies only.
   - T-11k: the `Clean up all` action is offered only in the final configured status column of a milestone board, and is absent when that status is `active` (including a single-status configuration whose only status is `active`), absent in every other column, absent in the `Unconfigured status` column, and absent in the `Tasks without a milestone` section.
   - T-11l: the preview lists exactly the IDs of that milestone's tasks in that column in ascending ID order, and cancel (no POST) archives nothing.
   - T-11m: confirming archives every matching task, returns `303` to `/?type=task&expand={milestone}`, and the re-read board no longer shows them; no other task is touched.
   - T-11n: with one induced archive failure at the archive destination of the first task in ascending order, the remaining tasks are still archived, the response is `200`, and the alert names the failed ID with its message. A duplicate archived file is invisible through duplicate-ID suppression, which drops the task from the board index, so the fixture places a directory at the destination; it takes the same refusal path (`already archived`).
   - T-11o: an identically named final status column in another milestone is untouched, and a without-milestone task carrying that status is untouched.
   - T-11p: no `papyrus task` CLI equivalent, no JSON API route, and no new write path are introduced; the existing status-form tests pass unchanged.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Baseline artifact:** MOCK-0001 — [`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`](ui/mockups/MOCK-0001-browser-requirements-review.html), approved 2026-11-03 (UTC). It remains the authoritative layout reference for the requirement categories and the shared shell.

**Extension artifact (TASK-11):** MOCK-0002 — produced by `@ui-designer` as a self-contained review package that reuses the MOCK-0001 shell and adds the unified Backlog view (tasks, milestones, releases) with list, view, edit, empty, error, and 320 px states. MOCK-0001 is not modified in place. Its status is `approved`, most recently for the kanban-board revision on 2026-11-12 UTC (TASK-102), which supersedes the baseline approval of 2026-11-04 (TASK-11).

### Screens to be covered by the extension package

1. **Shared shell** (reused from MOCK-0001): header, sidebar with three entries (Business requirements, Non-functional requirements, Backlog), `main`, footer, one `h1`.
2. **Backlog board** (`/?type=task`): milestones as expandable/collapsible sections grouped by release with a bold named separator, each expanded milestone showing its tasks as a kanban board (one column per configured status, empty columns visible), a tasks-without-milestone section after all milestones, cards showing ID + title link, drag and non-drag status change, the failed-persistence state, and 320 px layout. The `Clean up all` revision adds the action in the final status column header, the server-rendered confirmation warning with the affected ID list, the empty-column warning state, and the post-action failure report; MOCK-0002 does not cover these and the new mockup does.
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
| AC-3 | The expanded interface renders in Firefox 115+ (NFR-005.001). |
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
- Assumption: AC #10 (real-browser verification in Firefox 115+) and NFR-005.001 are deferred follow-ups, not blocking for this ticket; tracked as a follow-up rather than a release gate.

## Open questions

- **OQ-1 (resolved: comments read-only in the browser)** — Confirmed by user: comments are displayed read-only; comment management stays CLI-only.
- **OQ-2 (recommended default: one value per line for list fields)** — Labels, dependencies, and acceptance criteria are stored as Markdown lists. The browser edits them as one value per line. If a different delimiter or rich editing is desired, the parsing rules change.
- **OQ-3 (resolved: per-line 200-character cap)** — Confirmed by user: labels, dependencies, and acceptance-criteria items are capped at 200 Unicode scalar values per line.
- **OQ-4 (resolved: new MOCK-0002 delivered)** — Confirmed by user: MOCK-0001 stays approved; the extension is delivered as approved MOCK-0002 reusing the approved shell.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-006, REQ-006.002, REQ-006.003, REQ-006.006, REQ-006.007, REQ-006.008, REQ-006.009, REQ-006.010, REQ-006.011, REQ-006.012, REQ-006.013, REQ-006.020, REQ-006.021, REQ-006.022, REQ-002, REQ-002.001, REQ-002.002, REQ-002.003, REQ-002.004, REQ-003, REQ-003.001, REQ-003.002, REQ-005.001, REQ-005.002, REQ-005.007, REQ-001.002 |
| Business rule | BR-004 (browser command behavior), BR-005 (task command behavior), BR-006 (milestone command behavior), BR-007 (release command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001, ADR-0003, ADR-0004, ADR-0005, ADR-0006, ADR-0010, ADR-0012, ADR-0013, ADR-0022, ADR-0032 |
| Ticket | TASK-6 (baseline), TASK-8, TASK-9, TASK-11, TASK-86, TASK-101, TASK-102, TASK-103, TASK-147 (`Clean up all` revision), TASK-205 (MS-030 architecture assessment) |
| Increment boundary | REQ-006.002 covers both stores; the documentation-store part was delivered by TASK-6/TASK-7, the backlog-store part by this ticket. |
| UI mockups | MOCK-0001 (approved baseline); MOCK-0002 (approved; kanban-board revision approved 2026-11-12 UTC under TASK-102) — `docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html` |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/task-management.md` (TASK-9), `docs/specifications/milestone-and-release-management.md` (TASK-8) |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5/TASK-10) |
| Request | RE-008 (milestone ordering by lowest task ordinal, lifecycle markers, cleanup) |
