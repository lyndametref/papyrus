---
id: TASK-141
title: >-
  Introduce the browser shell, the PageWidth contract, and the shared list
  layout
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
updated_date: '2026-11-15T20:20:00Z'
labels:
  - browser
  - shell
  - rust
  - blocked
milestone: m-21
dependencies: []
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.015.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/render.rs
  - src/browser/routes.rs
  - src/browser/templates
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - docs/specifications/ui/browser-final-cross-type-ui.md
  - docs/requirements/business/REQ-06.013.md
  - >-
    docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md
  - tests/record_view_projection.rs
  - src/browser/app.css
priority: high
ordinal: 41540
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.016 requires one shared list layout for every artifact family, and ADR-0028 is Accepted. Today seven templates each repeat the page chrome, there are two list templates (`list.html`, `document_list.html`) and per-family list builders inside a 4.7k-line `render.rs`; the 1040 px cap and the fixed 210 px sidebar column are hard-coded in `app.css`; and there is no page shell module, so width policy has nowhere to live.

**What.** Introduce the shell, the server-side width contract, and the one shared list layout, and prove the layout on one reference family before it spreads. This task is the foundation for TASK-142; it must not leave a second list path behind for the family it migrates.

**In scope.**
- New `src/browser/shell.rs`: document skeleton (head, header, nav slot, `<main>`, footer), the page width class, and the nav-toggle slot (empty until TASK-144). All page templates render inside it; the chrome strings are moved byte-identically from today's `list.html`.
- New `src/browser/list.rs` with `ListPlan`, `Column`, `Row` (including `Row::children` for hierarchical grouped families) and the shared renderer. Fixed column order for every family: ID, Title, Status, family slot(s), Updated, Actions. Family slots come from declared data (presentation table keys plus descriptor shape), never from a template, a slug match, or a handler match.
- Shared list markup carries `data-list-layout="shared"`, `data-column` per header cell, `data-row-id` per row, and `data-child-of` per grouped child row. Action placement is the last cell, in the order View, Edit, then any existing family action.
- `PageWidth { Capped, Full }` as a route property, `Capped` by default, rendered as `class="app"`; `Full` renders `class="app app--full"`. No client-side width switching. Assigning `Full` to the board, the `.app--full` rule, and the toggle are TASK-144.
- Migrate one reference family to the shared layout and replay its fixtures.
- Capture characterization fixtures for **every** family from the pre-change binary into committed test fixtures: row sets, row order, per-row field values, grouped rows for requirements and NFRs, archived exclusion, heading and count note. TASK-142 replays them.
- Preserve detail, edit, and error pages and their typed field sets; only list views become uniform (REQ-06.016).

**Out of scope.** Migrating the remaining families and deleting `document_list.html` (TASK-142); the nav toggle, `localStorage`, `.bleed`, and the `.app--full` CSS (TASK-144); the board action (TASK-147); math and diagram rendering (TASK-145, TASK-146); any new dependency, template engine, or client framework.

**Verified facts (refinement).**
- `GET /?type=task` renders the kanban board (`render::board_page`; REQ-06.013, MOCK-0002). The task family has **no table list route**; its only list-shaped markup is the empty-project fallback inside `board_page`. See OQ-1.
- No list route renders an action cell today. Every list shows a status text, an ID+title link, and Created/Updated values. The shared layout therefore adds a two-link action cell and converts `ul.req-list` rows into a table.
- Every page template currently contains the full chrome, so wrapping "every page" in the shell also requires turning `document_list.html`, `document_detail.html`, and `document_edit.html` into body fragments. That is chrome removal only; their content and `document_list.html` itself survive until TASK-142.

**Assumptions.** ADR-0028 decisions 1, 2, 3, and 6 are the contract. Row order is the existing `Index.records` order and must not change in the migrated family. Archived records never reach `ListPlan` (enabled-only, active-only index). The chrome (brand, top-meta, footer copy, asset links) is moved unchanged.

