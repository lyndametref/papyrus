---
id: TASK-147
title: Add the per-milestone `Clean up all` board action
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
updated_date: '2026-11-16T06:19:00Z'
labels:
  - browser
  - backlog
  - specification
  - ui-gated
milestone: m-21
dependencies: []
references:
  - docs/requirements/business/REQ-06.013.md
  - docs/requirements/business/REQ-02.004.md
  - docs/requirements/business/REQ-05.007.md
  - src/browser/mod.rs
  - src/browser/render.rs
  - docs/specifications/browser-tasks-milestones-and-releases.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/requirements/business/REQ-02.003.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/routes.rs
  - src/task.rs
  - tests/browser_cli.rs
priority: high
ordinal: 41595
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.013 was revised on 2026-11-15: the final configured task status column of each milestone board must offer a "Clean up all" action that archives that column's tasks for the current milestone only, with a warning that lists the affected task IDs, a confirmation step, and per-task failure reporting. The board, its drag-and-status control, and its release separators already exist; the bulk action does not.

**What.** The board action, its server-rendered confirmation flow, its per-task archive behaviour, its failure report, and the visibility rule.

**Verified facts (refinement).**
- The final configured task status is the last entry of `Config::statuses` (`src/config.rs`; the default list ends with `Done`). `render::board_columns` renders exactly `cfg.statuses` in configured order and then an optional trailing `Unconfigured status` column, so "last column" means the last *configured* entry, never the unconfigured column.
- `board_columns(cfg, tasks, expand: Option<&str>)` (`src/browser/render.rs`) builds each board's columns. `board_milestone_section` passes `Some(milestone_id)`; the `Tasks without a milestone` section passes `None`, which is the natural gate for "milestone board only".
- The existing status write path is a status-only `POST /doc/{id}` form (`status_change=1`, `version`, `status`, `expand`) handled by `board_status_submit`: success is `303` to `/?type=task[&expand={ms}]`, failure re-renders the board with `BoardAlert`. `POST /?type=task` is `405` today; the `/?type=config` alias already accepts POST, so letting the board route accept POST follows an existing pattern.
- The task archive entry point is `crate::task::archive(ctx, id)` (`src/task.rs`): it retires legacy sections, refreshes `updated_date`, then moves the file to `<backlog_directory>/archive/tasks/` through `record::store::move_record`. It preserves `status` and the task ID, and it refuses with `Validation("already archived: <id>")` when the destination file already exists. Calling it once per task needs no second write path (ADR-0013) and changes no archive semantics (REQ-02.004, REQ-05.007, ADR-0022).
- There is no CSRF token anywhere in the browser: cross-origin POSTs are rejected by `validate_host`/`validate_origin` in `handle_connection`, and `form_post` requires `application/x-www-form-urlencoded` plus a valid version token. One bulk form cannot carry one version token per task.
- A milestone board's task set is the enabled task index filtered by `milestone_id`; archived tasks never enter that index.

**In scope.**
- The action appears in the header of the final configured task status column of every milestone board (open or closed), only when that status differs from `active`. It appears only there: not in other columns, not in the trailing `Unconfigured status` column, and not in the `Tasks without a milestone` section.
- Activating it renders a server-rendered warning that names the milestone, states the count, and lists the affected task IDs in ascending ID order with their titles; an explicit confirm is required, and cancel is a plain link that archives nothing.
- On confirmation, every task in that column for the current milestone is archived through the existing `task::archive` entry point; a failure is reported for that task and does not abort the remaining archives; successfully archived tasks leave the board on the re-read page.
- Scope is the current milestone board only, never project-wide; archived tasks stay excluded from the board; task status changes and every other board behaviour are unchanged.
- No CLI equivalent, no JSON API, no new write path, no new dependency.
- Tests: visibility rule for a non-`active` and an `active` final status plus a single-status configuration, cancel, full success, partial failure with per-task reporting and no abort, milestone scoping with an identically named status in another milestone, and archived exclusion afterwards.

