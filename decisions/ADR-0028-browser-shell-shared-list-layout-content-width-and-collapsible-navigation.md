---
id: ADR-0028
title: "Browser Shell: Shared List Layout, Content-Width Contract, and Collapsible Navigation"
created_date: '2026-11-15T00:00:00Z'
updated_date: '2026-11-20T20:09:06Z'
labels: []
references: []
---
# Browser Shell: Shared List Layout, Content-Width Contract, and Collapsible Navigation

## Status
accepted
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

   *Revision (2026-11-20, amended by ADR-039): `view.html` and `edit.html` are single descriptor-driven templates for every document type; the typed field sets are not kept, and `document_detail.html`/`document_edit.html` are deleted. The conflict page renders through `edit.html` and the document not-found page through `view.html`, so the template set stays five. The shell module and every other rule of this decision are unchanged.*

2. **One shared list layout with a server-built column plan.** `browser::list` builds, per list page:

   ```rust
   pub struct ListPlan { pub heading: String, pub count_note: String, pub columns: Vec<Column>, pub rows: Vec<Row>, pub empty: Option<EmptyState> }
   pub struct Column { pub key: &'static str, pub label: &'static str, pub class: &'static str }
   pub struct Row { pub id: String, pub title: String, pub href: String, pub cells: Vec<String>, pub labels: Vec<String>, pub references: Vec<String>, pub actions: Vec<(String, String)>, pub children: Vec<Row> }
   ```

   - The fixed column order for every family is: ID, Title, Status, family slot(s), Created, Updated, Actions. Only the *labels and presence* of the family slot(s) vary, and they come from the descriptor plus the presentation table (ADR-0020), never from a per-family template or a `match` on slug.
   - Family slots are declared data: task → `Milestone`, `Assignee`; milestone → `Release`; release → none; decision → `Type`; requirement/nfr → `Group`; audit → `Audit type`; rule → `Applicability`; ui-mockup → `Task`, `Specs`; generic document → none (ID, Title, Status, Created, Updated, Actions).
   - Slot keys are reserved-key safe: a declared slot key may not be `id`, `title`, `status`, `created`, `updated`, or `actions`, and may not repeat inside one family. Structural columns are fixed, so a slot can never duplicate or reorder one.
   - The Title cell carries the record's labels and references below the link, rendered by the shared list from escaped values. They are general front-matter metadata, not family data, so they are structural (never a slot) and identical for every family.
   - Grouped families (requirements/NFR) use the hierarchical row variant (`Row::children`): the group row plus its member rows in the same table body, not a second template.
   - Action placement is fixed: the last cell, in the order `View`, `Edit`, and any family action that already exists. Archive/delete/create controls stay at page level (unchanged).
   - Archived records remain excluded because indexing is enabled-only and active-only (ADR-0020 decision 7); the layout never sees them.

   *Revision (2026-11-15, architecture review at TASK-141 blocker resolution): the fixed order previously read `ID, Title, Status, family slot(s), Updated, Actions`, the slot table declared release → `Status`, and the row carried no title, labels, or references. The order now includes `Created`, release declares no slot, and the row carries the title plus labels/references in the Title cell, because REQ-06.004 is a must-have requirement whose acceptance criteria require the browser list to show the same front-matter values as the CLI, including `created_date`, `labels`, and `references`. Dropping them would have required a business-requirement change, not an architecture decision. Resolved question 3 is satisfied structurally by this order instead of by ad-hoc per-family slots. No other part of decision 2 changed.*

   *Revision (2026-11-20, amended by ADR-039): family slot labels and presence come from the descriptor's seed entry alone; the presentation table is no longer consulted for a document type, and a document field never becomes a column. Slot keys may not be any reserved field key of ADR-036 decision 23. Reference rendering in a row is decided by the descriptor's seed `reference_visibility` per type and role, not by the content model. The fixed column order, the structural columns, the Title-cell labels, the grouped row variant, and the action placement are unchanged.*

3. **The width decision is made on the server, per route.** `routes.rs` gains a route property:

   ```rust
   pub enum PageWidth { Capped, Full }
   ```

   `Capped` is the default for every route. `Full` is set only for the backlog kanban board route (when REQ-06.013 lands) and the UI-mockup preview route (when it exists). The shell renders `class="app app--full"` for `Full` and `class="app"` for `Capped`. Client JavaScript never changes the page width class, so no-JS behaviour and server tests agree.

   Inside a `Capped` page, a full-width-eligible region (rendered diagram figure, and later an embedded mockup) uses a `.bleed` block: `width: 100%`, with its own `overflow-x: auto` wrapper, so wide content scrolls inside the region and the page never gains a horizontal scrollbar. `Capped` pages keep `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px` exactly as today.

