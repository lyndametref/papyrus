---
id: TASK-134
title: >-
  Align the milestone descriptor and record layer with the close-only milestone
  lifecycle
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T13:59:00Z'
updated_date: '2026-11-15T14:14:00Z'
labels:
  - registry
  - lifecycle
milestone: m-20
dependencies: []
references:
  - src/registry.rs
  - src/record/descriptor.rs
  - src/record/store.rs
  - src/import.rs
  - src/milestone.rs
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - TASK-130
  - TASK-132
priority: high
ordinal: 41400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** ADR-0017 constraint 3 and ADR-0023 decision 2 record that the milestone descriptor carries `ArchivePolicy::None` and that `<backlog_directory>/archive/milestones/` is a **read/import location only**. The code still says otherwise:

- `MILESTONE_TEMPLATE.archive` is `FRONTMATTER_ARCHIVE` in `src/registry.rs`.
- `RecordDescriptor::resolve_archived_directory` returns the archive folder for `"task" | "milestone"` in `src/record/descriptor.rs`, so the record layer still exposes a milestone lifecycle archive destination.
- `relocate_if_archived` in `src/record/store.rs` creates the archived folder and moves any active-folder record whose front-matter `status` is `archived`. A legacy active-folder milestone with `status: archived` is therefore silently moved, and a read/search path creates `backlog/archive/milestones/`, contradicting the close-only lifecycle (`papyrus milestone close <id>`, REQ-03.001, ADR-0022 decision 5) and the m-19 checkpoint.

**What.** Align the descriptor and record layer with the accepted ADRs: the milestone template declares `ArchivePolicy::None`; `resolve_archived_directory` exposes no lifecycle archive location for milestones; the legacy/import folder stays readable and import-writable; `relocate_if_archived` preserves and reports an `archived` milestone instead of moving it; a regression test locks the invariant. No user-visible CLI change beyond preventing the invalid auto-archive.

**In scope.**
- `src/registry.rs`: set the milestone template `archive` field to `ArchivePolicy::None` and update the registry test `archive_policies_follow_the_layout` so the milestone expectation is `ArchivePolicy::None`; adjust any other test or assertion that assumes the milestone `FRONTMATTER_ARCHIVE`.
- `src/record/descriptor.rs`: return `None` from `resolve_archived_directory` for `milestone`. If a separate read/import-only accessor is needed so `resolve_read_directories` still returns the active folder, `<backlog_directory>/archive/milestones/`, and `<backlog_directory>/milestones/archived/`, add it (name it for read/import, not for archiving). Keep task and document behavior unchanged.
- Preserve read resolution: `milestone view`, `edit`, and `delete` by ID, search, browser views, migration scans, and milestone number allocation (`read_dirs` in `src/milestone.rs` feeds `next_number_in_dirs`) must still see both legacy folders, read-only.
- `src/record/store.rs`: `archive_inner` and `relocate_if_archived` must never resolve or create a milestone archive destination. For a milestone whose stored `status` is `archived` (legacy data), keep the file in place and report the condition to the caller; no destination directory is created and no move occurs.
- `src/import.rs`: leave the import path writing archived source milestones to `<backlog_directory>/archive/milestones/`; import remains the only writer that creates that folder.
- Update the `resolve_archived_directory` / read-directory documentation comments to state that the milestone folder is read/import-only and not a lifecycle archive destination.
- Regression test: in a temporary project, place a milestone in `backlog/milestones/` with `status: archived`, run the read/search/edit path that previously relocated it, and assert the file is still in `backlog/milestones/` and `backlog/archive/milestones/` does not exist. Also assert both legacy locations still resolve for reads and that import still writes archived milestones to `backlog/archive/milestones/`.

**Out of scope.** REQ-06.013 text (TASK-135), `docs/specifications/milestone-and-release-management.md` (TASK-136), ADRs, browser UI changes, new CLI options or verbs, deleting, migrating, or restoring legacy archived milestone files, the release lifecycle, and the task/document archive policies.

**Assumptions.** The accepted ADRs are authoritative and the close-only milestone lifecycle is already delivered by TASK-130/TASK-132. "Report" means exposing the preserved record condition to the calling path so it is not silent; it adds no new exit code, error, or output line. `archived` remains only a legacy stored value, never a reachable milestone lifecycle state. No new configuration key or record type is introduced.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The milestone template archive_policy is ArchivePolicy::None in src/registry.rs, and archive_policies_follow_the_layout asserts None for milestone.
- [x] #2 RecordDescriptor::resolve_archived_directory returns None for milestone, so no production caller can obtain a milestone lifecycle archive destination.
- [x] #3 Milestone read directories still include the active folder, <backlog_directory>/archive/milestones/, and <backlog_directory>/milestones/archived/, so legacy records remain resolvable for view, edit, delete, search, and browser, and ID allocation never reuses their numbers.
- [x] #4 Import still writes archived source milestones to <backlog_directory>/archive/milestones/; no other command or read path creates that folder.
- [x] #5 relocate_if_archived leaves an active-folder milestone whose stored status is archived in place and reports the condition; it creates no directory and performs no move for a milestone.
- [x] #6 A regression test proves that a read/search path on such a milestone leaves the file in backlog/milestones/ and leaves backlog/archive/milestones/ absent.
- [x] #7 Task archive/restore, document archive, and release close behavior are unchanged; papyrus milestone close <id> still rewrites open -> closed in place and moves nothing.
- [x] #8 No CLI command, flag, output, or exit-code change is introduced beyond preventing the invalid auto-archive, and the full test suite passes.
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Milestone descriptor is now ArchivePolicy::None and the record layer exposes no milestone lifecycle archive destination: resolve_archived_directory/resolve_archive_directory return None for milestones. Legacy locations remain read/import-only via resolve_read_import_directory (backlog/archive/milestones) plus the milestones/archived alias; import is the only writer and read paths never create the folder. relocate_if_archived preserves a milestone with legacy status archived in place, warns on stderr, creates no folder and moves nothing. Task/document/release behavior and the CLI surface are unchanged. Evidence: registry/record/milestone tests for policy, read dirs, allocation, legacy read, import destination and no-create regression; full suite 856 passed / 0 failed / 22 ignored; clippy and fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