**Decisions taken in this refinement (recorded in the specification).**
- Confirmation is server-rendered on the board page: a `GET /?type=task&expand={ms}&cleanup={ms}` preview (no writes, milestone stays expanded, action replaced by the warning region) and a confirm form that POSTs to the existing board route `/?type=task`. No script dialog, because the board must work with JavaScript disabled under the existing CSP, and no new path.
- The action is offered whenever the visibility rule holds, including when the column is empty; the warning then states that nothing will be archived and offers no confirm control. The visibility rule therefore depends only on the configured status, never on data.
- On any archive failure the board re-renders with `200` and a `role="alert"` region naming the archived/total counts and listing each failed ID with its message, because the request did archive records and only part of it failed.
- The bulk form carries no per-task version token: archiving is per-record idempotent (an existing destination fails per task), and the request keeps the baseline Host/Origin and media-type guards.
- The confirmed set is re-derived from current files at POST time, so the preview list is a warning of what will be archived, not a fixed list of hidden IDs.

**Out of scope.** A CLI bulk command, project-wide archival, changes to task archive semantics, milestone archive behaviour (milestones have no archive lifecycle), a single-task archive from the browser record view, board layout changes beyond the action and the warning, and any client-side script change.

**Assumptions.**
- Assumption: the final configured status is compared with `active` as an exact, case-sensitive string.
- Assumption: "current milestone" means the milestone whose board hosts the action; the without-milestone section is not a milestone board and gets no action.
- Assumption: failures are reported as task ID plus the message returned by the archive path; no new error taxonomy is introduced.
- Assumption: archiving through `task::archive` refreshing `updated_date` is the documented CLI archive behaviour and is accepted here.
- Assumption: no new dependency, ADR, CLI surface, or JSON API is needed.

**UI gate — mockup required, no waiver.** The ticket adds three visual states that MOCK-0002 does not contain: a new control inside a column header, a confirmation warning region above the columns, and a post-action failure list. A control in a column header and a new warning region are layout changes, not copy changes, so this is not layout-equivalent to the existing board controls and AC8 cannot be satisfied by a waiver.
- UI description to update: `docs/specifications/ui/browser-backlog-records-ui.md` — §4 (backlog view: action placement, warning region, empty-column warning, failure report), §8 (active elements), §9 (MOCK register), §10 (approval status).
- Registry: add the row to `docs/specifications/ui/mockups/README.md`.
- `@ui-designer` allocates the next sequential MOCK id (MOCK-0016 is taken; MOCK-0017 is expected), reuses the approved MOCK-0002 board and 320 px conventions, and must obtain explicit user approval. MOCK-0002 is not renumbered or replaced; a new package is produced.
- Gated: the final visible markup, its CSS, and the mockup-recorded presentation details. Not gated: the route/handler work, the archive loop, and the behaviour tests, which can land before approval but must not be released with an unapproved visual design.

**Specification.** `docs/specifications/browser-tasks-milestones-and-releases.md` §3 "Clean up all (per-milestone bulk archive)" clauses 1–13 plus the revised out-of-scope bullet and the new presentation-detail items; `docs/specifications/browser-managed-artifacts-and-lifecycles.md` scope bullet and §3 clause 3 (they no longer claim that every task lifecycle operation is CLI-only).

**Interactions (do not take on their scope).** TASK-141 wraps pages in the new shell and TASK-144 makes the board `PageWidth::Full` and adds the navigation toggle; the new markup must sit inside whatever chrome wrapper exists, must not add fixed widths, and must not add inline styles or scripts. TASK-142 does not touch the board. TASK-148 later ports browser wording into the specification corpus; the board clauses stay canonical in the specification named above.