4. **Navigation collapse is a client-side, per-browser preference with no server state.**
   - Markup: one `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` in the shell header on **every** page, and `<nav id="site-nav">` for the existing sidebar.
   - State: `localStorage` key `papyrus.nav` with values `"collapsed"`/`"expanded"`. Collapsed state sets `data-nav="collapsed"` on `<html>`; CSS then removes the sidebar from the grid (`grid-template-columns: minmax(0, 1fr)`), hides it from the accessibility tree (`hidden` attribute set by script), and drops the `.main` left border and padding, so the reclaimed width is real layout width, not a negative margin. A retained zero-width first track must not be used: the sidebar is `display: none` while collapsed, so the remaining grid item auto-places into that zero-width track and the content column collapses to 0 px; the reclaim comes from removing the sidebar item, zeroing the gap, and removing the main column border and padding (210 + 30 + 30 = 270 px).
   - No flash of the wrong state: a new asset `/assets/nav-state.js` (≈15 lines) is loaded **synchronously in `<head>`**, reads the stored value, and sets `document.documentElement.dataset.nav` before first paint. `app.js` stays `defer` and only handles the toggle and the existing form guard.
   - Default: expanded. When no stored value exists, the state is expanded at every viewport width; the current "auto-collapse below 480 px" behaviour is removed, because it competes with a persisted user preference and REQ-06.015 makes the collapsed state the user's choice. `localStorage` is the only persistence mechanism (constraint 5): no cookie, no server state, no configuration key.
   - Degradation: with JavaScript disabled the button is not rendered (`<noscript>`-style server-side omission is not possible without JS detection, so the button carries `hidden` and a two-line asset removes `hidden`), the nav is expanded and fully usable, and any `localStorage` throw (private mode, quota, disabled storage) is caught and ignored, leaving in-memory state for the session.
   - Rejected alternatives: a cookie (ADR-0005 forbids cookies and sessions, and it would make the server stateful per client), server-side state in the project or user config (a write path, a concurrency surface, and per-user state in a project file), a query parameter on every link (breaks persistence across navigation and deep links), and a CSS-only checkbox (state cannot survive navigation).

   *Revision (2026-11-16): the collapsed grid previously read `grid-template-columns: 0 minmax(0, 1fr)`. That mechanism is defective and was proven so by measurement: with the sidebar `display: none`, `.main` auto-places into the zero-width first track and its rendered width becomes 0 px at 600, 720 and 1440 px. The collapsed state now uses a single track, `grid-template-columns: minmax(0, 1fr)`, which is what the shipped implementation does. No other part of decision 4 changed.*

5. **CSP is unchanged by this ADR.** The nav script is an external same-origin asset, so `script-src 'self'` covers it; the toggle uses class/attribute changes only, so `style-src 'self'` is untouched. Only ADR-0025 (`font-src`) and ADR-0026 (`style-src 'unsafe-inline'` for Mermaid's SVG `<style>`) relax the policy, and only for their stated reasons.

6. **No new dependency, template engine, or client framework.** Lists are rendered server-side into the same `{{name}}` placeholder templates; the two scripts are classic scripts; the CSS is the existing stylesheet plus new rules.

7. **UI-mockup previews are out of scope here; only the mockup full-width class is defined.** REQ-06.015 presupposes a mockup preview surface; none exists, and the stored review packages contain their own HTML/CSS/JS (ADR-0005 and the mockup specification currently forbid serving the package at all). This ADR fixes only the `PageWidth::Full` classification and the `.app--full` class that a mockup preview will use, and records the binding constraint: any future preview route must serve the package as a **sandboxed, opaque-origin document** (`sandbox` without `allow-same-origin`, its own CSP, `X-Frame-Options`/`frame-ancestors` adjusted only for that route), because serving an author-supplied HTML package same-origin would give it access to the browser write API. That route requires its own ADR before implementation and is not part of this decision.

8. **Accessibility and browser support are part of the contract.** The toggle is keyboard operable with a visible focus ring, `aria-expanded` and `aria-controls` stay in sync, the collapsed nav is removed with `hidden` rather than only visually hidden, and the layout is verified at the existing 320/480/720 px breakpoints and in Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001).

   The shared list is part of the same contract: every header cell keeps `scope="col"`, every row action link carries an accessible name that includes the record ID (`aria-label="View {id}"`, `aria-label="Edit {id}"`), because twenty identical "View" links are ambiguous to screen-reader and voice users, and the grouped child rows expose their parent through `data-child-of` plus a visual indent that is not colour-only.

