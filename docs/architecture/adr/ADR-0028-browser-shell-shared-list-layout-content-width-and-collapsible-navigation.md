---
id: ADR-0028
title: "Browser Shell: Shared List Layout, Content-Width Contract, and Collapsible Navigation"
created_date: '2026-11-15T00:00:00Z'
updated_date: '2026-11-15T15:58:31Z'
labels: []
references: []
---
# Browser Shell: Shared List Layout, Content-Width Contract, and Collapsible Navigation

## Status

Accepted (2026-11-15) — owner decision 2026-11-15: one shared shell and list layout, server-side `PageWidth` width contract, `localStorage` navigation persistence (default expanded), and the mockup full-width class only (preview route out of scope).

## Date

2026-11-15

## Context

REQ-06.016 requires every browser list view — tasks, milestones, releases, decisions, and every document type including audit records and technical specifications — to use one shared list layout with fixed header, row/column structure, and action placement, with family-specific fields in designated slots and no bespoke list template. REQ-06.015 requires the 1040 px cap to stay for ordinary pages, to be lifted for the backlog kanban board and UI-mockup previews, and to allow diagram/visualization content to use full width inside otherwise-capped pages; it also requires a control on every page that collapses and expands the left navigation, reclaims its width, persists across navigation, and defaults to expanded.

Current shape:

- Seven templates: `list.html`, `view.html`, `edit.html`, `error.html`, `document_list.html`, `document_detail.html`, `document_edit.html`. `list.html` and `document_list.html` are near-duplicates; `browser::render` (4.7 k lines) builds per-family list markup with its own headers, counts, and action cells.
- `app.css` hard-codes the cap in one rule: `.app { max-width: 1040px; margin: 0 auto; padding: 0 24px 56px; }`, and the sidebar is a fixed `210px` grid column (`.layout { grid-template-columns: 210px minmax(0, 1fr); }`).
- `app.js` (68 lines of logic) contains a sidebar toggle that exists only below 480 px; `data-collapsed` lives on the type list, the state is not persisted, and the toggle is hidden at wider widths.
- The backlog kanban board (REQ-06.013) and a UI-mockup preview route do not exist yet; REQ-06.015 names both as uncapped surfaces, so the width contract has to be defined before they are built.
- ADR-0005 fixes: no cookies or sessions, JavaScript as optional progressive enhancement, embedded assets, `Cache-Control: no-store`, `X-Frame-Options: DENY`, and a CSP without `'unsafe-inline'` for scripts. ADR-0020 fixes registry-derived navigation and list routes and states that no new asset is introduced.

Forces: one layout for a type set that changes at runtime (ADR-0017, ADR-0020); no template engine (ADR-0005); no per-family render branches for lists; the nav state must survive navigation without a server, a session, or a cookie; and `X-Frame-Options: DENY` plus `frame-ancestors 'none'` mean an embedded mockup preview is a separate decision, not a side effect of a width class.

## Decision

1. **A new `browser::shell` module owns page chrome.** It produces the document skeleton (head, header, nav, `<main>`, footer), the width class, and the nav toggle. Templates are reduced to: `shell.html` (chrome with `{{sidebar}}` and `{{main}}` slots), `list.html` (the single shared list layout), `view.html`, `edit.html`, `error.html`. `document_list.html` is deleted; its content model becomes slots in the shared list. `view.html`/`edit.html` keep their typed field sets — REQ-06.016 mandates a uniform **list** layout only, and collapsing detail/edit forms is out of scope.

2. **One shared list layout with a server-built column plan.** `browser::list` builds, per list page:

   ```rust
   pub struct ListPlan { pub heading: String, pub count_note: String, pub columns: Vec<Column>, pub rows: Vec<Row> }
   pub struct Column { pub key: &'static str, pub label: &'static str, pub class: &'static str }
   pub struct Row { pub id: String, pub href: String, pub cells: Vec<String>, pub actions: Vec<(String, String)>, pub children: Vec<Row> }
   ```

   - The fixed column order for every family is: ID, Title, Status, family slot(s), Updated, Actions. Only the *labels and presence* of the family slot(s) vary, and they come from the descriptor plus the presentation table (ADR-0020), never from a per-family template or a `match` on slug.
   - Family slots are declared data: task → `Milestone`, `Assignee`; milestone → `Release`; release → `Status`; decision → `Type`; requirement/nfr → `Group`; audit → `Audit type`; rule → `Applicability`; ui-mockup → `Task`, `Specs`; generic document → none (ID, Title, Status, Updated, Actions).
   - Grouped families (requirements/NFR) use the hierarchical row variant (`Row::children`): the group row plus its member rows in the same table body, not a second template.
   - Action placement is fixed: the last cell, in the order `View`, `Edit`, and any family action that already exists. Archive/delete/create controls stay at page level (unchanged).
   - Archived records remain excluded because indexing is enabled-only and active-only (ADR-0020 decision 7); the layout never sees them.

