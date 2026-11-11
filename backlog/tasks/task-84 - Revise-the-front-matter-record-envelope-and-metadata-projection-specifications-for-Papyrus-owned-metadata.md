---
id: TASK-84
title: >-
  Revise the front-matter, record-envelope, and metadata-projection
  specifications for Papyrus-owned metadata
status: To Do
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - specification
  - front-matter
  - native-layout
  - release-1.0
milestone: m-16
dependencies: []
references:
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-01.005.md
  - docs/specifications/shared-record-envelope-and-front-matter-patcher.md
  - >-
    docs/specifications/render-one-metadata-projection-in-cli-and-browser-views.md
  - docs/specifications/task-management.md
priority: high
type: docs
ordinal: 39010
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0012 and ADR-0013 remove the Backlog.md key-set contract, the storage-ID translation, and the legacy-to-Backlog body conversion, and ADR-0012 lists five specifications that must be revised. Those specifications, plus task-management and milestone-and-release-management, still describe storage IDs, a storage boundary, a two-direction round trip, and body conversion, and the search specification does not state that front matter is indexed. This task aligns the metadata, adoption, projection, and serializer-removal specifications with the accepted decisions so the implementation tasks follow one contract.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The specifications shared-record-envelope-and-front-matter-patcher.md, migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md, expose-front-matter-metadata-options-on-all-record-commands.md, render-one-metadata-projection-in-cli-and-browser-views.md, and remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md no longer state a storage ID, a storage-boundary translation, a Backlog.md key set, or Backlog.md-compatible serialization.
- [ ] #2 Front-matter adoption is specified as front-matter-only, idempotent, and per-record on first read and first write, with legacy metadata section retirement on write and no Backlog-layout conversion or relocation.
- [ ] #3 The metadata projection specification states one RecordView whose id is the Papyrus ID for CLI text, --json, browser JSON, and browser HTML, and records the removal of any storage-ID JSON field as a breaking change.
- [ ] #4 task-management.md and milestone-and-release-management.md use Papyrus IDs in examples, commands, and references and describe no storage-ID input form.
- [ ] #5 The search specification states that whole-file text, now including front matter, is indexed, so metadata values are searchable.
- [ ] #6 No revised specification retains a byte-identical Backlog.md body guarantee or a Backlog.md round-trip acceptance gate.
- [ ] #7 Revised specifications keep their Sources and Traceability sections and link ADR-0012 and ADR-0013.
<!-- AC:END -->
