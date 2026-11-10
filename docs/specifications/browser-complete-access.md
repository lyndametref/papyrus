# Feature Specification — Complete Browser Access for All Papyrus Content

**Ticket:** TASK-19
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0012`](ui/mockups/MOCK-0012-browser-final-cross-type-review.html) — approved 2026-11-10 (TASK-60); UI description [`ui/browser-final-cross-type-ui.md`](ui/browser-final-cross-type-ui.md). Reuses the approved baseline shell [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html) and approved patterns [`MOCK-0010`](ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html), [`MOCK-0011`](ui/mockups/MOCK-0011-browser-agent-skills-review.html). No new mockup is required for this ticket.

## Purpose

Final integration increment of the offline, project-local web interface: one navigation (the sidebar) and one content inventory (the landing view at `GET /`) expose every record type Papyrus manages, reusing the per-type list/view/edit views delivered by TASK-6, TASK-11, TASK-15, TASK-23, TASK-24, TASK-40, TASK-25, TASK-26, TASK-57, TASK-72, and TASK-77. This task does not re-implement per-type rendering; it consolidates those increments into a single entry point and verifies cross-type consistency. It completes NFR-01.002 (display all Papyrus content in the local browser) and advances NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, and NFR-05.001 for the browser as a whole. The transport, asset delivery, rendering, security, configuration, and accessibility baseline of TASK-6 and ADR-0005/ADR-0006 are reused unchanged.

## Scope

### In scope

- An inventory hub at `GET /` listing every supported record type as a card with an illustrative count, with the represented types linking to their per-type list view (`/?type=<category>`). The requirements list view moves to `GET /?type=req`; `GET /?type=req` and the requirements list behavior are unchanged.
- The persistent sidebar in the final order established by the approved MOCK-0012 and the per-type increments: Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows. Gating by `document_types.*.enabled` for the gated types is retained; `Managed artifacts & lifecycles` and `Agent skills & workflows` remain always present.
- `aria-current="page"` and the non-colour cue (bold + underline) on the active sidebar entry, including on the inventory hub (no entry active) and on the lifecycle and skills pages.
- The `?type=` category selector continues to accept exactly the existing values (`req`, `nfr`, `task`, `milestone`, `release`, `decision`, `glossary`, `ui-spec`, `ui-mockup`, `business-rule`, `audit-record`, `tech-spec`, plus the `lifecycle` and `skills` aliases). Any other value, including `all`, remains `400`.
- Cross-type ID resolution: a record ID is resolved by exact match across all twelve record categories; an ID matching records in more than one category is `500` with a generic message; a deprecated or unknown ID is `404`.
- Deprecated records of every document type are hidden from lists, counts, and the inventory, and are unreachable by URL, consistently with the per-type increments.
- Disabled or absent document types contribute no records and no error to the inventory or to any list; their sidebar entry is omitted.
- Empty states for every category, including the inventory when no records exist.
- Front-matter metadata projection (REQ-06.004) is reused on every per-type list/detail view and on the inventory cards where applicable; `created_date` and `updated_date` remain read-only in the browser.
- Embedded assets, CSP, security headers, Host/Origin validation, request limits, timeouts, offline operation, browser baseline (Firefox/Edge/Safari per NFR-05.001), accessibility, and the 320 CSS-pixel responsive layout, all reused unchanged.
- Unit and integration tests extending `tests/browser_cli.rs` (and siblings) for the inventory hub and for cross-type consistency, including a cross-type test that creates one record of every supported type and verifies it is reachable through the inventory and its per-type view.

### Out of scope

- Re-implementing per-type list/view/edit rendering, persistence, validation, or JSON APIs; those remain owned by their dedicated increments (TASK-6, TASK-11, TASK-15, TASK-23, TASK-24, TASK-40, TASK-25, TASK-26, TASK-57, TASK-72, TASK-77).
- Adding a new record type, configuration key, `?type=` value, or API endpoint.
- Cross-type search in the browser UI; search remains CLI-only (TASK-18).
- Creating, deprecating, restoring, archiving, or deleting records from the browser; these remain CLI-only.
- Real-browser GUI rendering verification in Firefox, Edge, or Safari; only markup-, asset-, and protocol-level verification is performed (consistent with sibling increments; NFR-05.001 real-browser verification is deferred).
- Packaging and release verification — TASK-20 / TASK-73.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006/ADR-0009 remain the baseline.
- Changes to approved mockups (MOCK-0001…MOCK-0012) or to any UI description document.

## Behavior

### 1. Route and navigation

1. `GET /` serves the inventory hub. `GET /?type=req`, `GET /?type=nfr`, and every other documented `?type=` value continue to serve their existing per-type list page. The baseline rule that `GET /` and `GET /?type=req` render the same page is superseded by this increment: `GET /` now renders the inventory hub, and `GET /?type=req` renders the requirements list.
2. `GET /lifecycle` and `GET /?type=lifecycle` serve the managed-artifacts-and-lifecycles page (TASK-72); `GET /skills` and `GET /?type=skills` serve the agent-skills page (TASK-77). These aliases are unchanged.
3. Any method other than `GET`/`HEAD` on a known path is `405` with an `Allow: GET, HEAD` header; an unknown path is `404`.
4. The sidebar is rendered on every page, including the inventory hub, the lifecycle page, and the skills page, using the final order in Scope. The active entry carries `aria-current="page"` and the non-colour cue; on the inventory hub no document-type entry is active.
5. The sidebar collapses behind the existing accessible toggle at 320 CSS pixels, matching the baseline.

### 2. Inventory hub

1. The page renders the shared shell: header (product name, project name, static offline/bind note), persistent left sidebar, `main`, footer, and exactly one `h1` with the text `Browser inventory` (per MOCK-0012). A subtitle states that the inventory lists every record type Papyrus manages.
2. The page renders one card per supported record type, in the final sidebar order, each showing the type name, the record count for that type (active records only), and a short description. The count is computed from the configured active store, exactly as the per-type list count; a missing or disabled store contributes `0` and no error.
3. A represented type's card links to its per-type list view (`/?type=<category>`). The represented types are all twelve record categories plus the two virtual pages: Managed artifacts & lifecycles (`/lifecycle`) and Agent skills & workflows (`/skills`). Every card is therefore clickable; cards for disabled document types are omitted rather than shown as zero-count.
4. A card for a type that has no records shows its empty-state note (for example "No technical specifications found.") and remains clickable.
5. The page renders no record list, count beyond the card count, form, or edit control of its own; per-type lists remain the place to browse records.
6. The page works with JavaScript disabled; cards are plain links.

### 3. Sidebar (final order and gating)

1. The sidebar lists entries in this order: Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows.
2. Gated entries (Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, UI specifications, UI mockups) are omitted when the corresponding `document_types.<key>.enabled` is `false` or the key is absent, exactly as in the per-type increments. Backlog, Decisions, Glossary, Managed artifacts & lifecycles, and Agent skills & workflows are always present.
3. The selected entry carries `aria-current="page"` and the non-colour cue (bold + underline). The Backlog entry is active for `task`, `milestone`, and `release` categories.
4. Labels use the approved casing from MOCK-0012 (`Business requirements`, `Business rules`, `Audit records`, `Technical Specifications` → `Technical specifications`? keep per-type label). The exact labels follow the per-type increments and MOCK-0012; no label is changed by this increment beyond ordering.

### 4. Cross-type consistency

1. ID resolution covers all twelve record categories (REQ, NFR, task, milestone, release, decision, glossary, UI spec, UI mockup, business rule, audit record, tech spec). An ID that matches no active record in any category is `404`; an ID that matches active records in more than one category is `500` with a generic message and no detail about which categories matched. Deprecated records are never matched.
2. Deprecated records of every document type are stored in that type's `deprecated/` subfolder, are never listed, counted, or resolved, and return `404` on view, edit, POST, and API routes. The HTML `404` page for a deprecated ID is identical to that for an unknown ID of the same type (per-type help text and single recovery link), so the response never reveals whether a hidden ID exists.
3. Disabled or absent document types contribute no records and no error to any list, count, search of the store, or ID resolution.
4. A missing or unreadable store directory is not a server error for list/inventory routes: the category renders its empty state and its API returns an empty array. An unreadable directory on a direct record request returns `500` with a generic message.
5. Every category's empty state uses the documented empty-state wording for that type and renders without horizontal scrolling for long unbreakable tokens.
6. Optimistic concurrency, atomic writes, path boundary enforcement, HTML escaping, and the version token contract are reused unchanged from the baseline and the per-type increments.

### 5. Metadata projection (REQ-06.004)

1. List and detail views for every record type read `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and the type-specific keys from the YAML front matter, exactly as in the per-type increments and ADR-0009's single projection.
2. The browser's editable field set is unchanged; `created_date` and `updated_date` are read-only. A save writes the changed values to the front matter, refreshes `updated_date`, preserves every other front-matter key and the rest of the file, and never copies metadata into the body.
3. The inventory card counts and links do not depend on front-matter parsing beyond what the per-type list already does.