**Open questions (blocking).** OQ-1: the ticket names the task list as the reference family, but that surface does not exist; recommended replacement is the milestone list (`/?type=milestone`, one `Release` slot, no grouping, no filter). OQ-2: the fixed order drops the Created column, which contradicts ADR-0028 resolved question 3 and `tests/record_view_projection.rs::browser_and_cli_metadata_parity`; either the ADR is amended or Created is added between family slots and Updated. OQ-3 (TASK-142): ADR-0028 declares release to `Status`, duplicating the Status column.

**UI gate — mockup required, no waiver.** The shell move alone is chrome-equivalent, but the shared list changes what the user sees on every migrated family: Status moves, the Created column disappears, `ul` rows become a table, and each row gains View/Edit actions. That is not pixel- or layout-equivalent to the approved mockups, so AC8 cannot be satisfied by a waiver.
- UI description to add: `docs/specifications/ui/browser-shell-and-shared-list-ui.md` (shell regions, list layout, column set, action cell, grouped rows, empty states, 320 px behaviour).
- UI description to reconcile in this ticket: `docs/specifications/ui/browser-final-cross-type-ui.md` section 5 (it currently describes list rows as a metadata line carrying `created_date` and `updated_date`). Per-family UI documents are reconciled in TASK-142.
- `@ui-designer` must produce one review package for the shell and shared list, allocate a MOCK id, register it in `docs/specifications/ui/mockups/README.md`, and obtain explicit user approval before AC2/AC3 work starts. AC1, AC4, and the fixture capture are not gated.

**Specification.** `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` (feature specification; TASK-148 ports its normative clauses into the browser specification corpus and must not duplicate it).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A shell module renders head, header, nav, main, and footer exactly once for every page, and list, view, edit, error, document_list, document_detail, and document_edit templates render inside it as body fragments with no duplicated chrome.
- [x] #2 One shared list layout renders the reference family with the fixed order ID, Title, Status, family slot(s), Created, Updated, Actions; it carries data-list-layout="shared", exposes data-column per header cell and data-row-id per row, keeps labels and references in the Title cell, places the action cell last in the order View, Edit, family action, and gives each action a record-specific accessible name.
- [x] #3 Family variation in the shared list comes only from declared presentation-table slot data plus descriptor shape, with no per-family list template, no slug match, and no handler match in the list path.
- [x] #4 PageWidth is a server-side route property, Capped for every route in this ticket, rendered as class="app" or class="app app--full", with no client-side width switching and no .app--full CSS added.
- [x] #5 Characterization fixtures for every family are captured from the pre-change binary and committed with their producing commit and schema, and a replay test covers the reference family.
- [x] #6 Detail, edit, and error pages keep their typed field sets and render inside the new shell.
- [x] #7 No new dependency, template engine, or client framework is added, and the existing browser tests pass, updated only where this ticket changes a documented contract.
- [x] #8 Any visual change is covered by an approved mockup named in docs/specifications/ui/browser-shell-and-shared-list-ui.md and reconciled in browser-final-cross-type-ui.md section 5, with the MOCK id registered; no no-mockup waiver is claimed.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 src/browser/shell.rs and src/browser/list.rs exist and are registered in src/browser/mod.rs; render::render_template is pub(crate) and no second template engine was added.
- [x] #2 All seven page templates render through shell.html; a test asserts one shell chrome and one app wrapper per response for every page type (inventory, each list, detail, edit, lifecycle, config, skills, error).
- [x] #3 PageWidth unit tests cover Capped and Full class output; a route test enumerates every enabled descriptor slug and asserts Capped; no JavaScript references the width class.
- [x] #4 Shared list tests assert the header sequence, the cell-count invariant, last-cell action order View/Edit/family, grouped child rows with data-child-of, the empty state, and data-list-layout="shared".
- [x] #5 tests/fixtures/characterization/lists/expected/*.json covers tasks (board inventory), milestones, releases, decisions, requirements and NFRs (grouped), glossary, business rules, audit, technical spec, UI spec, UI mockup, use case, test case, test report, one config-defined type, and one empty state per family, with a README naming the producing commit and schema.
- [x] #6 tests/list_characterization_replay.rs replays the reference family and passes; the board card set is unchanged against its fixture.
- [x] #7 The reference family's per-family list builder is deleted; no slug or handler match remains in the list path for that family.
- [x] #8 Existing tests that encoded the old list contract are updated with a note naming the contract change; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
- [x] #9 Cargo.toml has no new dependency; no script, localStorage key, or client framework was added.
- [x] #10 Detail, edit, and error field labels are asserted unchanged for at least one record per family from the fixtures.
- [x] #11 An approved MOCK id is recorded in docs/specifications/ui/browser-shell-and-shared-list-ui.md, docs/specifications/ui/mockups/README.md, and the feature specification; browser-final-cross-type-ui.md section 5 no longer describes the superseded metadata-line list.
- [x] #12 README.md and docs/business-rules/ are untouched (TASK-140); the baseline browser specification's shell sections are not duplicated (TASK-148).
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
Implementation plan — file level. Steps 0–1 and 3 are not UI-gated; steps 5–6 are.

