---
id: TASK-131
title: >-
  Correct the milestone and release specification to the close-only milestone
  lifecycle
status: Done
assignee:
  - @business-analyst
created_date: '2026-11-15T14:00:00Z'
updated_date: '2026-11-15T14:27:00Z'
labels:
  - docs
  - specification
milestone: MS-021
dependencies:
  - TASK-129
references:
  - docs/specifications/milestone-and-release-management.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - TASK-130
priority: medium
ordinal: 41420
---


## Description
**Why.** `docs/specifications/milestone-and-release-management.md` still specifies the removed milestone archive lifecycle: `papyrus milestone archive <id>` in Scope, milestone lifecycle `active -> archived`, archived milestones hidden from `list`, view/edit of archived milestones, a "Milestones - archive and delete" behavior section, `milestone archive` in the inputs/outputs summary, "already-archived milestone" as a validation example, archive assumptions, and OQ-3 naming `backlog/archive/milestones/` as the canonical archive folder. TASK-130 delivered the close-only lifecycle: `papyrus milestone close <id>` moves `open -> closed` in place and the archive and restore verbs were removed (ADR-0022 decision 5; ADR-0023 decision 2). The page is now an implementer-facing contradiction and part of the residual milestone drift found by the m-19 checkpoint.

**What.** Correct the page to the close-only milestone lifecycle, remove every milestone archive claim, state the legacy read/import-only role of `backlog/archive/milestones/` and the legacy alias, keep the release lifecycle unchanged, and record the superseded wording in the Revision line.

**In scope.**
- Purpose and Scope: remove `archive` from the milestone verb list, keep `delete`, and state the milestone lifecycle `open -> closed` through `papyrus milestone close <id>` in place; no milestone archive or restore. Keep the release lifecycle `planned -> released` through `papyrus release close <id>`, with no release archive, restore, or delete.
- Behavior §1.4 storage: active and closed milestones live in `<backlog_directory>/milestones/`; `<backlog_directory>/archive/milestones/` and `<backlog_directory>/milestones/archived/` are legacy **read** locations; the one-way import is the only writer that creates/or writes `backlog/archive/milestones/`, and no milestone command creates it.
- §3 list: lists the milestones in the active folder, including `closed` ones; there is no archived milestone state; legacy archived files are not listed.
- §4 view and §5 edit: remove the "archived milestone" lifecycle cases; state that legacy files found in either legacy location remain readable (and editable in place) as legacy data, with no movement.
- §6: rename to "Milestones - close and delete"; document `papyrus milestone close <id>` (open -> closed, in place, refreshed `updated_date`, exit 4 when already closed or not open) and `delete`; remove the archive command and the "already-archived milestone" failure.
- Inputs and outputs: remove `milestone archive <id>` and add `milestone close <id>` with its output.
- Validation and error behavior: replace "already-archived milestone" with the close-only conflict wording.
- Assumptions: replace the two archive assumptions and the relationship assumption so they no longer claim a milestone archive move; state that legacy archived milestone files are read and left in place, and that a closed milestone remains in the active folder and stays listed.
- OQ-3: mark it resolved by the close-only lifecycle (ADR-0022 decision 5, ADR-0023 decision 2): `backlog/archive/milestones/` is a read/import location only, no milestone unarchive or restore exists.
- Revision line: add a 2026-11-15 entry that records the correction and explicitly notes the superseded `active -> archived` and `papyrus milestone archive` wording.
- Keep the release sections (create/list/view/edit/close, relationship, persistence) otherwise intact.

**Out of scope.** Code (TASK-134), REQ-06.013 (TASK-135), ADR text, business rules, other specification pages, requirement IDs, and any new milestone feature or verb. Do not change the release lifecycle, the dangling-release behavior, or the sanitization rules.

**Assumptions.** The implementation already matches the close-only lifecycle after TASK-130; this page is being corrected to it, not defining new behavior. Legacy archived milestone files may still exist on disk and stay readable/editable in place, but no command produces, moves, or restores that state. `milestone delete` continues to resolve records in the active and legacy read locations.

## Acceptance Criteria
- [x] Purpose and Scope no longer list milestone archive; the milestone verbs are create, list, view, edit, close, and delete, and the milestone lifecycle is stated as open -> closed in place through papyrus milestone close <id> with no archive or restore.
- [x] Behavior 1.4 states that active and closed milestones stay in <backlog_directory>/milestones/, that backlog/archive/milestones/ and milestones/archived/ are legacy read locations, that import is the only writer to backlog/archive/milestones/, and that no milestone command creates it.
- [x] milestone list is specified as listing the active folder including closed milestones, with no archived milestone state and legacy archived files not listed.
- [x] The archived-milestone view/edit lifecycle cases are removed and replaced with legacy-file wording: readable (and editable in place) as legacy data, never moved.
- [x] The archive-and-delete section is replaced by a close-and-delete section that documents papyrus milestone close <id> and the open -> closed conflict behavior; papyrus milestone archive is gone.
- [x] milestone archive is removed from Inputs and outputs and milestone close <id> is added; the validation example no longer mentions an already-archived milestone.
- [x] The archive assumptions and OQ-3 are rewritten to the read/import-only folder and the close-only lifecycle, with no remaining claim that a milestone is archived or moved.
- [x] The Revision line records the 2026-11-15 correction and explicitly notes the superseded active -> archived/papyrus milestone archive wording.
- [x] The release lifecycle remains planned -> released through papyrus release close <id> with no archive, restore, or delete, and no code, ADR, requirement, or business-rule file changes.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
milestone-and-release-management.md corrected to the delivered lifecycle: milestones use create/list/view/edit/close/delete; papyrus milestone close <id> is the only open -> closed transition, writing status + updated_date in place through the atomic writer (exit 4 naming status on a non-open close, exit 3 unknown, --json {id,status:closed}); no archive/restore, no archived milestone state, no archive-folder lifecycle; closed milestones stay in backlog/milestones and remain listed; backlog/archive/milestones and milestones/archived are legacy read locations with the one-way import as the only writer. Releases keep planned -> released via release close and are never archived. Revision note names the superseded active -> archived / milestone archive wording; traceability updated. Evidence: milestone_cli 4/4, release_close_cli 8/8, cli_help_golden 2/2, no_drift 10/10.
