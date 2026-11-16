# Feature Specification — Browser Shell UI: Shared List Layout, Width Classes, and Collapsible Navigation

**Tickets:** TASK-148 (publisher; contract fixed by TASK-141, TASK-142, TASK-144)
**Milestone:** m-21
**Status:** Normative, published 2026-11-16 from the landed contract
**Requirements:** REQ-06.016 (one shared list layout), REQ-06.015 (content width and navigation visibility), REQ-06.004 (browser/CLI metadata parity), REQ-06.013 (uncapped per-milestone board), REQ-05.007 (archived records never listed)
**ADR:** ADR-0028 (Accepted, amended 2026-11-15 and 2026-11-16) and ADR-0005 as amended by ADR-0027
**UI:** MOCK-0016 (shell and shared list, registry `draft`) and MOCK-0018 (navigation toggle and collapsed layout, registry `draft`); layout description `docs/specifications/ui/browser-shell-and-shared-list-ui.md`
**Contract detail:** `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` (TASK-141/142/144 delivery record and implementation detail)

## 1. Purpose and scope

This page is the normative browser-shell contract for page chrome, the shared list layout, the two content-width classes, and the collapsible navigation. It states behaviour; visual measurements and pixel values are owned by the UI description and the approved mockups.

### 1.1 In scope

- The page shell: header, document-type sidebar, main region, and footer rendered once per response.
- The shared list layout used by every list view, including the fixed column order, family slots, Title-cell labels and references, action placement, and grouped rows.
- The `Capped` and `Full` page-width classes and the `.bleed` full-width region inside a capped page.
- The navigation toggle, its `papyrus.nav` persistence, the default-expanded state, and behaviour without JavaScript.
- The accessibility contract for the shell, the shared list, and the collapsed navigation.
- The supported-browser floor (NFR-05.001) and the response-header and asset rules that the shell depends on.

### 1.2 Non-goals

- Detail and edit field sets: REQ-06.016 mandates a uniform **list** layout only; collapsing detail/edit forms is out of scope.
- The board's card content (REQ-06.013). This contract fixes only the board's `Full` width class.
- A mockup preview route or endpoint (ADR-0028 decision 7; needs its own ADR).
- Math and diagram rendering internals — see `docs/specifications/browser-math-and-diagram-rendering.md`.
- Visual measurements, spacing, colour, and iconography — owned by the UI description and MOCK-0016/MOCK-0018.
- Any template engine, client framework, or new dependency (ADR-0028 decision 6).

## 2. Shell regions

### 2.1 Regions

| Region | Markup | Content |
|---|---|---|
| Header | `<header class="site-header">` | The navigation toggle, the `P` mark, `Papyrus`, the project name, and the static top-meta note. |
| Sidebar | `<aside class="sidebar">` containing `<nav id="site-nav" aria-label="Document types">` | Grouped document-type entries; the current entry carries `aria-current="page"`. |
| Main | `<main class="main">` | The page body fragment only; no chrome. |
| Footer | `<footer class="site-footer">` | The shared footer copy. |

Each region appears exactly once per response, including error pages.

### 2.2 Invariants

- Exactly one `<div class="app…">` wrapper per response.
- Body fragments only: no header, sidebar, footer, or width wrapper is rendered inside `<main>`.
- One `h1` per page; heading levels are not skipped.
- The shell owns the navigation toggle and the width class. It never renders list actions, filters, or page headings other than page chrome.

## 3. Shared list contract

### 3.1 Column plan

The fixed order for every family is:

`ID, Title, Status, family slot(s), Created, Updated, Actions`

- `Row.cells.len() == columns.len() - 3`, because ID, Title, and Actions are structural.
- Reserved keys `id`, `title`, `status`, `created`, `updated`, and `actions` are never declared as a slot, never repeated, and never reordered.
- A family with two slots has eight header cells; a family with no slot has six. Only the slot cells vary; the six structural columns stay in place.
- Archived records never reach the list: the index is enabled-only and active-only (REQ-05.007).