3. **The width decision is made on the server, per route.** `routes.rs` gains a route property:

   ```rust
   pub enum PageWidth { Capped, Full }
   ```

   `Capped` is the default for every route. `Full` is set only for the backlog kanban board route (when REQ-06.013 lands) and the UI-mockup preview route (when it exists). The shell renders `class="app app--full"` for `Full` and `class="app"` for `Capped`. Client JavaScript never changes the page width class, so no-JS behaviour and server tests agree.

   Inside a `Capped` page, a full-width-eligible region (rendered diagram figure, and later an embedded mockup) uses a `.bleed` block: `width: 100%`, with its own `overflow-x: auto` wrapper, so wide content scrolls inside the region and the page never gains a horizontal scrollbar. `Capped` pages keep `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px` exactly as today.

4. **Navigation collapse is a client-side, per-browser preference with no server state.**
   - Markup: one `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` in the shell header on **every** page, and `<nav id="site-nav">` for the existing sidebar.
   - State: `localStorage` key `papyrus.nav` with values `"collapsed"`/`"expanded"`. Collapsed state sets `data-nav="collapsed"` on `<html>`; CSS then removes the sidebar from the grid (`grid-template-columns: 0 minmax(0, 1fr)`), hides it from the accessibility tree (`hidden` attribute set by script), and drops the `.main` left border, so the reclaimed width is real layout width, not a negative margin.
   - No flash of the wrong state: a new asset `/assets/nav-state.js` (≈15 lines) is loaded **synchronously in `<head>`**, reads the stored value, and sets `document.documentElement.dataset.nav` before first paint. `app.js` stays `defer` and only handles the toggle and the existing form guard.
   - Default: expanded. When no stored value exists, the state is expanded at every viewport width; the current "auto-collapse below 480 px" behaviour is removed, because it competes with a persisted user preference and REQ-06.015 makes the collapsed state the user's choice. `localStorage` is the only persistence mechanism (constraint 5): no cookie, no server state, no configuration key.
   - Degradation: with JavaScript disabled the button is not rendered (`<noscript>`-style server-side omission is not possible without JS detection, so the button carries `hidden` and a two-line asset removes `hidden`), the nav is expanded and fully usable, and any `localStorage` throw (private mode, quota, disabled storage) is caught and ignored, leaving in-memory state for the session.
   - Rejected alternatives: a cookie (ADR-0005 forbids cookies and sessions, and it would make the server stateful per client), server-side state in the project or user config (a write path, a concurrency surface, and per-user state in a project file), a query parameter on every link (breaks persistence across navigation and deep links), and a CSS-only checkbox (state cannot survive navigation).

5. **CSP is unchanged by this ADR.** The nav script is an external same-origin asset, so `script-src 'self'` covers it; the toggle uses class/attribute changes only, so `style-src 'self'` is untouched. Only ADR-0025 (`font-src`) and ADR-0026 (`style-src 'unsafe-inline'` for Mermaid's SVG `<style>`) relax the policy, and only for their stated reasons.

6. **No new dependency, template engine, or client framework.** Lists are rendered server-side into the same `{{name}}` placeholder templates; the two scripts are classic scripts; the CSS is the existing stylesheet plus new rules.

7. **UI-mockup previews are out of scope here; only the mockup full-width class is defined.** REQ-06.015 presupposes a mockup preview surface; none exists, and the stored review packages contain their own HTML/CSS/JS (ADR-0005 and the mockup specification currently forbid serving the package at all). This ADR fixes only the `PageWidth::Full` classification and the `.app--full` class that a mockup preview will use, and records the binding constraint: any future preview route must serve the package as a **sandboxed, opaque-origin document** (`sandbox` without `allow-same-origin`, its own CSP, `X-Frame-Options`/`frame-ancestors` adjusted only for that route), because serving an author-supplied HTML package same-origin would give it access to the browser write API. That route requires its own ADR before implementation and is not part of this decision.

8. **Accessibility and browser support are part of the contract.** The toggle is keyboard operable with a visible focus ring, `aria-expanded` and `aria-controls` stay in sync, the collapsed nav is removed with `hidden` rather than only visually hidden, and the layout is verified at the existing 320/480/720 px breakpoints and in Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001).

## Consequences

**Positive:**

- REQ-06.016 becomes structural: a new document type or family cannot get its own list markup, because the only list template is shared and the only variation is declared slot data.
- Deleting `document_list.html` and the per-family list builders removes a recurring source of "this list forgot the archive filter / forgot the action cell" defects.
- The width rule is one enum on the route plus two CSS classes, which is testable per route instead of asserted by eye.
- The nav preference is per browser, survives navigation, costs one small asset, and adds no server state, cookie, or write path.
- Removing the automatic sub-480 px collapse eliminates a state the user could not control.

**Negative:**

- `browser::render` loses about a dozen list builders, which is a large mechanical refactor with visual-diff risk on every list page.
- The shared column plan flattens family-specific presentation: e.g. the requirement list's per-group counts and the ui-mockup's `Specs` cell must be expressed as slot content, which may be visually narrower than today.
- A blocking head script is a new (small) render-blocking cost on every page and a new asset that must work before `app.js`.
- Removing the automatic narrow-viewport collapse means a phone-width first load shows the expanded nav until the user collapses it.
- The `.bleed` breakout technique must be checked against every supported browser; `100vw` interacts badly with scrollbars if used naively, hence the `overflow-x: auto` wrapper rule.

