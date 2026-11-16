---
id: TASK-148
title: >-
  Revise packaging and browser specifications for bundled assets and the shell
  contract
status: Implemented
assignee:
  - '@business-analyst'
created_date: '2026-11-15T16:14:00Z'
updated_date: '2026-11-16T09:37:00Z'
labels:
  - docs
  - specifications
milestone: m-21
dependencies:
  - TASK-142
  - TASK-144
  - TASK-145
  - TASK-146
references:
  - docs/requirements/business/REQ-06.016.md
  - docs/requirements/business/REQ-06.017.md
  - docs/requirements/business/REQ-06.015.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - >-
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - docs/specifications
  - docs/architecture/adr/ADR-0026-mermaid-only-offline-diagram-rendering.md
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - docs/specifications/browser-math-and-diagram-rendering.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/rust-cli-and-debian-package-baseline.md
  - docs/packaging.md
priority: medium
ordinal: 41600
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Gate — cleared 2026-11-16.** ADR-0026 is `Accepted` (owner scope decision 2026-11-15; mechanics completed 2026-11-16 under delegated architecture authority and reported to the owner through the orchestrator). The diagram clauses are no longer blocked: the `mermaid`-only fence set, the `plantuml`/`puml` source block with its note, the 11.17.2 pin, the 100,000-byte and 1,000-line caps, the client failure contract, and the scoped diagram CSP (`style-src 'self' 'unsafe-inline'`) may be written. No page may still describe ADR-0026 as `Proposed` or the diagram CSP as undecided.

**Implementation state the wording must respect.** Math is landed: KaTeX 0.16.11 is vendored and served, the strict `CSP` in `src/browser/http.rs` already contains `font-src 'self'`, and the shell, width classes, and shared list are landed by TASK-141/TASK-142/TASK-144. Mermaid 11.17.2 is **not** embedded yet: `src/browser/assets.rs` has no Mermaid entry, `src/browser/http.rs` defines only the strict string, and `src/browser/vendor/` has no Mermaid directory. TASK-146 implements it; TASK-150 verifies it. The specifications state the accepted diagram contract and mark the bundle, the diagram CSP, and the Mermaid notice as landing with TASK-146 — they must not claim behaviour that has not run.

## Why

ADR-0025 (amended 2026-11-16), ADR-0027, and ADR-0028 falsified wording that many pages still carry: "no web font", "no `url(`", an asset-free package, per-family list markup, a CSP without `font-src`, and no width/navigation rules. ADR-0025 and ADR-0027 assign that revision to the business analyst / tech writer.

## What

Publish the browser shell UI specification and correct every affected packaging and browser specification to the landed contract, with the diagram clauses written from the accepted ADR-0026.

## Settled by this refinement — do not re-litigate

1. **New page.** `docs/specifications/browser-shell-ui.md`, titled `Feature Specification — Browser Shell UI: Shared List Layout, Width Classes, and Collapsible Navigation`, with the fixed outline in the implementation plan. It is normative for behaviour, cites `docs/specifications/ui/browser-shell-and-shared-list-ui.md` and MOCK-0016/MOCK-0018 for visuals, and does not restate visual measurements. `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` stays the delivery record and contract detail; its only change is a header pointer (no clause edits).
2. **CSP home.** The two literal strings are stated normatively once, in `browser-requirements.md` §9.7, with a four-row response-class table. `browser-math-and-diagram-rendering.md` §7.1/§17.1 already carries the same two literals and must stay byte-identical. No third copy: every other page names the class and cites §9.7. Exact literals are in the plan.
3. **Measured figures.** Binary 4,730,296 B (4.51 MiB; 4.73 MB decimal) measured on `target/release/papyrus` on 2026-11-16; served KaTeX assets 558,541 B (CSS 23,335 + JS 275,414 + 20 WOFF2 259,792, from `src/browser/vendor/MANIFEST.md`); Mermaid 3,572,661 B per ADR-0026 decision 4 (not on disk); projected 8,302,957 B = 7.92 MiB against the 12 MiB (12,582,912 B) ceiling, headroom 4,279,955 B = 4.08 MiB. **Correction:** the refinement brief's "KaTeX assets 580,487 B" is wrong by 21,946 B; use the measured 558,541 B and record the correction in the plan's evidence section.
4. **Indexes.** No generated index exists for `docs/specifications/`. The only generated index in scope is `docs/architecture/adr/index.html` (HTML shell plus embedded JSON `data` payload). Because this task edits no ADR, regeneration must be a byte-identical no-op; the requirement, business-rule, and glossary indexes likewise. Index work is verify-and-diff; do not invent a specification index.
5. **Historical evidence stays.** The dated `Smoke test results (TASK-…)` and `Verification results (TASK-…)` sections of `docs/packaging.md` are evidence, not contract, and are not rewritten. The live procedure (`## Packaging`, §3.3, §3.4, §3.6, `## Runtime dependency license audit`) is.
6. **No requirement or ADR edits.** Conflicts are reported in the ticket, not resolved in prose.

## Scope — 26 page targets plus 12 UI descriptions

- **Create (1):** `docs/specifications/browser-shell-ui.md`.
- **Browser specifications (14):** `browser-requirements.md`, `browser-tasks-milestones-and-releases.md`, `browser-managed-artifacts-and-lifecycles.md`, `browser-config-defined-document-types.md`, `browser-audit-records.md`, `browser-business-rules.md`, `browser-decision-records.md`, `browser-glossary-terms.md`, `browser-technical-specifications.md`, `browser-ui-specifications.md`, `browser-ui-mockups.md`, `browser-use-cases-test-cases-and-test-reports.md`, `browser-complete-access.md`, `browser-agent-skills-and-workflows.md`.
- **Packaging (10):** `docs/packaging.md` and the eight `docs/specifications/*-milestone-packaging-and-verification.md` pages that carry the "No external references … web font" item (backlog-discovery, cli, complete-content-management, decision-records, glossary-terms, search-browser, technical-specifications, use-cases-and-test-documentation).
- **Baseline (1):** `docs/specifications/rust-cli-and-debian-package-baseline.md` (TASK-143 section gains the accepted Mermaid row and notice).
- **Pointer only (1):** `docs/specifications/browser-shell-shared-list-layout-and-page-width.md`.
- **UI descriptions (12):** only their list-row and navigation statements, per plan step 7. Approved visuals, MOCK ids, and measurements are untouched.

