---
id: TASK-098
title: Render the backlog as a per-milestone kanban board with card status change
status: Done
assignee: []
created_date: '2026-11-11T21:15:00Z'
updated_date: '2026-11-13T11:08:00Z'
labels:
  - browser
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-092
  - TASK-096
  - TASK-097
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/specifications/browser-spec-revisions-task-101.md
  - docs/specifications/browser-spec-revisions-task-102.md
priority: high
type: feature
ordinal: 39145
---


## Description
## Purpose

Replace the flat `GET /?type=task` task list with the per-milestone kanban board required by REQ-06.013, and let a status be changed from that board. The work is board rendering plus a status-only write that reuses the existing `POST /doc/{id}` endpoint, the ADR-0006 version token, and the single shared patcher/writer (ADR-0013 `record::patch` + `record::store::write_atomic`). No new route, endpoint, module, crate, dependency, non-functional requirement, or ADR is introduced. The approved mockup is MOCK-0002 (kanban-board revision approved 2026-11-12 UTC under TASK-102).

## Current state (verified in the repository)

- `GET /?type=task` renders a flat per-category row list: `browser::list_page` -> `render::list_page` -> `render::backlog_list_section` (heading `Tasks`, subheading `Task records in this project.`).
- `GET /?type=milestone` and `GET /?type=release` render their own row lists and must not change.
- Task status lives in front matter (`status:`) for Papyrus-native records, with a fallback to the `## Status` section for records without front matter (`task::task_from_document`).
- `browser::form_post` is the only HTML write handler (`POST /doc/{id}`, `application/x-www-form-urlencoded`, `version` required, 409 on stale token, 415 on wrong media type). It applies `record::edit::*_ops` through `save_ops` -> `edit::apply` -> `record::store::write_atomic`. Its existing task branch treats absent form fields as *clear* for milestone/assignee/lists, so a partial form post cannot be used to change only the status.
- `browser::api_post` (`POST /api/doc/{id}`) already accepts a partial JSON body, but the board must also work with JavaScript disabled, so it cannot be the only submission path.
- The mockup shell, sidebar (`Backlog` entry -> `/?type=task`, `aria-current="page"`), error pages, CSP (`script-src 'self'`, no inline script/style), and the single `app.js` progressive-enhancement script are reused unchanged.

## Scope (in)

1. `GET /?type=task` renders the combined Backlog board (h1 `Backlog`), replacing the flat task row list. Milestone and release pages are untouched.
2. Release grouping, milestone sections, status columns, cards, and the tasks-without-a-milestone section as specified below, including ordering and empty states.
3. Cards show the task ID and title as a link to `/doc/{id}`, plus assignee and labels per approved MOCK-0002.
4. Status change from the board by dragging a card into another column, and by an equivalent labelled select plus `Change status` button on the card that needs no pointer and no script.
5. A status-only write through the existing endpoint, version token, and shared writer that changes nothing but the status (plus the routine `updated_date` refresh).
6. The board failure state: card stays in its stored column and the failure is reported.
7. New board styles in `app.css` and drag-and-drop in `app.js` (classic script, no fetch, no module, no inline handler).
8. Specification bookkeeping: `docs/specifications/browser-tasks-milestones-and-releases.md` section 3 and `docs/specifications/ui/browser-backlog-records-ui.md` section 4 (updated during refinement; implement against them).
9. Tests: update the three affected existing tests and add the board/status-change suite.

## Scope (out)

- Changing `/?type=milestone`, `/?type=release`, the record views, the edit forms, the JSON API shapes, search, or the CLI.
- A new route or endpoint (for example `/doc/{id}/status`), a second write path, a new module, crate, dependency, ADR, or NFR.
- A second stage/save step on the board, inline (in-card) status editing beyond the card form, bulk status change, or reordering columns.
- Creating, archiving, restoring, or deleting tasks/milestones/releases from the browser; editing any field other than status from the board; moving tasks between milestones.
- Redesigning MOCK-0002 or allocating a new MOCK id (the approved revision already covers the board, including drag, the select control, the failure state, and 320 px).
- Changing `docs/requirements/` or `docs/business-rules/`.

