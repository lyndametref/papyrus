---
id: TASK-93
title: >-
  Apply the record view layout, reference links, NFR grouping, glossary landing,
  and title branding in the browser
status: Implemented
assignee: []
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-13T06:38:00Z'
labels:
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-86
  - TASK-87
  - TASK-88
references:
  - docs/requirements/business/REQ-06.006.md
  - docs/requirements/business/REQ-06.009.md
  - docs/requirements/business/REQ-06.010.md
  - docs/requirements/business/REQ-06.011.md
  - docs/requirements/business/REQ-06.012.md
  - docs/requirements/business/REQ-01.005.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-glossary-terms.md
  - >-
    docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md
  - >-
    docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html
  - docs/specifications/browser-spec-revisions-task-86.md
priority: high
type: feature
ordinal: 39100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.006, REQ-06.009, REQ-06.010, REQ-06.011, and REQ-06.012 define presentation rules the browser does not yet implement: non-functional requirements sit in a catch-all section, record metadata is spread across the page, references are plain text, the glossary landing page shows dates and references, and the header emphasizes Papyrus instead of the project. These changes share the rendering layer and the approved mockup MOCK-0014, so they are delivered together.

Scope (in scope):
- Apply the shared single record view layout from browser-requirements.md §4c to every managed Markdown record type with a single record view: a summary card showing only record ID, created date, and updated date (plus the owning group for a sub-requirement); labels as a taglist under the title; dependencies and references at the end as native HTML disclosure sections shown only when the record has at least one value.
- Render task dependencies, a task's milestone, references values, and a sub-requirement's owning group as local hyperlinks to the referenced record; render unresolved values as plain text without an error; work offline.
- List non-functional atomic requirements under their owning group derived from the ID prefix, with the group title or group number fallback, graceful empty groups, and no "Other requirements" catch-all.
- Adjust the glossary landing page to omit created/updated dates and references and show labels as tags; glossary detail views follow the shared record view layout.
- Rebrand the shared header on every page so the configured project name is large and bold and the Papyrus mention is smaller, leaving the P mark unchanged.
- Ensure routes, links, form values, and JSON carry Papyrus IDs only, with no Backlog.md storage-ID wording or translation.

Out of scope (handled by other tasks):
- Configuration view editing (TASK-94).
- Per-artifact descriptions and read-only configuration on the lifecycle page (TASK-95).
- Console verbosity / debug logging (TASK-96).
- Legacy serializer and browser write-path removal (TASK-97).

The approved design authority is MOCK-0014 and the revised browser specifications from TASK-86. No new MOCK id is allocated and no UI is invented outside MOCK-0014.

Assumptions:
- Assumption: MOCK-0014 (approved 2026-11-12 UTC) is the sole visual authority for these five surfaces; existing MOCK-0001..MOCK-0013 revisions are separate follow-ups and do not block this task.
- Assumption: disclosure sections are collapsed by default per the recommended default OQ-4, unless MOCK-0014 specifies otherwise.
- Assumption: the configured project name comes from project configuration (REQ-01.002) and is already available to the shared header.
- Assumption: browser record indexes already resolve records by Papyrus ID, so reference links use /doc/{id} with no storage-ID translation.
- Assumption: NFR grouping is derived in the browser from the ID prefix only; no Group field is added to requirement files.

Open questions:
- OQ-1: whether the approved MOCK-0014 requires any disclosure section to default open; follow the mockup if it differs from the collapsed-by-default recommendation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Every single record view shows a summary card with only the record ID, created date, and updated date, plus the owning group for a sub-requirement.
- [ ] #2 Labels render as a taglist under the title, and dependencies and references render at the end as native disclosure sections that appear only when the record has at least one value and work with JavaScript disabled.
- [ ] #3 Task dependencies, a task milestone, references values, and a sub-requirement owning group render as links to the referenced record; an unresolved value renders as plain text without an error; links are local and work offline.
- [ ] #4 Non-functional atomic requirements are listed under their owning group with the group title, a missing group falls back to the group number, an empty group still renders, and no Other requirements catch-all remains.
- [ ] #5 The glossary landing page omits created and updated dates and references and shows labels as tags, while glossary detail views follow the record view layout.
- [ ] #6 The shared header renders the configured project name in large bold type and the Papyrus mention smaller on every page, leaving the P mark unchanged.
- [ ] #7 Routes, links, form values, and JSON carry Papyrus IDs only.
- [ ] #8 Tests or fixtures cover the group derivation, unresolved references, disclosure sections, and header rendering.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation follows approved MOCK-0014 for all five surfaces; no UI is invented outside the approved mockup.
- [ ] #2 cargo build succeeds with no warnings introduced by this change.
- [ ] #3 cargo test passes, including new or updated browser tests for group derivation, unresolved references, disclosure sections, header rendering, glossary landing, NFR grouping, and Papyrus-ID-only routing.
- [ ] #4 cargo clippy --all-targets -- -D warnings and cargo fmt --check pass.
- [ ] #5 No requirement or business-rule files were modified.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review MOCK-0014 and the revised browser specifications (browser-requirements.md sections 4a-4c and 10a, browser-glossary-terms.md section 3a, browser-spec-revisions-task-86.md section 9) to fix the exact markup, states, and responsive behavior for the five surfaces.
2. Header branding: update the shared header template and CSS so the configured project name renders large and bold, the Papyrus mention renders smaller, and the P mark is unchanged; apply on every page including error pages.
3. Single record view layout: refactor the summary card to show only record ID, created date, and updated date, plus the owning group for a sub-requirement; remove status, milestone, assignee, labels, and references from the card; render labels as a read-only taglist under the title; render dependencies and references at the end as native <details> sections shown only when at least one value exists.
4. Reference links: resolve task dependencies, milestone, references, and owning group through the browser record index; render resolved values as /doc/{id} links and unresolved values as plain text without errors or 404s; verify offline operation.
5. NFR grouping: derive the owning group from the NFR ID prefix; remove the "Other requirements" catch-all for the non-functional category; render group titles or number fallbacks; render empty groups with an explicit empty state and "No requirements" count; update the JSON index ordering and group title (or null) accordingly.
6. Glossary landing: omit created/updated dates and references; render labels as tags; keep glossary detail views on the shared single record view layout; leave other document-type landing pages unchanged.
7. ID-only audit: verify routes, links, form values, and JSON use Papyrus IDs only; remove any remaining Backlog.md storage-ID wording or translation from browser code and specs touched by this task.
8. Tests: add or extend browser tests/fixtures covering group derivation, unresolved references, disclosure sections, header rendering, glossary landing, NFR grouping, and Papyrus-ID-only routing.
9. Verification: cargo build, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check; manual smoke comparison against MOCK-0014.
<!-- SECTION:PLAN:END -->
