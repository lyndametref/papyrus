# Feature Specification — Browser Shell, PageWidth Contract, and Shared List Layout (TASK-141)

**Ticket:** TASK-141
**Milestone:** m-21
**Status:** Contract settled 2026-11-15 (ADR-0028 amendment); MOCK-0016 registered 2026-11-15 (`draft`; approval waived for the implementation run, see §7.3). AC1–AC8 unblocked
**Requirements:** REQ-06.016 (shared list layout), REQ-06.015 (content width and navigation visibility, width clause only), REQ-06.004 (browser list parity with the CLI)
**ADR:** ADR-0028 (Accepted, amended 2026-11-15) — decisions 1, 2, 3, 6, and 9 are the contract for this ticket; decisions 4, 5, 7, and 8 belong to TASK-144/TASK-148/ADR-0029
**UI:** mockup required — see §7. A no-mockup waiver is **not** available (see §7.1).
**Downstream:** TASK-142 (remaining families, `document_list.html` deletion), TASK-144 (nav toggle, width CSS), TASK-148 (specification and UI-spec port)

## 1. Purpose and scope

Introduce the three structural pieces ADR-0028 fixes, and prove the shared list on one reference family before TASK-142 spreads it:

1. `browser::shell` — one page skeleton that every page renders inside, instead of seven templates that each repeat the chrome.
2. `PageWidth { Capped, Full }` — a server-side, per-route content-width decision rendered as a class on the `.app` wrapper.
3. `browser::list` — one server-built list model (`ListPlan`) and one template path for list pages, with family variation expressed only as declared column data.

This specification is the feature's canonical detail until TASK-148 ports the normative clauses into `docs/specifications/browser-requirements.md` (§3.6 navigation model, §4a shared header) and publishes the shell UI specification page. TASK-148 owns that port; this page must not be duplicated there, only cited and superseded.

### Non-goals

- Migrating the other families (releases, decisions, document types, requirements, NFRs) — the milestone list is the reference family (§5.6); the rest is TASK-142.
- Deleting `document_list.html` or the per-family list builders (TASK-142).
- The nav toggle, `/assets/nav-state.js`, `localStorage`, `data-nav`, `.bleed`, and the `.app--full` and collapsed-grid CSS rules (TASK-144).
- Kanban board changes (REQ-06.013; TASK-147 owns the board action).
- Math and diagram rendering (TASK-145, TASK-146).
- Detail, edit, and error page *content* changes (REQ-06.016 mandates a uniform list layout only).

## 2. Scope boundary against the ticket's premise

The ticket as filed says the task family is the reference implementation and that no visual change happens. Both statements fail against the current code:

- `GET /?type=task` renders the per-milestone kanban board (`render::board_page`, REQ-06.013, approved mockup MOCK-0002). The task family's only table-shaped list is the empty-project fallback inside `board_page` (`backlog_list_section` when no tasks and no milestones). There is no task list route to migrate, and replacing the board with a table would contradict REQ-06.013.
- No list route currently renders an actions cell. Every list shows a status text, an `ID` + title link, and Created/Updated values (`document_list_page` as `<table>` columns `Status, ID, Title, Created, Updated`; `backlog_list_section` and `list_section` as `<ul class="req-list">` rows). ADR-0028 decision 2 as amended adds a trailing `Actions` cell in the order `View`, `Edit`, then a family action, and fixes the order `ID, Title, Status, family slot(s), Created, Updated, Actions`. That reorders Status, converts `<ul>` rows to a table, adds the labels/references line in the Title cell, and adds two links per row. That is a visible change on every migrated list.

Resolved in §8: milestone is the reference family (§5.6); `created_date` stays as a structural Created column (§5.5).

## 3. Shell contract (ADR-0028 decision 1)

### 3.1 Module

`src/browser/shell.rs` owns the page chrome. It does not know about records, descriptors, or lists.

