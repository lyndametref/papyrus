---
id: TECH-074
title: "Browser Shell, PageWidth Contract, Shared List Layout, and Collapsible Navigation (TASK-141, TASK-144)"
created_date: '2026-11-16T17:46:55Z'
updated_date: '2026-11-18T20:22:32Z'
labels: []
references:
  - ADR-0005
  - ADR-0017
  - ADR-0020
  - ADR-0028
  - ADR-0029
  - MOCK-0002
  - MOCK-0016
  - MOCK-0018
  - NFR-001.002
  - NFR-001.003
  - NFR-005.001
  - REQ-005.007
  - REQ-006.003
  - REQ-006.004
  - REQ-006.009
  - REQ-006.012
  - REQ-006.013
  - REQ-006.014
  - REQ-006.015
  - REQ-006.016
  - REQ-007.001
  - REQ-007.002
  - TASK-141
  - TASK-142
  - TASK-144
  - TASK-145
  - TASK-146
  - TASK-147
  - TASK-148
  - TASK-149
  - TASK-150
  - ADR-034
  - REQ-006.018
  - RE-005
  - ADR-035
  - REQ-006.019
  - RE-006
  - TASK-178
status: archived
---
# TECH-074 Browser Shell, PageWidth Contract, Shared List Layout, and Collapsible Navigation (TASK-141, TASK-144)

> **Revised (TASK-170, 2026-11-18):** browser surfaces removed per ADR-034; this building block no longer provides the removed Configuration, Managed artifacts and lifecycles, or Agent skills and workflows surfaces. See ADR-034.

> **Revised (TASK-178, 2026-11-19):** reference visibility in the shared browser list is per artifact family per ADR-035 and REQ-006.019 (source RE-006). `ContentModel::Generic` (config-defined document types) renders no reference list in any list row; `ContentModel::Typed` and `ContentModel::HtmlMockup` families keep it; labels render for every family. Single-document view, CLI list and view, JSON API, `RecordView`, and front matter are unchanged.

## Description
## 1. Purpose and scope

Introduce the three structural pieces ADR-0028 fixes, and prove the shared list on one reference family before TASK-142 spreads it:

1. `browser::shell` — one page skeleton that every page renders inside, instead of seven templates that each repeat the chrome.
2. `PageWidth { Capped, Full }` — a server-side, per-route content-width decision rendered as a class on the `.app` wrapper.
3. `browser::list` — one server-built list model (`ListPlan`) and one template path for list pages, with family variation expressed only as declared column data.

This specification is the delivery record and implementation detail for the shell, the `PageWidth` contract, and the shared list. The normative behaviour is published in `docs/specifications/browser-shell-ui.md` (TASK-148), which cites this page for implementation detail and accepted differences; this page must not be duplicated there, only cited and superseded.

### Non-goals

- Migrating the other families (releases, decisions, document types, requirements, NFRs) — the milestone list is the reference family (§5.6); the rest is TASK-142 (delivered).
- Deleting `document_list.html` or the per-family list builders (TASK-142, delivered).
- Kanban board changes (REQ-006.013; TASK-147 owns the board action). TASK-144 changes only the board's **width class**, not its card content.
- Math and diagram rendering (TASK-145, TASK-146). TASK-144 ships the `.bleed` mechanism those tickets consume; it ships no rendering.
- A mockup preview route or endpoint (ADR-0028 decision 7; needs its own ADR).
- Detail, edit, and error page *content* changes (REQ-006.016 mandates a uniform list layout only).
- Any change to `script-src`, `style-src`, or the CSP constant (ADR-0028 decision 5; §4.5).

## 2. Scope boundary against the ticket's premise

The ticket as filed says the task family is the reference implementation and that no visual change happens. Both statements fail against the current code:

- `GET /?type=task` renders the per-milestone kanban board (`render::board_page`, REQ-006.013, approved mockup MOCK-0002). The task family's only table-shaped list is the empty-project fallback inside `board_page` (`backlog_list_section` when no tasks and no milestones). There is no task list route to migrate, and replacing the board with a table would contradict REQ-006.013.
- No list route currently renders an actions cell. Every list shows a status text, an `ID` + title link, and Created/Updated values (`document_list_page` as `<table>` columns `Status, ID, Title, Created, Updated`; `backlog_list_section` and `list_section` as `<ul class="req-list">` rows). ADR-0028 decision 2 as amended adds a trailing `Actions` cell in the order `View`, `Edit`, then a family action, and fixes the order `ID, Title, Status, family slot(s), Created, Updated, Actions`. That reorders Status, converts `<ul>` rows to a table, adds the labels/references line in the Title cell, and adds two links per row. That is a visible change on every migrated list.

Resolved in §8: milestone is the reference family (§5.6); `created_date` stays as a structural Created column (§5.5).

## 3. Shell contract (ADR-0028 decision 1)

### 3.1 Module

`src/browser/shell.rs` owns the page chrome. It does not know about records, descriptors, or lists.

```rust
pub const SHELL_TEMPLATE: &str = include_str!("templates/shell.html");
pub const NAV_TOGGLE: &str = "<button class=\"nav-toggle\" type=\"button\" aria-expanded=\"true\" aria-controls=\"site-nav\" hidden>Navigation</button>";

pub struct Page<'a> {
    pub title: &'a str,
    pub project: &'a str,
    pub bind_note: &'a str,
    pub sidebar: &'a str,     // rendered nav fragment
    pub main: &'a str,        // rendered page body fragment
    pub width: PageWidth,
}

pub fn render(page: Page<'_>) -> String;
pub fn width_class(width: PageWidth) -> &'static str; // "app" | "app app--full"
```