9. **The migration to the shared layout is dispatched by declared data, and its reference family is the milestone list.**

   - The reference family for the first migration is `milestone` (`/?type=milestone`): it is a real list route with a populated and an empty state, it carries one family slot (`Release`), it is the browser side of the REQ-06.004 parity surface, and it is the only family whose list is both a `<ul>` row list and a parity target. The task family cannot be the reference: `GET /?type=task` is the REQ-06.013 kanban board, and the task family has no list route to migrate.
   - While both list paths exist, `list_page` chooses the shared builder or the legacy builder from a declared presentation field, `list_layout: ListLayout { Shared, Legacy }` (default `Legacy`). No `match` on a slug or handler is introduced for the migration. TASK-142 sets every family to `Shared` and deletes the field together with the legacy builders in the same change, so no transitional branch can survive.
   - The task family has no shared-list surface and gets none from this ADR. REQ-06.013 owns the backlog view; the task slots stay declared for a future task list, which requires its own decision.

### Amendment (2026-11-20) — ADR-039

- **Amended:** decision 1 (one descriptor-driven `view.html` and `edit.html`; no typed field sets for documents) and decision 2 (seed-only slots and labels; reference visibility from the seed per type and role).
- **Amended downstream constraint 10:** the prohibited slot-key set is the union of the structural keys (`id`, `title`, `status`, `created`, `updated`, `actions`) and the ADR-036 decision 23 reserved field keys (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`). Slots are declared only in the seed entry; a document field, a general metadata key, or a handler may not introduce a column. The structural column set itself is unchanged.
- **Not changed:** decisions 3 to 9, downstream constraints 1 to 9 and 11 to 15, the width contract, the navigation model, CSP, the archive-exclusion rule, and this record's accepted status.

### Amendment (2026-11-20) — descriptor model cross-check

- **Decision 1 — one generic edit renderer.** The clause that `view.html`/`edit.html` keep their typed field sets is superseded exactly as ADR-039 decision 4 records: there is one descriptor-driven edit form for every document model, built from the field map (ADR-039 decision 22), and no per-family or per-type edit template survives. Conflict reuses the same form; not-found reuses `view.html`.
- **Decision 2 — reserved field keys versus slots.** The prohibited slot-key set is the union of the structural keys (`id`, `title`, `status`, `created`, `updated`, `actions`) and the reserved field keys of ADR-036 decision 23 (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`), as ADR-039 decision 6 records. Slots are seed data. A document field never becomes a column, whether or not its key is reserved, because a document descriptor's slots come from its seed entry and the seed declares none (ADR-039 decision 25). The reserved-key rule bounds the naming of a seed slot for a bespoke family; it is not a licence to add one.
- **Not changed:** decisions 3-9, downstream constraints 1-9 and 11-15, the width contract, CSP, the navigation model, and this record's accepted status. This note introduces no rule beyond ADR-039 decisions 4, 6, and 25.
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

## Amendment (2026-11-15) — TASK-141 blocker resolution

Resolved by architecture review while TASK-141 was blocked on two contract questions and no owner was available. This section and the revision notes inside decisions 2 and 9 take effect immediately so implementation can start; explicit owner confirmation is requested, and if it is refused the amendment is withdrawn in favour of a superseding ADR.

