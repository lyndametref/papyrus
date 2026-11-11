---
id: TASK-92
title: Implement papyrus import backlog
status: To Do
assignee: []
created_date: '2026-11-11T20:43:00Z'
labels:
  - import
  - native-layout
  - release-1.0
milestone: m-14
dependencies:
  - TASK-83
  - TASK-88
  - TASK-89
  - TASK-90
references:
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - >-
    docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md
  - docs/requirements/non-functional/NFR-04.004.md
priority: high
type: feature
ordinal: 39090
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0011 makes the one-way import the only bridge from a Backlog.md store, and ADR-0010 decision 11 requires it in the same release as the native layout because unimported records become invisible once native stems are the only recognised form. The command must plan all IDs before writing, reuse the shared reader and writer, and never persist a source ID or a mapping store. This task delivers that command as ADR-0013 stage 6.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus import backlog --from <dir> [--source-backlog-dir <dir>] [--dry-run] [--allow-existing] [--report <path>] exists with the documented options and exit codes, and the target project must be initialized.
- [ ] #2 Every destination path is contained in the project directory, and --report accepts only a path inside the project.
- [ ] #3 Import covers backlog/tasks, backlog/archive/tasks, backlog/milestones, and backlog/archive/milestones; docs, drafts, decisions, config.yml, and locks are not read as records and are reported as excluded.
- [ ] #4 Phase one allocates consecutive Papyrus numbers per family from the maximum present in every active, archived, and deprecated location, minimum 1, with milestones starting at 1; no source number is preserved and no mapping file, sidecar, marker, or extra front-matter key is written.
- [ ] #5 Field mapping follows ADR-0011 decision 5: Papyrus id, verbatim title, normalized status with a warning on mismatch, assignee as a YAML list, validated or approximated timestamps, translated milestone and dependency references with warnings for unresolvable entries, copied references, documentation, type, and ordinal, preserved unknown keys, and no milestone release value.
- [ ] #6 Body mapping converts the documented marker sections into the native section layout, drops acceptance-criteria ordinals and marker comments, and appends unknown sections and free text verbatim.
- [ ] #7 Collision handling matches ADR-0011 decision 7: same-title records import with a possible-duplicate warning, a non-empty target family requires --allow-existing, source anomalies are reported, and a mid-import failure aborts with a summary of the records already written.
- [ ] #8 A dry run performs the full plan and writes nothing, and the source directory is byte-identical after a real import.
- [ ] #9 Reading reuses the shared front-matter and document parser and writing reuses record::store, with no Backlog-specific parser, serializer, or direct file write in the import module.
- [ ] #10 Command help matches the specification, and tests cover active and archived records, reference translation, dangling references, status mismatch, duplicate source numbers, same-title targets, existing-target refusal, dry run, and source immutability.
<!-- AC:END -->