## Out of scope

Requirement text (`docs/requirements/`), ADR text (`docs/architecture/adr/`), implementation (TASK-141 to TASK-147, TASK-149, TASK-150), release notes, the man page, `README.md` (it makes no asset or math claim; ADR-0025's downstream note naming "README math wording" has no target), new mockups or MOCK ids, and any new product decision.

## Conflict register — report, do not resolve in prose

- **C1 — requirement vs code.** REQ-06.016 requires every list view, naming tasks among the families, to use the shared layout. The landed code has no task list: `/?type=task` is the REQ-06.013 board, as ADR-0028 decision 9 and downstream constraint 13 record. The specifications state the board exception; the requirement-vs-code gap stays open.
- **C2 — requirement vs code.** REQ-06.015 says a collapse control "is available on every page". With JavaScript disabled the landed shell emits the button carrying `hidden` and only `nav-state.js` reveals it, so no control is offered without JS. ADR-0028 decision 4 chose this deliberately; the specification records it as an accepted deviation and the literal requirement remains unmet.
- **C3 — specification vs requirement.** `browser-requirements.md` L25 and L348 and nine per-family pages say "Edge 100+ / Safari 100+", while NFR-05.001 (updated 2026-11-15) requires Edge 130+ and Safari 18+; OQ-9 even claims NFR-05.001 names "Safari version 100". The requirement wins: correct the specifications to NFR-05.001 and remove the stale "100" wording.
- **C4 — stale ADR text, out of scope.** ADR-0020 decision 5 still names `document_list.html`, deleted by TASK-142 under ADR-0028 constraint 1. Not edited here.
- **C5 — specification vs landed CSP.** `browser-requirements.md` §6.7 says the policy "needs no `'unsafe-inline'`", falsified by the accepted diagram class. Correct to: no server-authored inline `style` attribute; the only `'unsafe-inline'` is `style-src` on a diagram response.
- **C6 — landed accessibility gap, out of prose scope.** The shell width contract gives the `/config` table, the board columns, and the `/skills` command `<pre>` `role="region"`, a label, `tabindex="0"`, and a visible focus ring. The diagram figure (`figure.diagram.bleed` with its `.diagram-scroll` child; `src/browser/app.js`, `src/browser/app.css`) scrolls horizontally but adds none of those hooks, so a keyboard reader cannot scroll a wide diagram. Owner: TASK-146 (implementation) and TASK-150 (release verification). Not changed by TASK-148, which edits no template or client code.
- **C7 — requirement vs delivery.** REQ-06.015 requires a dedicated full-width layout class that lifts the 1040 px cap for UI mockup content. ADR-0028 decision 7 keeps the mockup preview route out of scope, so no surface exercises such a class; the landed `.app--full` is used only by the backlog board. The literal criterion remains unmet; closing it needs a requirement change or a new ADR, not prose.

## Assumptions

- **A1.** The landed code is authoritative for behaviour; the ADRs are authoritative for accepted-but-unimplemented clauses (Mermaid).
- **A2.** Diagram clauses describe the accepted contract and name their implementation status (TASK-146 implementation, TASK-150 verification).
- **A3.** Verification wording may name checks that run under TASK-150, provided the page says so.
- **A4.** Line numbers in the plan are anchors captured on 2026-11-16 at `ace3227`; match the quoted text, not the number.
- **A5.** UI description documents are layout descriptions: reconcile only their list-row and navigation statements; approved visuals, MOCK ids, and measurements stay.

**Dependencies:** TASK-142, TASK-144, TASK-145 (completed); TASK-146 (Refined) owns the Mermaid implementation this page describes; TASK-150 verifies it.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 docs/specifications/browser-shell-ui.md exists with the pinned heading structure and states the shell regions, the shared list contract (seven-column order, slot table, Title-cell labels/references, action placement, grouped rows), Capped/Full/.bleed, the navigation toggle and papyrus.nav persistence, the default-expanded state, no-JS behaviour, accessibility, and the NFR-05.001 browser floor.
- [x] #2 Every specification that asserted "no web font", "no url(" or an asset-free package now describes bundled same-origin fonts, the static asset table, the MANIFEST.md digests, and the single copyright file with one section per bundled asset family.
- [x] #3 The two CSP strings are recorded per response class in browser-requirements.md section 9.7, byte-identical to browser-math-and-diagram-rendering.md section 17.1 and src/browser/http.rs; the diagram string is scoped to has_diagram responses, /assets/ responses carry none, and no page holds a third copy.
- [x] #4 Every browser page that described per-family list markup points at the shared list contract, and the superseded wording is removed (five-column status/ID/title/created/updated order, first-paragraph previews, row-meta lines, the "Other requirements" subsection, document_list.html).
- [x] #5 The eight packaging-and-verification pages and docs/packaging.md check asset digests, notices, offline rendering, and the 12 MiB budget with the measured numbers, and no page still asserts "no web font" or "no url(".
- [x] #6 Affected generated indexes under docs/ are regenerated or verified byte-identical, and no file under docs/requirements/ or docs/architecture/adr/ is modified.
- [x] #7 The diagram clauses state the accepted ADR-0026 contract (mermaid-only fence set, plantuml/puml note, 11.17.2 pin, caps, failure contract, diagram CSP) and no page describes ADR-0026 as proposed or the diagram CSP as undecided.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 The new page is published with the pinned headings and cites its requirements, ADRs, MOCK ids, and the delivery record.
- [x] #2 All 26 page targets and the 12 UI descriptions are updated as listed in the implementation plan; no step is skipped or merged.
- [x] #3 The verification sweep in plan step 10 returns the expected results, with only the documented exceptions.
- [x] #4 The two CSP literals are byte-identical in browser-requirements.md section 9.7 and browser-math-and-diagram-rendering.md section 17.1 and equal src/browser/http.rs.
- [ ] #5 git diff --name-only docs/requirements docs/architecture/adr is empty, and docs/architecture/adr/index.html is byte-identical after regeneration.
- [x] #6 The measured figures are recorded with their source: binary 4,730,296 B, KaTeX 558,541 B (the brief's 580,487 B corrected), Mermaid 3,572,661 B, projected 8,302,957 B / 7.92 MiB, headroom 4,279,955 B.
- [x] #7 Conflicts C1-C5 are left unresolved in prose and reported in the final summary; any newly found conflict is added to the register rather than fixed.
- [x] #8 Diagram clauses name their implementation status (TASK-146) and verification owner (TASK-150); no page claims the diagram CSP is already emitted.
- [x] #9 The dated result sections of docs/packaging.md and the clauses of browser-math-and-diagram-rendering.md are unchanged.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
# Implementation plan (TASK-148)

## 0. Preconditions and evidence (record before editing)

1. `git rev-parse HEAD` (expected `ace3227` or later on m-21) and `git status` clean.
2. `stat -c %s target/release/papyrus` → **4,730,296 B** (4.51 MiB; 4.73 MB decimal). Re-run `cargo build --release` first if the file is older than the TASK-145 merge.
3. `src/browser/vendor/MANIFEST.md` served-asset totals: KaTeX CSS 23,335 + JS 275,414 + 20 WOFF2 259,792 = **558,541 B**. The brief's "580,487 B" is a 21,946 B over-count; the measured figure is authoritative.
4. Confirm Mermaid is absent: no `src/browser/vendor/mermaid/`, no `/assets/mermaid/` row in `src/browser/assets.rs`, and `src/browser/http.rs` line 16 defines only `CSP`. Mermaid's **3,572,661 B** and SHA-256 `581ed7d7…35390eb8` come from ADR-0026 decision 4.
5. Budget arithmetic to quote everywhere: 4,730,296 + 3,572,661 = **8,302,957 B = 7.92 MiB**; ceiling 12 MiB = 12,582,912 B; headroom **4,279,955 B = 4.08 MiB**.
6. `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` and `browser-math-and-diagram-rendering.md` are contract sources, not targets (step 8 is a pointer only).

## 1. Canonical text blocks (reuse; do not paraphrase the literals)

**B1 — assets and fonts (replaces every "no web font"/"no `url(`" claim).**
Bundled, same-origin fonts replace the earlier system-font-only rule (ADR-0027 decision 7). The first-party stylesheet `/assets/app.css` still uses the system font stack and contains no `url(`. Two stylesheets ship: `/assets/app.css` and the byte-identical vendored `/assets/katex/katex.min.css`, whose 60 relative `url(fonts/…)` values resolve to `/assets/katex/fonts/*.woff2` entries of the static asset table. Only the WOFF2 form is served. No served CSS contains `@import`; no reference is remote; there is no `<img>` element, no `srcset`, no icon font, and no CDN reference.

**B2 — asset table, digests, notices.**
Every browser byte comes from the static table `src/browser/assets.rs` (`Asset`, `ASSETS`, `lookup`); no request path is joined to a filesystem path. `/assets/` paths match exactly, `GET`/`HEAD` only (`405` otherwise), a miss is `404`, `HEAD` sends `Content-Length` with no body, and asset responses carry no CSP. Every entry has a row in `src/browser/vendor/MANIFEST.md` (vendored: path, package, version, licence, upstream URL, bytes, SHA-256; first-party: path, bytes, SHA-256), and a unit test fails the build on a missing row, an extra row, or a byte/digest mismatch. The package still contains exactly `/usr/bin/papyrus` and `/usr/share/doc/papyrus/copyright`; that single copyright file carries the Papyrus GPL-3.0-or-later block plus one section per bundled asset family — KaTeX code MIT, KaTeX fonts OFL-1.1, and Mermaid 11.17.2 MIT once TASK-146 lands.

**B3 — offline verification.**
Served HTML, CSS, and JS contain no `http://`, `https://`, protocol-relative `//`, remote `@import`, remote `url(...)`, `<img>`, or CDN reference. Every `url(...)` in every served stylesheet resolves to an entry of the asset table, and the only such values are KaTeX's 60 `url(fonts/…)` entries. A page with math renders offline in Firefox 100+, Edge 130+, and Safari 18+ with its WOFF2 fonts served from `/assets/katex/fonts/…` under `Cache-Control: no-store`. A page with a Mermaid figure does the same once TASK-146 lands; TASK-150 verifies it.

**B4 — CSP per response class (no literals).**
Every `text/html` response carries exactly one of the two CSP strings normatively fixed in `docs/specifications/browser-requirements.md` §9.7: the strict string on every HTML response whose pre-pass did not set `has_diagram`, and the diagram string — identical except `style-src 'self' 'unsafe-inline'` — only when the rendering pre-pass set `has_diagram`. `/assets/` responses and non-HTML responses carry no CSP. `script-src 'self'` never gains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash. The diagram string lands with TASK-146 and is verified by TASK-150.

**B5 — navigation.**
One `<button class="nav-toggle" type="button" aria-expanded="true" aria-controls="site-nav">` sits in the header on every page, left of the brand, with the stable accessible name `Navigation`; server output carries `hidden` and the synchronous `/assets/nav-state.js` reveals it. It collapses and expands `<nav id="site-nav">`, persists the choice in `localStorage` under `papyrus.nav` (`"collapsed"`/`"expanded"`), and defaults to expanded when no value is stored, at every viewport width. With JavaScript disabled the navigation is expanded, visible, and usable, and no toggle is offered. No cookie, server state, or configuration key stores the state.

**B6 — supported browsers.**
Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001), verified at 320, 480, and 720 CSS px. `:has()`, container queries, `color-mix()`, `@layer`, subgrid, `dialog`, popover, `inert`, optional chaining, top-level await, and private class fields are forbidden.

