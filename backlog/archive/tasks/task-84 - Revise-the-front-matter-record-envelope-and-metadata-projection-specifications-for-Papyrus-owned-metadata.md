---
id: TASK-84
title: >-
  Revise the front-matter, record-envelope, and metadata-projection
  specifications for Papyrus-owned metadata
status: Done
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-12T06:50:00Z'
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
  - >-
    docs/specifications/migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md
  - >-
    docs/specifications/expose-front-matter-metadata-options-on-all-record-commands.md
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
  - docs/specifications/milestone-and-release-management.md
  - docs/specifications/search-complete-unified.md
priority: high
type: docs
ordinal: 39010
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Revise the front-matter, record-envelope, and metadata-projection specifications (plus task-management.md, milestone-and-release-management.md, and the search specification) so they describe Papyrus-owned metadata per ADR-0012 and ADR-0013.

Scope (in):
- shared-record-envelope-and-front-matter-patcher.md
- migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md
- expose-front-matter-metadata-options-on-all-record-commands.md
- render-one-metadata-projection-in-cli-and-browser-views.md
- remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
- task-management.md
- milestone-and-release-management.md
- search specification (search-complete-unified.md and per-type search-*.md as needed)

Scope (out):
- Code changes or implementation of the envelope, identity, or projection.
- Requirements and business rules (read-only).
- UI mockups.

Required changes:
1. Remove statements of storage IDs, storage-boundary translation, Backlog.md key set, and Backlog.md-compatible serialization.
2. Specify front-matter adoption as front-matter-only, idempotent, and per-record on first read and first write; retire legacy metadata sections on write; no Backlog-layout conversion or relocation.
3. Specify one RecordView whose id is the Papyrus ID for CLI text, --json, browser JSON, and browser HTML; record removal of any storage-ID JSON field as a breaking change.
4. Use Papyrus IDs in examples, commands, and references in task-management.md and milestone-and-release-management.md; describe no storage-ID input form.
5. State in the search specification that whole-file text, including front matter, is indexed, so metadata values are searchable.
6. Remove any byte-identical Backlog.md body guarantee and Backlog.md round-trip acceptance gate.
7. Keep Sources and Traceability sections; link ADR-0012 and ADR-0013.

Assumptions:
- ADR-0012 and ADR-0013 are accepted and authoritative; ADR-0007, ADR-0008, and ADR-0009 are superseded.
- Documentation-only change; no code or requirement edits.
- No UI mockups required.

Dependencies: none (ADRs already accepted).

Open questions:
- Whether ADR references to superseded ADR-0007/0008/0009 are replaced throughout or supplemented with ADR-0012/0013 (default: supplement, then replace where contradictory).
- Whether all per-type search specs or only search-complete-unified.md need edits (default: all that mention storage IDs or Backlog compatibility).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The five named front-matter/envelope/projection/serializer specs no longer state a storage ID, storage-boundary translation, Backlog.md key set, or Backlog.md-compatible serialization.
- [ ] #2 Front-matter adoption is specified as front-matter-only, idempotent, and per-record on first read and first write, with legacy metadata-section retirement on write and no Backlog-layout conversion or relocation.
- [ ] #3 The metadata projection spec states one RecordView whose id is the Papyrus ID for CLI text, --json, browser JSON, and browser HTML, and records removal of any storage-ID JSON field as a breaking change.
- [ ] #4 task-management.md and milestone-and-release-management.md use Papyrus IDs in examples, commands, and references and describe no storage-ID input form.
- [ ] #5 The search specification states that whole-file text, including front matter, is indexed, so metadata values are searchable.
- [ ] #6 No revised specification retains a byte-identical Backlog.md body guarantee or a Backlog.md round-trip acceptance gate.
- [ ] #7 Revised specifications keep their Sources and Traceability sections and link ADR-0012 and ADR-0013.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria verified by documentation review.
- [ ] #2 Each revised spec retains its Sources and Traceability sections.
- [ ] #3 ADR-0012 and ADR-0013 are linked in each revised spec.
- [ ] #4 A grep check confirms no storage-ID, Backlog key-set, Backlog-compatible serialization, or Backlog round-trip gate remains in revised specs.
- [ ] #5 No requirement or business-rule files were modified.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Revised 8 specifications for Papyrus-owned metadata per ADR-0012/ADR-0013: shared-record-envelope-and-front-matter-patcher.md, migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write.md, expose-front-matter-metadata-options-on-all-record-commands.md, render-one-metadata-projection-in-cli-and-browser-views.md, remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md, task-management.md, milestone-and-release-management.md, search-complete-unified.md. Neutralized storage-ID/Backlog-compat wording, updated ADR/NFR references, added Papyrus ID handling and breaking-change note, ensured Traceability links ADR-0012 and ADR-0013.
<!-- SECTION:NOTES:END -->
