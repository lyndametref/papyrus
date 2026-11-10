---
id: TASK-19
title: Complete browser access for all Papyrus content
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-10T19:09:00Z'
labels: []
milestone: m-12
dependencies:
  - TASK-23
  - TASK-24
  - TASK-25
  - TASK-26
  - TASK-60
  - TASK-67
  - TASK-72
  - TASK-77
references:
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/requirements/business/REQ-06.004.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/browser-complete-access.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - docs/specifications/ui/browser-final-cross-type-ui.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
ordinal: 30300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Final integration increment of the offline, project-local web interface: one navigation (sidebar) and one content inventory (landing view at GET /) expose every record type Papyrus manages, reusing the per-type list/view/edit views delivered by TASK-6, TASK-11, TASK-15, TASK-23, TASK-24, TASK-25, TASK-26, TASK-40, TASK-57, TASK-72, and TASK-77. This task does not re-implement per-type rendering; it consolidates those increments into a single entry point and verifies cross-type consistency. It completes NFR-01.002 (display all Papyrus content in the local browser).

Key behavior:
- GET / serves the inventory hub (one card per supported record type in final sidebar order, with active-record count and link); GET /?type=req serves the requirements list (the previous default GET / -> requirements-list behavior is superseded).
- Sidebar final order: Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows. Gated entries omitted when document_types.<key>.enabled is false/absent; the two virtual pages are always present.
- ?type= accepts exactly the documented values plus lifecycle/skills aliases; any other value returns 400.
- Cross-type ID resolution covers all twelve record categories: unique ID resolves; ID present in more than one category returns 500; deprecated or unknown returns 404.
- Deprecated records of every document type are hidden from lists/counts/inventory and unreachable by URL.
- Disabled/absent document types contribute no card, records, or error.
- Front-matter metadata (REQ-06.004) reused unchanged; created_date/updated_date read-only in the browser.
- Baseline security, assets, CSP, Host/Origin, limits, offline, accessibility, and 320 px rules reused unchanged.

UI: approved mockup MOCK-0012 (TASK-60); UI description docs/specifications/ui/browser-final-cross-type-ui.md.

Specification: docs/specifications/browser-complete-access.md
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 GET / renders the inventory hub with one card per supported record type in final sidebar order, showing active-record counts and links for represented types; GET /?type=req renders the requirements list.
- [ ] #2 The persistent sidebar on every page lists entries in final order (Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows), with the active entry marked aria-current="page" and the non-colour cue; gated entries are omitted when disabled.
- [ ] #3 ?type= accepts exactly the documented values (req, nfr, task, milestone, release, decision, glossary, ui-spec, ui-mockup, business-rule, audit-record, tech-spec, plus lifecycle and skills aliases); any other value returns 400.
- [ ] #4 Cross-type ID resolution returns the record for a unique ID, 500 for an ID present in more than one category, and 404 for deprecated or unknown IDs.
- [ ] #5 Deprecated records of every document type are absent from inventory counts and unreachable by URL, returning the per-type 404 page identical for deprecated and unknown IDs.
- [ ] #6 Disabled or absent document types contribute no card, no records, and no error on the inventory or any list.
- [ ] #7 Empty states render for every category and for the inventory without horizontal scrolling for long unbreakable tokens.
- [ ] #8 Front-matter metadata (REQ-06.004) is reused unchanged on every list/detail view; created_date and updated_date remain read-only in the browser.
- [ ] #9 The interface works without JavaScript; inventory cards are plain links; sidebar toggle works at 320 CSS pixels.
- [ ] #10 No external resources are referenced in any response; CSP and security headers unchanged.
- [ ] #11 Build, tests, clippy, and formatting pass with existing dependencies only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Inventory hub integration test covers card order, counts, links, empty state, disabled types, and no external resources.
- [ ] #3 Cross-type integration test creates one record of every supported type and verifies reachability, ID resolution, deprecated/unknown 404, and 500 for duplicate IDs.
- [ ] #4 Feature specification docs/specifications/browser-complete-access.md reviewed and aligned.
- [ ] #5 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #6 No direct backlog file edits; all changes via backlog CLI.
- [ ] #7 Browser performs no writes outside the canonical project directory and no network access.
<!-- DOD:END -->
