---
id: TECH-061
title: "Browser Specification Revisions (TASK-86)"
created_date: '2026-11-16T17:46:52Z'
updated_date: '2026-11-18T22:26:40Z'
labels: []
references:
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - MOCK-0001
  - MOCK-0004
  - MOCK-0010
  - MOCK-0013
  - NFR-001.002
  - NFR-001.003
  - NFR-002.002
  - NFR-004
  - NFR-004.001
  - NFR-004.002
  - NFR-005.001
  - REQ-001.002
  - REQ-001.003
  - REQ-001.005
  - REQ-005.010
  - REQ-006.002
  - REQ-006.003
  - REQ-006.004
  - REQ-006.006
  - REQ-006.007
  - REQ-006.008
  - REQ-006.009
  - REQ-006.010
  - REQ-006.011
  - REQ-006.012
  - TASK-86
  - TASK-87
  - TASK-93
  - TASK-94
  - TASK-95
  - ADR-034
  - REQ-006.018
  - RE-005
status: draft
---
# TECH-061 Browser Specification Revisions (TASK-86)

> **Revised (TASK-201, 2026-11-18):** the configuration view (section 3), the `/config` route and `Configuration` sidebar entry, the read-only lifecycle configuration clauses (section 8, items 3-5), and the related mockup and open-question items are retired per ADR-034 and REQ-006.018; configuration editing is CLI-only and the browser serves no configuration or lifecycle page. The retired clauses are retained below as history. See TECH-080.

## Description
## Purpose

Revise the browser specification corpus so that it states the new browser contract introduced by REQ-006.003 (as extended) and REQ-006.006 through REQ-006.012: non-functional requirements grouped by their owning group, an error-only console with a browser-only debug flag, an editable configuration view (retired per ADR-034; configuration editing is CLI-only), shared-header branding, record references as links, a concise glossary landing page, a consistent single record view layout, and curated per-artifact descriptions with read-only configuration on the lifecycle page (the browser lifecycle page and its read-only configuration display are retired per ADR-034). The revisions also remove any remaining Backlog.md storage-ID wording from browser specs so that routes, links, form values, and JSON carry Papyrus IDs only (ADR-0010, ADR-0012, ADR-0013).

This ticket is documentation/specification work only. It does not change requirements, business rules, or implementation code. Implementation of the revised contract is delivered by TASK-93, TASK-94, and TASK-95; UI approval is delivered by TASK-87.

## Scope

### In scope

- Revising `docs/specifications/browser-requirements.md` for NFR grouping, console verbosity, the configuration view (retired per ADR-034), shared header branding, reference links, and the shared single record view layout.
- Revising `docs/specifications/browser-glossary-terms.md` for the glossary landing page and the single record view layout.
- Revising `docs/specifications/browser-managed-artifacts-and-lifecycles.md` for curated per-artifact descriptions and read-only per-section configuration (the read-only configuration display is retired per ADR-034; the document stays outside the browser).
- Revising every other browser specification that renders a record view or a record reference so that it references the shared single record view layout and the shared reference-link rule, and contains no Backlog.md storage-ID wording.
- Recording which mockup packages require revision or creation.
- Updating traceability sections in the revised specifications.

### Out of scope

- Implementing the revised browser behavior (TASK-93, TASK-94, TASK-95).
- Designing or approving UI mockups (TASK-87, `@ui-designer`).
- Changing requirements (`docs/requirements/`) or business rules (`docs/business-rules/`).
- Changing CLI, server, or template implementation.
- Creating, deprecating, restoring, or renaming records.

## Behavior

### 1. Non-functional requirement grouping (AC #1)

The revised `browser-requirements.md` must state all of the following:

1. Non-functional atomic requirements are listed under their owning group entry, matching the grouping used for business requirements. There is no trailing catch-all subsection.
2. The owning group is derived in the browser from the requirement ID prefix: `REQ-XX.YYY` -> `REQ-XX`, `NFR-XX.YYY` -> `NFR-XX`. No `Group` field is added to non-functional atomic requirement files and the requirement file format is unchanged.
3. A group heading shows the group record's title (for example `NFR-004 Persistence`). When the group record is missing, the group number is used (`NFR-004`).
4. A group with no atomic requirements still renders as a group row with no member rows; the shared section count note is computed from the rows actually emitted.
5. Every atomic requirement appears under exactly one group; none is dropped.
6. The JSON index (`/api/requirements`, `type=nfr`) is ordered by group and then by requirement ID, consistent with the list page, and carries the resolved group title or `null` when the group record is missing (falling back to the group number in the rendered HTML).

### 2. Console verbosity (AC #2)

The revised `browser-requirements.md` must state all of the following:

