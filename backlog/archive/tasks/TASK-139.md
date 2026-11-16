---
id: TASK-139
title: Add collapsible navigation and content-width classes to the browser shell
status: Done
assignee:
  - @frontend-dev
created_date: '2026-11-15T16:13:00Z'
updated_date: '2026-11-16T07:39:00Z'
labels:
  - browser
  - css
  - accessibility
milestone: MS-022
dependencies:
  - TASK-136
  - TASK-138
references:
  - docs/requirements/business/REQ-06.015.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - src/browser/app.css
  - src/browser/app.js
  - src/browser/assets.rs
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - docs/specifications/ui/browser-shell-and-shared-list-ui.md
  - src/browser/shell.rs
  - src/browser/render.rs
  - src/browser/routes.rs
  - src/browser/templates/shell.html
  - src/browser/vendor/MANIFEST.md
  - src/browser/http.rs
  - tests/browser_shell_cli.rs
  - >-
    docs/specifications/ui/mockups/MOCK-0018-browser-navigation-toggle-and-collapsed-layout-review.html
priority: high
ordinal: 41570
---


## Description
**Why.** REQ-06.015 requires a control on every page that collapses the left navigation and reclaims its width, with the state persisted in `localStorage` and defaulting to expanded, and it requires the 1040 px cap to stay everywhere except the backlog board and full-width regions. ADR-0028 decisions 3-8 fix the mechanics. TASK-141 shipped the shell, `PageWidth { Capped, Full }`, and `BrowserRoutes::page_width`, but no route returns `Full`, no toggle exists in the shell header, and `app.js` still carries a sub-480 px sidebar toggle that is unpersisted and hidden at wider widths.

**Authority.** REQ-06.015; ADR-0028 decisions 3-8 and downstream constraints 4-9 and 15; feature specification `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` §3.4 (toggle, assets, CSS), §4.2-§4.5 (board width, `.app--full`, `.bleed`, heading overflow), §7.4 (UI gate). The specification carries the full rationale; this ticket carries the instructions needed to implement it.

## Current state (verified 2026-11-16)

