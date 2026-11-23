---
id: TECH-050
title: "Browser Page for Managed Artifacts and Lifecycles"
created_date: '2026-11-16T17:46:50Z'
updated_date: '2026-11-19T21:42:42Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - ADR-0022
  - ADR-0023
  - BR-001
  - BR-006
  - BR-007
  - BR-008
  - MOCK-0001
  - MOCK-0009
  - MOCK-0010
  - NFR-001.002
  - NFR-001.003
  - REQ-001.002
  - REQ-001.003
  - REQ-003.001
  - REQ-003.002
  - REQ-004.001
  - REQ-005.006
  - REQ-005.007
  - REQ-005.008
  - REQ-006.003
  - REQ-006.006
  - REQ-006.007
  - REQ-006.008
  - REQ-006.009
  - REQ-006.010
  - REQ-006.011
  - REQ-006.012
  - REQ-006.013
  - REQ-007.001
  - TASK-114
  - TASK-127
  - TASK-147
  - TASK-20
  - TASK-6
  - TASK-66
  - TASK-72
  - TASK-86
  - TASK-95
  - ADR-034
  - REQ-006.018
  - RE-005
status: draft
---
# TECH-050 Browser Page for Managed Artifacts and Lifecycles

> **Revised (TASK-170, 2026-11-18):** browser surfaces removed per ADR-034; this building block no longer provides the removed Configuration, Managed artifacts and lifecycles, or Agent skills and workflows surfaces. See ADR-034.

## Description
> **Superseded in part (TASK-114):** the fixed twelve-section set and the "covered set does not change with `document_types`" rule are replaced by the registry-derived three-category page in [`browser-config-defined-document-types.md`](browser-config-defined-document-types.md) (REQ-006.003 revision of 2026-11-13). The page route, shell, and read-only configuration rules in this document remain in force.

> **Revised (TASK-127):** the section 4 lifecycle wording now follows the settled archive lifecycles: documents `active -> archived` with no restore (plus configured custom statuses, front-matter only), UI mockups `pending -> approved -> archived` (supersede, no restore), milestones `open -> closed` via `papyrus milestone close`, releases `planned -> released` with no archive, decisions unchanged (Nygard status `deprecated`). Sources: ADR-0022, ADR-0023, REQ-003.001, REQ-003.002, REQ-005.007, REQ-007.001.

> **Revised (TASK-147, 2026-11-15):** the scope bullet and section 3 clause 3 no longer assert that every record lifecycle operation is CLI-only, because the 2026-11-15 revision of REQ-006.013 gives the Backlog board task status change and the per-milestone `Clean up all` archive action. This page still offers no lifecycle operation, and creating, restoring, and deleting records remain CLI-only.
## Purpose

Add a read-only page to the offline, project-local web interface that describes every managed artifact/document type and its lifecycle, per REQ-006.003. The page is a single static view with one section per fixed record type; task states are derived from the configured task statuses (REQ-001.002); every other type uses the lifecycle documented for that type. It advances NFR-001.002 (display all Papyrus content) by giving contributors a single place to learn what each record type is and how it moves through its states.

## Scope

### In scope

- A new page route that serves the lifecycle page, reusing the shared shell, sidebar, header, footer, CSP, escaping, and responsive rules of the baseline (TASK-6, ADR-0005/ADR-0006).
- A persistent sidebar entry labelled `Managed artifacts & lifecycles`, placed at the end of the document-type list, that links to the page and carries `aria-current="page"` when active.
- A page body with one section (`h2`) for each of the twelve fixed record types, in this order: Tasks, Milestones, Releases, Decisions, Business requirements, Non-functional requirements, Glossary terms, UI specifications, Technical specifications, UI mockups, Business rules, Audit records.
- A task section whose state list and transitions are derived from the configured task statuses (REQ-001.002).
- A lifecycle sentence for every other section that uses the documented lifecycle for that type (see section 4).
- An explicit statement on the page that Backlog.md `docs`, `drafts`, and project configuration files are not covered.
- Unit/integration tests extending `tests/browser_cli.rs` (or a sibling file) for the new page.

### Out of scope