## Behavior contract

### A. Page and routing

1. `GET /?type=task` (and `HEAD`) returns `200 text/html` with h1 `Backlog`, subheading `Tasks grouped by milestone and status; releases separate milestones.`, and the sidebar entry `Backlog` marked `aria-current="page"`. The flat task rows are gone; no other route changes.
2. `GET /?type=task&expand={milestone-id}` expands that milestone instead of the default. Only an active milestone ID is accepted; an unknown, archived, or malformed value is ignored and the default applies.
3. `GET /?type=milestone` and `GET /?type=release` keep the current rendering, ordering, empty states, and wording.

### B. Grouping and ordering

1. The board renders, in this order: (a) one group for milestones with no release, (b) one group per active release in ascending release ID order, (c) the tasks-without-a-milestone section.
2. The no-release group is introduced by an `h2` reading `Milestones without a release` and is omitted when no milestone belongs to it.
3. Each release group starts with an `h2` separator whose text is `{release ID} — {release title}`. It is bold with a 2 px bottom border, so the boundary does not depend on colour.
4. Milestones inside a group follow ascending milestone ID. A release with no active milestone renders its separator and the text `No milestones in this release.`
5. Archived tasks and archived milestones are never rendered or counted (the active directories are the only source, as today).
6. Every record-derived string is HTML-escaped.

### C. Milestone sections and columns

1. Each milestone is a native `<details>` disclosure with a `<summary>` reading `{milestone ID} — {milestone title}`; it works with JavaScript disabled.
2. Default expansion: exactly the first milestone in render order that has at least one active task is `open`; all other milestones are collapsed. When `expand` names an active milestone, that milestone is `open` instead (all others collapsed, including the default one).
3. An expanded milestone renders one `<section class="board-column">` per configured task status, in `cfg.statuses` order, each with an `h3` carrying the status name and a `data-status` attribute equal to the configured value.
4. A column with no task shows `No tasks in this status.`
5. When at least one active task has a stored status that is not a configured status (including no status at all), a trailing column with the heading `Unconfigured status` and `data-status=""` is rendered after the configured columns. It is omitted when no such task exists.
6. The board is nested inside the milestone section; milestones remain the grouping unit.

### D. Cards

1. Cards appear in ascending task ID order within a column (the task index is already ID-sorted).
2. A card is an `<article class="board-card" draggable="true" data-task="{task ID}">` containing:
   - an `<a href="/doc/{task ID}">` whose text is the task ID plus the task title;
   - a line `Assignee: {assignee}` or `Assignee: (none)`;
   - `Labels:` followed by framed tags when labels exist;
   - the card status form (section G).
3. Long titles wrap inside the card; no text is truncated.
4. A card links only to the record view; the board does not link to the edit form.

### E. Tasks without a milestone

1. A task belongs here when its stored milestone value is absent/empty or does not resolve to an active milestone (a dangling or archived reference).
2. The section is an expandable `<details open>` placed after all milestone sections, with a `<summary>` reading `Tasks without a milestone`, and it renders the same column structure and cards as a milestone (columns in configured status order, empty columns visible, cards by ascending ID).
3. The section is omitted entirely when no task belongs to it.

### F. Empty states

1. When the project has neither active tasks nor active milestones, the board is replaced by the existing `Tasks` category empty state (`No tasks in this project.` plus the command-line-only note) and no groups are rendered.
2. When milestones exist but have no tasks, the groups and empty columns render as described above (no page-level empty state).
3. Wrapping and empty-state wording reuse the baseline rules (section 10.4 of `browser-requirements.md`).

### G. Status change: request