### 3.2 Family slots

| Family | Declared slot(s) |
|---|---|
| Task | `Milestone`, `Assignee` (no list surface today; see below) |
| Milestone | `Release` |
| Release | none |
| Decision | `Type`, `Superseded by` |
| Requirement / NFR | `Group` |
| Audit record | `Audit type` |
| Business rule | `Applicability` |
| UI mockup | `Task`, `Specs` |
| Generic document (config-defined type) | none |

Slots come from declared presentation data, never from a `match` on a slug, a handler, or a per-family template. The task family has no shared-list surface while REQ-06.013 owns the backlog view: `/?type=task` is the kanban board, not a list.

### 3.3 Title cell: labels and references

The Title cell renders the record link, then the record's front-matter labels and references below it, from escaped values. Labels and references are general front-matter metadata, not family data: they are rendered for every family, are never a slot, and never reorder a column. This preserves REQ-06.004 parity when the list shows the same values as the CLI.

### 3.4 Action cell

The action cell is always last, in the order `View`, `Edit`, then any family action that already exists.

- `View` links to `/doc/{id}`; `Edit` links to `/doc/{id}/edit`.
- Every link carries a record-specific accessible name: `aria-label="View {id}"` and `aria-label="Edit {id}"`.
- Archive, delete, and create controls stay at page level and are not row actions.

### 3.5 Grouped rows

Grouped families (requirements and NFRs) use the hierarchical row variant:

- The group row and its member rows share one `<tbody>`.
- Child rows carry `data-child-of="{group id}"`.
- Rows are emitted one group at a time in ascending group ID order; members ascend by ID.
- A member whose group row is absent from the filtered index renders as a top-level row in ID order. It is never dropped, duplicated, or attached to a synthetic group.
- The count note is `N group(s) · M requirement(s)`, computed from the rows actually emitted, including orphan members.

### 3.6 Empty state and count note

- One shared empty state for every family: `No {noun} in this project.` plus the help line `Records are created from the command line; this browser does not create records.`
- One muted `(none)` for an unset status, date, or slot value.
- The count note is shown for zero records as `0 <noun>` (or `0 groups · 0 requirements` for grouped families).
- A group with no members still renders its group row and no member rows.

### 3.7 Stable markup hooks

- `data-list-layout="shared"` on the list table marks the shared path.
- `data-column="{key}"` on every header cell.
- `data-row-id` on every row and `data-child-of` on every grouped child row.
- The table scrolls inside its own container at 320 CSS px and never creates page-level horizontal scrolling.
- Exact classes and spacing belong to the UI description, not to this page.

## 4. Content-width classes

### 4.1 `Capped`

`Capped` is the default for every route. It renders `class="app"` with `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px`.

### 4.2 `Full`

`Full` renders `class="app app--full"` with `max-width: none`. It is set only for the backlog board (`PageRoute::Board`) while REQ-06.013 owns that view. Any other uncapped route requires an ADR-0028 amendment. JavaScript never reads or writes the width class.

### 4.3 `.bleed` inside a capped page

A full-width-eligible region inside a `Capped` page uses `.bleed`:

- `width: 100%` plus its own `overflow-x: auto`.
- It carries `role="region"`, an `aria-label`, `tabindex="0"`, and a visible `:focus-visible` ring when it scrolls.
- No `100vw`, no negative margins, no fixed-position breakout.
- Used by rendered diagram figures (ADR-0026 decision 10) and later by mockup content.

### 4.4 Width invariants

- The route decides the width on the server, so the no-JavaScript layout equals the scripted layout.
- `.app--full` lifts only the cap and keeps centring and padding.
- The 481–720 CSS px section heading wraps beneath the count instead of widening the page.
- No page-level horizontal scrolling at 320, 480, or 720 CSS px.

## 5. Collapsible navigation

### 5.1 Toggle

One `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` sits in the header on every page, left of the brand, with the stable accessible name `Navigation`.