- Listing, viewing, editing, creating, archiving, deleting, or searching any record from this page; no lifecycle operation is offered here, and the browser's task lifecycle operations (status change and the board's `Clean up all` action of REQ-006.013) live on the Backlog board. Creating, restoring, and deleting records remain CLI-only.
- Any API endpoint for the page; the page is HTML only.
- Adding a new record type, changing configured `document_types`, or deriving the covered set from configuration. The covered set is fixed by REQ-006.003 and does not change with `document_types`.
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
3. The paragraph also states that the states and transitions are derived from the configured task statuses (REQ-001.002). It does not claim that task lifecycle operations are CLI-only, because the Backlog board offers status change and the per-milestone `Clean up all` archive action (REQ-006.013); creating, restoring, and deleting tasks remain CLI-only and are not described as browser operations on this page.
4. Only the configured status names are rendered. No per-state description is required for custom statuses; the configured order is the transition order.
5. When the configured status list is empty or invalid, the server falls back to its configured/default status handling rather than rendering an empty task section; the task section never renders an empty state list.

### 4. Lifecycle wording for other types

Each section's lifecycle paragraph uses the documented lifecycle for that type:

| Section | Lifecycle paragraph |
| --- | --- |
| Milestones | `Lifecycle: open → closed. The only transition is open → closed through papyrus milestone close <id>; milestones are not archived and have no restore. Closing a milestone is separate from task workflow status.` |
| Releases | `Lifecycle: planned → released. released is delivery completion, distinct from archiving; releases are not archived. The transition is performed explicitly via papyrus release close <id>.` |
| Decisions | `Lifecycle: proposed → accepted \| rejected; accepted → deprecated \| superseded.` |
| Business requirements | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| Non-functional requirements | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| Glossary terms | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| UI specifications | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| Technical specifications | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| UI mockups | `Lifecycle: pending → approved → archived. Archived is reached by archiving or by supersession via ui-mockup supersede; there is no restore.` |
| Business rules | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |
| Audit records | `Lifecycle: active → archived. Archiving sets status to archived and moves the file to archived/; there is no restore. Plus any configured custom statuses, which are front-matter values only.` |

1. The wording above is normative for this page and is taken from REQ-003.001, REQ-003.002, REQ-004.001, REQ-005.006, REQ-005.007, REQ-005.008, and REQ-007.001, with the lifecycle operations of ADR-0022 and the layout contract of ADR-0023. It intentionally differs from the shorthand sentences in MOCK-0010 where those conflict with the documented lifecycles.
2. The page does not state that a release becomes released when its milestone set is complete, that a milestone closes when its tasks are complete, that documents move through a Draft → Approved phase, or that audit records are immutable; none of those is documented.
3. All lifecycle sentences are static text; they are not derived from project data.

### 4a. Per-artifact descriptions and read-only configuration (TASK-86)

1. Description text is curated content shipped with the browser binary, written for the page, available offline (NFR-001.003), and not derived from requirement files, record content, or configuration.
2. Each of the twelve fixed sections includes a good description of its artifact on top of its lifecycle and existing status.
3. Each section shows read-only effective configuration: the artifact's `enabled` state, its ID prefix, and its storage folder. The task section additionally shows the configured statuses.
4. Configuration values on this page are display-only. Changes are made only through the configuration view (section 4d of `browser-requirements.md`).
5. The page points to the configuration view for changes, and does not offer editing.
6. The covered set remains fixed and is not derived from configured `document_types`.

### 5. Exclusions statement

1. The subtitle (section 1.5) states that Backlog.md `docs`, `drafts`, and project configuration files are not covered.
2. No section, heading, link, or text on the page describes Backlog.md `docs`, `drafts`, or project configuration files.

### 6. Rendering, security, and accessibility

1. All dynamic values (project name, bind note, configured status names) are HTML-escaped before rendering. Status names are rendered as text, never as markup.
2. The page carries the strict response class fixed in `browser-requirements.md` §9.7 (the same CSP and security headers as every other HTML response).
3. Bundled, same-origin fonts apply only inside rendered math; the page-level font stack stays the system stack, and styles come from `/assets/app.css`. The page loads no external stylesheet, script, font, image, or network resource.
4. The page works with JavaScript disabled.
5. Focus order follows document order; every interactive control (the header navigation toggle on every page, sidebar links) has a visible label or accessible name.
6. The header navigation toggle collapses and expands the sidebar at every width; the default is expanded, the choice persists in `papyrus.nav`, and without JavaScript the navigation stays expanded and usable (`browser-shell-ui.md` §5).

### 7. Tests