TASK-144 replaces the TASK-141 `nav_toggle: &'a str` field with `shell::NAV_TOGGLE`: decision 1 makes the shell the owner of the toggle, so no page renderer supplies it and no path can omit it. `render` fills the `{{nav_toggle}}` slot from the constant, so the toggle is byte-identical on every page.

`render::render_template` becomes `pub(crate)` so `shell` and `list` reuse the existing placeholder engine. No new engine is introduced (ADR-0028 decision 6).

### 3.2 Template set

- New `src/browser/templates/shell.html`: doctype, `lang`, `head` (charset, viewport, title, stylesheet, synchronous `/assets/nav-state.js`, deferred `app.js`), `<div class="{{app_class}}">`, `<header class="site-header">` with a `.header-lead` group (toggle plus brand), top-meta, `<div class="layout">`, `{{sidebar}}`, `<main class="main">{{main}}</main>`, `</div>`, `<footer class="site-footer">`, close.
- Header markup (TASK-144): `<header class="site-header"><div class="header-lead">{{nav_toggle}}<div class="brand">…</div></div><p class="top-meta">…</p></header>`. The lead group is what keeps the toggle left of the brand and both at the header start; `justify-content: space-between` on the header alone would centre the brand between them.
- Sidebar markup: the existing `<aside class="sidebar">` keeps its groups and its `<ul id="type-list">`, and its `<nav>` gains the id used by the toggle: `<nav id="site-nav" aria-label="Document types">`. The sub-480 px `<button class="sidebar-toggle">` is deleted together with its CSS (ADR-0028 decision 4: one toggle, in the header).
- `list.html`, `view.html`, `edit.html`, `error.html` become body fragments (their page-specific placeholders only, no chrome).
- `document_list.html`, `document_detail.html`, `document_edit.html` also become body fragments in this ticket. They keep their current content model and their `Status, ID, Title, Created, Updated` table until each family migrates; only the duplicated chrome is removed. The milestone list leaves that table in this ticket (§5.6). `document_list.html` did **not** survive TASK-142: it was deleted in that change together with the legacy list builders once every family rendered through `ListPlan` (§5.6). `document_detail.html` and `document_edit.html` remain body fragments.
- Invariant: the string `<div class="app` occurs exactly once per response, and each of `<header`, `<nav`, `<main`, `<footer` occurs exactly once. Exactly one `<button class="nav-toggle"` occurs per response, on every page, including error pages.

The chrome strings (brand mark, top-meta copy, footer long/short copy, asset links) are byte-identical to today's `list.html`; this ticket moves them, it does not reword them.

### 3.3 Rendering

Every page-rendering function in `render.rs` stops interpolating its own chrome and instead renders its body fragment, then wraps it with `shell::render`. `Chrome` keeps `project`, `bind_note`, and `routes`; the width is passed to `Page::width` from the route property in §4.

### 3.4 Collapsible navigation and its assets (TASK-144)

**Markup.** `shell::NAV_TOGGLE` (above) is the only declaration of the control:

- `class="nav-toggle"`, `type="button"`, `aria-expanded="true"`, `aria-controls="site-nav"`.
- Visible text `Navigation` is the accessible name and never changes; state is carried only by `aria-expanded`, so screen-reader users keep a stable name. No icon font, no `innerHTML`, no `aria-label` duplicating the text.
- `hidden` is present in the server output on every page. Without JavaScript the control cannot do anything, so it must not be offered; `/assets/nav-state.js` removes `hidden` (contract below). With JavaScript the button is visible on every page including error pages.
- The keyboard focus ring is the stylesheet's existing `button:focus-visible` rule; the toggle adds no focus handling of its own. It is a native `<button>`, so Enter and Space activate it.
- MOCK-0018 draws a decorative chevron inside the button. It is optional: if implemented it carries `aria-hidden="true"`, is part of neither the accessible name nor the toggle logic, and any direction change is driven by CSS from `.nav-toggle[aria-expanded]` — no script writes the button's content.

**State.** The collapsed state is one attribute on the document element: `<html lang="en" data-nav="collapsed">`. The expanded state is the **absence** of `data-nav`; `data-nav="expanded"` is never written. `localStorage` key `papyrus.nav` holds `"collapsed"` or `"expanded"`; only the exact value `"collapsed"` collapses (a missing key, any other value, or a storage error means expanded). No cookie, no server state, no config key (ADR-0028 constraint 5).

**CSS.**

```css
.nav-toggle { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px;
  border: 1px solid var(--line); border-radius: 5px; background: var(--soft);
  color: var(--ink); font: inherit; font-size: 13px; font-weight: 700; cursor: pointer; }
.nav-toggle[hidden] { display: none; }            /* the UA rule is lost once display is set */
.header-lead { display: flex; align-items: center; gap: 12px; min-width: 0; }
.brand { min-width: 0; }
.project { overflow-wrap: anywhere; }

:root[data-nav="collapsed"] .layout { grid-template-columns: minmax(0, 1fr); gap: 0; }
:root[data-nav="collapsed"] .sidebar { display: none; }
:root[data-nav="collapsed"] .main { border-left: 0; padding-left: 0; }
```

