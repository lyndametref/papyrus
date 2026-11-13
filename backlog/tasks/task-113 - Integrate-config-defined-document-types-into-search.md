---
id: TASK-113
title: Integrate config-defined document types into search
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-107
  - TASK-110
documentation:
  - docs/requirements/business/REQ-06.014.md
  - docs/requirements/business/REQ-07.002.md
priority: high
ordinal: 41070
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Search still walks a hard-coded set of document keys, so records of a type added in configuration cannot be found even though the CLI manages them. Once the registry is the type model, search must enumerate enabled document types, index only their active directories, and read the same metadata projection as the CLI and browser. Disabled types and archived records must stay invisible, exactly as they are in the other surfaces.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Search enumerates enabled Document descriptors from the artifact registry instead of a hard-coded document key list; no static DOC_KEYS or per-type search arm remains.
- [ ] #2 Records of a config-defined type are searchable by title, description, labels, and references with the same matching, scoring, and output format as default document types.
- [ ] #3 Only active directories are indexed; archived records and archived/ directories are never returned, and a disabled type produces no results.
- [ ] #4 Unified search and per-type search commands both use the registry-derived set, and a document type added or disabled in configuration changes results without a rebuild.
- [ ] #5 Tests create records for a user-added type, assert hits in unified and per-type search, and assert exclusion for archived records, disabled types, and a deprecated/ folder.
<!-- AC:END -->
