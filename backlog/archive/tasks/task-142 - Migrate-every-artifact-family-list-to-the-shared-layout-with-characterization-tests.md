---
id: TASK-142
title: >-
  Migrate every artifact family list to the shared layout with characterization
  tests
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:12:00Z'
updated_date: '2026-11-16T06:48:00Z'
labels:
  - browser
  - shell
  - tests
milestone: m-21
dependencies:
  - TASK-141
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.014.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/render.rs
  - src/browser/templates/document_list.html
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - src/browser/list.rs
  - src/browser/routes.rs
  - src/browser/mod.rs
  - tests/fixtures/characterization/lists/README.md
  - docs/requirements/business/REQ-05.007.md
priority: high
ordinal: 41550
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-06.016 forbids a bespoke list view for any family, and ADR-0028 downstream constraint 1 requires that `document_list.html` and every per-family list builder not survive. TASK-141 landed the shell, `PageWidth`, `browser::list`, and migrated the milestone list as the reference. Every other family still renders through `document_list.html` (config-defined types) or the per-family builders `render::list_section` / `render::backlog_list_section`, so the prohibition is not yet met.

**What.** Move every remaining family onto `ListPlan`, delete the second list path, and replay the TASK-141 characterization fixtures (schema `papyrus.list-characterization/1`, producing commit `d60f71c`) with every accepted difference recorded in this ticket.

**Settled contract (refinement).**

1. **Fixed columns, every family:** `id, title, status, <declared slots in order>, created, updated, actions`. Only slot presence and labels vary; `id`, `title`, `status`, `created`, `updated`, and `actions` are structural and reserved. Seven header cells for one slot, eight for two, six for none.
2. **Exact slot table:**

| Family | Slug(s) | Declared slots, in order | List status filter |
|---|---|---|---|
| Task | `task` | Milestone `milestone`, Assignee `assignee` | none - no list route; REQ-06.013 board |
| Milestone | `milestone` | Release `release` (already shared) | none |
| Release | `release` | none (status is the structural Status column) | none |
| Decision | `decision` aggregate; `adr`/`bdr` aliases | Type `type`, Superseded by `superseded_by` | none |
| Business requirements | `req` | Group `group` | yes |
| Non-functional requirements | `nfr` | Group `group` | yes |
| Business rules | `rule` (legacy `business-rule`) | Applicability `applicability` | yes |
| Audit records | `audit` | Audit type `audit_type` | yes |
| Technical specifications | `tech-spec` | none | yes |
| UI specifications | `ui-spec` | none | yes |
| UI mockups | `ui-mockup` | Task `task`, Specs `specs` | yes |
| Glossary terms | `glossary` | none | yes |
| Use cases | `use-case` | none | yes |
| Test cases | `test-case` | none | yes |
| Test reports | `test-report` | none | yes |
| Config-defined document types | any (e.g. `note`) | none | yes |