1. By default the browser command outputs only errors to the console.
2. `--debug` is an option of the `browser` command only. It is not a global option and is not accepted by any other command.
3. With `--debug`, each received request is logged as exactly one line containing the method, the path, the response status, and the duration. Request headers and bodies are never logged.
4. Errors are output whether or not `--debug` is set.
5. The man page and help text are updated to document `--debug` as browser-only.

### 3. Configuration view (AC #3) (retired)

> **Retired (TASK-201, 2026-11-18):** per ADR-034 and REQ-006.018 the browser serves no configuration view; `GET /config`, `GET /config/edit`, `POST /config`, and `GET /?type=config` return `404`, and there is no `Configuration` sidebar entry. Clauses 1-8 below record the superseded requirement and are retained as history.

The revised browser specification must state all of the following:

1. The browser serves a configuration view for the current project.
2. Editing uses `papyrus config get/set`, so folder changes are relocated by the single relocation routine (REQ-001.002, ADR-0010 decision 8). The browser never moves a folder itself.
3. No path outside the project directory can be defined; every path passes `Context::ensure_within`.
4. The view shows effective configuration values, including values that come from built-in defaults, and marks which keys are explicitly set in `papyrus.config.yml`.
5. Editable keys are exactly the keys `papyrus config set` accepts. The view applies the same keys, validation, and precedence as the CLI and cannot persist a key or value the CLI would reject (REQ-001.003).
6. No configuration key is treated as secret in the current configuration, and the view never reads or displays environment-variable values as editable configuration.
7. The view links to the lifecycle page for per-artifact read-only configuration and does not duplicate editing there.
8. The route, sidebar entry, and form encoding are design decisions recorded in the revised specification (recommended default: `/config`, with `/?type=config` as an alias; subject to TASK-87 approval).

### 4. Shared header branding (AC #4)

The revised `browser-requirements.md` shared-shell section must state all of the following, and every per-type browser specification that describes the shared shell must reference it rather than restating it:

1. On every browser page, the project name renders large and bold.
2. The "Papyrus" mention renders in smaller letters.
3. The "P" mark keeps its current size and styling.
4. The project name shown is the configured project name (REQ-001.002).
5. The rule applies to the shared header of every page.

### 5. Record references as links (AC #5)

The revised browser specifications must state all of the following:

1. Every rendered record reference that resolves to a Papyrus record is a hyperlink: task dependencies, a task's milestone, `references` values, and a sub-requirement's owning group.
2. Activating a link navigates to the referenced record's view; the browser back button returns to the originating page.
3. A reference that does not resolve to an existing record is rendered as plain text and raises no error.
4. Links are generated in the page and work offline (NFR-001.003).
5. Unresolved references are not styled as errors and do not produce 404 responses.

### 6. Glossary landing page (AC #6)

The revised `browser-glossary-terms.md` must state all of the following:

1. The glossary landing (list) page omits created and updated dates.
2. Labels are shown as a list of tags on the glossary landing page, using the same taglist presentation as the single record view layout (section 7).
3. References are not shown on the glossary landing page.
4. Glossary detail views are unchanged by this requirement and follow the single record view layout (section 7).
5. Other document-type landing pages are unchanged.

### 7. Single record view layout (AC #7)

The revised `browser-requirements.md` must define a shared single record view layout, and every per-type browser specification that has a single record view must state that it follows this layout in addition to its type-specific fields:

1. A summary card at the beginning of the view shows only the record ID, the created date, and the updated date.
2. A sub-requirement additionally shows its owning group in the summary card, and nothing else is added to the card.
3. Dependencies and references appear at the end of the view as collapsible sections, each shown only when the record has at least one such value.
4. Collapsible sections use native HTML disclosure elements and work with JavaScript disabled (NFR-005.001).
5. Labels are shown as a taglist under the title.
6. The layout applies to every managed Markdown record type that has a single record view. UI mockup HTML views are excluded.
7. The summary card does not show status, milestone, assignee, labels, or references; those remain in their existing locations outside the card, except labels which move to the taglist under the title.

### 8. Per-artifact descriptions and read-only configuration (AC #8) (read-only configuration display retired)

The revised `browser-managed-artifacts-and-lifecycles.md` must state all of the following:

1. Description text is curated content shipped with the browser binary, written for the page, available offline (NFR-001.003), and not derived from requirement files, record content, or configuration.
2. Each of the twelve fixed sections includes a good description of its artifact on top of its lifecycle and existing status.
3. **Retired (ADR-034):** each section shows read-only effective configuration: the artifact's `enabled` state, its ID prefix, and its storage folder. The task section additionally shows the configured statuses.
4. **Retired (ADR-034):** configuration values on this page are display-only. Changes are made only through the CLI (`papyrus config set/unset`); the former configuration view (section 3) is retired.
5. **Retired (ADR-034):** the page points to the configuration view for changes, and does not offer editing.
6. The covered set remains fixed and is not derived from configured `document_types`.