### 6. Rendering, security, and accessibility

1. All baseline rules from `docs/specifications/browser-requirements.md` sections 6, 9, and 10 apply unchanged: embedded templates and assets, `{{name}}` placeholder substitution, HTML escaping, CSP and security headers, Host/Origin validation, request limits, timeouts, no external resources, no inline scripts/styles.
2. The interface renders and is usable without JavaScript, including the inventory cards (plain links) and the sidebar toggle.
3. Focus order follows document order; every interactive control has a visible label or accessible name.
4. The 320 CSS-pixel responsive layout collapses the sidebar behind the existing accessible toggle and stacks card content; no horizontal scrolling occurs.

## Tests

Integration tests extend `tests/browser_cli.rs` (and siblings) with at least the following scenarios:

- T-19a: `GET /` returns `200` with `Content-Type: text/html; charset=utf-8`, renders the inventory hub with `h1` text `Browser inventory`, one card per supported record type in final order, and no external resource reference.
- T-19b: each represented type's card links to its per-type list (`/?type=<category>`); following the link renders that category's list with `aria-current="page"` on the matching sidebar entry. Disabled document types contribute no card.
- T-19c: the inventory shows the correct active-record count for a project containing records of every supported type, and shows `0` (or the empty-state note) for a type with no records.
- T-19d: `GET /?type=req` (and every other documented `?type=` value) still renders its per-type list exactly as before this increment; `GET /` no longer renders the requirements list.
- T-19e: `GET /?type=all` (and any other undocumented value) returns `400`.
- T-19f: the sidebar on the inventory hub, on a per-type list, on the lifecycle page, and on the skills page lists entries in the final order, with the active entry marked `aria-current="page"` and the non-colour cue; gated entries are omitted when their type is disabled.
- T-19g: cross-type ID resolution: an ID present in exactly one category resolves to that record; an ID present in two categories returns `500`; a deprecated ID and an unknown ID return `404` with the per-type `404` page.
- T-19h: deprecated records of at least three document types are absent from the inventory counts and return `404` on view, edit, POST, and API, with identical HTML `404` bodies for deprecated and unknown IDs.
- T-19i: a project with a disabled document type (for example `tech-spec.enabled: false`) shows no card for it and no error on the inventory; `?type=tech-spec` renders the empty state.
- T-19j: the inventory and every per-type list/edit view render without JavaScript, keep controls labelled and stacked at 320 CSS pixels, and contain no `http://` or `https://` reference.
- T-19k: an offline asset scan finds no external origin in inventory, list, view, edit, lifecycle, skills, and error responses.
- T-19l: build, tests, clippy, and formatting pass with existing dependencies only.