**Open questions.** None blocking. Resolved in this refinement: empty-column behaviour, the confirmation mechanism, the failure response code, the absence of a per-task version token, and the without-milestone section treatment.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The header of the final configured task status column (the last entry of the configured task statuses) of every milestone board offers a Clean up all action only when that status differs from active; no other column, the Unconfigured status column, and the Tasks without a milestone section never offer it.
- [x] #2 Activating the action renders a server-rendered warning that names the milestone, states the count, and lists the affected task IDs with titles, and requires an explicit confirm; cancel is a plain link that archives nothing.
- [x] #3 On confirmation, every task in that column for the current milestone is archived through the existing task archive entry point (papyrus task archive semantics), with no second write path.
- [x] #4 An archive failure is reported for that task and does not abort the remaining archives; the alert names each failed ID with its message and archived tasks are gone from the re-read board.
- [x] #5 The action archives only the hosting milestone's tasks: an identically named column in another milestone and without-milestone tasks carrying that status are untouched, and no project-wide archive exists.
- [x] #6 No CLI equivalent, no JSON API route, no new write path, and no change to the board's status form or to task status editing is introduced.
- [x] #7 Tests cover the visibility rule for a non-active final status, an active final status, and a single-status configuration, plus cancel, full success, partial failure with per-task reporting and no abort, milestone scoping, and archived exclusion afterwards.
- [x] #8 An approved mockup for the column action, the confirmation warning, and the failure report, or a recorded no-mockup waiver, exists before merge, with its MOCK id recorded in the UI description and the mockup registry.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 src/browser/render.rs renders the action only in the final configured status column header of a milestone board and only when that status is not active; the confirmation region lists the affected IDs with titles and shows the empty-column warning without a confirm control.
- [x] #2 src/browser/mod.rs serves the GET preview and the POST confirm on /?type=task only; no other /?type= route accepts POST, and no /api/ route, CLI command, or help/man text was added.
- [x] #3 The archive loop calls crate::task::archive once per task and collects failures without aborting; no new write path, move routine, or archive implementation exists in src/browser/ (ADR-0013).
- [x] #4 tests/browser_cli.rs covers visibility (non-active final status, active final status, single status), preview and cancel, full success, partial failure with per-task reporting and no abort, milestone scoping with an identically named column elsewhere, and archived exclusion afterwards; cargo test passes.
- [x] #5 Existing board tests t103a-t103g and the status-change tests t103d/t103e pass unchanged, proving card, column, grouping, expansion, and status-form behaviour are untouched.
- [x] #6 cargo clippy --all-targets -- -D warnings and cargo fmt --check pass; Cargo.toml has no new dependency.
- [x] #7 The approved MOCK id is recorded in docs/specifications/ui/browser-backlog-records-ui.md (§4, §8, §9, §10), docs/specifications/ui/mockups/README.md, and the two approval placeholders in docs/specifications/browser-tasks-milestones-and-releases.md; MOCK-0002 is not renumbered.
- [x] #8 docs/requirements/ and docs/business-rules/ are untouched, no ADR was added or amended, and docs/specifications/browser-shell-shared-list-layout-and-page-width.md was not edited.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Implementation plan — file level. Steps 0, 5, and 6 are UI-gated; steps 1–4 and 7 are not.

0. Preconditions
   - `@ui-designer` produces the review package (expected MOCK-0017), records it in `docs/specifications/ui/mockups/README.md` and `docs/specifications/ui/browser-backlog-records-ui.md` (§4, §8, §9, §10), and the user approves it. Do not write final markup/CSS before that.
   - Re-read the specification: `docs/specifications/browser-tasks-milestones-and-releases.md` §3 "Clean up all (per-milestone bulk archive)" clauses 1–13 are the contract; do not diverge from them.

1. `src/browser/render.rs` — board markup (gated)
   - Add `pub struct BoardCleanup<'a> { pub milestone: &'a Record, pub status: &'a str, pub tasks: &'a [&'a Record] }`.
   - Change `board_columns(cfg, tasks, expand: Option<&str>)` to `board_columns(cfg, tasks, milestone: Option<&Record>, cleanup: Option<&BoardCleanup<'_>>)`; keep the card form's hidden `expand` value derived from `milestone.id` so the existing status form stays byte-identical (the card builder keeps its current markup).
   - In `board_columns`, compute `final_status = cfg.statuses.last()`; render `board_cleanup_action(milestone, final_status)` inside the `<section class="board-column">` of that status only, and only when `milestone` is `Some`, `final_status != Some("active")`, and a cleanup preview is not already open. Markup: `<p class="board-column-action"><a class="link board-cleanup" href="/?type=task&expand={ms}&cleanup={ms}" aria-label="Clean up all {status} tasks in {ms}">Clean up all</a></p>` after the `<h3>`. No `<script>`, no `style` attribute.
   - Add `fn board_cleanup_confirm(cleanup: &BoardCleanup<'_>) -> String`: `<section class="board-confirm" role="region" aria-label="Clean up all {status} tasks in {ms}">` with a heading that names milestone and status, a count sentence, the word `warning`, and `<ul>` of `ID — title` in ascending ID order. When `tasks` is non-empty, a `<form method="post" action="/?type=task">` with hidden `cleanup=1`, `milestone={ms}`, `expand={ms}`, a submit `Archive {n} tasks`, and `<a class="link" href="/?type=task&expand={ms}">Cancel</a>`; when empty, the "nothing will be archived" sentence and only the cancel link (no form).
   - `board_milestone_section` passes `Some(milestone)` and renders the confirm region immediately after `<summary>`, before `.board-columns`, only for the previewed milestone. The `Tasks without a milestone` call passes `None` for both parameters.
   - Generalise the alert: `BoardAlert { heading, message, details: &[String], subheading: &str }`; render `details` as a `<ul>` inside the existing `<div class="alert" role="alert">`; `board_page` uses `alert.subheading` instead of the hardcoded "Status change could not be persisted." and the two existing status-failure call sites pass that same string.
   - `board_page(..., cleanup: Option<&BoardCleanup<'_>>)`; `list_page` and the inventory fallback pass `None`.

