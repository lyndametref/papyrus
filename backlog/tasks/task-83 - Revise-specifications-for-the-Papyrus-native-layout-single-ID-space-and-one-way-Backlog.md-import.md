---
id: TASK-83
title: >-
  Revise specifications for the Papyrus-native layout, single ID space, and
  one-way Backlog.md import
status: To Do
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - specification
  - native-layout
  - import
  - release-1.0
milestone: m-16
dependencies: []
references:
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/non-functional/NFR-04.004.md
  - docs/requirements/business/REQ-01.005.md
  - docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md
  - >-
    docs/specifications/translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md
  - docs/specifications/verify-backlog-md-round-trip-interoperability.md
priority: high
type: docs
ordinal: 39000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0010, ADR-0011, ADR-0012, and ADR-0013 (accepted 2026-11-11) replace the Backlog.md-layout architecture of ADR-0007/0008/0009. ADR-0010 decision 9 and ADR-0012 downstream constraint 5 declare three existing specifications superseded in intent, and they still mandate storage IDs, an m-0 offset, lazy Backlog-layout conversion, and a two-direction round-trip gate. Leaving them in place gives implementers contradictory instructions for the largest change in the release. This task brings the layout, identity, and import specifications in line with the accepted decisions and fixes the upgrade-notes contract the release must ship.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The specifications migrate-tasks-and-milestones-to-the-backlog-md-layout.md, translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md, and verify-backlog-md-round-trip-interoperability.md are recorded as superseded, name the superseding ADR, and no longer state a live behaviour contract.
- [ ] #2 A specification defines the Papyrus-native record layout and single ID space: configuration-driven directories per record family, <ID>.md filenames (decisions <ID>-<slug>.md), tolerant read stems, front-matter id as authoritative, allocation without offsets with milestones starting at 1, archive and deprecate as moves, and the intentional invisibility of Backlog.md stems. It is traceable to ADR-0010, REQ-01.005, and REQ-01.002.
- [ ] #3 A specification defines papyrus import backlog: invocation and options, source scope and exclusions, planned ID allocation, field and body mapping, status normalization, collision and duplicate handling, the one-way non-idempotent caveat, exit codes, and containment. It agrees with ADR-0011.
- [ ] #4 The README command documentation carries the import command with its options, the one-way caveat, and the excluded-scope statement, and no longer claims Backlog.md file compatibility.
- [ ] #5 No specification states a Backlog.md storage ID, an m-0 offset, dual ID spaces, or a Backlog.md round-trip or file-compatibility gate as current behaviour.
- [ ] #6 The upgrade-notes contract is defined: affected source directories, the command to run, reallocation of task IDs and milestone numbers, keeping the source store until verification, and Papyrus owning the records after import.
- [ ] #7 Revised specifications keep their Sources and Traceability sections and link the superseding architecture decisions.
<!-- AC:END -->