- The sidebar item leaves the grid and the template collapses to one `minmax(0, 1fr)` track with `gap: 0`; with the main border and padding removed that returns 210 + 30 + 30 = 270 px to page content. MOCK-0018 annotates the 240 px column-and-gap band it draws; the extra 30 px is the removed main padding. Reclaimed width is real grid width, never a negative margin. A retained zero-width first track must not be used: with the sidebar `display: none`, `.main` auto-places into that track and its width becomes 0.
- `.sidebar { display: none }` already hides the pre-`app.js` paint; `app.js` additionally sets `hidden` on `#site-nav` so the nav is removed from the accessibility tree by attribute, not by styling alone (ADR-0028 decision 4 and 8).
- `.brand`/`.header-lead` `min-width: 0` and `.project { overflow-wrap: anywhere }` keep a long project name inside the header at 320 px now that the toggle shares the row.
- The `@media (max-width: 480px)` block keeps `.layout { display: block }`, drops `.sidebar-toggle { display: block }` and `.sidebar ul[data-collapsed="true"] { display: none }`, and keeps `.app { padding: 0 12px 32px }`. The collapsed rules are media-query independent, so collapse works at every breakpoint; at ≤480 px `.sidebar { display: none }` is the whole reclamation.

**Script split.** Two classic same-origin scripts, both covered by the existing `script-src 'self'`:

| Asset | Loading | Owns |
| --- | --- | --- |
| `/assets/nav-state.js` (new, ≤ 1 KB, ≈ 15 lines) | synchronous in `<head>`, after the stylesheet and before the deferred `app.js` | reading `papyrus.nav`, setting `data-nav` before first paint, removing `hidden` from `.nav-toggle` |
| `/assets/app.js` (existing) | `defer` | the click handler and `hidden`/`aria-expanded` synchronisation, plus the unchanged dirty-form guard, tag editing, and board drag-and-drop |

`nav-state.js` contract:

1. Read `localStorage.getItem("papyrus.nav")` inside `try`/`catch`; a throw means "no stored value" and the session continues in memory.
2. If the value is `"collapsed"`, set `document.documentElement` `data-nav="collapsed"` immediately (before first paint), so no expanded frame is ever painted. Otherwise write nothing.
3. Remove `hidden` from the first `.nav-toggle`: on `DOMContentLoaded` when `document.readyState === "loading"`, otherwise immediately. This is the only reveal path for the control.
4. No dependency on `app.js`, `app.css`, or any global. No `innerHTML`, no `eval`, no storage of anything except the key above.

`app.js` contract:

- Delete the existing `.sidebar-toggle` / `#type-list` / `matchMedia("(max-width: 480px)")` block and its `applyViewport` change listener. The automatic sub-480 px collapse is removed: the persisted preference alone decides (ADR-0028 decision 4).
- On load, synchronise the DOM with `documentElement.dataset.nav === "collapsed"`: set `aria-expanded` to `"false"`/`"true"` on `.nav-toggle` and add/remove `hidden` on `#site-nav`.
- On click, flip the state, apply it by attribute only (never an inline style), write `"collapsed"`/`"expanded"` to `papyrus.nav` inside `try`/`catch`, and keep `aria-expanded` and `hidden` in sync. A storage throw must not stop the visual toggle.
- Nothing else changes: the dirty-form guard, tag editing, and board drag-and-drop blocks stay byte-identical, and no script reads or writes the width class.

**CSP.** `http::CSP` is unchanged: the new asset is same-origin and external, so `script-src 'self'` covers it, and the toggle changes classes and attributes only, so `style-src 'self'` is untouched (ADR-0028 decision 5). No inline `<script>`, no `onclick`, no `style=` attribute is added.

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

### 4.1 Route property (TASK-141, delivered)

- `Capped` renders `class="app"`; `Full` renders `class="app app--full"`. The class string is decided on the server; no script reads or writes it, so the no-JS layout equals the scripted layout.
- `.app` keeps `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px` unchanged.

### 4.2 Board classification (TASK-144)

- `BrowserRoutes::page_width(&self, route: PageRoute) -> PageWidth` returns `Full` for `PageRoute::Board` and `Capped` for `PageRoute::Other`; width is decided by route identity, never by a slug or record type.
- `render::board_page` passes `PageRoute::Board`; every other renderer (lists, detail, edit, inventory, config, skills, errors) passes `PageRoute::Other`; the board renderer is reached only for a descriptor with the `Task` handler, so a disabled task family leaves no route that can render `Full`.
- `Full` for any other route requires an ADR-0028 amendment (downstream constraint 4). The mockup preview route remains out of scope (decision 7) and no route or endpoint is added for it; `.app--full` is the named class that route will use.

### 4.3 Width CSS

```css
.app--full { max-width: none; }
```

- `.app--full` lifts only the cap; `.app`'s centring and `0 24px 56px` padding are inherited, so no page content touches the window edge.
- Client JavaScript cannot add or remove the class; a test asserts `app.js` and `nav-state.js` contain no `app--full`.

### 4.4 `.bleed` regions inside a capped page (TASK-144 mechanism, TASK-145/TASK-146 consumers)

A full-width-eligible region inside a `Capped` page is a `.bleed` block that takes the full width of the page's content column and owns its horizontal scrolling:

```css
.bleed { width: 100%; overflow-x: auto; }
.bleed:focus-visible { outline: 2px solid var(--ink); outline-offset: 2px; }
```

- Wide content scrolls inside the block, so the page never gains a horizontal scrollbar (ADR-0028 downstream constraint 7). The surrounding prose stays at its own narrower measure (`.text`, `.meta`, `.form` keep their existing `max-width`).
- A scrollable `.bleed` block carries `role="region"`, an `aria-label`, and `tabindex="0"`, matching `.table-scroll`, so keyboard users can reach the scroller in every supported browser.
- No `100vw`, no negative horizontal margins, and no `position: fixed` breakout: viewport units include the scrollbar width, which is the page-level overflow that constraint 7 forbids. A region that must reach the window edges is not expressible under this ADR and needs an amendment.
- On a `Full` page the same block spans the full padded window width, because the ancestor cap is absent.