**B7 — the two CSP literals (byte-exact; single normative home `browser-requirements.md` §9.7).**

Strict (`http::CSP`):

```
default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

Diagram (`http::CSP_DIAGRAM`, accepted under ADR-0026 decision 6):

```
default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

| Response | String |
|---|---|
| `text/html` whose pre-pass did not set `has_diagram`: lists, board, record/document views without a figure, edit, inventory, lifecycle, configuration, skills, and every 400/404/405/415/422/500 page | strict |
| `text/html` record or document view whose pre-pass set `has_diagram` | diagram |
| Every `/assets/*` response, including the Mermaid bundle and the WOFF2 fonts | none |
| JSON, text, and every non-HTML response | none |

Status note to keep beside the table: the strict string is live; the diagram string is accepted under ADR-0026 and lands with TASK-146 (verified by TASK-150).

**B8 — measured figures.**
Binary 4,730,296 B (4.51 MiB); served KaTeX assets 558,541 B; Mermaid 3,572,661 B (pinned, TASK-146); projected binary 8,302,957 B = 7.92 MiB; ceiling 12 MiB = 12,582,912 B; headroom 4,279,955 B = 4.08 MiB.

**B9 — asset verification block for packaging pages.**
(a) *Digests:* `cargo test` includes the manifest test, which asserts the manifest row set equals `ASSETS[].path` with no duplicate or missing row and recomputes SHA-256 and byte length for every entry. (b) *Notices:* `dpkg-deb --fsys-tarfile papyrus_<version>_amd64.deb | tar -xO ./usr/share/doc/papyrus/copyright` shows the GPL-3.0-or-later block plus one section per bundled family (KaTeX MIT, KaTeX fonts OFL-1.1, Mermaid MIT once TASK-146 lands), and `dpkg-deb -c` still lists exactly the binary and the copyright file. (c) *Offline rendering:* B3. (d) *Budget:* B8, asserting `stat -c %s` on the built binary is ≤ 12,582,912 B.

