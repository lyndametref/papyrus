---
id: TASK-69
title: Migrate tasks and milestones to the Backlog.md layout
status: Refined
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-08T21:35:00Z'
labels:
  - front-matter
  - migration
  - backlog-interop
milestone: m-12
dependencies:
  - TASK-64
  - TASK-68
references:
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md
priority: high
type: feature
ordinal: 30070
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement lazy, per-record migration of task and milestone files to the Backlog.md on-disk layout (ADR-0009 Stage 4) under REQ-05.011 and NFR-04.003, REQ-05.010, and REQ-01.005. Each task or milestone file gains front matter on its first read and on its first write. Files already in the Backlog.md storage shape keep bodies, marker comments, and section order byte-identical apart from the front-matter block (and, for Backlog.md-created milestones, the additive general keys and release). Legacy Papyrus-shape files are converted on first read or first write: heading dropped, retired metadata sections moved to front matter, description and acceptance criteria mapped to marker sections, unknown sections and free text preserved verbatim, with a before/after diff reported per converted file. Migration relocates legacy and archived files to the Backlog.md paths and storage IDs, translates display IDs to storage IDs at the boundary, detects conflicts per ADR-0008 decision 9, and is idempotent. Builds on TASK-63, TASK-64, TASK-65, TASK-68. Full behavior, scope, and acceptance criteria are in docs/specifications/migrate-tasks-and-milestones-to-the-backlog-md-layout.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each active and archived task and milestone file gains a front-matter block on its first read and on its first write; the Papyrus display ID is unchanged before and after (REQ-01.005, REQ-05.011).
- [ ] #2 A file already in the Backlog.md storage shape keeps its body, marker comments, and section order byte-identical apart from the front-matter block; a task or milestone whose front-matter block is already complete is not rewritten at all (no timestamp refresh, no write) (NFR-04.003, REQ-05.011).
- [ ] #3 A legacy Papyrus-shape task or milestone is converted on first read or first write: the '# <ID> <title>' heading is dropped, retired metadata sections move to front matter, '## Description' and '## Acceptance Criteria' map to the Backlog.md marker sections, and every unknown section and free-text block is preserved verbatim in file order (REQ-05.011).
- [ ] #4 Each converted legacy file is relocated to the Backlog.md path and storage filename, its front-matter 'id' becomes the storage form (task 'TASK-<n>', milestone 'm-<n>'), and 'milestone:'/'dependencies:' reference values are translated to storage form (REQ-01.005, ADR-0008 decision 4).
- [ ] #5 Each converted legacy file is reported in the per-file inventory with action 'converted-legacy-to-backlog-md', its resulting key list, and a unified before/after diff (REQ-05.011).
- [ ] #6 Milestone front matter carries the general keys plus 'release'; Backlog.md reads, lists, updates, and preserves the additive general keys and 'release' (NFR-04.003, REQ-05.010).
- [ ] #7 Conflict and duplicate cases follow ADR-0008 decision 9: duplicate storage IDs, front-matter/filename mismatches, legacy-versus-storage pairs with the same title, unknown visible IDs, and reused numbers are detected and reported per record; migration stops on the conflicting record and never merges, deletes, or renumbers (REQ-01.005).
- [ ] #8 Legacy archive paths ('tasks/archived/', 'milestones/archived/') are read for compatibility, consolidated into the Backlog.md archive paths ('archive/tasks/', 'archive/milestones/'), and never written again; new archives use only the Backlog.md paths (ADR-0008 decision 2).
- [ ] #9 Migration is idempotent: a record carrying a complete front-matter block is not re-migrated on subsequent reads or writes, and re-running produces byte-identical content (REQ-05.011).
- [ ] #10 A first-read migration that cannot complete its atomic write (read-only media, missing permission, I/O error, or failed relocation) leaves the original file byte-identical, removes temporary files, and returns the error to the caller (ADR-0006, REQ-05.011).
- [ ] #11 Dry-run traversal over the descriptor-claimed task and milestone corpus reports 'would-migrate', 'would-convert', or 'already-migrated' per file without writing, and is available to tests and internal callers; no migration subcommand is added (REQ-05.011).
- [ ] #12 Only descriptor-claimed task and milestone files are touched; prose Markdown, HTML/UI mockups, and Backlog.md 'docs'/'drafts' records are never modified (ADR-0007 decision 11, ADR-0008 decision 11).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation builds on record::patch and record::store only; no second writer is introduced (ADR-0009 decision 8).
- [ ] #2 Unit/property tests cover storage-shaped byte-identity, legacy conversion with unknown-section preservation, before/after diff reporting, conflict detection, idempotency, and read-path write failure.
- [ ] #3 NFR-04.003 round-trip verification is updated to include task and milestone records migrated lazily from both input shapes (ADR-0008 decision 10).
- [ ] #4 No-drift checks pass: no ID formatting outside record::identity and no fs::write on record paths outside record::store.
- [ ] #5 Documentation and specification references are updated; no direct edits to backlog task files.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation pass halted by user instruction after new-model fix attempt. Quality review still Incomplete. Accepted deferrals: active legacy files convert in place (no storage-filename relocation) and new archives keep the legacy path; both belong to TASK-70. Remaining blockers: (1) Backlog.md milestone rename / task edit drop additive+required front-matter keys (NFR-04.003 interop gate, TASK-71); (2) task/milestone archive+restore bypass lazy migration; (3) task list does not surface duplicate/mismatch conflicts while same-title-distinct records are over-blocked; (4) editing a migrated Backlog-shaped record replaces the body heading (body corruption); (5) create still formats IDs outside record::identity (TASK-70); (6) no-drift DoD item conflicts with TASK-70 scope. Tests: cargo test 464 passed. Branch task/TASK-69 holds committed WIP; not merged.
<!-- SECTION:NOTES:END -->