### 4.5 Section heading overflow (TASK-144, deferred from TASK-141)

The shared list section heading (`.section-head`: `h2` plus `.count`) overflows the page horizontally between roughly 481 and 720 CSS px on every list route, because `.count { flex: none }` cannot shrink and the `h2` cannot shrink below its longest word, so the row is wider than the main column. The fix is layout-only and applies to the shared list and the board's empty-project fallback:

```css
.section-head { flex-wrap: wrap; }
```

- When heading and count do not fit on one line, the count wraps beneath the heading instead of widening the page. No copy, order, or breakpoint changes.
- The same defect class is closed at the header: `.header-lead`, `.brand`, and `.project` get the min-width/`overflow-wrap` rules in §3.4 so the toggle does not push the header past the viewport at 320 px.
- The heading elements break inside long tokens instead of widening the page: `h1, .h1, h2, h3 { overflow-wrap: anywhere }` (the `.h1` class is carried by `<h1 class="h1">` on list, detail, edit, and board heads).
- Preformatted blocks scroll inside themselves: `pre { max-width: 100%; overflow-x: auto }`, with `pre:focus-visible` drawing the same 2 px ring as the other scrollers. A long command line therefore never widens the page.
- The board columns (`.board-columns`) and the `/config` effective-configuration table (`.table-scroll`) are focusable labelled scroll regions: `role="region"`, an `aria-label`, `tabindex="0"`, and a visible `:focus-visible` ring on the scroller itself. The `/skills` command scroller (a `<pre>` with `role="region"`, `aria-label` naming the command, and `tabindex="0"`) carries the same semantics.

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
    pub labels: Vec<String>,            // front-matter labels, rendered in the Title cell for every family
    pub references: Vec<String>,        // front-matter references; rendered only for Typed and HtmlMockup families (ADR-035)
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
- `labels` and `references` are general front-matter metadata, not family data. `labels` render in the Title cell below the link for every family. `references` render in the Title cell only for `ContentModel::Typed` and `ContentModel::HtmlMockup` families; a `ContentModel::Generic` (config-defined document type) row renders no reference block at all, omitted rather than emptied (ADR-035, REQ-006.019). Visibility is resolved from the descriptor content model, never from a slug, a presentation override, or a per-family template. `Row.references` still carries the `RecordView` projection for every family; only the list rendering omits it for `Generic`. `list.rs` escapes the values, because `Row` carries values, not markup. `href` is a route the builder computes, never a stored value, so a stored `href` key cannot reach the cell.
- `empty` is `Some` when the filtered index has no records for the family; the table is then replaced by the empty block. TASK-142 pins one shared pair for every family: message `No {noun} in this project.` and help `Records are created from the command line; this browser does not create records.` `{noun}` is the same singular/plural noun the count note uses (`business requirements`; `notes` for a config-defined type). The legacy per-family variants (requirement group wording and the cross-family hint, the technical-specification/use-case/test-case/test-report "were found in the configured directory" wording, the "Archived … are hidden" sentence) are recorded as accepted differences in TASK-142 and reconciled by the specification port owned by TASK-148. Empty copy never lives behind per-family builder code (AC3).
- `count_note` is supplied by the caller and is the only other variable string. Flat families use `list::count_note(slug, record count)` (`3 milestones`, `1 milestone`, `0 releases`); grouped families use the group/member note (`1 group · 3 requirements`, `0 groups · 0 requirements`), counted from the rows actually emitted. A config-defined type falls back to its lower-case slug (`2 notes`).
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

The header carries seven cells for the reference family. A family with two slots carries eight and a family with no slot carries six: only the slot cells vary, the six structural cells are fixed in place. The `row-tags` list renders for every family; the `row-refs` list and its `<ul>` element are omitted entirely for `ContentModel::Generic` families, while `ContentModel::Typed` and `ContentModel::HtmlMockup` families keep them (ADR-035, REQ-006.019).

Stable test hooks, independent of styling: `data-list-layout="shared"`, `data-column="{key}"`, `data-row-id`, `data-child-of`. Exact classes and spacing remain design decisions and are fixed by the approved mockup (§7), not by this document.

### 5.3 Family slots are declared data

- Slot declarations live in the existing presentation table (`routes.rs::Presentation`) as an ordered `list_slots: &'static [(&'static str, &'static str)]` (key, label), added with a `const fn with_slots` constructor so the table stays `const`. Keys: `milestone`, `assignee`, `release`, `type`, `superseded_by`, `group`, `audit_type`, `applicability`, `task`, `specs`.
- `routes::list_slots(slug) -> &'static [(&'static str, &'static str)]` returns the declared slots; a slug with no presentation entry (every config-defined type) returns an empty slice.
- `list::slot_cell(key, record) -> String` maps a slot key to its value. This is a `match` on the slot key, never on a slug, a handler, or a template.
- Slot order per ADR-0028: task → `Milestone`, `Assignee`; milestone → `Release`; release → none; decision → `Type`, `Superseded by`; requirement and NFR → `Group`; audit → `Audit type`; rule → `Applicability`; ui-mockup → `Task`, `Specs`; generic document → none.
- `Status` is structural for every family; declaring `status` as a slot is invalid (reserved key, §5.1). A release therefore shows one status cell, read from `record.status` verbatim.
- A list builder never branches on `descriptor.slug`. `TypeDescriptor` (already on hand) supplies id shape, category, and content model; the presentation table supplies slot keys and labels.