### 9. Papyrus IDs and mockup identification (AC #9)

The revised browser specifications must satisfy all of the following:

1. Routes, links, form values, and JSON use Papyrus IDs only. No specification text refers to Backlog.md storage IDs, offsets, or translation at any boundary (ADR-0010, ADR-0012, ADR-0013).
2. Every revised specification's traceability section cites REQ-006.003, REQ-006.006, REQ-006.007, REQ-006.008, REQ-006.009, REQ-006.010, REQ-006.011, REQ-006.012, and ADR-0010, ADR-0012, ADR-0013 as applicable.
3. The mockup impact is recorded:
   - Existing packages that render record views or shared shells (MOCK-0001 through MOCK-0013) require revision for the shared header branding, the single record view layout, and reference links. Revision does not renumber or replace a MOCK id.
   - MOCK-0010 additionally requires revision for per-artifact descriptions and read-only configuration on the lifecycle page (the read-only configuration display is retired per ADR-034).
   - MOCK-0001 additionally requires revision for NFR grouping under owning groups.
   - MOCK-0004 additionally requires revision for the glossary landing page (dates omitted, labels as tags, no references).
   - A new mockup package is required for the consolidated single record view layout, branding, links, grouping, and glossary landing; the configuration-view and lifecycle per-artifact configuration parts are retired per ADR-034. It is created and approved under TASK-87 and receives the next sequential MOCK id. No package is approved for implementation until the user explicitly approves it.

## Target specifications

| Specification | Required change |
|---|---|
| `docs/specifications/browser-requirements.md` | NFR grouping; console verbosity; configuration view; shared header branding; reference links; single record view layout; ID-only wording. |
| `docs/specifications/browser-glossary-terms.md` | Glossary landing page; single record view layout reference. |
| `docs/specifications/browser-managed-artifacts-and-lifecycles.md` | Curated descriptions; read-only per-section configuration (retired per ADR-034). |
| `docs/specifications/browser-tasks-milestones-and-releases.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-decision-records.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-ui-specifications.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-ui-mockups.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-business-rules.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-audit-records.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-technical-specifications.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-use-cases-test-cases-and-test-reports.md` | Single record view layout reference; reference links. |
| `docs/specifications/browser-agent-skills-and-workflows.md` | Shared header branding reference; ID-only wording. |

## Assumptions

- Assumption: This ticket edits specification documents only; no code, requirements, or business rules are changed.
- Assumption: The shared single record view layout is defined once in `browser-requirements.md` and referenced by per-type specifications, rather than duplicated.
- Assumption: The configuration view route is a design decision; the recommended default is `/config` with `/?type=config` as an alias, subject to TASK-87 approval. If a different route is chosen, the revised specification records it and no other behavior changes. Retired per ADR-034: the browser serves no configuration view.
- Assumption: Existing MOCK ids are not renumbered; revisions are new files or in-place approved updates per the registry's normal process.
- Assumption: No new ADR is required; the revisions follow ADR-0010, ADR-0012, and ADR-0013.
- Assumption: Standard documentation definition of done applies.

## Open questions

- OQ-1 (recommended default: sidebar entry labelled `Configuration` linking to `/config`) — Whether the configuration view is reached from the sidebar or from a header link is a design decision. The recommended default is a sidebar entry after the document types, consistent with the lifecycle and skills pages. Subject to TASK-87. Closed by ADR-034: no configuration view, `/config` route, or sidebar entry exists.
- OQ-2 (recommended default: per-type specifications reference the shared layout by section number) — Whether each per-type specification restates the shared single record view layout or only references it. Restating risks drift; referencing is preferred. Subject to reviewer preference during refinement review.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-006.003, REQ-006.006, REQ-006.007, REQ-006.008, REQ-006.009, REQ-006.010, REQ-006.011, REQ-006.012, REQ-006.002, REQ-006.004, REQ-005.010, REQ-001.002, REQ-001.003, REQ-001.005, NFR-001.002, NFR-001.003, NFR-002.002, NFR-004.001, NFR-004.002, NFR-005.001 |
| ADR | ADR-0010, ADR-0012, ADR-0013 |
| Ticket | TASK-86; downstream TASK-87, TASK-93, TASK-94, TASK-95 |
| UI mockups | MOCK-0001..MOCK-0013 (revision required); new package under TASK-87 |
