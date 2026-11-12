---
id: TASK-86
title: >-
  Revise the browser specifications for requirements grouping, verbosity,
  configuration view, branding, links, glossary landing, record layout, and
  per-artifact configuration
status: Done
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-12T07:11:00Z'
labels:
  - specification
  - browser
  - release-1.0
milestone: m-16
dependencies: []
references:
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.006.md
  - docs/requirements/business/REQ-06.007.md
  - docs/requirements/business/REQ-06.008.md
  - docs/requirements/business/REQ-06.009.md
  - docs/requirements/business/REQ-06.010.md
  - docs/requirements/business/REQ-06.011.md
  - docs/requirements/business/REQ-06.012.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - docs/specifications/browser-glossary-terms.md
  - docs/specifications/browser-spec-revisions-task-86.md
priority: high
type: docs
ordinal: 39030
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Revise the browser specification corpus for the Release 1.0 contract changes introduced by REQ-06.003 (extended) and REQ-06.006 through REQ-06.012: NFR grouping under owning groups; error-only console with a browser-only --debug flag; editable configuration view; shared-header branding; record references as links; concise glossary landing page; shared single record view layout; curated per-artifact descriptions and read-only configuration on the lifecycle page; Papyrus IDs only in routes, links, form values, and JSON. Specification work only; no code, requirements, or business rules change. Implementation is delivered by TASK-93, TASK-94, and TASK-95; UI approval by TASK-87. Feature spec: docs/specifications/browser-spec-revisions-task-86.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The requirements browser specification places non-functional atomic requirements under their owning group derived from the ID prefix, shows the group record title, falls back to the group number when the group record is missing, renders empty groups as entries with no children, and has no Other requirements catch-all.
- [ ] #2 A browser specification defines error-only console output by default and a browser-command-only --debug flag that logs one line per received request with method, path, response status, and duration and never logs headers or bodies.
- [ ] #3 A browser specification defines the configuration view: effective values including built-in defaults, explicitly set keys marked, editable keys exactly those config set accepts with the same validation and precedence, no path outside the project, no environment-variable value shown as editable, and edits delegated to config set so folder changes are relocated.
- [ ] #4 The shared-header specification states that the project name renders large and bold, the Papyrus mention renders smaller, and the P mark keeps its current size on every browser page.
- [ ] #5 The browser specifications state that every rendered record reference (task dependencies, a task milestone, references values, and a sub-requirement owning group) is a link to the referenced record, that an unresolved value renders as plain text without an error, and that links work offline.
- [ ] #6 The glossary browser specification states that the landing page omits created and updated dates and references and shows labels as tags, and that glossary detail views follow the single record view layout.
- [ ] #7 A browser specification defines the single record view: a summary card with only the record ID, created date, and updated date, plus the owning group for a sub-requirement; labels as a taglist under the title; and dependencies and references at the end as native disclosure sections shown only when the record has at least one value.
- [ ] #8 The managed artifacts and lifecycle browser specification defines curated per-artifact description text shipped with the binary and available offline, plus read-only effective configuration per section (enabled, ID prefix, storage folder, and configured statuses for tasks) that points to the configuration view for changes.
- [ ] #9 Revised browser specifications use Papyrus IDs only in routes, links, form values, and JSON, and identify which mockup packages need revision or creation.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All target browser specifications revised and internally consistent.
- [ ] #2 Shared single record view layout and reference-link rule are defined once in browser-requirements.md and referenced by per-type specs.
- [ ] #3 NFR grouping, console verbosity, configuration view, branding, links, glossary landing, and per-artifact config are stated in the relevant specs.
- [ ] #4 No Backlog.md storage-ID wording remains in revised browser specs.
- [ ] #5 Traceability sections cite REQ-06.003, REQ-06.006 through REQ-06.012, ADR-0010, ADR-0012, and ADR-0013.
- [ ] #6 Mockup impact (revisions and new package under TASK-87) is recorded.
- [ ] #7 Requirements and business rules files are unchanged.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Applied browser specification revisions per docs/specifications/browser-spec-revisions-task-86.md. Updated browser-requirements.md (NFR grouping, console verbosity, configuration view, shared header branding, reference links, single record view layout), browser-glossary-terms.md (glossary landing, single record view layout, reference links), browser-managed-artifacts-and-lifecycles.md (curated descriptions, read-only per-section config), and all other browser specs (shared layout/reference links, ID-only wording, traceability). Neutralized storage-ID wording.
<!-- SECTION:NOTES:END -->