- **OQ-1 — reference family: milestone list (confirmed against the refiner's recommendation).** `/?type=milestone` is the only family that is simultaneously a real list route, a populated *and* empty state, a family slot carrier (`Release`), a `<ul>` → `<table>` migration, and the browser side of the REQ-06.004 parity surface. The task family is not eligible: `/?type=task` is the REQ-06.013 board and no task list route exists. The milestone list is not grouped and has no status filter, so the grouped row variant and the filter path are proven by `ListPlan` unit tests in TASK-141 and first exercised end-to-end by migrating requirements/NFRs first in TASK-142.
- **OQ-2 — `created_date` stays, as a structural column: option (a).** The fixed order becomes `ID, Title, Status, family slot(s), Created, Updated, Actions`. The alternative (keep the order, relax parity to Updated-only on lists) was rejected because REQ-06.004 is a must-have requirement whose acceptance criteria require the browser *list* to show the same front-matter values as the CLI, including `created_date`, `labels`, and `references`; relaxing it is a business-requirement change, not an architecture one. `labels` and `references`, which today survive only in an ad-hoc metadata line, are carried in the Title cell so list parity does not regress when the list builders are deleted.
- **OQ-3 — release declares no family slot.** `Status` is the fixed Status column for every family; declaring it as a slot produced two status cells. Reserved-key validation (decision 2) prevents the class of error, not just this instance. No data is lost: the release list shows `planned`/`released` from the same `record.status` the slot would have read.
- **Task family has no shared-list surface.** REQ-06.016 governs list views; the task family's browser view is the REQ-06.013 board. TASK-142 therefore treats tasks as a board-regression check, not a list migration.
- **Mockup brief changes.** The pending shell/shared-list mockup must show the seven-column order above, the Title-cell tag line, and the row action cell with per-row accessible names. The mockup gate is unchanged: AC2/AC3 work still waits for the approved MOCK id.
- **Not changed by this amendment:** decisions 1, 3, 4, 5, 6, 7, 8; `PageWidth`; CSP; the navigation model; the archive-exclusion rule.

**Precedence while the feature specification is stale.** `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` still states the pre-amendment column order and leaves OQ-1..OQ-3 open (this review had no write access to `docs/specifications/`). This ADR governs until that specification is updated; the required specification edits are listed in TASK-141.

**Ticket text to apply (TASK-141).** The reference family is `milestone` (not `task`); AC2/DoD#4 use the header sequence `ID, Title, Status, family slot(s), Created, Updated, Actions`; the parity assertion is `created` **and** `updated` in the row cells plus `labels`/`references` in the title cell; the transitional dispatch is the `list_layout` presentation field. TASK-142 migrates requirements/NFRs first, then ui-mockup, then the flat document families, then config-defined types, and treats the task family as a board-regression check only.

## Amendment (2026-11-16) — collapsed grid track correction

The collapsed-layout mechanism in decision 4 was wrong and is corrected; the status stays `Accepted` because no shipped behaviour is changed and the defective detail was never released.

- **Correction — the collapsed grid has one track, not two.** `grid-template-columns: 0 minmax(0, 1fr)` is replaced by `grid-template-columns: minmax(0, 1fr)`. Measurement disproved the two-track form: with the sidebar `display: none`, `.main` auto-places into the retained zero-width first track and renders 0 px wide at 600, 720 and 1440 px. A zero-width first track is therefore not a valid way to reclaim the sidebar's width.
- **Width reclaim is unchanged as a contract.** The 270 px comes from removing the sidebar item (210 px), zeroing the gap, and removing the `.main` left border and padding (30 + 30 px); the reclaim is real layout width, not a negative margin.
- **Not changed by this amendment:** decisions 1, 2, 3, 5, 6, 7, 8, 9; `PageWidth`; CSP; the navigation model; the archive-exclusion rule; the status.
- **Precedence.** The shipped implementation already uses the single-track form; this amendment brings the ADR in line with it. `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` is corrected separately by the tech writer.

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
10. The structural column set of a shared list is fixed at `id`, `title`, `status`, family slot(s), `created`, `updated`, `actions`. No family adds, removes, or reorders a structural column, and no slot key may be a reserved key (`id`, `title`, `status`, `created`, `updated`, `actions`) or repeat.
11. Labels and references are rendered for every family in the Title cell by the shared list from escaped values. They are never a slot and never a per-family template concern.
12. While two list paths exist, the choice between them is the declared presentation `list_layout` field. A `match` on a slug or handler for a list, even transitional, violates constraint 2.
13. The task family has no shared-list surface while REQ-06.013 owns the backlog view. Adding a task list requires its own decision; the declared task slots do not imply one.
14. Every list row action link carries a record-specific accessible name; a bare repeated `View`/`Edit` label is not conformant.
15. Seven columns at 320 px must not introduce page-level horizontal scrolling: the shared table scrolls inside its own container, on the same principle as constraint 7. The exact treatment (scroll container or stacked rows) is the mockup's decision, not a per-family one.

## References

- `docs/requirements/business/REQ-06.016.md` — one shared list layout for every artifact family.
- `docs/requirements/business/REQ-06.015.md` — content width and navigation visibility.
- `docs/requirements/business/REQ-06.013.md` — the kanban board that must render uncapped.
- `docs/requirements/business/REQ-06.009.md`, `REQ-06.012.md`, `REQ-06.014.md` — shared header, record layout, config-defined list views.
- `docs/requirements/business/REQ-05.007.md` — archived records never listed.
- `docs/requirements/business/REQ-06.004.md` — CLI/browser metadata parity; source of the `Created` column and the Title-cell labels/references rule.
- `docs/requirements/non-functional/NFR-01.002.md`, `NFR-01.003.md`, `NFR-05.001.md`.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — templates, assets, CSP, no cookies, progressive enhancement.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — descriptor fields that feed the column plan.
- `docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md` — registry-derived navigation and list routes (constraint 5 amended here).
- `docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md`, `ADR-0026-mermaid-only-offline-diagram-rendering.md` — full-width diagram regions.
- `docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md` — asset table used by the new script.
- Implementation: `src/browser/shell.rs` (new), `src/browser/list.rs` (new), `src/browser/render.rs`, `src/browser/routes.rs`, `src/browser/templates/`, `src/browser/app.css`, `src/browser/app.js`, `src/browser/assets.rs`.