## 2. Ordered file-level changes

### Step 1 — create `docs/specifications/browser-shell-ui.md`

Use exactly this heading structure (order and numbering are fixed):

```
# Feature Specification — Browser Shell UI: Shared List Layout, Width Classes, and Collapsible Navigation
(header block: Tickets, Milestone, Status, Requirements, ADR, UI, Contract detail)
## 1. Purpose and scope
### 1.1 In scope
### 1.2 Non-goals
## 2. Shell regions
### 2.1 Regions
### 2.2 Invariants
## 3. Shared list contract
### 3.1 Column plan
### 3.2 Family slots
### 3.3 Title cell: labels and references
### 3.4 Action cell
### 3.5 Grouped rows
### 3.6 Empty state and count note
### 3.7 Stable markup hooks
## 4. Content-width classes
### 4.1 `Capped`
### 4.2 `Full`
### 4.3 `.bleed` inside a capped page
### 4.4 Width invariants
## 5. Collapsible navigation
### 5.1 Toggle
### 5.2 Stored preference: `papyrus.nav`
### 5.3 Default state
### 5.4 Without JavaScript
### 5.5 Script order and assets
## 6. Accessibility
## 7. Supported browsers
## 8. Response headers and assets
## 9. Traceability
```

Required content per section:

- **Header block.** Tickets `TASK-148 (publisher; contract fixed by TASK-141, TASK-142, TASK-144)`; Milestone m-21; Status `Normative, published 2026-11-16 from the landed contract`; Requirements REQ-06.016, REQ-06.015, REQ-06.004, REQ-06.013, REQ-05.007; ADR-0028 (Accepted, amended 2026-11-15 and 2026-11-16) and ADR-0005 as amended by ADR-0027; UI MOCK-0016 and MOCK-0018 (registry `draft`) and `docs/specifications/ui/browser-shell-and-shared-list-ui.md`; Contract detail `docs/specifications/browser-shell-shared-list-layout-and-page-width.md`.
- **§1.1** the shell, the shared list, the two page-width classes with `.bleed`, the navigation toggle and its persistence, the accessibility contract, and the browser floor. **§1.2** detail/edit field sets, the board's card content (REQ-06.013), mockup preview route (ADR-0028 decision 7), math/diagram rendering internals (cite `browser-math-and-diagram-rendering.md`), visual measurements (owned by the UI description), and any template engine or client framework.
- **§2.1** one table of regions: header `site-header` (toggle, `P` mark, `Papyrus`, project name, static top-meta); `<nav id="site-nav" aria-label="Document types">` inside `<aside class="sidebar">` (grouped entries, `aria-current="page"`); `<main class="main">`; footer `site-footer`. Each region exactly once per response.
- **§2.2** exactly one `<div class="app…">` wrapper per response; body fragments only (no chrome inside `main`); one `h1` per page; the shell owns the toggle and the width class; the shell never renders list actions, filters, or headings other than page chrome.
- **§3.1** fixed order `ID, Title, Status, family slot(s), Created, Updated, Actions`; `Row.cells.len() == columns.len() - 3`; reserved keys `id`, `title`, `status`, `created`, `updated`, `actions` (never declared as a slot, never repeated, never reordered); a family with two slots has eight header cells, no-slot families six.
- **§3.2** slot table: task → `Milestone`, `Assignee`; milestone → `Release`; release → none; decision → `Type`, `Superseded by`; requirement/NFR → `Group`; audit record → `Audit type`; business rule → `Applicability`; UI mockup → `Task`, `Specs`; generic document → none. Slots come from declared presentation data, never from a `match` on a slug or a per-family template. Task family has no list surface (board, REQ-06.013, per C1).
- **§3.3** labels and references render inside the Title cell below the link, from escaped values, for every family; they are never a slot and never reorder a column.
- **§3.4** actions are the last cell in the order `View`, `Edit`, then any existing family action; `View` → `/doc/{id}`, `Edit` → `/doc/{id}/edit`; each link carries a record-specific accessible name (`aria-label="View {id}"`, `aria-label="Edit {id}"`).
- **§3.5** grouped families (requirements/NFRs) use the hierarchical row variant: group row plus member rows in the same `<tbody>`, child rows carrying `data-child-of="{group id}"`, group chosen then members ascending by ID; an orphan member renders as a top-level row and is never dropped, duplicated, or attached to a synthetic group; the count note is `N group(s) · M requirement(s)` computed from the emitted rows.
- **§3.6** one shared empty state (`No {noun} in this project.` plus `Records are created from the command line; this browser does not create records.`), one muted `(none)` for unset status/date/slot, and the count note for zero records.
- **§3.7** stable hooks `data-list-layout="shared"`, `data-column="{key}"`, `data-row-id`, `data-child-of`; the table scrolls inside its own container at 320 px and never creates page-level horizontal scrolling; exact classes and spacing are the UI description's.
- **§4.1** `Capped` is the default for every route and renders `class="app"` with `max-width: 1040px; margin: 0 auto; padding: 0 24px 56px`.
- **§4.2** `Full` renders `class="app app--full"` with `max-width: none` and is set only for the backlog board (`PageRoute::Board`) while REQ-06.013 owns that view; any other uncapped route needs an ADR-0028 amendment; JavaScript never reads or writes the width class.
- **§4.3** `.bleed` is `width: 100%` plus its own `overflow-x: auto`, carries `role="region"`, an `aria-label`, `tabindex="0"`, and a visible `:focus-visible` ring when it scrolls; no `100vw`, no negative margins, no fixed-position breakout; used by rendered diagram figures (ADR-0026 decision 10) and later by mockup previews.
- **§4.4** the route decides the width on the server, so no-JS layout equals scripted layout; `.app--full` lifts only the cap and keeps centring and padding; the 481–720 px section heading wraps beneath instead of widening the page.
- **§5.1** B5's markup facts, including that the toggle is a native button, keyboard-operable (Enter/Space), uses the stylesheet focus ring, keeps a stable name, and any chevron is decorative (`aria-hidden`, no script writes button content).
- **§5.2** key `papyrus.nav`, values `"collapsed"`/`"expanded"`; only the exact value `"collapsed"` collapses; a missing key, another value, or a storage exception means expanded and the session continues in memory; `/assets/nav-state.js` sets `data-nav="collapsed"` on `<html>` before first paint and `app.js` keeps `aria-expanded` and `hidden` in sync; `data-nav="expanded"` is never written; no cookie, no server state, no configuration key.
- **§5.3** expanded is the default at every viewport width; the former sub-480 px auto-collapse is gone.
- **§5.4** without JavaScript the page renders expanded, the nav is visible and usable, the toggle stays `hidden`, and no width or nav state depends on script.
- **§5.5** `/assets/app.css` in `<head>`, then synchronous `/assets/nav-state.js`, then `defer` `/assets/app.js`; math pages additionally link the KaTeX stylesheet and script, diagram pages the Mermaid script before `app.js` (ADR-0026 decision 4); all are classic same-origin scripts under `script-src 'self'`.
- **§6** label/`scope="col"` rules for the shared list; `aria-expanded`/`aria-controls` sync; collapsed nav removed with `hidden`, not merely visually hidden; grouped child rows expose the parent through `data-child-of` plus a non-colour visual indent; focus order follows document order; no keyboard trap; 320/480/720 px states.
- **§7** B6 verbatim, plus the verification statement (TASK-150 owns the release evidence).
- **§8** B4 verbatim, plus B2's one-sentence asset-router summary, citing §9.7 for the literals and `browser-math-and-diagram-rendering.md` §16–§17 for the rendering assets.
- **§9** traceability table: REQ-06.016, REQ-06.015, REQ-06.004, REQ-06.013, REQ-05.007; NFR-05.001, NFR-01.003; ADR-0028, ADR-0027, ADR-0025, ADR-0026, ADR-0005; MOCK-0016, MOCK-0018; TASK-141, TASK-142, TASK-144, TASK-146, TASK-148; implementation files `src/browser/shell.rs`, `list.rs`, `routes.rs`, `render.rs`, `templates/shell.html`, `app.css`, `app.js`, `nav-state.js`, `assets.rs`, `http.rs`.