1. An integration test starts `papyrus browser --no-open` on a free port and asserts:
   - `GET /lifecycle` returns `200` with `Content-Type: text/html; charset=utf-8`;
   - the response body contains the `h1` text `Managed artifacts and lifecycles`;
   - the response body contains all twelve section headings in order;
   - the response body contains the configured default status chain `ToDo → Initiated → Refined → Implemented → Validated → Done`;
   - the response body contains the exclusion statement;
   - the response body contains the sidebar entry `Managed artifacts &amp; lifecycles`;
   - the response contains no `http://`, `https://`, or protocol-relative reference, and every `url(` in a served stylesheet resolves to an asset-table entry.
2. The same test asserts `GET /?type=lifecycle` returns the same page (alias).
3. The same test asserts `POST /lifecycle` returns `405` with an `Allow` header.
4. A second test (or a parameterised case) initialises a project with a custom status list (for example `["Backlog", "Doing", "Done"]`) and asserts the task section renders that exact chain and does not render the default chain.
5. A third test (TASK-95) initialises a project with a disabled document type (`document_types.<key>.enabled: false`) and asserts `GET /lifecycle` returns `200`, still renders twelve sections, and shows `enabled: false` in that section's read-only configuration line without an error.
6. A fourth test (TASK-95) renames a document type's ID prefix and storage folder, plus the base document directory, and asserts the rendered read-only configuration line shows the new prefix and joined folder while the section's curated description text stays unchanged.
7. The TASK-95 tests also assert the read-only configuration line is display-only: the section links to `/config` for changes and the page contains no form or input editing control.

## Assumptions

- Assumption: The page is read-only and renders no project records; it does not call into the record index or any store.
- Assumption: The route is `/lifecycle` with `/?type=lifecycle` as an alias, matching the UI description's recommended default (OQ-2) and keeping the sidebar/filter model consistent.
- Assumption: The sidebar entry is always present and is not affected by `document_types.*.enabled`, because the page describes a fixed set independent of configuration.
- Assumption: Lifecycle wording uses the documented lifecycles from requirements/README/BR (per user selection during refinement), not the shorthand sentences in MOCK-0010 where those conflict.
- Assumption: Custom task statuses have no per-state descriptions; only their names and order are rendered.
- Assumption: No per-state one-line descriptions are added beyond the single lifecycle sentence per section; MOCK-0010 OQ-6 is resolved to one sentence per section.

## Open questions

- None. Material behavior (covered set, task-status derivation, lifecycle wording, route, exclusions) is resolved by REQ-006.003, the approved MOCK-0010 layout, and the user's selection of documented lifecycle wording during refinement.

## Related artifacts

- Requirement: `docs/requirements/business/REQ-006.003.md` (primary); `docs/requirements/business/REQ-006.006.md`; `docs/requirements/business/REQ-006.007.md`; `docs/requirements/business/REQ-006.008.md`; `docs/requirements/business/REQ-006.009.md`; `docs/requirements/business/REQ-006.010.md`; `docs/requirements/business/REQ-006.011.md`; `docs/requirements/business/REQ-006.012.md`; `docs/requirements/business/REQ-001.002.md`; `docs/requirements/business/REQ-001.003.md`; `docs/requirements/business/REQ-003.001.md`; `docs/requirements/business/REQ-003.002.md`; `docs/requirements/business/REQ-004.001.md`; `docs/requirements/business/REQ-005.006.md`; `docs/requirements/business/REQ-005.007.md`; `docs/requirements/business/REQ-005.008.md`; `docs/requirements/business/REQ-007.001.md`; `docs/requirements/non-functional/NFR-001.002.md`; `docs/requirements/non-functional/NFR-001.003.md`
- UI description: `docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md`
- UI mockup: `docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html` (approved)
- Baseline browser spec: `docs/specifications/browser-requirements.md`
- Task status source: `docs/business-rules/BR-001-papyrus-command-behavior.md`, `docs/business-rules/BR-006-milestone-command-behavior.md`, `docs/business-rules/BR-007-release-command-behavior.md`, `docs/business-rules/BR-008-papyrus-decision-command-behavior.md`
- Milestone: `m-12`
- Dependency: TASK-66 (Done), TASK-86
- Implementation: TASK-95 (per-artifact descriptions and read-only configuration, section 4a)
- ADR: ADR-0010, ADR-0012, ADR-0013
path: /home/lyly/Documents/Dev/papyrus/docs/specifications/browser-managed-artifacts-and-lifecycles.md