1. Each card contains exactly one `<form method="post" action="/doc/{task ID}" class="card-status">` with:
   - a hidden `version` input carrying the card's ADR-0006 version token;
   - a hidden `status_change` input with the value `1`, marking a status-only submission;
   - a hidden `expand` input carrying the card's active milestone ID, or an empty value for the no-milestone section and for cards in the unconfigured column;
   - a `<label for="status-{task ID}">Status</label>` and `<select id="status-{task ID}" name="status">` listing every configured status in configured order, with the card's current status selected (no selection when the current status is not configured);
   - a `<button type="submit">Change status</button>`.
2. The select plus button is the complete non-drag path: it is a plain form post, so it works with JavaScript disabled, with the keyboard only, and at 320 px. The button is always visible; the script does not auto-submit on `change`, so arrow-keying through options never triggers a request.
3. Cards are `draggable="true"` and columns are drop targets (`data-status`). With the script enabled, dropping a card on a column sets that card's select to the column status and submits the same form (no fetch, no new endpoint). Dragging is enhancement only; every operation remains available through the form.
4. No inline event handler attribute, inline script, or inline style is added; the CSP is unchanged.

### H. Status change: server

1. `browser::form_post` handles the submission after media-type (415), version-token syntax (400), and record resolution (404) checks. A `status_change` submission for a non-task record is `400`.
2. The submitted status is trimmed and must equal a configured status; otherwise the response is `422` with the board failure state and the message `Status is not a configured status.` and no write happens.
3. A version token that does not match the current record is `409` with the board failure state and the message `Could not change the status of {ID}: the record changed on disk. The card remains in its original column ({stored status}). Reload the view and try again.` and no write happens.
4. With a valid token the handler builds a status-only patch: `SetFrontMatter("status", value)` when the document has front matter, otherwise `SetSection("## Status", value)`. It never adds a body `## Status` section to a record that carries front matter, and it touches no other field.
5. The patch goes through `save_ops` (the existing `edit::apply` + LF normalization + `save_content` byte comparison) and `record::store::write_atomic`. A write error is `500` with the board failure state and the message `Could not change the status of {ID}. Nothing was saved. Try again.`
6. Success returns `303 See Other` with `Location: /?type=task` plus `&expand={milestone ID}` when the card has a resolvable milestone, and no `expand` parameter otherwise. No cached copy is kept; the redirect target re-reads the files (save-then-reread).
7. No-op safety: submitting the status the card already has produces a byte-identical file (the shared writer skips the write and `updated_date` is not refreshed).

### I. Failure reporting

1. A failed change re-renders the whole board from freshly read data with the affected card in its stored column, the affected milestone expanded, and an alert above the groups: `<div class="alert" role="alert">` with heading `Status change failed` and the message from H.2/H.3/H.5.
2. The response status is the failure status (409, 422, or 500). No other card, group, or column changes.
3. The failure page keeps the standard shell, sidebar, h1, and subheading `Status change could not be persisted.`

### J. Presentation, script, accessibility

1. `app.css` gains board styles only: horizontal equal-width columns inside a milestone, vertical stacking at narrow widths (single column at 320–480 px), card border/padding, drag affordance, framed label tags, the bold 2 px release separator, the empty-column text, and a visible focus outline for the select and button. No colour-only status or release cue is introduced.
2. `app.js` gains drag-and-drop only (dragstart, dragover, drop, cleanup), guarded so a missing `dataTransfer`, missing form, or missing select never throws. No `fetch`, no module, no dynamic import, no new file.
3. Keyboard order: card link, then status select, then `Change status`; the select is labelled by the card's `Status` label and carries `aria-label` fallback text naming the task. All controls meet the baseline focus-visible rule.
4. At 320 CSS px the sidebar toggle, columns, cards, select, and button remain operable and stacked; the board introduces no horizontal scrolling requirement.

### K. Unchanged surfaces