2. `src/browser/mod.rs` — routes, handlers, archive loop
   - `render_board_page(state, cfg, expand, alert, cleanup)`: thread the new parameter; every existing call site passes `None`.
   - `list_page`: for the task handler, read `cleanup` with `http::query_param`. When present and non-empty: resolve the milestone in `enabled_index_for_handler(ctx, cfg, state, Milestone)` (`404` when missing or archived); read `final_status = cfg.statuses.last()`; `404` when `final_status == Some("active")`; build `Vec<&Record>` from the task index filtered by `milestone_id == Some(ms)` and `status == Some(final_status)` in index order (ascending ID); render with `Some(BoardCleanup { .. })` and `expand = Some(ms)`.
   - Route branch for `path == "/"` with a `type` parameter: `Method::Post` dispatches to `board_cleanup_submit` only when the resolved descriptor handler is `Task`; other types keep responding `405`; the `Method::Other` arm returns `405` with `Allow: GET, HEAD, POST` for the task board.
   - `fn board_cleanup_submit(request, state)`: media type must be `application/x-www-form-urlencoded` (`415`); `parse_form`; require `cleanup == "1"` and a non-empty `milestone` (`400`); load config; `final_status = cfg.statuses.last()`; if `Some("active")` re-render the board `422` with an alert ("Clean up all is not available for this status configuration."); resolve the milestone in the enabled milestone index (`404`); read a fresh task index and build the set as in step 2; loop `crate::task::archive(&state.context(), &task.id)`, collecting `Err` as `(task.id.clone(), err.to_string())` and continuing on error; if no failures, `Response::redirect("/?type=task&expand={ms}")` (303); otherwise re-render the board with `200` and `BoardAlert { heading: "Clean up all finished with failures", message: "Archived {ok} of {total} tasks in {ms}.", details: ["{id} — {message}", ...], subheading: "Some tasks could not be archived." }`, `expand = ms`.
   - Do not add a bulk helper in `src/task.rs`, do not modify `task::archive`, and do not touch `form_post`/`board_status_submit`. The only new server behaviour outside rendering is the preview query and the confirm POST.

3. `src/browser/app.css` (gated) — `.board-column-action`, `.board-confirm`, `.board-confirm ul`, `.alert ul`; reuse existing spacing tokens; no fixed widths, no `!important`, no inline styles, stack correctly at 320 px inside the current `@media (max-width: 480px)` block.

4. `tests/browser_cli.rs` — the six tests in the test plan below, using a `cleanup_fixture` built from the existing helpers (`create_task`, `set_task_status`, `set_statuses`, `create_milestone`).

5. Docs (gated on the MOCK id) — update `docs/specifications/ui/browser-backlog-records-ui.md` §4/§8/§9/§10, `docs/specifications/ui/mockups/README.md`, and the two "id is recorded when approved" places in `docs/specifications/browser-tasks-milestones-and-releases.md`. Do not edit `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` (TASK-141/144) or `docs/business-rules/`/`docs/requirements/`.