No clause may be copied from the delivery record verbatim beyond the facts above; cite that page for implementation detail. Link the new page from `browser-shell-shared-list-layout-and-page-width.md` (step 8) and from `browser-requirements.md` §4.6, §9.7, §10 (step 2).

### Step 2 — `docs/specifications/browser-requirements.md` (19 edits)

1. L22 scope bullet: replace "no CDN, no external fonts" with the sentence "embedded assets are served from one static table — first-party files, vendored KaTeX code and its 20 WOFF2 fonts, and (from TASK-146) the vendored Mermaid bundle are same-origin; there is no CDN, no remote font, no network fetch, and no new dependency."
2. L25: "Firefox 100+, Edge 100+, Safari 100+" → "Firefox 100+, Edge 130+, Safari 18+ (NFR-05.001)" (C3).
3. §4.6 L128: after the shell sentence, add "The header carries the navigation toggle that collapses and expands the sidebar; the expanded state is the default and the choice persists in `localStorage` under `papyrus.nav` (see `browser-shell-ui.md` §5)."
4. §4.9 L131: replace the per-group row description (group link, member count, nested member links) with "The list renders through the shared list contract in `browser-shell-ui.md` §3: group rows and member rows in one table, the Group slot carrying the stored reference, and the shared count note. The legacy nested-link rows and the per-group member count are not carried over."
5. §4.10 L132: delete the "Other requirements" subsection rule and replace with the orphan-member rule (`browser-shell-ui.md` §3.5): an unresolved or missing group reference renders the member as a top-level row in ID order with the stored value shown in the Group slot (`REQ-99 (not found)` or `(none)`), never a trailing catch-all section.
6. §4.11 L135: replace "No requirements in this group" with the shared empty-state rule (`browser-shell-ui.md` §3.6); a group with no members still renders its group row.
7. §6.1 L255: replace the template list and the two-asset sentence with: `shell.html`, `list.html`, `view.html`, `edit.html`, `error.html`, plus the `document_detail.html`/`document_edit.html` body fragments; all embedded with `include_str!` and served through the static asset table (B2), which also serves `/assets/nav-state.js`, the KaTeX stylesheet/script, and the built-in WOFF2 fonts.
8. §6.5 L259: replace with B1 plus "the page-level font stack stays the system stack; KaTeX fonts apply only inside rendered math."
9. §6.6 L260: replace the two-item script description with: `nav-state.js` (synchronous, applies the stored navigation state and reveals the toggle) and `app.js` (`defer`: toggle behaviour, dirty-form guard, tag editing, board status change, math and diagram render passes). Cite `browser-shell-ui.md` §5.5 and `browser-math-and-diagram-rendering.md` §6/§16.
10. §6.7 L261: replace with: no server-rendered page emits an inline `<script>`, an inline event-handler attribute, or an inline `style` attribute; KaTeX's own `style` attributes are dropped under `style-src 'self'` and that cosmetic deviation is accepted (ADR-0025 decision 6 revision); the only `'unsafe-inline'` in the policy is `style-src` on a diagram response (C5, B7).
11. §6.9 L263: keep, and add "every asset URL is a root-relative `/assets/` path from the static table."
12. §9.7 L334: replace the single CSP sentence with B7 verbatim — both fenced literals, the four-row class table, the `X-Frame-Options`/`nosniff`/`Referrer-Policy`/`no-store` sentence unchanged, and the status note that the diagram string is accepted under ADR-0026 and lands with TASK-146. This is the sole normative copy.
13. §9.10 L341: keep "no remote font"; add "bundled fonts are same-origin table entries (B1)."
14. §10.1 L348: replace the browser versions with B6's first sentence; add "the system font stack plus the bundled KaTeX WOFF2 fonts" to the feature list.
15. §10.3 L350: replace "the single script is a classic script" with the two-script loading order (B5/§5.5) and the conditional math/diagram bundles; keep "no module loading, no `fetch`, no dynamic import, no framework."
16. §10.4 L353–354: replace the narrow-width toggle bullets with B5 plus: on record view/edit pages the type entry keeps `aria-current="page"`; the collapsed navigation is removed with `hidden`; the toggle is present on every page at every width.
17. T-24 L514: keep "no external URL"; append "and every `url(` in a served stylesheet resolves to an asset-table entry (KaTeX's 60 font paths)."
18. T-27 L517: replace with "Assets are embedded and served from one exact-match table: representative `/assets/` GET/HEAD returns 200 with the documented MIME, `Content-Length`, `Cache-Control: no-store` and no CSP; a miss is 404 and a non-GET/HEAD is 405; the manifest digest test passes; no served CSS has a remote `url(` or `@import`."
19. T-28 L518 and OQ-9 L550: T-28 → assert `.nav-toggle` with `aria-expanded`/`aria-controls="site-nav"`, expanded default, `papyrus.nav` persistence, collapsed `hidden`; OQ-9 → delete the "Safari version 100" premise and state that NFR-05.001's floor is Firefox 100+/Edge 130+/Safari 18+.