1. `GET /api/tasks`, `GET /api/doc/{id}`, `POST /api/doc/{id}`, record views, edit forms, error pages, search, and every CLI command keep their current behavior, shapes, and wording; `POST /api/doc/{id}` with `{version,status}` continues to work unchanged and is not the board's path.
2. No requirements, business rules, ADRs, configuration keys, or record formats change.

## Resolved questions (decisions recorded in the specifications)

- R1 (submission mechanism, was TASK-101/102 OQ-2): the board posts a status-only form to the existing `/doc/{id}` endpoint with the version token and a `status_change` marker. Reason: absent form fields currently mean *clear*, and the JSON API cannot serve a JavaScript-disabled client.
- R2 (flat task list, was TASK-101/102 OQ-1): the board replaces the flat `/?type=task` row list; no alternate flat task list remains.
- R3: the non-drag control is the mockup's select plus a visible `Change status` submit button; the script does not auto-submit on change.
- R4: tasks whose status is not configured are shown in a trailing `Unconfigured status` column so no active task disappears.
- R5: tasks whose milestone value is absent, dangling, or archived appear in the tasks-without-a-milestone section.
- R6: a release without milestones keeps its separator and shows an explicit empty group; after a successful change the board returns with the card's milestone expanded via `expand`.

## Assumptions

- A1: Card contents stay at ID, title, assignee, and labels, as approved in MOCK-0002. The TASK-67 rule that browser list rows print created/updated dates and references applies to row lists; the board card is a design-decided surface and does not print dates. The parity test is updated to assert the row-list projection on the milestone list instead of the removed task rows.
- A2: The `expand` parameter is an internal board affordance used by the redirect and by direct links; it is not a public API and accepts only an active milestone ID.
- A3: `status_change` is a private marker used only by this board; no other client is expected to send it, and it is ignored by the JSON API.
- A4: The existing write/no-op semantics (`save_content` byte comparison) mean a same-value submit does not refresh `updated_date`; this satisfies "only the status field changes".
- A5: No new MOCK id is needed; MOCK-0002's approved revision already covers the board, drag, select, failure state, and 320 px.
- A6: `updated_date` refreshing on a real status change is the normal write behavior (ADR-0013 decision 9) and is not counted as an extra field change.
- A7: Tests may add fixtures through the CLI only; no test writes record files directly except to simulate out-of-band concurrent edits (existing practice).
- A8: The three affected existing tests may be adjusted where the approved board replaces a row list; assertions about persisted data, status codes, and escaping must not be weakened.

## Dependencies

- Completed: TASK-97 (single writer / no legacy path), TASK-101 (board specification), TASK-102 (approved MOCK-0002 board revision).
- Downstream: TASK-98 (release verification) and TASK-73/TASK-100 consume this ticket.

## Tests to add or update

- New: board layout (columns per configured status in order, empty column placeholder, unconfigured column), release grouping (no-release group first, separator text/weight, empty release group, milestone order), no-milestone section (absent, dangling, and archived milestone references), card content/order/links/escaping, archived exclusion, `expand` handling, and the empty-project state.
- New: status change through the form (303, `Location`, persisted value, file diff limited to `status` and `updated_date`, reload shows the new column), stale token (409, no write, card in stored column, alert text), unconfigured status (422, no write), unknown/archived ID (404), non-task ID with the marker (400), wrong media type (415), and no-op submit (byte-identical file).
- New: markup/a11y gates — `draggable="true"`, column `data-status`, one labelled select and one submit button per card, no `onclick`/inline script, board CSS narrow-width rule, and a 320 px operable control.
- Update: `tests/browser_cli.rs::t11a_list_backlog_by_type` (task case becomes board assertions), `tests/backlog_discovery_smoke.rs` (task page still contains TASK-001), `tests/record_view_projection.rs::browser_and_cli_metadata_parity` (row-list metadata asserted on the milestone list; add a milestone fixture).
- Gates: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (including `tests/no_drift.rs`) with the existing dependency set only.

## Blast radius

