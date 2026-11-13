---
id: TASK-114
title: Serve registry-derived browser views for every enabled document type
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
labels:
  - rust
  - browser
milestone: m-17
dependencies:
  - TASK-109
  - TASK-110
documentation:
  - >-
    docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md
  - docs/requirements/business/REQ-06.014.md
priority: high
ordinal: 41080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The browser route table, sidebar, list/detail pages, edit handlers, and not-found logic are written per built-in type through `Category`. A type added in configuration has no browser presence, a disabled type can still be browsed, and the archived location is not distinguished from an unknown record. REQ-06.014 requires list, detail, and edit views for every enabled `document_types` entry with a status display and a validated status filter. This task moves the browser onto the registry and the two generic view families while preserving the existing typed pages.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The route table, navigation, list pages, and API endpoints are assembled at startup from registry.enabled(Document); no second document-type list and no Category enum survive.
- [ ] #2 Every enabled config-defined type gets a list view, a detail view, an edit view, its sidebar entry, and one canonical /api/docs/<slug> endpoint; existing built-in API names remain aliases.
- [ ] #3 List and detail views render status read-only for every document type; the edit form offers title, description, labels, and references and never a status control or a save path for status.
- [ ] #4 The list status filter offers exactly active plus the type's declared custom statuses in configured order, never archived; a value outside the set is rejected rather than applied, and a type with no custom statuses offers only active.
- [ ] #5 Archived records and archived/ are never indexed or reachable, an archived ID and an unknown ID return the same page, and a disabled type has no navigation, list, detail, edit, or API response.
- [ ] #6 The lifecycle page lists every document type declared in configuration, enabled or disabled, grouped by the three artifact categories, with main statuses, custom statuses, prefix, folder, and enabled state read-only.
- [ ] #7 Built-in typed pages keep their existing field sets and output, selected by descriptor; views use the type's configured folder and prefix and the shared RecordView projection.
- [ ] #8 Tests cover a user-added type end to end in the browser, the filter set and rejection, read-only status, archived-versus-unknown equivalence, disabled visibility, and an unchanged built-in page fixture.
<!-- AC:END -->