Also update the traceability table (L556+) to add `browser-shell-ui.md`, ADR-0026, ADR-0027, ADR-0028.

### Step 3 — per-family browser specifications (11 files)

For every file: replace the list section's legacy row/preview/metadata claims with a pointer plus the family's slot facts, and fix the baseline/browser/AC lines.

| File | Edits |
|---|---|
| `browser-audit-records.md` | L19 scope bullet, L70–72 list clauses 1–3 (drop the content preview; cite shared layout + `Audit type` slot), L170 baseline bullet (B3 + B6), L188 T-26k (add digest/CSP-class assertions), L200 mockup-coverage line (shared layout, no preview), L215 AC-2 (drop "content preview"; add Title-cell labels/references, action cell), L226 AC-13, L239 and L247 "100+" wording (C3). |
| `browser-business-rules.md` | L19, L70–72 (drop description preview; `Applicability` slot), L193, L213 T-25l, L225, L240 AC-2, L251 AC-13, L263/L273 "100+". |
| `browser-decision-records.md` | L56 list clause (shared layout; `Type`, `Superseded by` slots), L124, L178 "100+". |
| `browser-glossary-terms.md` | L58–60 (drop definition preview), L158 mockup coverage, L128, L213, L216 assumption (preview no longer exists). |
| `browser-technical-specifications.md` | L21, L70–74 (drop description preview), L168, L199, L216 AC-2. |
| `browser-ui-specifications.md` | L20 list bullet (drop description preview; cite shared layout), and any AC/test row repeating the preview. |
| `browser-ui-mockups.md` | L75–79 clause 3 ("rows show no status, task, supersession, or metadata preview") → shared layout with `Task` and `Specs` slots, Status column, Title-cell labels/references; L199 baseline, L269 "100+". |
| `browser-use-cases-test-cases-and-test-reports.md` | L19, L71–76 (drop preview and metadata line; explain Created/Updated columns and Title-cell labels/references), L214, L248, L258 OQ-3 (preview resolved as removed), L267 AC-2. |
| `browser-tasks-milestones-and-releases.md` | L58 (milestone/release rows → shared layout; milestone `Release` slot, release no slot; board unchanged), L226 baseline (B3/B6; drop "no inline styles" absolute), L234 T-11a (add `data-list-layout="shared"`), L277 AC-3 and L292 assumption ("100+" → NFR-05.001). |
| `browser-config-defined-document-types.md` | L31 scope ("shared document list layout (Status, ID, Title, Created, Updated)") → seven-column shared list with no family slot; L162–171 field-order/metadata-line statements → shared layout; L485–492 implementation bullets (`document_list_page`, five-column order) → `ListPlan`/`list_page`; L513–516 template creation (`document_list.html`) → deleted, `list.html` body fragment; L531 T-114-01 and L533 T-114-03 (five-column expectations, "Status immediately left of ID") → seven-column order and Title-cell labels/references; L596–603 A9; L611–613 MOCK-0015 note (historical, keep) and L619–621 UI note (seven columns). |
| `browser-managed-artifacts-and-lifecycles.md` | L104 CSP sentence → cite §9.7/B4; L105 "no external … font" → B1; L107 and L108 sidebar-toggle/narrow-collapse statements → B5 (header toggle, every page, default expanded); L119 test assertion (no external resource) → B3. |
| `browser-agent-skills-and-workflows.md` | L119 and L131 asset/font statements → B1/B2 (`/assets/app.js`, `/assets/nav-state.js`, asset table); L130 CSP → cite §9.7; L133 sidebar toggle → B5; L146 test assertion → B3. |
| `browser-complete-access.md` | L25 baseline bullet (browser floor), L82 asset/CSP bullet, L83 no-JS bullet (nav expanded, no toggle), L91 T-19a, L101 T-19k (add CSP class + digest), L116 "100+", L145 AC-9, L163 320 px row. |