## Assumptions

- Assumption: The inventory hub replaces the previous default `GET /` → requirements-list behavior; `GET /?type=req` becomes the canonical requirements list route. This follows the approved MOCK-0012 route table (`GET /` = inventory). Existing tests that call `GET /` expecting the requirements list are updated as part of this increment; no product behavior other than the landing route changes.
- Assumption: The inventory card counts are active-record counts computed by the same store reads as the per-type lists; they are not cached and not persisted.
- Assumption: "Represented types" in the approved MOCK-0012 are all twelve record categories plus the two virtual pages, because every category already has a per-type list view delivered by earlier increments. Cards for disabled document types are omitted rather than shown as disabled.
- Assumption: Real-browser verification in Firefox, Edge, and Safari (NFR-05.001) remains a deferred follow-up, consistent with sibling increments.
- Assumption: No new dependencies, crates, or ADRs are introduced; the change is confined to the existing `src/browser/` module and its tests.
- Assumption: The metadata projection (REQ-06.004, ADR-0009) is already implemented by the per-type increments; this increment adds no new metadata keys and changes no editable field set.

## Open questions

- **OQ-1 (non-blocking, product)** — Whether the inventory hub should show counts only or also a one-line preview per type (for example the most recently updated record). MOCK-0012 uses illustrative counts only; this specification adopts counts only. If the product prefers previews, only the card template and its tests change.
- **OQ-2 (non-blocking, follow-up)** — Real-browser verification in Firefox, Edge, and Safari 100+ is deferred, consistent with sibling tickets.
- **OQ-3 (non-blocking, consistency)** — The approved MOCK-0012 prototype marks only four types as clickable because it fully prototypes only those; the implementation marks every type clickable because every type already has a per-type list view. If the product prefers a restricted clickable set, only the card link rendering changes.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-02.001, REQ-02.002, REQ-03.001, REQ-03.002, REQ-04.001, REQ-04.002, REQ-05.001–REQ-05.007, REQ-06.002, REQ-06.004 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior) |
| ADR | ADR-0005, ADR-0006, ADR-0007, ADR-0009 |
| Ticket | TASK-6 (baseline), TASK-11, TASK-15, TASK-23, TASK-24, TASK-25, TASK-26, TASK-40, TASK-57, TASK-58, TASK-60 (UI design), TASK-66, TASK-67, TASK-72, TASK-76, TASK-77, TASK-19 |
| UI mockups | MOCK-0001…MOCK-0011 (approved baseline); MOCK-0012 (approved) — `docs/specifications/ui/mockups/MOCK-0012-browser-final-cross-type-review.html` |
| UI description | `docs/specifications/ui/browser-final-cross-type-ui.md` (TASK-60) |
| Related specification | `docs/specifications/browser-requirements.md` (TASK-6 baseline), `docs/specifications/browser-tasks-milestones-and-releases.md`, `docs/specifications/browser-decision-records.md`, `docs/specifications/browser-glossary-terms.md`, `docs/specifications/browser-ui-specifications.md`, `docs/specifications/browser-ui-mockups.md`, `docs/specifications/browser-business-rules.md`, `docs/specifications/browser-audit-records.md`, `docs/specifications/browser-technical-specifications.md`, `docs/specifications/browser-managed-artifacts-and-lifecycles.md`, `docs/specifications/browser-agent-skills-and-workflows.md`, `docs/specifications/search-complete-unified.md` (sibling "complete" increment) |
| Milestone | m-12 |