- `src/record/edit.rs` (+ unit tests), `src/browser/render.rs`, `src/browser/mod.rs`, `src/browser/app.css`, `src/browser/app.js`, two specification files, and the affected test files. No new files except tests if a separate test module is chosen; no manifest changes.

## Acceptance Criteria
- [x] The Backlog view at `GET /?type=task` renders the board (h1 `Backlog`, `Backlog` sidebar entry `aria-current="page"`, no flat task rows) while `GET /?type=milestone` and `GET /?type=release` keep their existing row lists, wording, and empty states unchanged.
- [x] Milestones are native expandable/collapsible `<details>` sections, exactly one being open by default (the first with an active task) or the active milestone named by `?expand=`; an expanded milestone shows one column per configured task status in configured order, empty columns read `No tasks in this status.`, and tasks whose status is not configured appear in a trailing `Unconfigured status` column that is rendered only when such tasks exist.
- [x] Milestones with no release are grouped first under the heading `Milestones without a release`; each active release follows in ascending release ID order behind an `h2` separator `{release ID} — {title}` that is bold with a 2 px bottom border and does not rely on colour; milestones ascend by ID inside a group; a release with no milestone shows `No milestones in this release.`
- [x] Tasks with no milestone value, or with a value that does not resolve to an active milestone, appear in an expandable `Tasks without a milestone` section after all milestone sections (same columns and cards, omitted when empty); archived tasks and archived milestones never appear on the board.
- [x] Each card links its task ID and title to `/doc/{id}`, shows the assignee (`(none)` when absent) and labels as framed tags, wraps long titles, escapes all record text, and cards are ordered by ascending task ID within a column.
- [x] Each card carries a plain `POST /doc/{id}` form with the version token, a status-only marker, the card milestone for the redirect, a labelled status select listing the configured statuses in order with the current one selected, and a `Change status` submit button: the select plus button changes status with no pointer and no script, and with the script enabled dragging a card into another column selects that column status and submits the same form; the script never auto-submits on `change`.
- [x] A status change persists immediately through the shared patcher and writer, changing only the `status` value (front-matter key when front matter exists, otherwise the `## Status` section) plus the routine `updated_date` refresh, survives a reload, and returns `303` to `/?type=task` with the card milestone expanded; a same-value submit leaves the file byte-identical.
- [x] A change that cannot be persisted leaves the card in its stored column and re-renders the board with a `role="alert"` failure message: `409` for a stale version token, `422` for a status that is not configured, `500` for a write failure, and no file is modified in any failure case.
- [x] Tests cover board rendering for every configured status including empty and unconfigured columns, release/no-release grouping and the empty release group, the no-milestone section including dangling and archived milestone references, card order/links/escaping, archived exclusion, `?expand=`, both status-change paths and their persistence, the failure paths, the empty-project state, and a status control operable without a pointer at 320 px; `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` (including `tests/no_drift.rs`) pass with the existing dependency set only.

## Definition of Done
- [x] #1 All nine acceptance criteria are checked with objective evidence (test names or CLI/file output recorded in the final summary).
- [x] #2 `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass, including `tests/no_drift.rs`, with no new dependency, crate, or ADR.
- [x] #3 `GET /?type=task` renders the board and `GET /?type=milestone` / `?type=release` are unchanged; a status change and a stale-token failure were verified manually at 1440 px and 320 px with the script enabled and disabled.
- [x] #4 `docs/specifications/browser-tasks-milestones-and-releases.md` section 3 and `docs/specifications/ui/browser-backlog-records-ui.md` section 4 match the implemented behavior; `docs/requirements/` and `docs/business-rules/` are untouched.
- [x] #5 No new route, endpoint, module, crate, dependency, ADR, inline script/style, or second write path was added; `POST /doc/{id}` and the shared writer remain the only HTML write path.
- [x] #6 The three affected existing tests were updated and no existing assertion about persisted data, status codes, or escaping was weakened.
- [x] #7 All Backlog changes were made with the `backlog` CLI; no task, draft, document, or decision markdown file was edited directly.