- Server output carries `hidden`; the synchronous `/assets/nav-state.js` reveals it.
- It is a native button, so Enter and Space activate it, and it uses the stylesheet focus ring.
- Any chevron is decorative: `aria-hidden="true"`, not part of the accessible name, and no script writes the button content.

### 5.2 Stored preference: `papyrus.nav`

- The key is `papyrus.nav`; the values are `"collapsed"` and `"expanded"`.
- Only the exact value `"collapsed"` collapses. A missing key, any other value, or a storage exception means expanded, and the session continues in memory.
- `/assets/nav-state.js` sets `data-nav="collapsed"` on `<html>` before first paint; `app.js` keeps `aria-expanded` and `hidden` in sync. `data-nav="expanded"` is never written.
- No cookie, no server state, and no configuration key stores the state.

### 5.3 Default state

Expanded is the default at every viewport width. There is no automatic narrow-viewport collapse.

### 5.4 Without JavaScript

Without JavaScript the page renders expanded, the navigation is visible and usable, and the toggle stays `hidden`. No width or navigation state depends on script.

### 5.5 Script order and assets

In `<head>`, after the stylesheet:

1. `/assets/app.css`;
2. synchronous `/assets/nav-state.js`;
3. `defer` `/assets/app.js`.

Math pages additionally link the KaTeX stylesheet and script; diagram pages add the Mermaid script before `app.js` (ADR-0026 decision 4). All are classic same-origin scripts under `script-src 'self'`.

## 6. Accessibility

- Every list header cell keeps `scope="col"`; every row action link carries a record-specific accessible name (§3.4).
- `aria-expanded` and `aria-controls` stay in sync with the toggle state.
- The collapsed navigation is removed with `hidden`, not merely visually hidden, so it leaves the accessibility tree.
- Grouped child rows expose their parent through `data-child-of` plus a non-colour visual indent.
- Focus order follows document order; there is no keyboard trap.
- The layout is usable at 320, 480, and 720 CSS px and at 200% zoom without page-level horizontal scrolling of text.

## 7. Supported browsers

Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001), verified at 320, 480, and 720 CSS px. `:has()`, container queries, `color-mix()`, `@layer`, subgrid, `dialog`, popover, `inert`, optional chaining, top-level await, and private class fields are forbidden. TASK-150 owns the release verification evidence.

## 8. Response headers and assets

Every `text/html` response carries exactly one of the two CSP strings normatively fixed in `docs/specifications/browser-requirements.md` §9.7: the strict string on every HTML response whose pre-pass did not set `has_diagram`, and the diagram string — identical except `style-src 'self' 'unsafe-inline'` — only when the rendering pre-pass set `has_diagram`. `/assets/` responses and non-HTML responses carry no CSP. `script-src 'self'` never gains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash. The diagram string is scoped to `has_diagram` responses and is verified by TASK-150.

Every browser byte comes from the static table `src/browser/assets.rs` (`Asset`, `ASSETS`, `lookup`); no request path is joined to a filesystem path. `/assets/` paths match exactly, `GET`/`HEAD` only (`405` otherwise), a miss is `404`, `HEAD` sends `Content-Length` with no body, and asset responses carry no CSP. Rendering assets are specified in `docs/specifications/browser-math-and-diagram-rendering.md` §16–§17.

## 9. Traceability

| Artifact | Reference |
|---|---|
| Requirements | REQ-06.016, REQ-06.015, REQ-06.004, REQ-06.013, REQ-05.007 |
| NFRs | NFR-05.001 (browser floor), NFR-01.003 (offline) |
| ADRs | ADR-0028, ADR-0027, ADR-0025, ADR-0026, ADR-0005 |
| UI | MOCK-0016, MOCK-0018, `docs/specifications/ui/browser-shell-and-shared-list-ui.md` |
| Tickets | TASK-141, TASK-142, TASK-144, TASK-146, TASK-148 |
| Implementation | `src/browser/shell.rs`, `list.rs`, `routes.rs`, `render.rs`, `templates/shell.html`, `app.css`, `app.js`, `nav-state.js`, `assets.rs`, `http.rs` |