3. **Grouped families (every family whose descriptor id shape is grouped: requirements, NFRs).** Rows are emitted one group at a time in ascending group ID: the group row, then one child row per member in ascending ID, all in one `<tbody>`; each child carries `data-child-of="{group id}"`. A group with no members renders as a group row with no children. A member whose group row is absent from the indexed set (unresolved reference, or a group row removed by a status filter) renders as a top-level row in ID order, with no `data-child-of`, and is never dropped, duplicated, or attached to a synthesised group. The Group slot shows the stored reference verbatim.
4. **Status filter (REQ-06.014).** Every enabled document type keeps today's single filter form and its 400 validation (`active` plus declared custom statuses in configured order; `archived` never offered), rendered above the shared section. Milestone, release, decision, and task views have no filter. The filter decides the index; `ListPlan` never filters. A filtered list with no matches shows the shared empty block (filter-specific empty copy is out of scope).
5. **Count note.** Flat families: `list::count_note(slug, record count)` (`3 milestones`, `1 milestone`, `0 releases`; a config-defined type uses its own noun, `2 notes`). Grouped families: `N group(s) · M requirement(s)` computed from the rows actually emitted - the group rows shown and the member rows shown, orphan members rendered top-level included - never from a synthesised group (`1 group · 3 requirements`, `0 groups · 0 requirements`).
6. **Empty state.** One shared pair for every family: message `No {noun} in this project.` and help `Records are created from the command line; this browser does not create records.`, with `{noun}` the same singular/plural noun the count note uses. Legacy per-family empty wording (requirement cross-family hint and group messages, the "were found in the configured directory" wording, "Archived ... are hidden") does not survive; it is an accepted difference owned by TASK-148.
7. **Heading, subheading, count noun, and slots resolve through the presentation.** `list_page` resolves them via the descriptor's representative navigation slug, so `?type=adr`, `?type=bdr`, and `?type=decision` all render the Decisions heading and the `Type` slot. Requirement and NFR lists therefore show `Business requirements` / `Non-functional requirements` instead of the shared legacy `Requirements`, and every list subheading is the presentation inventory note rather than the legacy per-family `<Family> records in this project.` sentence.
8. **Task family.** `/?type=task` stays the REQ-06.013 board: no shared table, no Actions cell, no status filter, per-milestone grouping unchanged. The task slots stay declared but unused.
9. **One path.** `ListLayout`, `routes::list_layout`, `Presentation::list_layout`, `with_layout`, `list_section`, `backlog_list_section`, `document_list_page`, and `document_list.html` are deleted in this change; no slug or handler match selects a list builder, and no per-family list template remains. Config-defined types (no presentation entry) migrate by deleting the `document_list_page` branch. The board's empty-project fallback keeps its own markup and copy inside `board_page`; it is board copy, not a list.
10. **`document_list.html` deletion is safe.** No route changes: `/?type=<slug>` already resolves to the list handler, and the generic branch now calls the shared builder. The template is referenced only by `DOCUMENT_LIST_TEMPLATE`, which is deleted with it.
11. **Fixture normalization.** Captured `""` or `"(none)"` status/date/slot values are the shared muted `(none)`; non-empty status stays verbatim. Captured `actions` (`[]`), `baseline_columns`, and `baseline_row_shape` are evidence only.

**Accepted visual differences (complete list).** Every difference between the captured pre-change markup and the delivered shared list is recorded here; nothing is left implicit.

- **A1 Shared layout.** Table instead of `<ul>`, the fixed header sequence, the `Created` column, the `View`/`Edit` action cell, the Title-cell labels/references, and the muted `(none)` for unset values. Applies to every populated fixture.
- **A2 First-paragraph preview removed.** The legacy list rows rendered the first paragraph of the record body: `definition` for glossary terms, `description` for UI specifications, business rules, audit records, technical specifications, use cases, test cases, and test reports. The shared row carries only the Title cell (link, labels, references) plus the fixed cells; no body preview is rendered for any family.
- **A3 Legacy row-metadata line removed.** The `row-meta` line (`created`, `updated`, `labels`, `references`, and every remaining front-matter key) is not carried over: `created`/`updated` are structural columns and `labels`/`references` are in the Title cell. Every other field it exposed already has a column or a declared slot; the one field with no other home, decision `superseded_by`, is now the declared `Superseded by` slot, so CLI/browser parity is preserved.
- **A4 Requirement and NFR heading.** `Requirements` becomes the presentation heading (`Business requirements`, `Non-functional requirements`); the grouped count note keeps its group/member shape.
- **A5 Subheading copy.** The list subheading is now the presentation inventory note, so the legacy `<Family> records in this project.` sentence is replaced: `Milestone records...` becomes `Milestones in this project.`, `Release records...` becomes `Releases in this project.`, `Decision records...` becomes `Decisions in this project.`, `Business requirement records...` becomes `Business requirements in this project.`, `Non-functional requirement records...` becomes `Non-functional requirements in this project.` The other families' subheadings already matched their presentation note. Fixtures do not capture a subheading, so this difference is asserted through the presentation rather than replayed.
- **A6 Shared empty state.** Every legacy per-family empty block (requirement cross-family hint and group messages, the technical-specification/use-case/test-case/test-report "were found in the configured directory" wording, the "Archived ... are hidden" sentence) is replaced by the one shared message/help pair; TASK-148 owns the per-family UI description port.
- **A7 Grouped row structure.** The `grouped` entry with the pseudo-group key `other` replays as top-level rows; the per-group member count, the `<article class="group">` wrapper, the `(not found)` annotation on an unresolved group reference, and the "No requirements in this group" note are not carried into the table. For every grouped family, a member whose group row is absent from the indexed set renders as a top-level row with no `data-child-of`, and the grouped count note is computed from the rows actually emitted.
- **A8 Config-defined count noun.** `note` count note changes from `2 records` to `2 notes` (a config-defined type uses its own noun), and every empty fixture's count note (`No records`, `No groups`, `""`) becomes `0 <noun>` (or `0 groups · 0 requirements`).