## Implementation Notes
## Implementation plan

1. **Status-only patch (`src/record/edit.rs`).** Add `pub fn task_status_ops(doc: &Document, status: &str) -> Vec<PatchOp>`: when `doc.front_matter.is_some()` emit only `PatchOp::SetFrontMatter("status", status)`; otherwise emit only `PatchOp::SetSection("## Status", status)`. Add unit tests: front-matter record gains no body `## Status` section; legacy section record keeps its order; the returned ops list contains exactly one op; `apply` on a front-matter record changes only the `status` and `updated_date` lines.

2. **Board renderer (`src/browser/render.rs`).** Add a small internal grouping model (`BoardGroup { release: Option<&Record>, milestones: Vec<(&Record, Vec<&Record>)> }` or equivalent borrowing the indexes) built from the task, milestone, and release indexes; then add:
   - `pub struct BoardAlert<'a> { pub heading: &'a str, pub message: &'a str }`;
   - `pub fn board_page(cfg: &Config, tasks: &Index, milestones: &Index, releases: &Index, expand: Option<&str>, alert: Option<BoardAlert<'_>>, chrome: Chrome<'_>) -> String`.
   Rules to implement: no-release group first (`h2 Milestones without a release`, omitted when empty); releases by ascending ID with `h2` separator `{REL id} — {title}` and `No milestones in this release.`; milestones by ascending ID as native `<details>` with `<summary>{MS id} — {title}`, `open` on exactly one milestone (the `expand` target when valid, else the first with a task); per-milestone columns from `cfg.statuses` with `h3` + `data-status`, placeholder `No tasks in this status.`; trailing `Unconfigured status` column (`data-status=""`) only when needed; cards by ascending ID with `/doc/{id}` link, `Assignee: …`, label tags, and the card status form; tasks-without-a-milestone `<details open>` after all groups (omitted when empty); page-level empty state when there are no tasks and no milestones. Reuse `LIST_TEMPLATE` with heading `Backlog` and subheading `Tasks grouped by milestone and status; releases separate milestones.` (or `Status change could not be persisted.` plus the alert on failure). Escape every record-derived value.

3. **Route and write handling (`src/browser/mod.rs`).**
   - `list_page`: for `Category::Task`, read the optional `expand` query parameter, build `records::build_index` for `Task`, `Milestone`, and `Release`, and return `render::board_page`. Keep the other categories on the current path unchanged.
   - `form_post`: after media-type, version-syntax, and record-resolution checks, branch on the `status_change` field (value `1`). Non-task record with the marker -> `400`; untrimmed/unknown status -> `render::board_page` with status `422` and alert `Status is not a configured status.`; version mismatch -> board with `409` and the "changed on disk" message naming the task and its stored status; otherwise `save_ops(ctx, &record, edit::task_status_ops(&record.doc, &status))`, and on `Err` -> board with `500` and the "Nothing was saved" message. Success -> `Response::redirect` `303` to `/?type=task` plus `&expand={milestone}` when the record's `milestone_id` resolves to an active milestone. Extract a `board_status_submit` helper plus a `board_page_for(state, expand, alert)` builder so the GET route and the failure paths share one code path.
   - Do not modify the existing per-type edit branches, `api_post`, or any route table entry.

4. **Styles (`src/browser/app.css`).** Add board-only classes: `.board-columns` (equal-width grid driven by a `--status-count` custom property or `repeat(N, minmax(0,1fr))`), `.board-column`, `.board-card`, `.board-drag`, `.board-status` (form/label/select/button), `.release-sep` (bold, 2 px bottom border), `.board-empty`, `.board-milestone` (`details`/`summary`), and label tags reused from `.tag`. Add a narrow-width rule (single column at 320–480 px, full-width select and button, focus-visible outline) alongside the existing breakpoints. No colour-only cue, no inline style.

