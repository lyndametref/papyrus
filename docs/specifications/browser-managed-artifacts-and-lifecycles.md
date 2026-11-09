# Feature Specification — Browser Page for Managed Artifacts and Lifecycles

**Ticket:** TASK-72
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready; UI mockup approved)
**UI mockups:** [`MOCK-0010`](ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html) (approved 2026-11-09, TASK-66) — UI description [`ui/browser-front-matter-and-lifecycle-ui.md`](ui/browser-front-matter-and-lifecycle-ui.md); reuses the approved baseline shell [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html). No new mockup is required for this ticket.

## Purpose

Add a read-only page to the offline, project-local web interface that describes every managed artifact/document type and its lifecycle, per REQ-06.003. The page is a single static view with one section per fixed record type; task states are derived from the configured task statuses (REQ-01.002); every other type uses the lifecycle documented for that type. It advances NFR-01.002 (display all Papyrus content) by giving contributors a single place to learn what each record type is and how it moves through its states.

## Scope

### In scope

- A new page route that serves the lifecycle page, reusing the shared shell, sidebar, header, footer, CSP, escaping, and responsive rules of the baseline (TASK-6, ADR-0005/ADR-0006).
- A persistent sidebar entry labelled `Managed artifacts & lifecycles`, placed at the end of the document-type list, that links to the page and carries `aria-current="page"` when active.
- A page body with one section (`h2`) for each of the twelve fixed record types, in this order: Tasks, Milestones, Releases, Decisions, Business requirements, Non-functional requirements, Glossary terms, UI specifications, Technical specifications, UI mockups, Business rules, Audit records.
- A task section whose state list and transitions are derived from the configured task statuses (REQ-01.002).
- A lifecycle sentence for every other section that uses the documented lifecycle for that type (see section 4).
- An explicit statement on the page that Backlog.md `docs`, `drafts`, and project configuration files are not covered.
- Unit/integration tests extending `tests/browser_cli.rs` (or a sibling file) for the new page.

### Out of scope

- Listing, viewing, editing, creating, deprecating, restoring, archiving, deleting, or searching any record from this page; all record lifecycle operations remain CLI-only, consistent with the baseline no-lifecycle-operations stance (REQ-05.007).
- Any API endpoint for the page; the page is HTML only.
- Adding a new record type, changing configured `document_types`, or deriving the covered set from configuration. The covered set is fixed by REQ-06.003 and does not change with `document_types`.
- Describing Backlog.md `docs`, `drafts`, or project configuration files.
- Changes to MOCK-0001..MOCK-0009, to MOCK-0010, or to any UI description document.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-20.

## Behavior

### 1. Route and navigation

1. `GET /lifecycle` serves the page. `GET /?type=lifecycle` serves the same page (alias) so the sidebar/filter model stays consistent. Any other method on a known path is `405` with an `Allow: GET, HEAD` header; an unknown path is `404`.
2. The sidebar entry is labelled exactly `Managed artifacts & lifecycles` and is placed after `UI Mockups` (the last document-type entry). It is always present; it is not gated by any `document_types.*.enabled` flag.
3. The sidebar entry links to `/lifecycle`. When the page is active, the entry carries `aria-current="page"` and the non-colour cue (bold + underline) used by every other entry.
4. The page renders the shared shell: header (product name, project name, static offline/bind note), persistent left sidebar, `main`, footer, and exactly one `h1` with the text `Managed artifacts and lifecycles`.
5. A subtitle paragraph states: `One section per managed record type. Backlog.md docs, drafts, and project configuration files are not covered.`
6. The breadcrumb is not required for this page; the sidebar is the navigation. (The page is not a record view.)

### 2. Page sections

1. The page renders exactly twelve sections, in the order listed in the Scope section, each as a `<section>` containing an `<h2>` whose text is the type name from that list.
2. Each section contains one lifecycle paragraph. No section contains a record list, count, link, or form.
3. Section order, headings, and count are fixed and do not depend on configuration, project contents, or enabled document types.

### 3. Task section

1. The task section heading is `Tasks`.
2. The lifecycle paragraph states the configured task statuses in their configured order, joined by ` → `, for example `States: ToDo → Initiated → Refined → Implemented → Validated → Done.` with default configuration.
3. The paragraph also states that the states and transitions are derived from the configured task statuses (REQ-01.002) and that task lifecycle operations are CLI-only.
4. Only the configured status names are rendered. No per-state description is required for custom statuses; the configured order is the transition order.
5. When the configured status list is empty or invalid, the server falls back to its configured/default status handling rather than rendering an empty task section; the task section never renders an empty state list.

### 4. Lifecycle wording for other types

Each section's lifecycle paragraph uses the documented lifecycle for that type:

| Section | Lifecycle paragraph |
| --- | --- |
| Milestones | `Lifecycle: active → archived. Archiving a milestone is separate from task workflow status.` |
| Releases | `Lifecycle: active → released. released is delivery completion, distinct from archiving; releases are not archived. The transition is performed explicitly via the documented release status command.` |
| Decisions | `Lifecycle: proposed → accepted \| rejected; accepted → deprecated \| superseded.` |
| Business requirements | `Lifecycle: active → deprecated (restored via restore).` |
| Non-functional requirements | `Lifecycle: active → deprecated (restored via restore).` |
| Glossary terms | `Lifecycle: active → deprecated (restored via restore).` |
| UI specifications | `Lifecycle: active → deprecated (restored via restore).` |
| Technical specifications | `Lifecycle: active → deprecated (restored via restore).` |
| UI mockups | `Lifecycle: pending → approved → deprecated (restored via restore; supersession via ui-mockup supersede).` |
| Business rules | `Lifecycle: active → deprecated (restored via restore).` |
| Audit records | `Lifecycle: active → deprecated (restored via restore).` |

1. The wording above is normative for this page and is taken from REQ-03.001, REQ-03.002, REQ-04.001, REQ-05.006, REQ-05.007, and REQ-05.008. It intentionally differs from the shorthand sentences in MOCK-0010 where those conflict with the documented lifecycles.
2. The page does not state that a release becomes released when its milestone set is complete, that a milestone closes when its tasks are complete, that documents move through a Draft → Approved phase, or that audit records are immutable; none of those is documented.
3. All lifecycle sentences are static text; they are not derived from project data.

### 5. Exclusions statement

1. The subtitle (section 1.5) states that Backlog.md `docs`, `drafts`, and project configuration files are not covered.
2. No section, heading, link, or text on the page describes Backlog.md `docs`, `drafts`, or project configuration files.

### 6. Rendering, security, and accessibility

1. All dynamic values (project name, bind note, configured status names) are HTML-escaped before rendering. Status names are rendered as text, never as markup.
2. The page sets the same CSP and security headers as every other page (`default-src 'none'; style-src 'self'; script-src 'self'; ...`).
3. The page loads no external stylesheet, script, font, image, or network resource; styles come from `/assets/app.css`.
4. The page works with JavaScript disabled.
5. Focus order follows document order; every interactive control (sidebar toggle at 320 px, sidebar links) has a visible label or accessible name.
6. The 320 CSS-pixel responsive layout collapses the sidebar behind the existing accessible toggle, matching the baseline.

### 7. Tests

1. An integration test starts `papyrus browser --no-open` on a free port and asserts:
   - `GET /lifecycle` returns `200` with `Content-Type: text/html; charset=utf-8`;
   - the response body contains the `h1` text `Managed artifacts and lifecycles`;
   - the response body contains all twelve section headings in order;
   - the response body contains the configured default status chain `ToDo → Initiated → Refined → Implemented → Validated → Done`;
   - the response body contains the exclusion statement;
   - the response body contains the sidebar entry `Managed artifacts &amp; lifecycles`;
   - the response contains no `http://` or `https://` reference and no external resource reference.
2. The same test asserts `GET /?type=lifecycle` returns the same page (alias).
3. The same test asserts `POST /lifecycle` returns `405` with an `Allow` header.
4. A second test (or a parameterised case) initialises a project with a custom status list (for example `["Backlog", "Doing", "Done"]`) and asserts the task section renders that exact chain and does not render the default chain.

## Assumptions

- Assumption: The page is read-only and renders no project records; it does not call into the record index or any store.
- Assumption: The route is `/lifecycle` with `/?type=lifecycle` as an alias, matching the UI description's recommended default (OQ-2) and keeping the sidebar/filter model consistent.
- Assumption: The sidebar entry is always present and is not affected by `document_types.*.enabled`, because the page describes a fixed set independent of configuration.
- Assumption: Lifecycle wording uses the documented lifecycles from requirements/README/BR (per user selection during refinement), not the shorthand sentences in MOCK-0010 where those conflict.
- Assumption: Custom task statuses have no per-state descriptions; only their names and order are rendered.
- Assumption: No per-state one-line descriptions are added beyond the single lifecycle sentence per section; MOCK-0010 OQ-6 is resolved to one sentence per section.

## Open questions

- None. Material behavior (covered set, task-status derivation, lifecycle wording, route, exclusions) is resolved by REQ-06.003, the approved MOCK-0010 layout, and the user's selection of documented lifecycle wording during refinement.

## Related artifacts

- Requirement: `docs/requirements/business/REQ-06.003.md` (primary); `docs/requirements/business/REQ-01.002.md`; `docs/requirements/business/REQ-03.001.md`; `docs/requirements/business/REQ-03.002.md`; `docs/requirements/business/REQ-04.001.md`; `docs/requirements/business/REQ-05.006.md`; `docs/requirements/business/REQ-05.007.md`; `docs/requirements/business/REQ-05.008.md`; `docs/requirements/non-functional/NFR-01.002.md`
- UI description: `docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md`
- UI mockup: `docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html` (approved)
- Baseline browser spec: `docs/specifications/browser-requirements.md`
- Task status source: `docs/business-rules/BR-001-papyrus-command-behavior.md`, `docs/business-rules/BR-006-papyrus-milestone-command-behavior.md`, `docs/business-rules/BR-007-papyrus-release-command-behavior.md`, `docs/business-rules/BR-008-papyrus-decision-command-behavior.md`
- Milestone: `m-12`
- Dependency: TASK-66 (Done)
path: /home/lyly/Documents/Dev/papyrus/docs/specifications/browser-managed-artifacts-and-lifecycles.md