```rust
pub const SHELL_TEMPLATE: &str = include_str!("templates/shell.html");

pub struct Page<'a> {
    pub title: &'a str,
    pub project: &'a str,
    pub bind_note: &'a str,
    pub sidebar: &'a str,     // rendered nav fragment
    pub main: &'a str,        // rendered page body fragment
    pub width: PageWidth,
    pub nav_toggle: &'a str,  // empty today; TASK-144 fills it
}

pub fn render(page: Page<'_>) -> String;
pub fn width_class(width: PageWidth) -> &'static str; // "app" | "app app--full"
```

`render::render_template` becomes `pub(crate)` so `shell` and `list` reuse the existing placeholder engine. No new engine is introduced (ADR-0028 decision 6).

### 3.2 Template set

- New `src/browser/templates/shell.html`: doctype, `lang`, `head` (charset, viewport, title, stylesheet, deferred `app.js`), `<div class="{{app_class}}">`, `<header class="site-header">` with brand, top-meta, and a `{{nav_toggle}}` slot, `<div class="layout">`, `{{sidebar}}`, `<main class="main">{{main}}</main>`, `</div>`, `<footer class="site-footer">`, close.
- `list.html`, `view.html`, `edit.html`, `error.html` become body fragments (their page-specific placeholders only, no chrome).
- `document_list.html`, `document_detail.html`, `document_edit.html` also become body fragments in this ticket. They keep their current content model and their `Status, ID, Title, Created, Updated` table until each family migrates; only the duplicated chrome is removed. The milestone list leaves that table in this ticket (§5.6) and every remaining family leaves it in TASK-142. Deleting the templates is TASK-142.
- Invariant: the string `<div class="app` occurs exactly once per response, and each of `<header`, `<nav`, `<main`, `<footer` occurs exactly once.

The chrome strings (brand mark, top-meta copy, footer long/short copy, asset links) are byte-identical to today's `list.html`; this ticket moves them, it does not reword them.

### 3.3 Rendering

Every page-rendering function in `render.rs` stops interpolating its own chrome and instead renders its body fragment, then wraps it with `shell::render`. `Chrome` keeps `project`, `bind_note`, and `routes`; the width is passed to `Page::width` from the route property in §4.

## 4. PageWidth contract (ADR-0028 decision 3)

```rust
// src/browser/routes.rs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PageWidth {
    #[default]
    Capped,
    Full,
}
```

- `BrowserRoutes::page_width(&self, slug: &str) -> PageWidth` returns `Capped` for every enabled slug in this ticket. A single documented arm marks the future `Full` surface (the backlog board, TASK-144). No other route may return `Full` without an ADR-0028 amendment.
- `Capped` renders `class="app"`; `Full` renders `class="app app--full"`. The class string is decided on the server; no script reads or writes it.
- `.app--full` CSS is **not** added here (TASK-144). This ticket only guarantees the class is emitted and unit-tested.
- `.app` keeps `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px` unchanged.

## 5. Shared list contract (ADR-0028 decision 2)

### 5.1 Model

```rust
// src/browser/list.rs
pub struct ListPlan {
    pub heading: String,
    pub count_note: String,
    pub columns: Vec<Column>,
    pub rows: Vec<Row>,
    pub empty: Option<EmptyState>,
}

pub struct Column {
    pub key: &'static str,   // "id" | "title" | "status" | family slot key | "created" | "updated" | "actions"
    pub label: &'static str, // header text
    pub class: &'static str, // cell class; "" when none
}

pub struct Row {
    pub id: String,
    pub title: String,
    pub href: String,                   // "/doc/{id}"; a route, never a stored value
    pub cells: Vec<String>,             // Status, family slot(s), Created, Updated — in that order
    pub labels: Vec<String>,            // front-matter labels, rendered in the Title cell
    pub references: Vec<String>,        // front-matter references, rendered in the Title cell
    pub actions: Vec<(String, String)>, // (label, href): View, Edit, family
    pub children: Vec<Row>,             // grouped families
}

pub struct EmptyState { pub message: String, pub help: Option<String> }
```

Rules:

- `columns` is the complete header sequence and always starts with `id` and `title`, names `status` before the family slot(s), places `created` and `updated` after them, and ends with `actions`.
- Reserved keys: `id`, `title`, `status`, `created`, `updated`, and `actions` are structural and may not be declared as a slot, must not be repeated inside one family, and may not be reordered (ADR-0028 decision 2 and downstream constraint 10).
- `Row.cells.len() == columns.len() - 3` (ID, Title, and Actions are structural). `ListPlan::render` asserts this and the column-shape rules; a violation is a programming error, not a 500.
- `title` is not duplicated inside `cells`; it is required because the ADR row sketch has no title field and the ID/Title cells are structural.
- `labels` and `references` are general front-matter metadata, not family data. The shared list renders them in the Title cell below the link and escapes them in `list.rs`, because `Row` carries values, not markup. `href` is a route the builder computes, never a stored value, so a stored `href` key cannot reach the cell.
- `empty` carries the per-family empty copy, because today's copy varies by family (requirement group messages, the technical-specification/use-case/test-case/test-report wording, the generic "Records are created from the command line" help line). Empty copy must not stay behind per-family builder code (AC3). When `empty` is `Some`, the table is not rendered.
- Cell values are rendered HTML built from escaped values; `render` applies no further escaping to `cells`, `actions`, or `children`.

### 5.2 Markup contract

```html
<section class="section" aria-label="{heading}">
  <div class="section-head"><h2 class="h2">{heading}</h2><p class="count">{count_note}</p></div>
  <table class="table list list--shared" data-list-layout="shared">
    <thead><tr>
      <th scope="col" data-column="id">ID</th>
      <th scope="col" data-column="title">Title</th>
      <th scope="col" data-column="status">Status</th>
      <th scope="col" data-column="{slot}">{Slot}</th>
      <th scope="col" data-column="created">Created</th>
      <th scope="col" data-column="updated">Updated</th>
      <th scope="col" data-column="actions">Actions</th>
    </tr></thead>
    <tbody>
      <tr data-row-id="{id}">
        <td class="id-cell">{id}</td>
        <td class="title-cell"><a class="link" href="/doc/{id}">{title}</a><ul class="row-tags">{labels}</ul><ul class="row-refs">{references}</ul></td>
        <td class="status-cell">…</td>
        <td class="slot-cell">…</td>
        <td class="created-cell">…</td>
        <td class="updated-cell">…</td>
        <td class="actions-cell"><a class="link" href="/doc/{id}" aria-label="View {id}">View</a> <a class="link" href="/doc/{id}/edit" aria-label="Edit {id}">Edit</a>{family}</td>
      </tr>
      <tr data-row-id="{child}" data-child-of="{parent}"> … </tr>
    </tbody>
  </table>
</section>
```

The header carries seven cells for the reference family. A family with two slots carries eight and a family with no slot carries six: only the slot cells vary, the six structural cells are fixed in place.

Stable test hooks, independent of styling: `data-list-layout="shared"`, `data-column="{key}"`, `data-row-id`, `data-child-of`. Exact classes and spacing remain design decisions and are fixed by the approved mockup (§7), not by this document.

### 5.3 Family slots are declared data

- Slot declarations live in the existing presentation table (`routes.rs::Presentation`) as an ordered `list_slots: &'static [(&'static str, &'static str)]` (key, label), added with a `const fn with_slots` constructor so the table stays `const`. Keys: `milestone`, `assignee`, `release`, `type`, `group`, `audit_type`, `applicability`, `task`, `specs`.
- `routes::list_slots(slug) -> &'static [(&'static str, &'static str)]` returns the declared slots; a slug with no presentation entry (every config-defined type) returns an empty slice.
- `list::slot_cell(key, record) -> String` maps a slot key to its value. This is a `match` on the slot key, never on a slug, a handler, or a template.
- Slot order per ADR-0028: task → `Milestone`, `Assignee`; milestone → `Release`; release → none; decision → `Type`; requirement and NFR → `Group`; audit → `Audit type`; rule → `Applicability`; ui-mockup → `Task`, `Specs`; generic document → none.
- `Status` is structural for every family; declaring `status` as a slot is invalid (reserved key, §5.1). A release therefore shows one status cell, read from `record.status` verbatim.
- A list builder never branches on `descriptor.slug`. `TypeDescriptor` (already on hand) supplies id shape, category, and content model; the presentation table supplies slot keys and labels.