5. **Progressive drag-and-drop (`src/browser/app.js`).** Add a guarded board block: on `dragstart` store the task ID from `data-task`; on `dragover` mark the column as a target and `preventDefault()`; on `drop` find the card's `select`, set its value to the column's `data-status`, then `form.submit()`. Clear drag state on `dragend`/`dragleave`. Guard for missing `dataTransfer`, form, or select; no `fetch`, no auto-submit on `change`, no new script file.

6. **Specification consistency.** The two files were updated during refinement; re-read them before coding and keep the implementation aligned: `docs/specifications/browser-tasks-milestones-and-releases.md` section 3 (rules B/C/D/E/F/G/H/I) and `docs/specifications/ui/browser-backlog-records-ui.md` section 4. If an implementation detail must differ, update the specification in the same change and note it in the task; do not touch `docs/requirements/` or `docs/business-rules/`.

7. **Tests (`tests/browser_cli.rs`, `tests/backlog_discovery_smoke.rs`, `tests/record_view_projection.rs`).**
   - Extend `backlog_fixture` usage or add a board fixture: two releases, one milestone without a release, two milestones under releases, tasks in every status plus an empty status, a task with no milestone, a task with a dangling milestone ID, a task with an unconfigured status, and an archived task/milestone.
   - Add `t103a` board layout, `t103b` release/no-release/no-milestone grouping and `expand`, `t103c` card contents, order, links, escaping and archived exclusion, `t103d` status-change success (303, `Location`, file diff, reload) and no-op submit, `t103e` failure paths (409/422/404/400/415) and markup/a11y gates (labelled select, submit button, `draggable`, `data-status`, no inline handlers).
   - Update `t11a_list_backlog_by_type`, `backlog_discovery_smoke`, and `browser_and_cli_metadata_parity` as described in the ticket's test section.

8. **Gates.** Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and confirm `tests/no_drift.rs` still passes (no new writer, no hardcoded record directory, no section parsing). Manual check of `/?type=task` at 1440 px and 320 px, with the script enabled and disabled, including one successful and one stale-token change.


## Notes
Implemented the Backlog board in render::board_page plus the task-route branch and status_change form branch. Board: release/no-release grouping, native details, configured-status columns in order, empty and trailing unconfigured columns, ID+title cards with assignee/labels and the version-token status form, tasks-without-a-milestone section, archived exclusion, ?expand handling, and board alert states. Status writes use record::edit::task_status_ops through save_ops and the shared atomic writer only. Verification: cargo test (174 unit + all integration suites, including 166 browser tests and no_drift), cargo clippy --all-targets -- -D warnings, cargo fmt --check all pass. Headless Firefox render checked at 1440 px and 320 px with JavaScript enabled and disabled; status change, same-value no-op, stale-token 409, unconfigured 422, and write-failure 500 are covered by t103d/t103e/t103f. No new route, endpoint, module, crate, dependency, ADR, inline script/style, or second write path.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
GET /?type=task now renders the per-milestone Backlog kanban board (h1 Backlog) while milestone and release category pages are unchanged. Milestones render as native details grouped by no-release then ascending releases; expanded milestones show one column per configured status in order, empty columns, and a trailing Unconfigured status column when needed. Cards show ascending ID, ID+title record link, assignee, labels, and a plain POST /doc/{id} status form with version token, status_change marker, milestone expand value, labelled select, and Change status button; drag-and-drop is script-only enhancement. Status-only writes go through record::edit::task_status_ops and the existing save_ops/atomic writer, changing only status plus updated_date, with 303 save-then-reread, same-value byte identity, and 400/409/422/500 board failure states. Verification: cargo fmt --check, cargo clippy --all-targets -- -D warnings, and cargo test all pass; t103a-f cover layout, grouping/expand, unconfigured/empty states, card contents/escaping, persistence/no-op, all failure paths, and the 1440/320 no-JS markup gates.