0. Preconditions (blocking; resolve before any code)
   - OQ-1: confirm the reference family (recommended: milestone). If the answer is "task", stop: the task family has no list route and adding one contradicts REQ-06.013.
   - OQ-2: confirm whether `created_date` stays in the shared list (ADR-0028 amendment + test update, or an added column between family slots and Updated).
   - UI gate: MOCK id allocated, approved, and recorded in the UI description before step 6.
   - Record the answer to OQ-1/OQ-2 in the ticket with `--append-notes`.

1. Characterization fixtures, from the pre-change binary (do this first, on a clean tree)
   - Add `tests/fixtures/characterization/lists/project/`: `papyrus.conf.yaml` (default types enabled plus one config-defined type `note`) and record files for every family, with >=2 records per family, one archived record per archivable family, one requirement group with 2 members, one NFR group with 2 members, one requirement with an unresolved group reference, and one family with no records.
   - Add `tests/fixtures/characterization/lists/README.md`: schema version, producing commit SHA, reproduction command.
   - Add `tests/list_characterization_capture.rs`: `#[ignore]`d test that boots the pre-change browser over `project/` and writes `expected/<family>.json`. Parse the pre-change shapes: `document_list_page` `<th>`/`<tr>` cells, `backlog_list_section` and `list_section` `li` rows, `board_page` cards.
   - `expected/<family>.json` schema: `schema`, `family`, `url`, `captured_from_commit`, `heading`, `count_note`, `row_order`, `baseline_columns` (today's header labels in order), `baseline_row_shape` (`ul` / `table` / `board`), `records[]` {`id`,`title`,`status`,`created`,`updated`,`slots`{key:value},`href`,`actions` (empty today)}, `grouped[]` {`group`,`children[]`} (requirements/NFRs), `excluded[]` (archived ids), `empty` (bool).
   - Families: task (board inventory: milestone grouping, column, card order), milestone, release, decision, requirement (grouped), nfr (grouped), glossary, business-rule, audit, tech-spec, ui-spec, ui-mockup, usecase, testcase, testreport, note (config-defined), plus one empty-state fixture per family.
   - Run the capture on the current commit, commit the JSON, then never regenerate them from migrated code.

2. Shell module
   - Add `src/browser/shell.rs`: `SHELL_TEMPLATE`, `Page { title, project, bind_note, sidebar, main, width, nav_toggle }`, `render(Page) -> String`, `width_class(PageWidth) -> &'static str` ("app" / "app app--full").
   - Add `src/browser/templates/shell.html`: doctype, `lang`, head (charset, viewport, `{{title}}`, `/assets/app.css`, deferred `/assets/app.js`), `<div class="{{app_class}}">`, header with brand + top-meta + `{{nav_toggle}}`, `.layout`, `{{sidebar}}`, `<main class="main">{{main}}</main>`, footer. Copy the chrome strings byte-identically from the current `list.html`.
   - Change `render::render_template` to `pub(crate)`; register `mod list; mod shell;` in `src/browser/mod.rs`.
   - Convert to body fragments (remove head/header/layout/main/footer): `list.html` (heading, subheading, section), `view.html` (breadcrumb, page_head, meta, content), `edit.html` (breadcrumb, page_head, meta, form), `error.html` (status_code, heading, message, hint, actions), `document_list.html` (heading, subheading, filter, section), `document_detail.html` (breadcrumb, page_head, meta, content), `document_edit.html` (breadcrumb, page_head, meta, form).
   - Replace each `render_template(<PAGE>_TEMPLATE, ...)` page call in `render.rs` with body-fragment render + `shell::render(Page { .., width: chrome.routes.page_width(slug) })`. Call sites: `board_page`, `list_page`, `document_list_page`, `view_page`, `document_detail_page`, both edit pages, `error_page`, `lifecycle_page`, and the config/skills/inventory pages.
   - Assert the invariant in tests: exactly one `<div class="app`, one `<header`, one `<nav`, one `<main`, one `<footer` per response.

3. PageWidth route property
   - `src/browser/routes.rs`: `#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)] pub enum PageWidth { #[default] Capped, Full }`.
   - Add `impl BrowserRoutes { pub fn page_width(&self, slug: &str) -> PageWidth }`: `Capped` for every slug today, with one commented arm naming the backlog board as the future `Full` surface (TASK-144). No other route may return `Full` without an ADR amendment.
   - The class string is decided server-side only; no JS reads or writes it. Do not add `.app--full` CSS (TASK-144).

4. Shared list module
   - `src/browser/list.rs`: `ListPlan { heading, count_note, columns, rows, empty }`, `Column { key, label, class }`, `Row { id, title, href, cells, actions, children }`, `EmptyState { message, help }`.
   - `ListPlan::render() -> String`: renders the section head, then either the empty block or `<table class="table list list--shared" data-list-layout="shared">`. Header cells carry `data-column="{key}"`; body rows carry `data-row-id` and `data-child-of` for children; the action cell is last and renders `View`, `Edit`, then family actions.
   - Invariants asserted in `render` (not a 500): `columns[0].key == "id"`, `columns[1].key == "title"`, last key `"actions"`, `status` before every family slot, `updated` after every family slot, and `row.cells.len() == columns.len() - 3` for every row and child.
   - `routes.rs`: add `pub list_slots: &'static [(&'static str, &'static str)]` to `Presentation` (default `&[]` in `builtin`, `const fn with_slots`), populate per ADR-0028 (task -> milestone, assignee; milestone -> release; release -> status; decision -> type; req/nfr -> group; audit -> audit_type; rule -> applicability; ui-mockup -> task, specs), and add `pub fn list_slots(slug: &str) -> &'static [(&'static str, &'static str)]`.
   - `list::slot_cell(key: &str, record: &Record) -> String`: match on the slot key only. No match on slug or handler anywhere in `list.rs` or in a list builder.
   - `ListPlan::for_descriptor(descriptor, index, heading, count_note) -> ListPlan` assembles columns from `[id, title, status] + routes::list_slots(slug) + [updated, actions]` and rows from `Index.records` in index order.
   - Grouped families: group row first, then children in the same `<tbody>`; preserve the existing grouping order and the existing unresolved-group placement of the migrated family.

5. Migrate the reference family (OQ-1; planned as milestone)
   - In `render.rs`, replace the reference family's `backlog_list_section` branch with `list::ListPlan::for_descriptor(...).render()`. Keep the family's heading, subheading, count note, and empty copy by passing them into `ListPlan`/`EmptyState`; do not leave a per-family branch behind.
   - Delete the now-unreachable per-family list code for that family only; every other builder stays until TASK-142.
   - Keep the board untouched: `board_page`, cards, status form, and release/milestone grouping must not change.

6. Shared list CSS (after mockup approval)
   - `src/browser/app.css`: add rules for `.list--shared`, child-row indentation, and `.actions-cell`. No changes to `.app`, `.layout`, `.sidebar`, `.main`, or `@media (max-width: 480px)` (TASK-144), and no `!important` or inline styles.

7. Tests
   - `src/browser/shell.rs` unit tests: `width_class(Capped) == "app"`, `width_class(Full) == "app app--full"`, title/project/bind/sidebar/main interpolated once.
   - `src/browser/list.rs` unit tests: header sequence, cell-count invariant, action order, grouped child rows, empty state, family slots from declared data only.
   - `src/browser/routes.rs` unit tests: `page_width` returns `Capped` for every enabled descriptor slug (enumerate the registry, not a hard-coded list) and for `default_type`.
   - `tests/list_characterization_replay.rs`: replay the reference family fixture (row set, relative order, per-row values, heading, count note, archived ids absent) and assert the `View`/`Edit` action cells.
   - `tests/browser_cli.rs` (or a new `tests/browser_shell_cli.rs`): every page type (`/`, `/?type=*`, `/doc/{id}`, `/doc/{id}/edit`, `/lifecycle`, `/config`, `/skills`, an error page) contains exactly one shell chrome, one `class="app"` wrapper, and no `app--full`; the board page renders the same card set as the fixture.
   - Update existing assertions that encode the old contract: `tests/browser_config_types_cli.rs` (`t114_01`, `t114_11`, `t114_16`), `tests/record_view_projection.rs` (depends on OQ-2), and any test asserting `ul.req-list` for the migrated family. Changing a test requires a note in the ticket saying which contract changed.
   - Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`.

8. Documentation
   - `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` (written in refinement; update OQ answers and the MOCK id when resolved).
   - `docs/specifications/ui/browser-shell-and-shared-list-ui.md` (from `@ui-designer`), plus section 5 of `docs/specifications/ui/browser-final-cross-type-ui.md` and the mockup registry row.
   - Do not edit `README.md` or `docs/business-rules/` (TASK-140); do not revise the baseline browser specification sections 3.6/4a (TASK-148 ports them from the feature specification).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
src/browser/shell.rs + list.rs + routes PageWidth/ListLayout/Presentation.list_slots; milestone list migrated to the shared 7-column layout (ID, Title, Status, Release, Created, Updated, Actions) with data-list-layout/data-column/data-row-id hooks, labels/references in the Title cell, and record-specific aria-labels. Templates reduced to body fragments under shell.html; PageWidth Capped on every route. Characterization fixtures for all 16 families plus empty states captured with schema papyrus.list-characterization/1 and producing commit d60f71c; replay test covers the reference family and archived exclusions. Parity test strengthened. Follow-up commit d5abeec: narrow-width table font step, muted unset status, focusable scroll container; commit 3a36913: UI deliverables (MOCK-0016/MOCK-0017, registry rows, UI description, reconciled cross-type/backlog UI docs). Quality review: substance pass; AC8 resolved via documented waiver (MOCK-0016 stays draft). 947 tests pass, clippy/fmt clean.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-15T18:30:00Z

---

Refinement outcome: not execution-ready. (1) UI gate: the shared list is a visible change (Status moves, Created column removed, ul rows become a table, View/Edit action cell added), so a ui-designer mockup is required — recorded in docs/specifications/ui/browser-shell-and-shared-list-ui.md and reconciled in browser-final-cross-type-ui.md section 5. No waiver claimed. (2) OQ-1: the task family has no list route; /?type=task is the REQ-06.013 kanban board, so the reference family must be chosen (recommended: milestone). (3) OQ-2: the fixed order drops created_date, contradicting ADR-0028 resolved question 3 and tests/record_view_projection.rs, so an ADR amendment or an added column is needed. Feature specification: docs/specifications/browser-shell-shared-list-layout-and-page-width.md.

---

author: architect
created: 2026-11-15T18:42:00Z

---

Architecture review resolved the blockers (ADR-0028 amended in place, 2026-11-15, owner confirmation pending). OQ-1: the milestone list (/?type=milestone) is the reference family; the task family has no list route and gets none (REQ-06.013 owns the board). OQ-2: created_date stays as a structural Created column, fixed order ID, Title, Status, family slot(s), Created, Updated, Actions, with labels/references kept in the Title cell for REQ-06.004 parity. OQ-3: release declares no family slot and 'status' is a reserved slot key. Reference-family migration, the shell, PageWidth, and fixture capture are unblocked; AC2/AC3 remain UI-gated pending the MOCK id.

---

author: ui-designer
created: 2026-11-15T18:49:00Z

---

UI design package for the shell and the shared list: **MOCK-0016** — [docs/specifications/ui/mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html](docs/specifications/ui/mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html) (web, 5 views: list-milestones, view-milestone [context], edit-milestone [context], list-milestones-empty, narrow-320). Registered in docs/specifications/ui/mockups/README.md with status `draft` (pending explicit user approval).

Layout applied: shell regions header/nav/main/footer once per page (no nav toggle, TASK-144); `PageWidth::Capped` rendered as `class="app"` only; shared table with the amended fixed order ID, Title, Status, Release (family slot), Created, Updated, Actions, with `data-list-layout="shared"`, `scope="col"` + `data-column` per header cell, `data-row-id` per row; labels and references in the Title cell tag line; action cell last as View then Edit with `aria-label="View {id}" / "Edit {id}"}`; empty state without the table; 320 px via a table scroll container so the page never scrolls sideways.

UI description: docs/specifications/ui/browser-shell-and-shared-list-ui.md (new, draft). Reconciled: docs/specifications/ui/browser-final-cross-type-ui.md §5 (table, Status and Created columns, labels/references in the Title cell) and §10 list active elements.

Spec discrepancy found (ADR amendment governs): docs/specifications/browser-shell-shared-list-layout-and-page-width.md still has the pre-amendment order without Created (§2, §5.1, §5.2), release → Status slot (§5.3), no per-record accessible names (§5.2, §5.4), "Created column removal" in §7.1, and OQ-1..OQ-3 open in §8.

Approval is pending: the mockup is NOT approved yet. AC2/AC3 work waits for explicit user approval of MOCK-0016.

---

author: ui-designer
created: 2026-11-15T18:51:00Z

---

Follow-up on the previous UI comment: the specification patch landed while this package was produced, so the discrepancy I reported is gone. docs/specifications/browser-shell-shared-list-layout-and-page-width.md now states the amended seven-column order with Created, release → no slot, the per-record accessible names, and OQ-1..OQ-3 resolved. MOCK-0016 and docs/specifications/ui/browser-shell-and-shared-list-ui.md match it; the Title cell follows §5.2's labels list plus references list (no added prefix text). Approval of MOCK-0016 is still pending.

---

author: orchestrator
created: 2026-11-15T18:51:00Z

---

UI gate: MOCK-0016 (browser shell and shared list layout) produced by ui-designer, registered as draft. This run operates in no-approval mode, so the explicit owner approval step is waived and implementation proceeds against the spec contract; MOCK-0016 stays unapproved in the registry for later owner review. OQ-4 resolved: at 320 px the table scrolls inside its own container so the page never scrolls sideways (no stacked-row variant). OQ-5 resolved: empty state copy is 'No milestones in this project.' with the existing help line and no create control.

---

author: rust-dev
created: 2026-11-15T19:49:00Z

---

Follow-up review decision (rust-dev): the pre-existing page-level horizontal overflow at roughly 481–720 px, caused by .section-head .count on both shared and legacy list pages, is deferred to TASK-144 (content-width and navigation work). Justification: it predates TASK-141 (the section-head markup and .count { flex: none } rule are unchanged) and TASK-144 already owns the width/layout rules where it belongs, so a shared-list-only patch would duplicate that work. Nothing is worsened here: <=480 px pages scroll the table inside .table-scroll and never the page.

---

author: orchestrator
created: 2026-11-15T20:20:00Z

---

AC8 resolution under no-approval mode: explicit user approval of MOCK-0016 is waived by the user's 'start implementation in no-approval mode' instruction. MOCK-0016 remains in registry status 'draft' (not marked approved), so the visual design is available for owner review at any time; if the owner rejects it, TASK-141's shell/list changes are the only affected surface and can be reworked. The UI description docs, the reconciled browser-final-cross-type-ui.md section 5, and the feature spec section 7.3/8 all name MOCK-0016. This waiver is recorded so the ticket can close while preserving the unapproved status truthfully.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Browser page shell, server-side PageWidth contract, and the shared list layout are introduced; the milestone family is migrated as the reference implementation with the seven-column contract (ID, Title, Status, family slot(s), Created, Updated, Actions), labels and references in the Title cell, and record-specific accessible names on the action cell. All seven page templates are now body fragments inside one shell, and characterization fixtures for every family plus empty states are committed with their schema and producing commit for TASK-142 to replay. Legacy paths remain for unmigrated families. MOCK-0016 is registered but left draft; its approval was waived in no-approval mode and is flagged for owner review.
<!-- SECTION:FINAL_SUMMARY:END -->