## Acceptance criteria mapping

| Check | Verifies |
|---|---|
| AC-1 | `GET /` renders the inventory hub with one card per supported type in final order, counts, and links; `GET /?type=req` still renders the requirements list. |
| AC-2 | The sidebar on every page lists entries in the final order with the active entry marked; gated entries are omitted when disabled. |
| AC-3 | `?type=` accepts exactly the documented values (plus `lifecycle`/`skills` aliases); any other value is `400`. |
| AC-4 | Cross-type ID resolution returns `404` for deprecated/unknown IDs, `500` for an ID present in more than one category, and the record for a unique ID. |
| AC-5 | Deprecated records are absent from inventory counts and unreachable by URL with the per-type `404` page. |
| AC-6 | Disabled/absent document types contribute no card, no records, and no error. |
| AC-7 | Empty states render for every category and for the inventory. |
| AC-8 | Metadata projection (REQ-06.004) is reused unchanged on every list/detail view. |
| AC-9 | The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable controls. |
| AC-10 | The interface works without JavaScript and keeps controls labelled and stacked at 320 CSS pixels. |
| AC-11 | Build, tests, clippy, and formatting pass with existing dependencies only. |

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. Layout only — no branding, colour palette, logo, or visual identity.

**Approved artifact (TASK-60):** [`MOCK-0012`](ui/mockups/MOCK-0012-browser-final-cross-type-review.html) — `approved` in the mockup registry on 2026-11-10 (UTC). One self-contained review package: review landing, inventory prototype, representative record views with front-matter metadata, lifecycle page, skills page, error pages, and 320 px states (21 views). Layout description: [`ui/browser-final-cross-type-ui.md`](ui/browser-final-cross-type-ui.md). MOCK-0001…MOCK-0011 remain the approved shell and pattern baseline.

### Screens covered by the approved package

1. **Shared shell** (reused): header, sidebar with twelve document-type entries plus two virtual pages, `main`, footer, one `h1` per page.
2. **Inventory hub** (`/`): type cards with counts; represented types clickable.
3. **Representative record views** (tasks, milestones, releases, technical specifications): list/detail/edit with front-matter metadata block.
4. **Lifecycle page** (`/lifecycle`): twelve sections.
5. **Skills page** (`/skills`): configuration block and grouped skills.
6. **Error pages**: reused 404/400/405/415/500 plus per-type 404 pages.
7. **320 CSS px**: list, edit, lifecycle, skills views; sidebar toggle.

### Layout decisions resolved by this specification

1. `GET /` becomes the inventory hub; `GET /?type=req` becomes the canonical requirements list, per the approved MOCK-0012 route table.
2. The sidebar uses the final order listed in section 3.1 and MOCK-0012 section 2.
3. Inventory cards are plain links and work without JavaScript.
4. Deprecated and unknown IDs use the per-type `404` page, identical for both cases.

## Related artifacts

- Requirement: `docs/requirements/business/REQ-02.001.md`, `REQ-02.002.md`, `REQ-03.001.md`, `REQ-03.002.md`, `REQ-04.001.md`, `REQ-04.002.md`, `REQ-05.001.md`–`REQ-05.007.md`, `REQ-06.002.md`, `REQ-06.004.md`
- Non-functional: `docs/requirements/non-functional/NFR-01.002.md`, `NFR-01.003.md`, `NFR-02.002.md`, `NFR-04.001.md`, `NFR-04.002.md`, `NFR-05.001.md`
- Business rule: `docs/business-rules/BR-004-browser-command-behavior.md`
- ADR: `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md`, `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md`, `docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md`, `docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md`
- Milestone: m-12
- Dependency: TASK-6, TASK-11, TASK-15, TASK-23, TASK-24, TASK-25, TASK-26, TASK-40, TASK-57, TASK-58, TASK-60, TASK-66, TASK-67, TASK-72, TASK-76, TASK-77