**Risks and mitigations:**

- *List regression across ~10 families*: characterization tests per family (header labels, column order, action cells, ordering, archived exclusion) captured before the refactor and replayed after it.
- *A family silently keeps bespoke markup*: a test enumerates every enabled descriptor and asserts the list route returns markup produced by the shared layout (a `data-list-layout="shared"` marker plus the same header sequence).
- *Nav state and layout fight each other*: one test loads a page with `data-nav="collapsed"` preset, navigates, and asserts the sidebar is absent and the main column is wider; a second runs with JavaScript disabled and asserts the nav is present.
- *Width class applied too broadly*: a route-level test asserts `Capped` for every list/detail/edit route and `Full` only for the kanban and preview routes.
- *Grouped-list flattening breaks the requirement hierarchy*: the hierarchical row variant is required by a test on the requirement list (group row plus member rows, same table).

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| One shell + one shared list layout, server-side width enum, `localStorage` nav state (this ADR) | Selected | Satisfies both requirements with no new dependency, no server state, and no CSP change. |
| Keep per-family list templates and "harmonise" their CSS | Rejected | REQ-06.016 explicitly rejects "consistent headers and columns" in favour of one shared layout. |
| Render lists client-side from the JSON API | Rejected | Violates the no-JS-required posture of ADR-0005 and adds a client rendering layer. |
| Collapse state in a cookie | Rejected | ADR-0005 forbids cookies and sessions; it also makes layout server state per client. |
| Collapse state in project configuration | Rejected | Per-user UI state in a project file, plus a new write path and concurrency surface. |
| Collapse state in a URL query parameter | Rejected | Does not persist across navigation; requires rewriting every link. |
| CSS-only collapse (checkbox hack) | Rejected | State cannot survive navigation, which REQ-06.015 requires. |
| Set the width class from JavaScript | Rejected | Breaks no-JS equality and makes the route contract untestable server-side. |
| Serve the mockup preview inside the same page markup | Rejected | Same-origin script execution from an author-supplied package; violates ADR-0005's security model and the mockup specification's "never served" rule. |

## Resolved questions (owner, 2026-11-15)

1. **Navigation persistence** — decided: `localStorage` under `papyrus.nav`, default expanded at every viewport width; no automatic narrow-viewport collapse (decision 4).
2. **UI-mockup scope** — decided: this ADR fixes the mockup full-width class (`PageWidth::Full` / `.app--full`) only; the preview route is out of scope and needs its own ADR (decision 7).
3. **Family-specific fields in the shared list** — decided: every field that exists today is kept as a declared slot; visual trimming is a design decision, not a structural one.
4. **Toggle position** — a design refinement (header, left of the brand, always visible); it carries no architectural consequence.

## Downstream constraints

1. `document_list.html` and any per-family list builder must not survive; every list view renders through the shared layout, including config-defined types.
2. Family variation is declared data (descriptor + presentation table); no `match` on a slug and no per-family template may be added for a list.
3. `PageWidth` is decided server-side per route; JavaScript may not change the page width class.
4. The 1040 px cap stays on every route except the kanban board and the mockup preview; a new uncapped route requires an amendment to this ADR.
5. Nav state is stored only in `localStorage` under `papyrus.nav`; no cookie, no server state, no config key.
6. The nav toggle must work without JavaScript (expanded, visible, usable) and must not rely on inline styles or inline scripts (CSP unchanged).
7. An in-page full-width region must never cause page-level horizontal scrolling; wide content scrolls inside its own container.
8. A mockup preview route may not be added under this ADR; it needs its own decision covering sandboxing, CSP, and `X-Frame-Options`.
9. ADR-0020 downstream constraint 5 ("no new asset or dependency") is amended by this ADR: one small same-origin script asset is added; the no-dependency and no-template-engine rules stand.

## References

- `docs/requirements/business/REQ-06.016.md` — one shared list layout for every artifact family.
- `docs/requirements/business/REQ-06.015.md` — content width and navigation visibility.
- `docs/requirements/business/REQ-06.013.md` — the kanban board that must render uncapped.
- `docs/requirements/business/REQ-06.009.md`, `REQ-06.012.md`, `REQ-06.014.md` — shared header, record layout, config-defined list views.
- `docs/requirements/business/REQ-05.007.md` — archived records never listed.
- `docs/requirements/non-functional/NFR-01.002.md`, `NFR-01.003.md`, `NFR-05.001.md`.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — templates, assets, CSP, no cookies, progressive enhancement.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — descriptor fields that feed the column plan.
- `docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md` — registry-derived navigation and list routes (constraint 5 amended here).
- `docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md`, `ADR-0026-mermaid-only-offline-diagram-rendering.md` — full-width diagram regions.
- `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md` — asset table used by the new script.
- Implementation: `src/browser/shell.rs` (new), `src/browser/list.rs` (new), `src/browser/render.rs`, `src/browser/routes.rs`, `src/browser/templates/`, `src/browser/app.css`, `src/browser/app.js`, `src/browser/assets.rs`.