**C3 completeness clause (all nine per-family pages, not only the rows named above).** Two shapes occur: the parenthetical `browser baseline (Firefox/Edge/Safari 100+)` and an acceptance row `renders in Firefox, Microsoft Edge, and Safari version 100 or later` (in `browser-decision-records.md` L165, `browser-glossary-terms.md` L203, `browser-tasks-milestones-and-releases.md` L277, and `browser-requirements.md` L348's feature list). Replace both with `Firefox 100+, Edge 130+, and Safari 18+ (NFR-05.001)`. The step-10 sweep is the completeness check: no `100+`/`Safari 100`/`Edge 100` string may remain in any `docs/specifications/browser-*.md` except `browser-shell-shared-list-layout-and-page-width.md`.

### Step 4 — `docs/packaging.md` (live procedure only)

1. L93–99 package contents: keep the two-entry rule and add "the copyright file now carries one section per bundled asset family (KaTeX code MIT, KaTeX fonts OFL-1.1, Mermaid MIT once TASK-146 lands)"; replace "compiled into the binary with `include_str!` and served from memory (`/assets/app.css`, `/assets/app.js`)" with B2's asset-table sentence and the representative paths.
2. L219–221 asset checks: replace with a probe list — `/assets/app.css` (`text/css`), `/assets/app.js` (`text/javascript`), `/assets/nav-state.js` (`text/javascript`), `/assets/katex/katex.min.css` (`text/css`), `/assets/katex/katex.min.js` (`text/javascript`), one `/assets/katex/fonts/KaTeX_Main-Regular.woff2` (`font/woff2`), and after TASK-146 `/assets/mermaid/mermaid.min.js`; each 200 with `Cache-Control: no-store`, non-empty, and no CSP header; a miss 404, `POST` 405.
3. L225–228 "No external references" bullet: replace with B3 + B4.
4. L261–267 §3.6: retitle to "Embedded assets, digests, notices, and the runtime file dependency"; keep the empty-working-directory test; add B9 (a)–(d) and B8.
5. `## Runtime dependency license audit` L436–468: keep the crate table and verdicts; add a paragraph: browser assets are not crates — their licences are recorded in `src/browser/vendor/MANIFEST.md` and in the package `copyright`, are compatible with GPL-3.0-or-later (MIT, OFL-1.1), and the NFR-06.002 release gate is unchanged.
6. Leave every dated result section untouched (Settled item 5).

### Step 5 — eight `*-milestone-packaging-and-verification.md` pages

Pattern per file (match on text, not line): (a) the scope bullet that says "no runtime asset files … are packaged, because they are compiled into the binary" gains "First-party and vendored browser assets are embedded through the static asset table, and the single copyright file carries one notice section per bundled asset family."; (b) the "320 px markup" checklist item → viewport meta plus B5 plus the 320/480/720 px no-horizontal-scroll assertion; (c) the "No external references" item → B3; (d) add one item for B4 and one for B9, numbered in the page's own sequence; (e) where a test table exists, add one row after `T-04`-style metadata row: notices, digests, and the 12 MiB budget per B9/B8.

| File | Anchor lines |
|---|---|
| `backlog-discovery-milestone-packaging-and-verification.md` | L20 scope, L109 320 px, L110 no-external, L132 §3.6, L190 table |
| `cli-milestone-packaging-and-verification.md` | L22, L132, L133, L157, L215 |
| `complete-content-management-milestone-packaging-and-verification.md` | L20, L135, L136, L159, L223 |
| `decision-records-milestone-packaging-and-verification.md` | L20, L125, L126, L148, L206 |
| `glossary-terms-milestone-packaging-and-verification.md` | L20, L119, L120, L142, L200 |
| `search-browser-milestone-packaging-and-verification.md` | L20, L103, L104, L126, L184 |
| `technical-specifications-milestone-packaging-and-verification.md` | L22, L132, L133, L155, L213 |
| `use-cases-and-test-documentation-milestone-packaging-and-verification.md` | L22, L132, L133, L155, L213 |

Also fix the per-page "320 px markup" assertions in scope bullets (e.g. backlog L25, cli L28, decision L25, glossary L25, search L22, technical L27, use-cases L27) that name the old sidebar toggle.

### Step 6 — `docs/specifications/rust-cli-and-debian-package-baseline.md` (TASK-143 section)

1. §2 L110: replace "Mermaid paths are added by its own task" with the accepted row: `/assets/mermaid/mermaid.min.js`, 3,572,661 B, `text/javascript; charset=utf-8`, MIT, Mermaid 11.17.2, SHA-256 `581ed7d74bd9048d0e3a91363927d72ef22942d7722546b27f7cc29e35390eb8`, landed by TASK-146.
2. §5 L137–144: extend the notice list with the Mermaid MIT section (own short name, full MIT text from `src/browser/vendor/mermaid/11.17.2/LICENSE` once TASK-146 lands) and state that the copyright file stays the only documentation file.

### Step 7 — `docs/specifications/ui/*.md` list and navigation statements (12 files)

Files: `browser-backlog-records-ui.md`, `browser-decision-records-ui.md`, `browser-requirements-ui.md`, `browser-business-rules-ui.md`, `browser-audit-records-ui.md`, `browser-technical-specifications-ui.md`, `browser-glossary-terms-ui.md`, `browser-ui-specifications-ui.md`, `browser-ui-mockups-ui.md`, `browser-use-cases-test-cases-and-test-reports-ui.md`, `browser-config-defined-document-types-ui.md`, `browser-final-cross-type-ui.md`.

Per file: (a) replace any list description that says rows carry a metadata line, a first-paragraph preview, a `Status, ID, Title, Created, Updated` column set, or a narrow-width sidebar toggle with a sentence pointing at `browser-shell-ui.md` §3 and §5; (b) keep the mockup-specific visuals, measurements, and MOCK ids; (c) where the file states the shared list is out of its scope, keep that and cite the shared contract. No new MOCK, no visual rework.

### Step 8 — pointer-only edit to the delivery record

`docs/specifications/browser-shell-shared-list-layout-and-page-width.md`: L19 ("canonical detail until TASK-148 ports …") and L5/L9 header lines → state that the normative shell/list/width/navigation clauses now live in `docs/specifications/browser-shell-ui.md` (published by TASK-148) and that this page remains the TASK-141/142/144 delivery record and implementation detail. No clause, table, or accepted-difference text changes.

### Step 9 — indexes

1. Regenerate/verify `docs/architecture/adr/index.html` from the ADR sources and diff. Expected outcome: byte-identical (this task edits no ADR). If it differs, stop and report — an ADR edit was made in error.
2. Verify `docs/requirements/business/index.html`, `docs/requirements/non-functional/index.html`, `docs/business-rules/index.html`, `docs/domain-model/business-glossary/index.html` and `docs/index.html` are unchanged (`git status` clean for those paths).
3. Record in the ticket that no generated specification index exists, so AC6's "specification index" is satisfied by the cross-references added in steps 1, 2, and 8.

### Step 10 — verification sweep (must all pass before reporting done)

```sh
grep -rn "no web font\|no web fonts" docs/specifications docs/packaging.md        # expect 0
grep -rn "remote \`url(\`\|no \`url(\`" docs/specifications docs/packaging.md      # expect 0
grep -rn "Status, ID, Title, Created, Updated" docs/specifications | grep -v shell-shared  # expect 0
grep -rn "Other requirements" docs/specifications/browser-*.md | grep -v shell-shared   # expect 0
grep -rn "row-meta\|metadata line\|first paragraph of" docs/specifications/browser-*.md docs/specifications/ui/*.md | grep -v shell-shared  # expect 0
grep -rn "Edge 100\|Safari 100\|Safari version 100\|100+" docs/specifications/browser-*.md  # expect 0
grep -rn "document_list.html" docs/specifications | grep -v shell-shared          # expect 0
grep -rn "Proposed" docs/specifications/browser-math-and-diagram-rendering.md docs/specifications/browser-shell-ui.md  # expect 0
# CSP equality: the two literals in browser-requirements §9.7 must equal http.rs:16 and the
# same two in browser-math-and-diagram-rendering §7.1/§17.1, byte for byte.
git diff --name-only docs/requirements docs/architecture/adr                      # expect empty
```

Then report: files changed, the conflict register (C1–C5) untouched, the measured numbers with their source, and any new conflict found while editing (do not fix it in prose).

## 3. Notes for the executor

- Match on quoted text: line numbers drift after the first edit in a file (A4).
- Never restate the CSP literals outside `browser-requirements.md` §9.7 and the existing rendering page; cite the class instead.
- Never turn the accepted-but-unimplemented Mermaid contract into a claim that it ships today.
- Do not touch `docs/requirements/`, `docs/architecture/adr/`, `docs/specifications/browser-math-and-diagram-rendering.md` clauses, or the dated result sections of `docs/packaging.md`.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Published `docs/specifications/browser-shell-ui.md` (pinned §1–§9). Corrected 26 page targets + 12 UI descriptions to the landed shell/list/width/navigation contract. CSP literals are single-homed in `browser-requirements.md` §9.7 and are byte-identical to `browser-math-and-diagram-rendering.md` §7.1/§17.1 and `http.rs` (strict count 1/1/2, diagram 1/1/1 across the three files).

Measured 2026-11-16: `stat -c %s target/release/papyrus` = 8,321,888 B (7.94 MiB) after Mermaid landed; the ticket's pre-Mermaid projection was 8,302,957 B (7.92 MiB); KaTeX 558,541 B (brief's 580,487 B corrected); Mermaid 3,572,661 B; ceiling 12,582,912 B; measured headroom 4,261,024 B (4.06 MiB). `cargo test --lib manifest_covers_every_asset` passed.