### 5.4 Rows, actions, archive exclusion

- Flat families keep the order `Index.records` already yields (ascending ID as indexed). Migrating a family must not reorder rows.
- Grouped families: the group row first, then one child row per member, all in the same `<tbody>`, each child carrying `data-child-of="{group id}"`. Rows are emitted one group at a time in ascending group ID order, children in ascending ID order; a group with no members renders as a group row with no children. A member whose group row is absent from the filtered index (unresolved reference, or a status filter removed the group row) renders as a top-level row in ID order and is never dropped, duplicated, or attached to a synthetic group. The Group slot shows the stored reference verbatim; the legacy "Other requirements" section, the per-group member count, and the `(not found)` annotation are not carried over.
- Actions are always the last cell, in the order `View`, `Edit`, then the family action if one exists today. `View` links to `/doc/{id}` and `Edit` to `/doc/{id}/edit`; both exist for every record today through the detail and edit routes, so no new route is implied. Each action link carries a record-specific accessible name (`aria-label="View {id}"`, `aria-label="Edit {id}"`), because repeated `View`/`Edit` labels are ambiguous to screen-reader and voice users (ADR-0028 decision 8 and downstream constraint 14).
- Archived records never reach `ListPlan`: the index is built enabled-only and active-only (ADR-0020 decision 7). No list code filters archive state, and no list code may add an archive filter.

### 5.5 List parity contract (REQ-006.004)

REQ-006.004 is a must-have requirement: the browser list must show the same front-matter values as the CLI for the same record. REQ-006.019 qualifies that parity for list views only: a `ContentModel::Generic` (config-defined document type) list row omits the reference list, while CLI list and view output keep it (ADR-035). The shared list therefore guarantees, for every family including config-defined types:

- `created` and `updated` are fixed cells in fixed positions, never slots and never optional.
- `labels` render in the Title cell, below the link, for every family, from escaped values.
- `references` render in the Title cell, below the labels, for `ContentModel::Typed` and `ContentModel::HtmlMockup` families; a `ContentModel::Generic` family omits the reference block entirely (ADR-035, REQ-006.019). The value is unchanged in front matter, `RecordView`, the JSON API, the single-document view, and CLI list and view.
- `status` renders verbatim as stored, including a custom status declared for the type; the list never renames or maps it to a display label (REQ-007.001, REQ-007.002).
- No family may drop, rename, or relocate any of these values, and no slot may stand in for one of them. The only declared variations are the family slot(s), their labels, the empty copy, and reference visibility by content model.

A family that needs another front-matter value declares it as a family slot, and a slot may not displace a structural column (ADR-0028 downstream constraints 10 and 11).

### 5.6 Reference family and transitional dispatch

The reference family is the milestone list (`/?type=milestone`, ADR-0028 decision 9):

- It is a real list route with a populated state and an empty state.
- It carries one family slot (`Release`), so the slot mechanism is exercised before TASK-142 spreads it.
- It is a `<ul>` row list that becomes a table, which is the largest structural change any family makes; a family already using a table would not prove the conversion.
- It is the browser side of the REQ-006.004 parity surface, so §5.5 is proven on the first migration.

The task family cannot be the reference: `GET /?type=task` is the REQ-006.013 kanban board and the task family has no list route to migrate (§2).

TASK-141 introduced the transitional dispatch `list_layout: ListLayout { Shared, Legacy }` in the presentation table. TASK-142 retired it: every family is `Shared`, and `ListLayout`, `routes::list_layout`, `Presentation::list_layout`, `with_layout`, the `Legacy` builders (`list_section`, `backlog_list_section`, `document_list_page`), and `document_list.html` are deleted in the same change, so no transitional branch, slug match, or per-family list template survives (ADR-0028 downstream constraints 1, 2, and 12). `list_page` always builds a `ListPlan`. The board's empty-project fallback keeps its own markup and copy inside `board_page`, because `/?type=task` is not a list route.

TASK-142's migration order is fixed by ADR-0028 (each step keeps the build green and replays its fixtures before the next):

1. Requirements and NFRs first — the only grouped families; the grouped row variant and the status filter are unit-tested in this ticket and first exercised end-to-end there.
2. UI mockups next — `Task` and `Specs` slots, the widest row.
3. The remaining flat document families, plus the release family (flat, no slot, no filter). Release and milestone stay in the Backlog category and render no status filter.
4. Decisions — the aggregate list, and its `?type=adr`/`?type=bdr` aliases resolve to the same presentation (heading and `Type` slot) as `?type=decision`.
5. Config-defined types last — they have no presentation entry and never had a `list_layout` value; they migrate when the dispatch and the generic `document_list_page` branch are deleted.

The task family is a board-regression check, not a list migration: its fixtures prove the REQ-006.013 board is unchanged.

### 5.7 Post-migration list contract (TASK-142)

The following is the settled behaviour once every family renders through `ListPlan`:

- **One path.** `list_page` (route) resolves the descriptor, applies the optional status filter, builds the index, and calls the shared builder. `ContentModel::Generic` no longer selects a different page.
- **Presentation slug.** The heading, subheading, count noun, and declared slots are resolved through the list's representative presentation slug (`BrowserRoutes::list_slug_for`: decision aliases share the aggregate presentation; every other family keeps its own), so decision descriptor aliases render the decisions presentation (heading and subheading `Decisions`, `Type` slot) rather than an empty slot set.
- **Status filter (REQ-006.014).** Every enabled document type keeps the single status filter (`active` plus declared custom statuses, configured order, `archived` never offered) rendered above the shared section. Milestone, release, decision, and task views have no filter. The filter decides the index; `ListPlan` never filters. A filter value outside the offered set stays a 400. A filtered list with no matches shows the shared empty block; filter-specific empty copy is out of scope.
- **Grouped rows.** As in §5.4; the group-aware count note is `N group(s) · M requirement(s)` computed from the rows actually emitted — the group rows shown and the member rows shown, orphan members included — never from a synthesised group.
- **Empty state.** One shared message and help pair (see §5.1), one muted `(none)` for an unset status, date, or slot value, and the count note for zero records.
- **Actions.** `View` then `Edit` in the last cell, each with a record-specific accessible name; the title cell carries the record's labels for every family and its references for Typed and HtmlMockup families only.
- **Reference visibility (REQ-006.019, ADR-035).** The shared renderer omits the reference block for `ContentModel::Generic` (config-defined document type) rows and keeps it for `ContentModel::Typed` and `ContentModel::HtmlMockup` families. The decision is read from the descriptor content model; no slug match, presentation override, per-family template, or new presentation field is added. Labels stay for every family. Single-document view, CLI list and view, JSON API, `RecordView`, and front matter are unchanged.
- **Task family.** `/?type=task` is still the REQ-006.013 board: no shared table, no `Actions` cell, no status filter, and the per-milestone grouping unchanged.

## 6. Characterization fixtures (AC5)

Fixtures guard TASK-142, so they must be produced from the **pre-change** binary and must not be regenerable from the migrated code path.