6. Verification — `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; confirm `Cargo.toml` is unchanged; confirm `git diff` touches no route other than `/?type=task` POST.

Test plan (all in `tests/browser_cli.rs`, using the existing `Server`/`get`/`post_form`/`form_body` helpers).

Fixture `cleanup_fixture`: default statuses (final `Done`); `MS-001` with `TASK-001` and `TASK-002` at `Done` (assign different IDs so order is observable) and `TASK-003` at `ToDo`; `MS-002` with `TASK-004` at `Done`; `TASK-005` without a milestone at `Done`; `TASK-006` at `Done` and then archived through the CLI.

- `t147a_cleanup_visibility`: with default configuration, `/?type=task` contains one `Clean up all` per milestone board (count equals the number of rendered milestones), inside `column_containing(&html, "Done")`, and none inside `column_containing(&html, "ToDo")`, none after the `Tasks without a milestone` marker, and none in the `data-status=""` column. With `set_statuses(&dir, &["ToDo", "Initiated", "active"], "ToDo")`, no `Clean up all` appears anywhere. With a single status (`set_statuses(&dir, &["Done"], "Done")`), the action appears in the only column.
- `t147b_cleanup_preview_and_cancel`: `GET /?type=task&expand=MS-001&cleanup=MS-001` returns `200`, contains the warning region, `MS-001`, the count `2`, and the IDs `TASK-001`, `TASK-002` in ascending order; no `TASK-003`, `TASK-004`, or `TASK-005` in that list; the confirm form posts to `/?type=task`; the cancel link is `/?type=task&expand=MS-001`. Assert every task file is byte-identical after the preview and `backlog/archive/tasks/` holds no file for them. `GET` the cancel target and assert nothing changed (cancel performs no POST).
- `t147c_cleanup_full_success`: `POST /?type=task` with `cleanup=1&milestone=MS-001&expand=MS-001` returns `303` with `Location: /?type=task&expand=MS-001`; `backlog/archive/tasks/TASK-001.md` and `TASK-002.md` exist with `status: Done` preserved; `/doc/TASK-001` and `/doc/TASK-002` return `404`; the re-read board contains neither ID; `TASK-004` and `TASK-005` are still present and `MS-002` still offers its action.
- `t147d_cleanup_partial_failure`: pre-create `backlog/archive/tasks/TASK-002.md` so `task::archive` refuses that task deterministically (destination exists); POST as above and assert `200`, `TASK-001` archived, `TASK-002` still in `backlog/tasks/` with `status: Done` and still on the board, and a `role="alert"` region containing the heading, `Archived 1 of 2`, and `TASK-002` with its message. Assert no other milestone's task was touched.
- `t147e_cleanup_milestone_scoping`: preview and confirm for `MS-002` and assert only `TASK-004` is listed, then archived; `TASK-001`/`TASK-002` (same `Done` status, other milestone) and `TASK-005` (no milestone) remain on disk and on the board. Also assert the `MS-001` preview after that lists nothing (empty warning, no confirm form).
- `t147f_cleanup_archived_exclusion_and_guards`: after a successful cleanup, the archived IDs are absent from `/?type=task`, `/doc/{id}` returns `404`, and a repeated confirm for the same milestone returns `303` while archiving nothing (empty set). Guard cases: `POST` with `cleanup=1` and an unknown or archived milestone returns `404`; with a missing `milestone` returns `400`; with `Content-Type: application/json` returns `415`; with a configuration whose final status is `active` returns `422`; and `POST /?type=req` still returns `405`. Existing `t103a`–`t103g` board tests must pass without modification.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Server-rendered board action: visible only in the last configured status column of a milestone section when that status is not active, including closed milestones; absent from other columns, the Unconfigured column, and the without-milestone section. GET preview at /?type=task&expand={ms}&cleanup={ms} renders a named role=region warning with the milestone, count and ascending task IDs/titles plus confirm/cancel; cancel is a plain GET. Confirm POSTs to /?type=task and archives each task through crate::task::archive, collecting per-task failures into a role=alert region without aborting, then redirects 303 to the expanded board. Guards: 400 empty/missing milestone, 404 unknown milestone, 405 other ?type= POST, 415 wrong media type, 422 active final status. Quality review: pass after fixes; t147d now induces failure on the first task and proves no abort (verified by a temporary abort-on-first-error patch that fails the test), plus single-status active and closed-milestone coverage; 953 tests pass, clippy/fmt clean. AC8 resolved by recorded waiver; MOCK-0017 stays draft.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-15T18:55:00Z

---

Refinement outcome: execution-ready; no blocking open questions. UI gate: mockup required, no waiver — the ticket adds a column-header control, a confirmation warning region, and a post-action failure list, which MOCK-0002 does not cover and which are layout changes rather than copy changes. The UI description to update is docs/specifications/ui/browser-backlog-records-ui.md (§4, §8, §9, §10) plus the mockup registry; @ui-designer allocates the next sequential id (expected MOCK-0017) and must obtain explicit user approval. Not gated: the route/handler work, the archive loop, and the behaviour tests. Specification: docs/specifications/browser-tasks-milestones-and-releases.md §3 "Clean up all (per-milestone bulk archive)" clauses 1–13, plus the revised out-of-scope bullet; docs/specifications/browser-managed-artifacts-and-lifecycles.md scope bullet and §3 clause 3 no longer claim that every task lifecycle operation is CLI-only. Decisions: preview via GET /?type=task&expand={ms}&cleanup={ms}, confirm POST to the existing board route, archive loop over crate::task::archive with per-task failure collection and a 200 alert on partial failure, no per-task version token (per-record archive is idempotent and Host/Origin validation still guards the request).

---

author: ui-designer
created: 2026-11-15T19:03:00Z

---

UI package produced: **MOCK-0017** — `docs/specifications/ui/mockups/MOCK-0017-browser-cleanup-all-board-action-review.html`, registry status `draft` (awaiting explicit user approval; do not implement as final UI yet).

Contents: review landing (ticket + one web card) and a 6-view web prototype — `list-milestones` (context), `board-action` (Clean up all in the final configured Done column only), `board-confirm` (populated warning region above the columns: heading names milestone and status, count, ascending ID list with titles, `Archive 2 tasks`, `Cancel`), `board-confirm-empty` (action still offered before activation; warning states nothing will be archived; no confirm control), `board-cleanup-failed` (role="alert" above the release groups: counts plus per-task failure messages; archived task gone; action offered again), `narrow-320` (stacked columns, warning reading order, full-width confirm control).

Recorded: registry row in `docs/specifications/ui/mockups/README.md` (web; 6 views; `draft`) and `docs/specifications/ui/browser-backlog-records-ui.md` §4 (new §4a), §8, §9, §10. MOCK-0002 is not modified or renumbered.

Resolves presentation items 7–9 of the specification. Note: the 320 px view draws the confirmation warning (where the layout risk is), so the column action itself is annotated in that view rather than drawn twice. ACs and status unchanged.

---

author: orchestrator
created: 2026-11-15T19:04:00Z

---

UI gate: MOCK-0017 (Clean up all board action, 6 web views including empty-column warning, failure report, and 320 px) produced by ui-designer and registered as draft. This run operates in no-approval mode, so the explicit owner approval step is waived and implementation proceeds against the spec contract; MOCK-0017 stays unapproved in the registry for later owner review. Contract note recorded by the designer: MOCK-0002's board screens omit the visible 'Change status' submit that UI description section 4 requires; MOCK-0017 draws it. MOCK-0002 is an approved baseline and is not modified.

---

author: orchestrator
created: 2026-11-16T06:02:00Z

---

AC8 resolution under no-approval mode: explicit owner approval of MOCK-0017 is waived by the user's 'start implementation in no-approval mode' instruction. MOCK-0017 remains registry status 'draft' (not marked approved), so the visual design is available for owner review; if rejected, only the board action markup/CSS added here is affected. MOCK-0017 is named in the UI description sections 4a/8/9/10, the mockup registry, and the specification.

---

author: orchestrator
created: 2026-11-16T06:06:00Z

---

Second review pass (launched before the fix landed) confirms all behaviour; the no-abort gap it reported is already fixed in 6427c92 with abort-proof. One residual coverage gap remains from that pass: t147b asserts the preview region, count, ids and controls but not the task titles, which AC2 requires. Reopened to Implemented so the test tightening is tracked; it will be applied once TASK-142's browser test edits finish, to keep one writer in tests/browser_cli.rs. Also recorded: the ticket's t103g reference is invalid (only t103a-t103f exist).

---

author: orchestrator
created: 2026-11-16T06:19:00Z

---

Final gap closed in 75ce5f8: t147b now asserts each affected task's exact rendered ID-plus-title pair and their ascending order (verified non-vacuous by a negative run). All AC and DoD evidence is now automated.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The milestone board gains a per-milestone Clean up all action in its final configured status column. It opens a server-rendered confirmation that names the milestone, the count, and every affected task in ascending ID order, and offers confirm or cancel so the board keeps working without JavaScript. Confirming archives each task through the existing task archive path, preserving status and landing in the archive/tasks directory, then returns to the expanded board. Per-task failures are reported together in an alert region without stopping the rest, archived tasks leave the board, and identically named columns in other milestones plus without-milestone tasks are never touched. No CLI command or JSON API is added.
<!-- SECTION:FINAL_SUMMARY:END -->