No ADR, requirement, business-rule, or source file was edited. `docs/architecture/adr/index.html` was not touched; pre-existing working-tree modifications to ADRs/index remain unstaged and are not part of this commit. No generated index exists for `docs/specifications/`.

Conflict register C1–C5 left unresolved in prose and reported to the caller. New deviation from the refined brief: TASK-146 has landed (`CSP_DIAGRAM` and the Mermaid asset are present), so pages state the diagram string is implemented by TASK-146 and verified by TASK-150 rather than describing it as future work.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @business-analyst
created: 2026-11-16T09:27:00Z

---

Correction round after independent review (2026-11-16 UTC). Scope bookkeeping:

(a) **Delivery record changed beyond pointer-only.** `docs/specifications/browser-shell-shared-list-layout-and-page-width.md` also received §3.4 grid track (`minmax(0, 1fr)` instead of a retained zero-width first track, because `.main` auto-places into it and collapses to 0), §4.2 `page_width(PageRoute)` (route identity instead of a slug), §4.3 heading/pre/scroll-region bullets, A3, §7.2 and OQ-2 wording. These corrections were right because the pointer-only version would otherwise have contradicted the landed implementation and the accepted-difference record.

(b) **Extra pages beyond the plan's 12 + 26.** Three UI descriptions (`browser-agent-skills-ui.md`, `browser-requirements-ui.md`, `browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md`) and two specification pages (`browser-configuration-view-single-record-layout-and-lifecycle-configuration.md`, `browser-spec-revisions-task-86.md`) were touched because they retained per-group member counts, old empty-state copy, or the 320 px auto-collapse.

(c) **`browser-ui-mockups.md` write contract.** Changed "metadata lines" to "metadata comment lines" (and the same in the insertion rule and T-40e) because the field-preserving write path only edits the `<!-- specs: -->` / `<!-- refs: -->` comments; the visible `<p>` lines are rewritten only when already present. The old wording overstated what is written and could imply whole-file regeneration.

Conflict register: added C6 (diagram `.bleed`/scroll-container accessibility gap; owners TASK-146/TASK-150) and C7 (REQ-06.015 mockup full-width class gap).

---

author: @business-analyst
created: 2026-11-16T09:37:00Z

---

Final consistency pass after re-review (2026-11-16 UTC).

- **Additional in-scope page:** `docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md` §2 (line 14) and §5 (line 42) carried the same 320 px auto-collapse claim; both now state expanded-by-default at every width with the persisted `papyrus.nav` toggle.
- **Adjacent same-class page:** `docs/specifications/browser-spec-revisions-task-102.md` §Surfaces line 38 (`320 px sidebar toggle`) was corrected to the same rule so the corpus has no residual width-specific navigation claim.
- **MOCK-0001 decision:** `docs/specifications/browser-requirements.md` §UI design handoff describes the approved 2026-11-03 package as built. Decision: leave that historical screen description intact (including the 320 px collapsed-sidebar line) and add a one-line historical note above the block stating that historical mockup descriptions are not updated to later contracts.
- **Other residual lines:** `browser-complete-access.md` §6.1 now says templates are embedded and rendered server-side, with only `/assets/*` files from the static asset table; `ui/browser-technical-specifications-ui.md` §2 and `ui/browser-use-cases-test-cases-and-test-reports-ui.md` §2 use the every-width persisted toggle; `ui/browser-use-cases-test-cases-and-test-reports-ui.md` §4 now uses the shared empty-state pair (`No {noun} in this project.` plus the command-line help line; delivery record A6) instead of the legacy "were found in the configured directory"/"Deprecated … are hidden" copy.

---
<!-- COMMENTS:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Documentation-only port to the landed contract. Created `browser-shell-ui.md` with the pinned outline and updated every browser and packaging page listed in the plan; removed every falsified assertion (no web font, no url(, asset-free package, per-family list markup, document_list.html, 100+ browser floor, CSP without font-src/unsafe-inline scoping). Verification: stale-string sweeps return only documented true statements; CSP literals match source; markdown links in touched pages resolve; manifest digest test passes; measured binary 8,321,888 B is under the 12,582,912 B ceiling. Conflicts C1–C5 reported, not resolved. DoD #5 cannot be marked because the pre-existing dirty tree already modifies ADRs; this task's commit excludes them.
<!-- SECTION:FINAL_SUMMARY:END -->