### 5.4 Rows, actions, archive exclusion

- Row order is the order `Index.records` already yields (ascending ID as indexed). Migrating a family must not reorder rows.
- Grouped families: the group row first, then one child row per member, all in the same `<tbody>`, each child carrying `data-child-of="{group id}"`. Members whose group reference is unresolved render as children of a synthetic trailing row only if the current family already shows them; the migrated family's current grouping behaviour is preserved, not redesigned.
- Actions are always the last cell, in the order `View`, `Edit`, then the family action if one exists today. `View` links to `/doc/{id}` and `Edit` to `/doc/{id}/edit`; both exist for every record today through the detail and edit routes, so no new route is implied. Each action link carries a record-specific accessible name (`aria-label="View {id}"`, `aria-label="Edit {id}"`), because repeated `View`/`Edit` labels are ambiguous to screen-reader and voice users (ADR-0028 decision 8 and downstream constraint 14).
- Archived records never reach `ListPlan`: the index is built enabled-only and active-only (ADR-0020 decision 7). No list code filters archive state, and no list code may add an archive filter.

### 5.5 List parity contract (REQ-06.004)

REQ-06.004 is a must-have requirement: the browser list must show the same front-matter values as the CLI for the same record. The shared list therefore guarantees, for every family including config-defined types:

- `created` and `updated` are fixed cells in fixed positions, never slots and never optional.
- `labels` and `references` render in the Title cell, below the link, from escaped values.
- `status` renders verbatim as stored, including a custom status declared for the type; the list never renames or maps it to a display label (REQ-07.001, REQ-07.002).
- No family may drop, rename, or relocate any of these values, and no slot may stand in for one of them. The only declared variation is the family slot(s), their labels, and the empty copy.

A family that needs another front-matter value declares it as a family slot, and a slot may not displace a structural column (ADR-0028 downstream constraints 10 and 11).

### 5.6 Reference family and transitional dispatch

The reference family is the milestone list (`/?type=milestone`, ADR-0028 decision 9):

- It is a real list route with a populated state and an empty state.
- It carries one family slot (`Release`), so the slot mechanism is exercised before TASK-142 spreads it.
- It is a `<ul>` row list that becomes a table, which is the largest structural change any family makes; a family already using a table would not prove the conversion.
- It is the browser side of the REQ-06.004 parity surface, so §5.5 is proven on the first migration.

The task family cannot be the reference: `GET /?type=task` is the REQ-06.013 kanban board and the task family has no list route to migrate (§2).

While both list paths exist, `list_page` selects the builder from a declared presentation field:

```rust
// src/browser/routes.rs
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ListLayout {
    Shared,
    #[default]
    Legacy,
}

// Presentation gains: pub list_layout: ListLayout
```

No `match` on a slug or handler is added for the migration, even temporarily (ADR-0028 downstream constraint 12). TASK-142 sets every family to `Shared` and deletes `ListLayout` together with the legacy builders in the same change, so no transitional branch can survive.

TASK-142's migration order is fixed by ADR-0028:

1. Requirements and NFRs first — the only grouped families; the grouped row variant and the status filter are unit-tested in this ticket and first exercised end-to-end there.
2. UI mockups next — `Task` and `Specs` slots, the widest row.
3. The remaining flat document families.
4. Decisions.
5. Config-defined types last — no presentation entry, therefore no slots.

The task family is a board-regression check, not a list migration: its fixtures prove the REQ-06.013 board is unchanged.

## 6. Characterization fixtures (AC5)

Fixtures guard TASK-142, so they must be produced from the **pre-change** binary and must not be regenerable from the migrated code path.