- `src/browser/shell.rs`: `Page` still has `pub nav_toggle: &'a str`, `shell::render` fills the `{{nav_toggle}}` slot from it, and `render.rs::page` passes `""`. `shell::width_class` already returns `"app"` / `"app app--full"`.
- `src/browser/templates/shell.html`: `<html lang="en">` without `data-nav`; head loads only `/assets/app.css` and the deferred `/assets/app.js`; header is `{{nav_toggle}}<div class="brand">...</div><p class="top-meta">...</p>` as three flex children.
- `src/browser/render.rs`: `build_sidebar` emits `<aside class="sidebar"><button class="sidebar-toggle" type="button" aria-expanded="true" aria-controls="type-list">...</button><nav aria-label="Document types">...<ul id="type-list">...`. `board_page` calls `page(chrome, None, ...)`, so the board is currently `Capped`.
- `src/browser/routes.rs`: `page_width(&self, _slug: &str)` returns `Capped` unconditionally; its unit test asserts `Capped` for every enabled slug.
- `src/browser/app.js` lines ~6-45 own `setCollapsed`/`applyViewport` over `.sidebar-toggle`, `#type-list`, and `matchMedia("(max-width: 480px)")`; the dirty-form guard starts at line ~47.
- `src/browser/app.css`: `.app { max-width: 1040px; margin: 0 auto; padding: 0 24px 56px; }`, `.layout { grid-template-columns: 210px minmax(0, 1fr); gap: 30px; }`, `.main { padding-left: 30px; border-left: 1px solid var(--line); }`, `.sidebar-toggle { display: none; ... }`, and in `@media (max-width: 480px)` the rules `.sidebar-toggle { display: block }` and `.sidebar ul[data-collapsed="true"] { display: none }`. There is no `.app--full`, no `.bleed`, and no `[data-nav]` rule.
- `src/browser/assets.rs` + `src/browser/vendor/MANIFEST.md`: 24 embedded assets, one manifest row each, digest enforced by `assets::tests::manifest_covers_every_asset_with_matching_bytes_and_digest`.
- TASK-141 deferred a live defect to this ticket: `.section-head` (list `h2` + `.count`, also used by the board's empty-project fallback) overflows the page horizontally between roughly 481 and 720 CSS px, because `.count { flex: none }` cannot shrink.

## In scope

### 1. Header toggle on every page

- Add `pub const NAV_TOGGLE: &str` to `shell.rs` holding exactly
  `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav" hidden>Navigation</button>`
- Remove `Page::nav_toggle`; `shell::render` fills the `{{nav_toggle}}` slot from `NAV_TOGGLE`. No page renderer supplies or can omit the toggle. Update `render.rs::page` and the two `shell.rs` unit-test constructors.
- Placement: header, immediately left of the brand, as the first child of a new `.header-lead` wrapper:
  `<header class="site-header"><div class="header-lead">{{nav_toggle}}<div class="brand">...</div></div><p class="top-meta">...</p></header>`.
  The wrapper is required: with three direct flex children, `justify-content: space-between` would centre the brand between toggle and meta.
- Accessible name is the visible text `Navigation` and is identical in both states; state is conveyed only by `aria-expanded`. No `aria-label` duplicating the text, no icon glyph as the only content. Native `<button>`, so Enter and Space activate it; the existing `button:focus-visible` rule provides the ring.
- Give the sidebar nav the id the button controls: `<nav id="site-nav" aria-label="Document types">`. Keep `<ul id="type-list">` unchanged.
- Delete the markup and CSS of the old `<button class="sidebar-toggle">` (one toggle, in the header).
- MOCK-0018 draws a decorative chevron inside the button. It is optional: if implemented it carries `aria-hidden="true"`, is part of neither the accessible name nor the toggle logic, and any direction change is driven by CSS from `.nav-toggle[aria-expanded]` (no script writes the button's content).
- `hidden` stays in the server output: without JavaScript the control cannot do anything, so it must not be offered. `/assets/nav-state.js` removes it.

### 2. Collapsed state and its CSS

- State lives on the document element only: collapsed is `<html lang="en" data-nav="collapsed">`; expanded is the absence of `data-nav`. Never write `data-nav="expanded"`.
- `app.js` additionally sets `hidden` on `#site-nav` when collapsed and removes it when expanded, so the nav leaves the accessibility tree by attribute and not only by styling (ADR-0028 decision 4 and 8).
- CSS to add:

```css
.nav-toggle { display: inline-flex; align-items: center; gap: 6px; padding: 6px 10px;
  border: 1px solid var(--line); border-radius: 5px; background: var(--soft);
  color: var(--ink); font: inherit; font-size: 13px; font-weight: 700; cursor: pointer; }
.nav-toggle[hidden] { display: none; }
.header-lead { display: flex; align-items: center; gap: 12px; min-width: 0; }
.brand { min-width: 0; }
.project { overflow-wrap: anywhere; }
:root[data-nav="collapsed"] .layout { grid-template-columns: 0 minmax(0, 1fr); gap: 0; }
:root[data-nav="collapsed"] .sidebar { display: none; }
:root[data-nav="collapsed"] .main { border-left: 0; padding-left: 0; }
```

  `.nav-toggle[hidden]` is mandatory: once `display` is set on the class, the UA `[hidden]` rule no longer applies.
- The zero column, `gap: 0`, and removed main border/padding return 210 + 30 + 30 = 270 px to page content (MOCK-0018 annotates the 240 px column-and-gap band; the last 30 px is the removed main padding). Do not use negative margins. `.app` and its 1040 px cap stay untouched, so the reclaimed width goes to content rather than to the window edges.
- `@media (max-width: 480px)`: keep `.app` padding, `.top-meta { display: none }`, `.layout { display: block }`, `.main { border-left: 0; padding-left: 0 }`; remove `.sidebar-toggle { display: block }` and `.sidebar ul[data-collapsed="true"] { display: none }`. Collapse must work at every breakpoint; at <=480 px `.sidebar { display: none }` is the whole reclamation.

### 3. `/assets/nav-state.js` (new, at most 1 KB, about 15 lines)

- Contract:
  1. `try { stored = localStorage.getItem("papyrus.nav") } catch (e) { stored = null }`.
  2. If `stored === "collapsed"`, set `data-nav="collapsed"` on `document.documentElement` immediately, before first paint. Otherwise write nothing.
  3. Remove `hidden` from the first `.nav-toggle` on `DOMContentLoaded` when `document.readyState === "loading"`, otherwise immediately. This is the only reveal path for the control.
  4. No other globals, no `innerHTML`, no `eval`, no dependency on `app.js`.
- Register it in `src/browser/assets.rs` as `Asset { path: "/assets/nav-state.js", mime: JS, bytes: include_bytes!("nav-state.js") }` and add a first-party row to `src/browser/vendor/MANIFEST.md` (`/assets/nav-state.js`, byte count, lowercase sha256). Refresh the existing `/assets/app.css` and `/assets/app.js` rows in the same edit.
- Load it synchronously in `<head>`, after the stylesheet and before the deferred `app.js`: `<script src="/assets/nav-state.js"></script>`. No `defer`, no `async`, no inline fallback.

### 4. `app.js`

- Delete the `.sidebar-toggle` / `#type-list` / `matchMedia("(max-width: 480px)")` block, `setCollapsed`, `applyViewport`, and the media-query listener. The automatic sub-480 px collapse is removed; the stored preference alone decides.
- Add one toggle block: on load, sync `aria-expanded` (`"false"`/`"true"`) and `hidden` on `#site-nav` from `document.documentElement.getAttribute("data-nav") === "collapsed"`; on click, flip the state, apply it by attribute only, `try { localStorage.setItem("papyrus.nav", next) } catch (e) {}`, and keep `aria-expanded`/`hidden` in sync. A storage throw must not stop the visual toggle.
- The dirty-form guard, tag editing, and board drag-and-drop blocks stay byte-identical. No script reads or writes the width class.

### 5. Width classes and the board

- `routes.rs::page_width` returns `Full` for the board's task slug and `Capped` for everything else, including `""`. Resolve the board slug from the route table (Backlog nav group, the same lookup `render.rs` already performs), not from a hard-coded `"task"`; a disabled or renamed task family must not leave a stale `Full`.
- `render::board_page` passes that slug to the shell instead of `None`.
- Add `.app--full { max-width: none; }`. `.app` keeps `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px` byte-identical.
- No route other than the board may be `Full`; the mockup preview route stays out of scope and gets no route or endpoint (ADR-0028 decision 7).

### 6. `.bleed`

- Add `.bleed { width: 100%; overflow-x: auto; }` and `.bleed:focus-visible { outline: 2px solid var(--ink); outline-offset: 2px; }`.
- Wide content scrolls inside the block, so the page never gains a horizontal scrollbar. Surrounding prose keeps its narrower measure (`.text`, `.meta`, `.form` are unchanged).
- A scrollable `.bleed` block carries `role="region"`, an `aria-label`, and `tabindex="0"` in its markup, matching `.table-scroll`.
- Do **not** use `100vw`, negative horizontal margins, or a fixed-position breakout: viewport units include the scrollbar width and reintroduce page-level overflow. Reaching the window edges from a capped page is not in this ADR; it needs an amendment.
- No page emits `.bleed` yet; TASK-145 and TASK-146 consume it. Verification is the CSS contract plus the no-scrollbar breakpoint pass, not a rendered diagram.

### 7. The 481-720 px list heading overflow (deferred here from TASK-141)

- Fix: `.section-head { flex-wrap: wrap; }` in the base rule. When the heading and the count note do not fit, the count moves under the heading instead of widening the page. No copy or order change; the 480 px block keeps its own rules.
- Same class of defect at the header: the `.header-lead`/`.brand` `min-width: 0` and `.project { overflow-wrap: anywhere }` rules above keep a long project name inside the viewport at 320 px now that the toggle shares the row.
- Reproduce/verify with a long heading and count: `/?type=req` (`Business requirements` + group/member count) and `/?type=nfr`.

### 8. Existing assertions that must change in the same commit

These currently assert the removed sub-480 toggle and the all-capped rule; leaving them untouched keeps the suite red:

- `tests/browser_shell_cli.rs`: `every_page_renders_the_shell_exactly_once` asserts `!html.contains("app--full")` for `/?type=task` - split into board-is-full and every-other-route-is-capped; `width_class_is_server_side_only` asserts `!css.contains(".app--full")` - invert it.
- `sidebar-toggle` / `aria-controls="type-list"` assertions become `nav-toggle` / `aria-controls="site-nav"`: `tests/backlog_discovery_smoke.rs`, `tests/browser_cli.rs`, `tests/search_browser_smoke.rs`, `tests/ui_mockups_smoke.rs`, `tests/business_rules_smoke.rs`, `tests/content_management_smoke.rs`, `tests/technical_specifications_smoke.rs`, `tests/audit_records_smoke.rs`, `tests/use_cases_test_cases_test_reports_smoke.rs`.
- `src/browser/routes.rs::tests::page_width_is_capped_for_every_enabled_slug` and the `src/browser/shell.rs` unit tests (the `nav_toggle` field disappears).

## Out of scope

- Board content or actions (TASK-147, delivered), the shell/list refactor (TASK-141/142, delivered), math and diagram rendering (TASK-145/146), a mockup preview route or endpoint, any change to `script-src`/`style-src`/the CSP constant, any new crate or client dependency, any server-side, cookie, or config-based persistence of the nav state.

## Assumptions

- `Assumption:` the toggle's visible label is `Navigation` with the state carried by `aria-expanded`; ADR-0028 fixes only that it sits in the header, left of the brand, always visible. The mockup may restyle it but may not rename or restructure it.
- `Assumption:` ADR-0028 decision 3 defines `.bleed` as `width: 100%` of its content column with its own `overflow-x: auto`. True viewport-edge breakout is deliberately not implemented because it needs the scrollbar-safe technique the ADR leaves to a later decision.
- `Assumption:` the board's `Full` classification keys off the Backlog nav group's task slug, because `/?type=task` is the only route whose descriptor handler is `Task`.

## Dependencies

- TASK-141 (delivered): shell, `PageWidth`, `page_width`, shared layout.
- TASK-143 (delivered): `assets.rs` table and `vendor/MANIFEST.md` digest test, which the new asset extends.
- Downstream: TASK-145/TASK-146 consume `.bleed`; TASK-148 ports §3.4/§4 into the browser specification corpus; TASK-149/TASK-150 verify packaging and the release.

## Test plan

Automated (Rust):

1. `routes::tests`: `page_width` returns `Full` for the board slug only - every other enabled slug, `""`, and the default type return `Capped`; the board slug is resolved from the route table.
2. `shell::tests`: `render` emits exactly one `<button class="nav-toggle"`, with `type="button"`, `aria-expanded="true"`, `aria-controls="site-nav"`, and `hidden`, for both widths; `width_class` unchanged.
3. `tests/browser_shell_cli.rs`, every page in `shell_pages()`: exactly one `nav-toggle`; `<nav id="site-nav"` present; no `data-nav` attribute in server output; nav element not `hidden`; button `hidden` present (this is the JavaScript-disabled state: expanded and usable, no dead control).
4. Route-level widths over HTTP: `/?type=task` contains `<div class="app app--full">`; `/`, every `?type=` list route, `/doc/MS-001`, `/doc/MS-001/edit`, `/lifecycle`, `/config`, `/skills`, `/nope` contain `<div class="app">` and not `app--full`.
5. CSP: the served `Content-Security-Policy` header equals `http::CSP` (unchanged constant) on `/`, `/?type=task`, and `/?type=req`; the head contains `<script src="/assets/nav-state.js"></script>` and no `<script>` element without `src`. `tests/browser_shell_cli.rs` currently exposes only a body-returning `Server::get`; add a raw/header accessor to the local helper rather than reaching into another test file's copy.
6. No inline script or style on shell pages: no `onclick=`, no ` onchange=`, no ` style=`, no inline `<script>` body.
7. Assets: `GET /assets/nav-state.js` is `200` with `text/javascript; charset=utf-8`, contains `papyrus.nav` and `data-nav`, and is under 1 KB; `assets::tests::manifest_covers_every_asset_with_matching_bytes_and_digest` passes with the new and re-digested rows.
8. Stylesheet contract (read `src/browser/app.css` and `src/browser/nav-state.js` as text, as `width_class_is_server_side_only` already does): `.app--full` exists with `max-width: none`; the four collapsed rules exist; `.nav-toggle[hidden]` exists; `.bleed` has `width: 100%` and `overflow-x: auto`; the file contains no `100vw`; `.section-head` base rule contains `flex-wrap: wrap`; neither script contains `app--full`.
9. Behavioural assets: `app.js` contains no `matchMedia`, no `sidebar-toggle`, and no viewport-collapse logic, and still contains the dirty-form guard marker `form[data-dirty-guard]`; `nav-state.js` contains `localStorage.getItem("papyrus.nav")` inside `try`, the `"collapsed"` comparison, and the `hidden` removal.

Manual, recorded in the implementation notes (no headless-browser harness exists in this repository and adding one is out of scope):

10. Firefox 100+, Edge 130+, Safari 18+ at 1440/720/480/320 px, with a cleared and a preset `papyrus.nav`: default expanded; collapse reclaims the width on every page; the state survives navigation and reload with no flash of the expanded sidebar; the button is absent with JavaScript disabled and the nav is expanded and usable.
11. No page-level horizontal scrollbar at 320/480/720 px, expanded and collapsed, for `/?type=req`, `/?type=nfr`, `/?type=milestone`, and `/?type=task` (board columns); wide tables scroll inside `.table-scroll` only. Specifically confirm the 481-720 px heading/count row at 600 px on `/?type=req`.
12. Keyboard pass: Tab reaches the toggle, the focus ring is visible, Enter and Space toggle, `aria-expanded` and the nav's `hidden` follow.

## UI gate

- Required. The change is chrome-visible on every page, so the mockup covers the header toggle (expanded and collapsed), the collapsed layout, the full-width board, a `.bleed` region, the 320/480/720 px states, the JavaScript-disabled state, and the heading-wrap fix.
- **MOCK-0018** (`docs/specifications/ui/mockups/MOCK-0018-browser-navigation-toggle-and-collapsed-layout-review.html`, registry status `draft`) is the package: 11 web views covering the toggle in both states with its accessibility contract, the expanded and collapsed body, the full-width board, a bleed band, 720/480/320 px, the JavaScript-disabled page, and the 600/320 px heading row. It also updated `docs/specifications/ui/browser-shell-and-shared-list-ui.md`, whose "no toggle in this ticket" statements are now superseded.
- AC9 passes on an approved MOCK-0018, or - when approvals are disabled for the run - on the registered `draft` MOCK id plus an approval waiver recorded in this ticket by the orchestrator. The implementer must not record the waiver or check AC9 on its own.

## Resolved during refinement (2026-11-16)

- **Stacking breakpoint (MOCK-0018 OQ-6).** The 480 px stacking breakpoint stays unchanged. Moving navigation stacking to 720 px would change every page's layout and belongs to a separate design decision, not to the width-and-nav ticket.
- **Collapsed width above the cap (MOCK-0018 OQ-7).** The 1040 px cap stays in the collapsed state: the reclaimed width goes to page content inside the capped column, never to the window edges (ADR-0028 downstream constraint 4). A collapsed page that grows past 1040 px would need an ADR amendment.
- **Board column minimum widths.** Not specified by this ticket; the mockup's values are illustrative. The board keeps `.board-column { flex: 1 1 0; min-width: 0 }` and its existing 480 px column stacking; no new minimum width is introduced.
- **Decorative chevron.** Optional and non-semantic, as described in item 1 above.
- **Reclaimed width.** 270 px total (210 px column + 30 px gap + 30 px main padding-left); MOCK-0018's 240 px annotation covers only the column and gap.

## Notes for downstream tickets

- TASK-148 ports §3.4 and §4.2-§4.5 of the specification into the browser specification corpus and reconciles the per-family UI documents; TASK-144 changes no `docs/specifications/ui/**` file itself.
- TASK-149's asset verification must include the new file's bytes and digest; TASK-150 covers the breakpoint matrix above.

## Acceptance Criteria
- [x] Every page, including error pages, contains exactly one `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` whose accessible name is the visible text `Navigation`; it is reachable by Tab, shows a visible focus ring, and is activated by Enter and Space. A decorative glyph may sit inside it only as `aria-hidden="true"` and outside the toggle logic.
- [x] Collapsing sets `data-nav="collapsed"` on `<html>`, drops the sidebar grid column and its gap, removes the main column left border and padding-left (210 px column + 30 px gap + 30 px padding returned to page content), and sets `hidden` on `#site-nav`, so the reclaimed width is real layout width and the nav leaves the accessibility tree.
- [x] The collapsed or expanded state persists in `localStorage` under the key `papyrus.nav` (values `collapsed` and `expanded`) across page navigation and reload, is applied by `/assets/nav-state.js` before first paint so no expanded frame is ever painted while collapsed, and uses no server state and no cookie.
- [x] The default state is expanded at every viewport width when no stored value exists, and the automatic sub-480 px collapse is removed together with `.sidebar-toggle`, its `matchMedia` logic, and the `data-collapsed` attribute.
- [x] With JavaScript disabled the server-marked-up page has no `data-nav`, an unhidden and usable expanded navigation, and a `hidden` toggle; and when `localStorage` throws the page stays expanded and the toggle still works for the session.
- [x] Standard pages stay capped at 1040 px and the board route renders full width: `BrowserRoutes::page_width` returns `Full` for the board task slug and `Capped` for every other route, `.app--full` exists with `max-width: none`, and the mockup full-width class exists with no preview route or endpoint.
- [x] A `.bleed` region takes the full width of a capped page content column and owns its horizontal scrolling (`width: 100%` plus `overflow-x: auto`, no `100vw`, no negative margins), and no page gains a horizontal scrollbar anywhere, including the list section heading and count row between 481 and 720 px, which wraps instead of overflowing.
- [x] Route tests assert `Capped` for every list, detail, edit, inventory, config, skills, and error route and `Full` only for the board, the served `Content-Security-Policy` header is byte-identical to `http::CSP`, and the pages add no inline script, inline style, or inline event handler.
- [x] MOCK-0018 is approved, or the orchestrator has recorded an explicit approval waiver in this ticket; the mockup stays registered with status `draft` in `docs/specifications/ui/mockups/README.md` until owner review, and the implementer does not record the waiver itself.

## Definition of Done
- [x] #1 The automated test plan in this ticket passes (`cargo test`), including the `routes` width test, the `shell` toggle test, the route-level width/CSP/no-inline assertions in `tests/browser_shell_cli.rs`, and the asset manifest digest test.
- [x] #2 `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` are clean.
- [x] #3 `/assets/nav-state.js` is registered in `src/browser/assets.rs` and in `src/browser/vendor/MANIFEST.md` with its byte count and lowercase sha256, and the `/assets/app.css` and `/assets/app.js` manifest rows are re-digested.
- [x] #4 No new crate, client library, or build step: `Cargo.toml` and `Cargo.lock` are unchanged and no Node toolchain is introduced.
- [x] #5 `http::CSP` and the served `Content-Security-Policy` header are unchanged, and no inline script, inline style, or inline event handler is added by this task.
- [x] #6 The `app.js` dirty-form guard blocks stay byte-identical in behaviour: typing marks the form dirty, reveals the warning, and sets `beforeunload`.
- [x] #7 The breakpoint and browser pass is executed and recorded in the implementation notes: 320, 480, 600, 720, and 1440 px in Firefox 100+, Edge 130+, and Safari 18+, in the expanded, collapsed, JavaScript-disabled, and board states, checking for no page-level horizontal scrollbar.
- [x] #8 The shell feature specification sections owned by this contract (§3.4, §4.2-§4.5, §7.4) match what was built, and any deliberate deviation is recorded there; the implementer changes no `docs/specifications/ui/**` file (TASK-148 ports the UI corpus).
- [x] #9 The UI gate is satisfied: the MOCK id is registered in the mockup registry and the shell UI description, and either the mockup is approved or the orchestrator has recorded an approval waiver in this ticket.
- [x] #10 All acceptance criteria are checked, and no item from the Out of scope list is touched.


## Notes
Delivered: NAV_TOGGLE in shell.rs on every page with aria-expanded/aria-controls, nav-state.js (679 B, synchronous in head, key papyrus.nav, storage exceptions caught) applying the collapsed state before first paint, app.js reduced to toggle sync plus the form guard with the sub-480 auto-collapse removed, collapsed grid as one minmax(0,1fr) track with the sidebar removed and the nav hidden, PageRoute-based full width (Board only), .app--full and .bleed with their own overflow containers, heading/pre/table/board scroll containment, and labelled focusable scroll regions on /config, the board columns and the 11 /skills command blocks. Measured Firefox 153.5.1esr headless (WebDriver BiDi, DPR 1): zero page overflow across 98 route/width/state samples at 320/480/481/500/600/720/1440 px for ?type=req|nfr|task|milestone|release|decision, /doc/TASK-001, /doc/TASK-001/edit, /config, /skills, /nope; collapsed .main 540 px at 600 and 992 px at 1440 (1380 for the full board), never 0; only ?type=task and its expand=/cleanup= variants render app app--full. Three review rounds found and fixed: a zero-width collapsed content column, task detail/edit wrongly full width, and 481-500 px plus 320 px page overflow. Edge 130 / Safari 18 / Firefox 100 remain unverified and are an explicit TASK-150 checklist item.


## Comments
author: refiner
created: 2026-11-16T06:33:00Z

---

Refined 2026-11-16 (task-refinement). Pinned: shell::NAV_TOGGLE markup and accessible name "Navigation"; data-nav="collapsed" on <html> with expanded as attribute absence; localStorage key papyrus.nav and both values; /assets/nav-state.js contract (pre-paint apply, hidden removal, caught storage errors, <=1 KB, asset table + MANIFEST digest); app.js ownership split with the sub-480 auto-collapse removed and the dirty-form guard preserved; .app--full, .bleed, and the collapsed grid rules; board PageWidth::Full via the route table slug. The 481-720 px .section-head overflow deferred by TASK-141 is fixed here by `.section-head { flex-wrap: wrap; }` plus header min-width rules and is covered by AC7 and tests 8/11. UI gate: MOCK-0018 registered as draft; no waiver recorded — the orchestrator records approval or waiver for AC9. Resolved from the mockup: 480 px stacking breakpoint stays (OQ-6); the 1040 px cap stays when collapsed (OQ-7); the decorative chevron is optional and non-semantic. Spec updated: docs/specifications/browser-shell-shared-list-layout-and-page-width.md §3.4, §4.2-§4.5, §7.4.

---

author: orchestrator
created: 2026-11-16T06:33:00Z

---

AC9 resolution under no-approval mode: explicit owner approval of MOCK-0018 (navigation toggle and collapsed layout, 11 web views) is waived by the user's 'start implementation in no-approval mode' instruction. MOCK-0018 remains registry status 'draft' for owner review, and the UI description names it. Design points settled with the designer: the 480 px stacking breakpoint and the 1040 px cap when collapsed are both retained; the decorative chevron is optional and aria-hidden; reclaimed width is 270 px. ADR note for the architect: decision 3's .bleed is implemented as width 100% of the content column with its own overflow container rather than a 100vw breakout, because a viewport breakout reintroduces the horizontal scrollbar that constraint 7 forbids; a true window-edge bleed would need an ADR amendment.

---

author: orchestrator
created: 2026-11-16T07:32:00Z

---

AC6 amended to the delivered contract: page width is a route property. BrowserRoutes::page_width(&self, route: PageRoute) returns Full for PageRoute::Board and Capped for PageRoute::Other; render::board_page passes Board, every other renderer passes Other. The earlier slug-based wording (page_width(slug)) is superseded and would have let /doc/TASK-001 and /doc/TASK-001/edit inherit full width, which an independent Firefox pass caught. Board-only Full is asserted over HTTP including the expand= and cleanup= variants.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
The browser shell gains one navigation toggle in its header, present on every page, that collapses the left navigation and gives its width back to the content. The collapsed state is stored in localStorage under papyrus.nav and applied by a small same-origin script before first paint, so there is no flash, the server keeps no state and sets no cookie; with JavaScript disabled the navigation stays expanded and usable, and storage failures are caught. Width is now a route property: only the task board renders full width, while lists, detail, edit, config, skills and error pages keep the 1040 px cap, and a bleed region inside a capped page scrolls its own content. Long headings, command blocks, configuration tables and board columns stay inside their containers, so no page gains a horizontal scrollbar between 320 and 1440 pixels.
