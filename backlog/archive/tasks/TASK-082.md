---
id: TASK-082
title: >-
  Design review: browser configuration view, single record view layout, and
  lifecycle configuration
status: Done
assignee: []
created_date: '2026-11-11T20:42:00Z'
updated_date: '2026-11-12T16:10:00Z'
labels:
  - design-review
  - browser
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-081
references:
  - docs/specifications/ui/mockups/README.md
  - docs/requirements/business/REQ-06.008.md
  - docs/requirements/business/REQ-06.012.md
  - docs/requirements/business/REQ-06.003.md
  - >-
    docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html
  - >-
    docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md
  - docs/requirements/business/REQ-06.006.md
  - docs/requirements/business/REQ-06.007.md
  - docs/requirements/business/REQ-06.009.md
  - docs/requirements/business/REQ-06.010.md
  - docs/requirements/business/REQ-06.011.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - docs/specifications/browser-glossary-terms.md
  - docs/specifications/browser-spec-revisions-task-86.md
  - >-
    docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md
priority: high
type: task
ordinal: 41624
---


## Description
Design the approved clickable UI review package for the revised browser contract introduced by REQ-06.003 and REQ-06.006 through REQ-06.012: an editable configuration view, a consistent single record view layout, curated per-artifact descriptions with read-only configuration on the lifecycle page, non-functional requirement grouping under owning groups, shared header branding, record references as links, and a concise glossary landing page.

This ticket is UI design work only. It does not change requirements, business rules, specifications, or implementation code; the revised browser contract is already specified in docs/specifications/browser-requirements.md (sections 4a-4d, 10a), docs/specifications/browser-managed-artifacts-and-lifecycles.md (section 4a), and docs/specifications/browser-glossary-terms.md (section 3a) by TASK-86. The feature specification is docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md.

Surfaces the package must cover (MOCK-0014):
- Configuration view: effective values with explicit/default markers, editable keys matching papyrus config set, folder relocation via config set, no path outside the project, link to the lifecycle page, route /config with /?type=config alias, sidebar entry "Configuration".
- Single record view layout: summary card (ID, created date, updated date; sub-requirement adds owning group), labels as taglist under the title, dependencies/references as native disclosure sections shown only when present.
- Shared header branding: project name large bold, "Papyrus" smaller, "P" mark unchanged, on every page.
- NFR grouping: non-functional requirements under owning group (ID prefix), group title or number fallback, empty group "No requirements", no catch-all section.
- Dependency and reference links: dependencies, milestone, references, owning group as links; unresolved references as plain text; works offline.
- Glossary landing page: no created/updated dates, labels as tags, no references; detail views follow single record layout.
- Lifecycle page per-artifact configuration: one section per fixed record type with curated description plus read-only effective configuration (enabled, ID prefix, storage folder; task section also statuses), link to configuration view.
- 320 px responsive layouts, error pages, accessibility annotations.

Assumptions:
- The revised browser contract is already specified by TASK-86; this ticket only designs the UI for it.
- MOCK-0014 is the next sequential global MOCK id and is never reused or renumbered.
- The mockup stays draft until the user explicitly approves it; only then may the registry be updated to approved and implementation begin.
- Existing MOCK ids are not renumbered; revisions to MOCK-0001..MOCK-0013 are separate follow-ups and do not block this package.
- Configuration view route/sidebar placement defaults (browser-requirements.md section 4d OQ-1) are subject to approval during this review.

Dependencies:
- TASK-86 (completed): revised browser specifications.

Downstream consumers (not dependencies of this ticket): TASK-93, TASK-94, TASK-95. They may not start until MOCK-0014 is approved.

## Acceptance Criteria
- [x] A new numbered mockup package exists under docs/specifications/ui/mockups/ with a review landing and clickable prototype covering the configuration view, the single record view layout, the lifecycle page per-artifact description and read-only configuration, non-functional requirement grouping, the shared header branding, dependency and reference links, and the glossary landing page.
- [x] The package has a matching UI description document under docs/specifications/ui/ and is registered in the mockup registry with its status.
- [x] The package is submitted for explicit user approval, and the registry records approved before the browser implementation tasks start.
- [x] The approved design covers every REQ-06.003 and REQ-06.006 to REQ-06.012 surface listed above and reuses the existing wireframe styles.

## Definition of Done
- [x] #1 Mockup package passes docs/specifications/ui/mockups/validate_mockups.sh with no errors for MOCK-0014
- [ ] #2 UI description document exists and is registered in the mockup registry with status draft
- [x] #3 Registry status updated to approved only after explicit user approval
- [x] #4 Every REQ-06.003 and REQ-06.006 to REQ-06.012 surface is represented in the prototype and UI description
- [x] #5 Feature specification exists and links the MOCK id, UI description, and requirement/spec references