**Fixture replay per family.** The "Accepted difference" column cites the entries above.

| Fixture(s) | Replay | Accepted difference |
|---|---|---|
| `milestone`, `release`, `decision`, `glossary`, `business-rule`, `audit`, `tech-spec`, `ui-spec`, `ui-mockup`, `use-case`, `test-case`, `test-report` | populated: row set/order, title, href, labels/references in the Title cell, status, slot values, created/updated, archived absence, heading, count note | A1; plus A2 for the preview families (`glossary`, `ui-spec`, `business-rule`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`); A3 and the `Superseded by` slot on `decision`; A5 (subheading); A8 (`note`) |
| `note` | as above | A1, A8 |
| `requirement`, `nfr` | as above, plus group->child nesting; count note preserved in shape | A1, A2 (description preview), A4 (heading), A5 (subheading), A7 (orphan members render top-level; group wrapper, per-group member count, `(not found)`, and "No requirements in this group" dropped; count note from rows shown) |
| `task` | board only: heading, count note, milestone grouping/card order, archived absence | no shared table/header is asserted |
| every `*-empty` | `empty: true` => no `<table>`, shared message/help, count `0 <noun>` (or `0 groups · 0 requirements`) | A6 and A8 (legacy count note `No records`, `No groups`, `""` becomes `0 <noun>`) |

**Tests to update (known legacy assertions).** `tests/browser_shell_cli.rs` (note/release "stay legacy" assertions become shared), `tests/browser_config_types_cli.rs` (t114_01 column order; t114_16 "non-document lists keep previous layout" is obsolete; glossary `row-status/req-id/...` order), `tests/browser_cli.rs` (t09 empty-project and empty-type copy; t09 grouped empty/orphan markup; technical-specification and use-case empty copy), `tests/release_document_types_smoke.rs` (note list column order). The reference-family milestone test (`reference_family_replays_its_characterization_fixture`) was folded into the generic per-family replay loop: coverage is preserved and no milestone-specific test remains, so the plan does not claim it stays. The `record_view_projection` shared assertions stay. New tests are listed in AC6/AC7 and in the Implementation Plan.

**In scope.** Migration of every remaining family; grouped row support; filter preservation; the count note, empty-state, slot, and accepted-difference contract above; deletion of the second list path and `document_list.html`; fixture replay; the new tests; the feature-specification update in §3.2, §5.1, §5.3, §5.4, §5.6, §5.7, and §6; UI-doc reconciliation is not in scope (TASK-148).

**Out of scope.** Detail, edit, and error pages; CLI list output; width classes and navigation (TASK-144); math and diagram rendering (TASK-145, TASK-146); new columns that do not exist today apart from the declared `Superseded by` decision slot; filter-aware empty copy; per-family UI description documents and the specification port (`docs/specifications/ui/**`, TASK-148); board behaviour (TASK-147).

**Assumptions.**
- The TASK-141 fixtures, seed projects, README, and capture test are the pre-change evidence and are neither regenerated nor edited.
- The board empty-project fallback is not a list view; its copy and structure stay as today.
- `serde_json` is already a dev-dependency; no new dependency is needed.
- Decision aliases are list routes of the same family and must show the same shared list as the canonical decisions route.
- The `Superseded by` decision slot presents the stored `superseded_by` reference; it is a declared slot because that value had no structural column and no other declared home.

**Dependencies.** TASK-141 (completed). Downstream: TASK-148 (specification port), TASK-149, TASK-150.

**UI gate.** No new mockup is produced: TASK-142 adds no visual contract beyond the one MOCK-0016 (draft) already defines for the shared list. The refiner records a no-mockup waiver naming MOCK-0016 and the 32 characterization fixtures; every visual difference is recorded above as A1-A8.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Every enabled artifact family and every enabled config-defined document type renders its list view through the shared layout; a disabled type has no list route (404). The task family has no list view and keeps the REQ-06.013 board.
- [x] #2 Every such list view renders the same header sequence - ID, Title, Status, the declared family slot(s) in order, Created, Updated, Actions - with the same row, column, and action placement; View then Edit (then any existing family action) are the last cell, and family-specific values appear only in the declared slot cells and the Title cell labels/references.
- [x] #3 document_list.html, document_list_page, list_section, backlog_list_section, ListLayout, routes::list_layout, Presentation::list_layout, and with_layout are deleted; no slug or handler match selects a list builder, no per-family list template remains, and ListPlan is the only list path.
- [x] #4 Requirements and NFRs render one group row followed by its member rows in the same tbody, each member carrying data-child-of; a group with no members renders as a group row with no children; for every grouped family, a member whose group row is absent from the indexed set renders as a top-level row with no data-child-of and is never dropped, duplicated, or attached to a synthesised group; the grouped count note reflects the records actually shown.
- [x] #5 All 32 TASK-141 fixtures (16 populated, 16 empty) are replayed per family, the task fixture is replayed against the board, and every accepted difference is recorded in this ticket.
- [x] #6 A test enumerates every enabled descriptor, resolves its representative list route, and asserts data-list-layout="shared" plus the exact data-column sequence including its declared slots with scope="col" on every header; the task descriptor is asserted on the board instead.
- [x] #7 Archived records stay absent from every list view and from the board's data-task cards.
- [x] #8 An approved mockup or a recorded no-mockup waiver exists before merge; the waiver is recorded in this ticket and names MOCK-0016 and the characterization fixtures.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 cargo test passes, including the extended replay loop, the new enumeration and archived-exclusion tests, and every updated legacy browser test.
- [x] #2 cargo fmt --check and cargo clippy --all-targets -- -D warnings are clean.
- [x] #3 No new dependency and no Cargo.toml change.
- [x] #4 document_list.html, document_list_page, list_section, backlog_list_section, DOCUMENT_LIST_TEMPLATE, ListLayout, routes::list_layout, Presentation::list_layout, and with_layout are deleted; no list code matches on a slug or handler; ListPlan is the only list path.
- [x] #5 The board empty-project fallback still renders its previous heading, count, message ("No tasks in this project."), and help, and the task board is otherwise unchanged.
- [x] #6 All 32 characterization fixtures are replayed per family; the task fixture is replayed against the board; the fixtures, seed projects, README, and capture test are unmodified.
- [x] #7 Every accepted visual difference (A1-A8, listed in the Description) is recorded in the Description and in the feature specification §6; no difference is left implicit.
- [x] #8 docs/specifications/browser-shell-shared-list-layout-and-page-width.md §3.2, §5.1, §5.3, §5.4, §5.6, §5.7, and §6 match the delivered code; docs/specifications/ui/** is untouched (TASK-148 owns the per-family UI reconciliation).
- [x] #9 The known legacy assertions listed in the Description are updated to the shared contract, and the generic per-family replay loop (which now covers the milestone reference family) plus the record_view_projection shared-list assertions pass.
- [x] #10 All eight acceptance criteria are checked with evidence before the task moves to a terminal status.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
**Baseline.** `cargo test --test list_characterization_replay`; boot `tests/fixtures/characterization/lists/project` and confirm every fixture URL resolves. Do not edit `tests/fixtures/characterization/lists/**` or `tests/list_characterization_capture.rs`.

**Reference - exact slot table (must match the Description).**

| Family | Slug(s) | Declared slots, in order | List status filter |
|---|---|---|---|
| Task | `task` | Milestone `milestone`, Assignee `assignee` | none (REQ-06.013 board) |
| Milestone | `milestone` | Release `release` | none |
| Release | `release` | none | none |
| Decision | `decision`; `adr`/`bdr` aliases | Type `type`, Superseded by `superseded_by` | none |
| Business requirements | `req` | Group `group` | yes |
| Non-functional requirements | `nfr` | Group `group` | yes |
| Business rules | `rule` (legacy `business-rule`) | Applicability `applicability` | yes |
| Audit records | `audit` | Audit type `audit_type` | yes |
| Technical specifications | `tech-spec` | none | yes |
| UI specifications | `ui-spec` | none | yes |
| UI mockups | `ui-mockup` | Task `task`, Specs `specs` | yes |
| Glossary terms | `glossary` | none | yes |
| Use cases | `use-case` | none | yes |
| Test cases | `test-case` | none | yes |
| Test reports | `test-report` | none | yes |
| Config-defined document types | any (e.g. `note`) | none | yes |

**1. Grouped row support - `src/browser/list.rs`, `src/browser/render.rs`, `src/browser/routes.rs`.**
- `ListPlan::for_descriptor` branches on `descriptor.id_shape == IdShape::Grouped` (declared data, never a slug): for each `index.groups` in ID order push the group `Row`, then one `Row` per position in `index.members[group.id]` (each position indexes `index.requirements`) into `children`; then push each position in `index.other` (same indexing) as a top-level `Row`, ascending ID, with no `data-child-of`. Reuse `row_for(record, slots)` for group, child, and orphan rows. No row is synthesised for a group that is absent from the indexed set.
- Add `grouped_count_note(&Index) -> String` (`N group(s) · M requirement(s)`) counted from the emitted rows (groups shown and members shown, orphans included), and a shared `noun(slug, count)` helper used by both `count_note` and the shared empty message (so `note` -> `notes`).
- `render::list_page` and `routes.rs`: keep the `filter-wrap` wrapper around the shared section; compute `count_note` grouped vs flat; set `req` and `nfr` to `ListLayout::Shared`.
- Unit tests in `list.rs`: group order, `data-child-of` on children, empty group row, unresolved member as a top-level row with no `data-child-of`, grouped count note from emitted rows, empty-message noun.

**2. UI mockups - `routes.rs`, verify `list.rs`.** Set `ui-mockup` to `Shared`; assert eight columns with `task`/`specs`; replay populated and empty; created/updated render muted `(none)`.

**3. Flat document families and release - `routes.rs`.** Set `glossary`, `rule`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`, `ui-spec`, `release` to `Shared`; replay each populated and empty fixture; assert the six-column shape for no-slot families and that a release shows exactly one status cell.

**4. Decisions - `routes.rs`, `render.rs`.** Set `decision` to `Shared`; declare the `Type` and `Superseded by` slots; in `list_page` resolve heading, subheading, count noun, and slots through `routes::nav_slug_for(slug)` so `?type=adr`, `?type=bdr`, and `?type=decision` all render the Decisions heading plus the `Type` and `Superseded by` slots; replay `decision` and `decision-empty`; assert all three URLs.

**5. Delete the second path - `src/browser/routes.rs`, `src/browser/render.rs`, `src/browser/mod.rs`, `src/browser/templates/`.**
- routes: delete `ListLayout`, `Presentation::list_layout`, `with_layout`, `routes::list_layout`, the default in `builtin`, the milestone `with_layout` call, and the two transitional tests; keep `list_slots` and its slot test; add a test that every presentation slot set passes `validate_slots`.
- render: delete `list_section`, `backlog_list_section`, `document_list_page`, `DOCUMENT_LIST_TEMPLATE`, and helpers left unused (`content_handler`, `category_short`, `record_view_list_meta`/`record_view_backlog_row_meta` with the legacy row-meta line unless still used by a detail page); keep `status_filter_html`, the filter wrapper, `requirement_link`/`member_list` (detail pages), and `row_status`/`row_dates` if still used; replace the board fallback with local board copy retaining heading, `No records`, message `No tasks in this project.`, and help; simplify `list_page` (drop `other_has_records`, always build `ListPlan`).
- mod.rs: drop the `other_has_records` computation and the `ContentModel::Generic` branch.
- delete `src/browser/templates/document_list.html`.
- Config-defined types and every remaining family now render shared without a presentation entry.

**6. Tests.**
- Extend `tests/list_characterization_replay.rs`: fixture-driven loop over all 32 `expected/*.json` with the normalization and accepted-difference rules below; add the grouped-row proof for `requirement` and `nfr`. The reference family is covered by this same loop: the dedicated `reference_family_replays_its_characterization_fixture` test was folded into it, so the plan no longer keeps a separate milestone test and coverage is preserved.
- New `tests/list_shared_layout.rs`: (a) enumerate every enabled descriptor from the seed project, resolve its representative route via `BrowserRoutes::nav_slug_for`, dedupe, and assert `data-list-layout="shared"`, the exact `data-column` sequence with declared slots, and `scope="col"` on every header; assert the task descriptor on the board instead; (b) assert every archived ID in every fixture is absent from its list, or from the board's `data-task` cards for tasks.
- Update the known legacy assertions listed in the Description.
- Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`.

**Reference - fixture replay and accepted differences (A1-A8 are defined in the Description).**

| Fixture(s) | Replay | Accepted difference |
|---|---|---|
| `milestone`, `release`, `decision`, `glossary`, `business-rule`, `audit`, `tech-spec`, `ui-spec`, `ui-mockup`, `use-case`, `test-case`, `test-report` | row set/order, title, href, labels/references in the Title cell, status, slot values, created/updated, archived absence, heading, count note | A1; A2 for the preview families (`glossary`, `ui-spec`, `business-rule`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`); A3 and the `Superseded by` slot on `decision`; A5 (subheading); A8 (`note`) |
| `note` | as above | A1, A8 |
| `requirement`, `nfr` | as above plus group->child nesting; count note preserved in shape | A1, A2 (description preview), A4 (heading), A5 (subheading), A7 (orphan members render top-level with no `data-child-of`; group wrapper, per-group member count, `(not found)`, "No requirements in this group" dropped; count note from rows shown) |
| `task` | board only: heading, count note, milestone grouping/card order, archived absence | no shared table/header is asserted |
| every `*-empty` | `empty: true` => no `<table>`, shared message/help, count `0 <noun>` (or `0 groups · 0 requirements`) | A6 and A8 (legacy count note `No records`, `No groups`, `""` becomes `0 <noun>`) |

Normalization: captured `""` or `"(none)"` status/date/slot values assert as `<span class="muted">(none)</span>`; a non-empty status asserts verbatim as `<span class="row-status tag">{value}</span>`; `actions`, `baseline_columns`, and `baseline_row_shape` are never replayed.

**7. Verify the specification.** Confirm `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` §3.2, §5.1, §5.3, §5.4, §5.6, §5.7, and §6 match the delivered code; do not edit `docs/specifications/ui/**` (TASK-148).
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Every family list now renders through ListPlan; the legacy path is deleted (document_list.html, document_list_page, list_section, backlog_list_section, ListLayout, list_layout, with_layout). Grouped req/NFR rows use one tbody with data-child-of, orphan members render top-level with no synthetic group, and grouped count notes count only emitted records. Decisions gained a declared Superseded by slot restoring CLI parity. 32 characterization fixtures replay, a descriptor-enumeration test asserts the shared marker plus exact data-column order and scope=col for every enabled family, and archived exclusion is asserted for all lists and the board. Accepted differences A1-A8 recorded in the ticket and spec section 6. Review fixes: 9f7afb3. Quality review: pass after fixes; 962 tests pass, clippy/fmt clean, every rendered link verified 200.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @task-refiner
created: 2026-11-15T18:31:00Z

---

Coordination from TASK-141 refinement. Fixture contract: tests/fixtures/characterization/lists/{project,expected/<family>.json} plus a README naming the producing commit and schema. Each fixture captures the semantic row model (row_order, per-record title/status/created/updated/slot values/href, grouped group->children, excluded archived ids, heading, count_note, empty) so it survives the header change; baseline_columns and baseline_row_shape are pre-change evidence and are not replayed. Two facts affect this task: (1) the task family has no list route (/?type=task is the REQ-06.013 board), so pick the reference family in TASK-141 before starting; (2) ADR-0028 declares release -> family slot Status, which duplicates the Status column - confirm or drop it here. Feature specification: docs/specifications/browser-shell-shared-list-layout-and-page-width.md.

---

author: @task-refiner
created: 2026-11-15T20:46:00Z

---

Refinement (TASK-142) - pinned decisions and UI waiver.

- Slot table confirmed: task->Milestone/Assignee; milestone->Release; release->none; decision->Type; req/nfr->Group; rule->Applicability; audit->Audit type; ui-mockup->Task/Specs; generic->none. `status` stays structural and is never a slot.
- Grouped families: group row then its children in one `<tbody>`, children carry `data-child-of`; an empty group renders as a group row; a member whose group row is filtered out renders top-level. The fixture pseudo-group `other` replays as top-level rows.
- Filter (REQ-06.014): kept unchanged for every document type (form above the shared section, same 400 rules); milestone/release/decision/task have none. `ListPlan` never filters.
- Count note: flat `list::count_note`; grouped `N group(s) · M requirement(s)` (preserves the fixture note).
- Empty state: one shared message/help pair for all families; legacy per-family empty copy is an accepted difference, reconciled by TASK-148.
- Heading: presentation heading; req/nfr move from `Requirements` to `Business requirements` / `Non-functional requirements` (accepted difference). Decision aliases (`?type=adr`, `?type=bdr`) resolve to the Decisions presentation.
- `list_layout`: all families become `Shared`; the enum, field, constructor, and function are deleted with the legacy builders and `document_list.html`. Config-defined types migrate by deleting that branch. No route changes; the board empty-project fallback keeps its own copy.
- Fixture replay: 32 fixtures; `""`/`"(none)"` normalize to the muted `(none)`; `actions`, `baseline_columns`, `baseline_row_shape` are evidence; `note` count note and every empty count note are accepted differences; the task fixture replays against the board.
- UI waiver: no new mockup. TASK-142 introduces no visual contract beyond MOCK-0016 (draft); the reviewed evidence is MOCK-0016 plus the 32 committed characterization fixtures. AC8 is satisfied by this waiver; `docs/specifications/ui/**` is not edited (TASK-148 owns the port).
- Spec updated: `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` §5.1, §5.4, §5.6, §5.7, §6.

---

author: @refiner
created: 2026-11-16T06:34:00Z

---

Quality-review text alignment (no code or test edits; the implementer fixed the three defects in parallel).

- Added a numbered "Accepted visual differences" list A1-A8 to the Description and cited it from both fixture-replay tables: A2 first-paragraph preview removed (glossary definition; ui-spec/business-rule/audit/tech-spec/use-case/test-case/test-report description); A3 legacy row-meta line removed with decision `superseded_by` now the declared `Superseded by` slot (CLI/browser parity kept); A4 requirement/NFR heading; A5 subheading copy now the presentation inventory note (`Milestone records...` -> `Milestones in this project.`, plus release, decision, business requirements, NFRs); A6 shared empty state (TASK-148 owns the UI port); A7 grouped row structure and orphan handling; A8 config-defined count noun.
- AC4 reworded to the corrected contract: every grouped family renders a member whose group row is absent from the indexed set as a top-level row with no `data-child-of`, never a synthesised group, and the grouped count note reflects the records actually shown. Other acceptance criteria kept as written.
- Milestone replay consolidation recorded: the dedicated `reference_family_replays_its_characterization_fixture` test was folded into the generic per-family loop. Description, Implementation Plan step 6, and DoD 9 no longer claim a separate milestone test stays; coverage is preserved.
- Spec `docs/specifications/browser-shell-shared-list-layout-and-page-width.md`: §3.2 no longer says `document_list.html` survives until TASK-142; §5.1 and §5.7 count the grouped note from the rows actually emitted; §5.3 adds the `superseded_by` slot (`Type`, `Superseded by`); §6 lists A1-A8 and records the folded reference-family test.
- DoD 7/8 updated to name A1-A8 and the amended spec sections (§3.2, §5.1, §5.3, §5.4, §5.6, §5.7, §6).

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The shared list layout now covers every artifact family and every config-defined document type, so no bespoke list view remains: document_list.html and all per-family list builders are gone, family variation comes only from declared slot data, and the fixed column order (ID, Title, Status, family slot(s), Created, Updated, Actions) holds everywhere. Grouped requirement and NFR lists render group rows with their children, orphan members appear top-level rather than as invented groups, and decisions keep their supersession reference through a declared slot. All 32 characterization fixtures replay, an enumeration test asserts the marker and header sequence for every enabled family, and archived records stay out of every list and the board.
<!-- SECTION:FINAL_SUMMARY:END -->