- **Location:** `tests/fixtures/characterization/lists/` — `project/` (the seed project), `expected/<family>.json` (one file per family), and `README.md` (schema version, the producing commit SHA, and the reproduction command).
- **Seed project:** a committed minimal project tree, not CLI-built at test time: `papyrus.conf.yaml` with the default types enabled plus one config-defined type (`note`), and record files covering every family, including at least one archived record per archivable family, one requirement group with two members, one NFR group with two members, one requirement whose group reference is unresolved, and one family with no records (for the empty state).
- **Families covered:** tasks (board inventory, since no task list exists), milestones, releases, decisions, business requirements (grouped), NFRs (grouped), glossary terms, business rules, audit records, technical specifications, UI specifications, UI mockups, use cases, test cases, test reports, and the config-defined `note` type. Plus one empty-state fixture per family.
- **Captured per fixture:** `schema`, `family`, `url`, `captured_from_commit`, `heading`, `count_note`, `row_order` (IDs in render order), `baseline_columns` (today's header labels in order), `baseline_row_shape` (`ul`, `table`, or `board`), `records` (per ID: `title`, `status`, `created`, `updated`, `labels`, `references`, declared slot values, `href`, and `actions` — empty today because no list renders action cells), `grouped` (group → child IDs, requirements and NFRs only), `excluded` (archived IDs that must stay absent), and `empty` (bool).
- **What is a target and what is evidence:** `baseline_columns` and `baseline_row_shape` are recorded as evidence of the pre-change contract and are **not** replayed, because they are exactly what the shared layout changes; `baseline_columns` is the only place the old five-column order survives, and it is evidence only. Captured `actions` (`[]` today) are evidence only; the action cell is asserted from the contract instead.
- **Replayed, normalized:** record set and relative row order, title, `href`, label values in every Title cell, reference values in the Title cell for Typed and HtmlMockup families and no reference block for the Generic family, slot values, created and updated, group→child nesting, archived IDs absent, and the heading and count note where the family's presentation heading and count contract are unchanged. A captured `""` or `"(none)"` status, date, or slot value is the shared muted `(none)`; a non-empty status stays verbatim. This refines the README's "replayed targets" line, which predates the pinned differences below.
- **Accepted differences (TASK-142, recorded in the ticket as A1–A8):**
  - **A1 Shared layout.** Table instead of `<ul>`, the fixed header sequence, the `Created` column, the `View`/`Edit` action cell, the Title-cell labels and, for Typed and HtmlMockup families, references, and the muted `(none)` for unset values; this is the intended change every populated fixture shares.
  - **A2 First-paragraph preview removed.** The legacy list rows rendered the first paragraph of the record body — `definition` for glossary terms and `description` for UI specifications, business rules, audit records, technical specifications, use cases, test cases, and test reports. The shared row carries only the Title cell (link, labels, and references where the family keeps them) plus the fixed cells; no body preview is rendered for any family.
  - **A3 Legacy row metadata removed.** The `row-meta` line (`created`, `updated`, `labels`, `references`, and every remaining front-matter key) is not carried over: `created`/`updated` are structural columns, `labels` are in the Title cell, and `references` are in the Title cell except for the Generic family (ADR-035). Every other field it exposed already has a column or a declared slot; the one field with no other home, decision `superseded_by`, is now the declared `Superseded by` slot (§5.3), so CLI/browser parity is preserved.
  - **A4 Requirement and NFR heading.** `Requirements` becomes the presentation heading (`Business requirements`, `Non-functional requirements`); the grouped count note keeps its group/member shape.
  - **A5 Subheading copy.** The list subheading is now the presentation inventory note, so the legacy `<Family> records in this project.` sentence is replaced: `Milestone records…` → `Milestones in this project.`, `Release records…` → `Releases in this project.`, `Decision records…` → `Decisions in this project.`, `Business requirement records…` → `Business requirements in this project.`, `Non-functional requirement records…` → `Non-functional requirements in this project.` The other families' subheadings already matched their presentation note. Fixtures do not capture a subheading, so this difference is recorded here and in the ticket, not replayed.
  - **A6 Shared empty state.** Every legacy per-family empty block (the requirement cross-family hint and group messages, the technical-specification/use-case/test-case/test-report "were found in the configured directory" wording, the "Archived … are hidden" sentence) is replaced by the one shared message/help pair in §5.1. The per-family UI description port is TASK-148.
  - **A7 Grouped row structure.** The `grouped` entry with the pseudo-group key `other` replays as top-level rows; the per-group member count, the `<article class="group">` wrapper, the `(not found)` annotation on an unresolved group reference, and the "No requirements in this group" note are not carried into the table. A member whose group row is absent from the indexed set renders as a top-level row with no `data-child-of` for every grouped family, and the grouped count note is computed from the rows actually emitted.
  - **A8 Config-defined count noun.** `note` count note changes from `2 records` to `2 notes` (a config-defined type uses its own noun), and every empty fixture's count note (`No records`, `No groups`, `""`) becomes `0 <noun>` (or `0 groups · 0 requirements`).
- **Producer:** `tests/list_characterization_capture.rs`, an `#[ignore]`d test that boots the current browser against `project/`, parses the pre-change markup (document table rows, `req-list` rows, board cards), and writes `expected/*.json`. It is run once on the pre-change commit and then kept for audit only.
- **Format decision:** JSON with `serde_json` (already a dev-dependency); no HTML golden files, because the header and cell markup is exactly what changes and byte-comparison would encode the old markup as the target.
- **Replay (TASK-142):** `tests/list_characterization_replay.rs` gains one loop over every populated and empty fixture (`expected/*.json`, 32 files). For each fixture it asserts the shared layout shows the same record set, the same relative row order, `created` and `updated` as cells, `labels` in every Title cell, `references` in the Title cell for Typed and HtmlMockup families and absent for the `note` Generic family (ADR-035, REQ-006.019), the same status/slot values, the same group→child nesting, the recorded archived IDs absent, the heading and count note where §6 keeps them, and — for the empty fixtures — no table plus the shared empty message, help, and zero count. It asserts the shared header sequence and the `View`/`Edit` actions separately, because those are the intended change. The task fixture is replayed against the board (heading, count note, milestone grouping/card order, archived exclusion; no shared table or header). The reference family is covered by this same loop: the dedicated `reference_family_replays_its_characterization_fixture` test was folded into it, so coverage is preserved and no milestone-specific test remains.
- **New contract tests (TASK-142):** a new `tests/list_shared_layout.rs` asserts (a) every enabled descriptor, resolved through its representative navigation route, returns `data-list-layout="shared"` and exactly the fixed `data-column` sequence including its declared slots (the task descriptor is excluded and asserted on the board instead; a disabled type stays 404), and (b) every archived ID recorded in every fixture is absent from its list, or from the board's `data-task` cards for the task family. The grouped-row proof (one `<tbody>`, group row before its `data-child-of` children, unresolved member as a top-level row) lives in the replay file beside the requirement and NFR fixtures.

## 7. UI gate

### 7.1 Outcome: mockup required

The shared list is a visible change on every migrated list: column reorder (`Status` before the family slots, `Created` and `Updated` after them), the new `Created` column as a fixed cell, a Title cell that now carries the record's labels and references, a table instead of `ul`, an added `View`/`Edit` action cell with record-specific accessible names, and a table header that does not exist today. The shell change alone is chrome-equivalent, but AC2 cannot be satisfied without the list change, so the ticket as a whole is a visual change. A no-mockup waiver would have to assert that the approved mockups still describe the delivered lists; they do not.

### 7.2 UI description documents to update

- New: `docs/specifications/ui/browser-shell-and-shared-list-ui.md` — shell regions, list layout, column set, action cell, grouped rows, empty states, 320 px behaviour, and the anchor for the new MOCK.
- Existing, must be reconciled rather than restated: `docs/specifications/ui/browser-final-cross-type-ui.md` §5 (list rows currently described as a row metadata with `created_date`/`updated_date`), `docs/specifications/ui/browser-config-defined-document-types-ui.md` §2 and its generic list section, and the per-family list sections of `browser-backlog-records-ui.md`, `browser-decision-records-ui.md`, `browser-requirements-ui.md`, `browser-business-rules-ui.md`, `browser-audit-records-ui.md`, `browser-technical-specifications-ui.md`, `browser-glossary-terms-ui.md`, `browser-ui-specifications-ui.md`, `browser-ui-mockups-ui.md`, `browser-use-cases-test-cases-and-test-reports-ui.md`.

TASK-141 updates the new shell document and `browser-final-cross-type-ui.md` §5, and registers the MOCK. The per-family UI description sections are reconciled by TASK-148's specification port; TASK-142 changes no `docs/specifications/ui/**` file.

### 7.3 Gate

`@ui-designer` produces one review package for the shell and shared list (MOCK id allocated, recorded in `docs/specifications/ui/mockups/README.md` and in this specification). **MOCK-0016** was produced and registered on 2026-11-15 with status `draft`; the implementation run proceeds in no-approval mode (orchestrator decision, TASK-141 comment 2026-11-15), so AC2/AC3 are implemented against the settled contract and the registry row stays `draft` for owner review. AC1, AC4, and AC5 (the fixture work in §6) were never gated. TASK-142 adds no visual contract of its own and records a no-mockup waiver naming MOCK-0016 and the 32 characterization fixtures; its accepted differences are listed in §6.

### 7.4 TASK-144 gate: header toggle and collapsed layout

TASK-144 changes chrome on every page (header control, reclaimed sidebar width, `.app--full` board, `.bleed`, the 481–720 px heading wrap), so a mockup is required and a no-mockup waiver is not available on visual grounds. The review package covers: the toggle left of the brand in expanded and collapsed states, the collapsed layout, the full-width board, a `.bleed` region, the 320/480/720 px states, the JavaScript-disabled state (no button, expanded nav), the accessibility annotations from §3.4, and the heading-wrap fix from §4.5.

It updates `docs/specifications/ui/browser-shell-and-shared-list-ui.md` (its §1/§2 "no toggle in this ticket" statements are superseded) and registers a new MOCK id in `docs/specifications/ui/mockups/README.md`. **MOCK-0018** (`docs/specifications/ui/mockups/MOCK-0018-browser-navigation-toggle-and-collapsed-layout-review.html`, registry status `draft`) is that package; it draws the 11 views listed here and its layout questions OQ-6 (stacking breakpoint) and OQ-7 (collapsed width above the cap) are closed by this specification: the 480 px stacking breakpoint stays, and the 1040 px cap stays in the collapsed state (ADR-0028 downstream constraint 4). AC9 is satisfied by an approved mockup, or — when approvals are disabled for the run — by the registered `draft` MOCK id plus an explicit approval waiver recorded by the orchestrator in the ticket. The implementer may not record that waiver and may not mark AC9 complete on its own.

## 8. Resolved questions (2026-11-15)

All three open questions and the ticket's task-family premise are settled by the ADR-0028 amendment (2026-11-15). Nothing on the contract remains open; only the MOCK id is pending (§7.3).

**OQ-1 — Which family is the reference implementation? Resolved: the milestone list.** `/?type=milestone` is the only family that is simultaneously a real list route with a populated and an empty state, a family slot carrier (`Release`), a `<ul>` → `<table>` migration, and the browser side of the REQ-006.004 parity surface (§5.6). The task family is not eligible: `GET /?type=task` is the REQ-006.013 board and no task list route exists (§2).

**OQ-2 — Does `created_date` survive in the shared list? Resolved: yes, as a structural column.** The fixed order is `ID, Title, Status, family slot(s), Created, Updated, Actions`, so the parity rule from ADR-0028 resolved question 3 is satisfied structurally instead of by ad-hoc per-family slots. `labels` and `references`, which today survive only in an ad-hoc row metadata, move into the Title cell. Dropping `created_date` would have relaxed REQ-006.004, a must-have requirement, which is a business-requirement change rather than an architecture decision (§5.5).

**OQ-3 — Release family slot `Status`. Resolved: release declares no family slot.** `Status` is structural for every family and the slot table now reads release → none (§5.3), so the release list shows one status cell. Reserved-key validation prevents the class of error, not just this instance.

**Task-family premise. Resolved: the task family has no shared-list surface.** REQ-006.016 governs list views and the task family's browser view is the REQ-006.013 board. TASK-142 treats tasks as a board-regression check, not a list migration; the declared task slots stay declared for a future task list, which needs its own decision (§5.6).

**MOCK id — resolved: MOCK-0016.** `docs/specifications/ui/mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html` is registered in `docs/specifications/ui/mockups/README.md` and in `docs/specifications/ui/browser-shell-and-shared-list-ui.md` with status `draft` (approval pending). The package shows the seven-column amended order, the Title-cell tag line, and the record-specific accessible names on the row actions. This implementation run operates in no-approval mode: the explicit owner-approval step is waived by the orchestrator, the implementation follows the settled contract, and the registry status stays `draft` until owner review.

## 9. Traceability

| Artifact | Reference |
| --- | --- |
| Requirements | REQ-006.016; REQ-006.015 (width **and** navigation visibility); REQ-006.004 (browser list parity, qualified for Generic list rows by REQ-006.019); REQ-006.019 (per-family reference visibility; source RE-006); REQ-006.013 (board, now `Full`); REQ-005.007 (archive exclusion); REQ-006.003, REQ-006.009, REQ-006.012, REQ-006.014 (shell, header, list routes) |
| NFRs | NFR-001.002; NFR-001.003; NFR-005.001 (Firefox 115+ at 320/480/720 px) |
| ADRs | ADR-0028 (decisions 1, 2, 3, 4, 5, 6, 7, 8, 9; downstream constraints 4–9 and 15; decision 2 and constraint 11 amended by ADR-035 for Generic lists); ADR-035 (per-family reference visibility); ADR-0005 (templates, CSP, progressive enhancement, no cookies); ADR-0017; ADR-0020 (decision 5 amended by ADR-0028 constraint 9) |
| Domain model | `docs/domain-model/` — requirement group and member requirement (grouped rows) |
| Business rules | `docs/business-rules/` — archive and lifecycle rules cited by the existing list pages |
| UI | MOCK-0016 (shell and shared list); TASK-144 header/collapsed-layout MOCK id in §7.4; `docs/specifications/ui/browser-shell-and-shared-list-ui.md` |
| Tickets | TASK-141; TASK-142; TASK-144; TASK-145; TASK-146; TASK-147; TASK-148; TASK-149; TASK-150; TASK-178 (revision) |
| Implementation | `src/browser/shell.rs`, `src/browser/list.rs`, `src/browser/render.rs`, `src/browser/routes.rs`, `src/browser/templates/shell.html`, `src/browser/app.css`, `src/browser/app.js`, `src/browser/nav-state.js` (new), `src/browser/assets.rs`, `src/browser/vendor/MANIFEST.md`, `src/browser/http.rs`, tests |