- **Location:** `tests/fixtures/characterization/lists/` — `project/` (the seed project), `expected/<family>.json` (one file per family), and `README.md` (schema version, the producing commit SHA, and the reproduction command).
- **Seed project:** a committed minimal project tree, not CLI-built at test time: `papyrus.conf.yaml` with the default types enabled plus one config-defined type (`note`), and record files covering every family, including at least one archived record per archivable family, one requirement group with two members, one NFR group with two members, one requirement whose group reference is unresolved, and one family with no records (for the empty state).
- **Families covered:** tasks (board inventory, since no task list exists), milestones, releases, decisions, business requirements (grouped), NFRs (grouped), glossary terms, business rules, audit records, technical specifications, UI specifications, UI mockups, use cases, test cases, test reports, and the config-defined `note` type. Plus one empty-state fixture per family.
- **Captured per fixture:** `schema`, `family`, `url`, `captured_from_commit`, `heading`, `count_note`, `row_order` (IDs in render order), `baseline_columns` (today's header labels in order), `baseline_row_shape` (`ul`, `table`, or `board`), `records` (per ID: `title`, `status`, `created`, `updated`, `labels`, `references`, declared slot values, `href`, and `actions` — empty today because no list renders action cells), `grouped` (group → child IDs, requirements and NFRs only), `excluded` (archived IDs that must stay absent), and `empty` (bool).
- **What is a target and what is evidence:** `baseline_columns` and `baseline_row_shape` are recorded as evidence of the pre-change contract and are **not** replayed, because they are exactly what the shared layout changes; `baseline_columns` is the only place the old five-column order survives, and it is evidence only. Everything decoded from records (set, order, values, grouping, exclusions, heading, count note) is replayed.
- **Producer:** `tests/list_characterization_capture.rs`, an `#[ignore]`d test that boots the current browser against `project/`, parses the pre-change markup (document table rows, `req-list` rows, board cards), and writes `expected/*.json`. It is run once on the pre-change commit and then kept for audit only.
- **Format decision:** JSON with `serde_json` (already a dev-dependency); no HTML golden files, because the header and cell markup is exactly what changes and byte-comparison would encode the old markup as the target.
- **Replay (TASK-142):** `tests/list_characterization_replay.rs` reads each fixture and asserts the shared layout shows the same record set, the same relative row order, `created` and `updated` as cells, `labels` and `references` in the Title cell, the same status/slot values, the same group→child nesting, the same heading/count note, and the recorded archived IDs absent; it asserts the shared header sequence and the `View`/`Edit` actions separately, because those are the intended change.

## 7. UI gate

### 7.1 Outcome: mockup required

The shared list is a visible change on every migrated list: column reorder (`Status` before the family slots, `Created` and `Updated` after them), the new `Created` column as a fixed cell, a Title cell that now carries the record's labels and references, a table instead of `ul`, an added `View`/`Edit` action cell with record-specific accessible names, and a table header that does not exist today. The shell change alone is chrome-equivalent, but AC2 cannot be satisfied without the list change, so the ticket as a whole is a visual change. A no-mockup waiver would have to assert that the approved mockups still describe the delivered lists; they do not.

### 7.2 UI description documents to update

- New: `docs/specifications/ui/browser-shell-and-shared-list-ui.md` — shell regions, list layout, column set, action cell, grouped rows, empty states, 320 px behaviour, and the anchor for the new MOCK.
- Existing, must be reconciled rather than restated: `docs/specifications/ui/browser-final-cross-type-ui.md` §5 (list rows currently described as a metadata line with `created_date`/`updated_date`), `docs/specifications/ui/browser-config-defined-document-types-ui.md` §2 and its generic list section, and the per-family list sections of `browser-backlog-records-ui.md`, `browser-decision-records-ui.md`, `browser-requirements-ui.md`, `browser-business-rules-ui.md`, `browser-audit-records-ui.md`, `browser-technical-specifications-ui.md`, `browser-glossary-terms-ui.md`, `browser-ui-specifications-ui.md`, `browser-ui-mockups-ui.md`, `browser-use-cases-test-cases-and-test-reports-ui.md`.

Per-family sections are reconciled in TASK-142; TASK-141 updates the new shell document and `browser-final-cross-type-ui.md` §5, and registers the MOCK.

### 7.3 Gate

`@ui-designer` produces one review package for the shell and shared list (MOCK id allocated, recorded in `docs/specifications/ui/mockups/README.md` and in this specification). **MOCK-0016** was produced and registered on 2026-11-15 with status `draft`; the implementation run proceeds in no-approval mode (orchestrator decision, TASK-141 comment 2026-11-15), so AC2/AC3 are implemented against the settled contract and the registry row stays `draft` for owner review. AC1, AC4, and AC5 (the fixture work in §6) were never gated.

## 8. Resolved questions (2026-11-15)

All three open questions and the ticket's task-family premise are settled by the ADR-0028 amendment (2026-11-15). Nothing on the contract remains open; only the MOCK id is pending (§7.3).

**OQ-1 — Which family is the reference implementation? Resolved: the milestone list.** `/?type=milestone` is the only family that is simultaneously a real list route with a populated and an empty state, a family slot carrier (`Release`), a `<ul>` → `<table>` migration, and the browser side of the REQ-06.004 parity surface (§5.6). The task family is not eligible: `GET /?type=task` is the REQ-06.013 board and no task list route exists (§2).

**OQ-2 — Does `created_date` survive in the shared list? Resolved: yes, as a structural column.** The fixed order is `ID, Title, Status, family slot(s), Created, Updated, Actions`, so the parity rule from ADR-0028 resolved question 3 is satisfied structurally instead of by ad-hoc per-family slots. `labels` and `references`, which today survive only in an ad-hoc metadata line, move into the Title cell. Dropping `created_date` would have relaxed REQ-06.004, a must-have requirement, which is a business-requirement change rather than an architecture decision (§5.5).

**OQ-3 — Release family slot `Status`. Resolved: release declares no family slot.** `Status` is structural for every family and the slot table now reads release → none (§5.3), so the release list shows one status cell. Reserved-key validation prevents the class of error, not just this instance.

**Task-family premise. Resolved: the task family has no shared-list surface.** REQ-06.016 governs list views and the task family's browser view is the REQ-06.013 board. TASK-142 treats tasks as a board-regression check, not a list migration; the declared task slots stay declared for a future task list, which needs its own decision (§5.6).

**MOCK id — resolved: MOCK-0016.** `docs/specifications/ui/mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html` is registered in `docs/specifications/ui/mockups/README.md` and in `docs/specifications/ui/browser-shell-and-shared-list-ui.md` with status `draft` (approval pending). The package shows the seven-column amended order, the Title-cell tag line, and the record-specific accessible names on the row actions. This implementation run operates in no-approval mode: the explicit owner-approval step is waived by the orchestrator, the implementation follows the settled contract, and the registry status stays `draft` until owner review.

## 9. Traceability

| Artifact | Reference |
| --- | --- |
| Requirements | REQ-06.016; REQ-06.015 (width clause); REQ-06.004 (browser list parity); REQ-06.013 (board unchanged); REQ-05.007 (archive exclusion); REQ-06.003, REQ-06.009, REQ-06.012, REQ-06.014 (shell, header, list routes) |
| NFRs | NFR-01.002; NFR-01.003; NFR-05.001 |
| ADRs | ADR-0028 (decisions 1, 2, 3, 6, 9); ADR-0005 (templates, CSP, progressive enhancement); ADR-0017; ADR-0020 (registry-derived routes, decision 7) |
| Domain model | `docs/domain-model/` — requirement group and member requirement (grouped rows) |
| Business rules | `docs/business-rules/` — archive and lifecycle rules cited by the existing list pages |
| Tickets | TASK-141 (this); TASK-142; TASK-144; TASK-147; TASK-148 |
| Implementation | `src/browser/shell.rs` (new), `src/browser/list.rs` (new), `src/browser/render.rs`, `src/browser/routes.rs`, `src/browser/templates/`, `src/browser/app.css`, tests |
