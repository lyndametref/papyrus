---
id: TASK-86
title: >-
  Revise the browser specifications for requirements grouping, verbosity,
  configuration view, branding, links, glossary landing, record layout, and
  per-artifact configuration
status: To Do
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-11T20:57:00Z'
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
priority: high
type: docs
ordinal: 39030
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.003 was extended and REQ-06.006 to REQ-06.012 were added on 2026-11-11. The browser specifications still describe a catch-all Other requirements section, verbose console output, configuration that cannot be edited from the browser, an undifferentiated record view, plain-text dependencies, and a glossary landing page with dates and references. Implementation cannot start on those surfaces without a specification and an approved mockup, so this task states the new browser contract and identifies the mockup work.
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
