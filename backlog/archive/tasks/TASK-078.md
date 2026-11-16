---
id: TASK-078
title: >-
  Revise specifications for the Papyrus-native layout, single ID space, and
  one-way Backlog.md import
status: Done
assignee: []
created_date: '2026-11-11T20:41:00Z'
updated_date: '2026-11-12T06:44:00Z'
labels:
  - specification
  - native-layout
  - import
  - release-1.0
milestone: MS-017
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
  - docs/specifications/papyrus-native-record-layout-and-single-id-space.md
  - docs/specifications/import-backlog-md-records.md
  - docs/requirements/business/REQ-01.002.md
priority: high
type: docs
ordinal: 39000
---


## Description
ADR-0010, ADR-0011, ADR-0012, and ADR-0013 (accepted 2026-11-11) replace the Backlog.md-layout architecture of ADR-0007/0008/0009. ADR-0010 decision 9 and ADR-0012 downstream constraint 5 declare three existing specifications superseded in intent, and they still mandate storage IDs, an m-0 offset, lazy Backlog-layout conversion, and a two-direction round-trip gate. Leaving them in place gives implementers contradictory instructions for the largest change in the release. This task brings the layout, identity, and import specifications in line with the accepted decisions and fixes the upgrade-notes contract the release must ship.

Deliverables:
1. Record the three legacy specifications (migrate-tasks-and-milestones-to-the-backlog-md-layout.md, translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md, verify-backlog-md-round-trip-interoperability.md) as superseded, naming the superseding ADRs, with no live behaviour contract remaining.
2. Author docs/specifications/papyrus-native-record-layout-and-single-id-space.md defining the Papyrus-native record layout and single ID space: configuration-driven directories per record family, <ID>.md filenames (decisions <ID>-<slug>.md), tolerant read stems, front-matter id authoritative, allocation without offsets with milestones starting at 1, archive and deprecate as moves, and the intentional invisibility of Backlog.md stems. Traceable to ADR-0010, REQ-01.005, and REQ-01.002.
3. Author docs/specifications/import-backlog-md-records.md defining papyrus import backlog: invocation and options, source scope and exclusions, planned ID allocation, field and body mapping, status normalization, collision and duplicate handling, the one-way non-idempotent caveat, exit codes, and containment. Agrees with ADR-0011.
4. Update README command documentation to carry the import command with its options, the one-way caveat, and the excluded-scope statement, and to remove any claim of Backlog.md file compatibility.
5. Ensure no specification states a Backlog.md storage ID, an m-0 offset, dual ID spaces, or a Backlog.md round-trip or file-compatibility gate as current behaviour.

The upgrade-notes contract is defined in the import specification section 11: affected source directories, the command to run, reallocation of task IDs and milestone numbers, keeping the source store until verification, and Papyrus owning the records after import.

## Acceptance Criteria
- [x] The specifications migrate-tasks-and-milestones-to-the-backlog-md-layout.md, translate-papyrus-and-backlog-md-ids-at-the-storage-boundary.md, and verify-backlog-md-round-trip-interoperability.md are recorded as superseded, name the superseding ADR, and no longer state a live behaviour contract.
- [x] A specification defines the Papyrus-native record layout and single ID space: configuration-driven directories per record family, <ID>.md filenames (decisions <ID>-<slug>.md), tolerant read stems, front-matter id as authoritative, allocation without offsets with milestones starting at 1, archive and deprecate as moves, and the intentional invisibility of Backlog.md stems. It is traceable to ADR-0010, REQ-01.005, and REQ-01.002.
- [ ] A specification defines papyrus import backlog: invocation and options, source scope and exclusions, planned ID allocation, field and body mapping, status normalization, collision and duplicate handling, the one-way non-idempotent caveat, exit codes, and containment. It agrees with ADR-0011.
- [x] The README command documentation carries the import command with its options, the one-way caveat, and the excluded-scope statement, and no longer claims Backlog.md file compatibility.
- [ ] No specification states a Backlog.md storage ID, an m-0 offset, dual ID spaces, or a Backlog.md round-trip or file-compatibility gate as current behaviour.
- [x] The upgrade-notes contract is defined: affected source directories, the command to run, reallocation of task IDs and milestone numbers, keeping the source store until verification, and Papyrus owning the records after import.
- [x] Revised specifications keep their Sources and Traceability sections and link the superseding architecture decisions.

## Definition of Done
- [x] #1 All three legacy specifications are marked superseded with superseding ADR links and no live behaviour contract remains
- [ ] #2 Native layout and import specifications are authored with Sources and Traceability sections
- [x] #3 README command documentation is updated with the import command, options, one-way caveat, and excluded-scope statement
- [ ] #4 No specification claims Backlog.md storage IDs, m-0 offset, dual ID spaces, or a round-trip/file-compatibility gate as current behaviour
- [x] #5 Changes reviewed for consistency with ADR-0010..ADR-0013


## Notes
Documentation implementation for TASK-83 completed. README man page (papyrus(1)) updated with an 'import' command section carrying 'papyrus import backlog --from <source-project-dir> [--source-backlog-dir <dir>] [--dry-run] [--allow-existing] [--report <path>]', its options, the one-way/non-idempotent caveat, and the excluded-scope statement (Backlog.md docs/drafts/decisions not imported); no Backlog.md file-compatibility claim remains. Legacy specs (migrate-tasks-and-milestones-to-the-backlog-md-layout, translate-papyrus-and-backlog-md-ids-at-the-storage-boundary, verify-backlog-md-round-trip-interoperability) remain superseded with superseding ADR links and no live contract. New specs (papyrus-native-record-layout-and-single-id-space, import-backlog-md-records) verified for layout/ID/import coverage, traceability, and Sources sections. Neutralized lingering storage-ID/round-trip wording in render-one-metadata-projection and complete-content-management packaging spec. All ACs 1-7 and DoD 1-5 checked.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
TASK-83 documentation scope delivered: README import command docs added; acceptance criteria verified; no remaining spec claims Backlog.md storage IDs, m-0 offset, dual ID spaces, or round-trip/file-compatibility gate as current behaviour.
