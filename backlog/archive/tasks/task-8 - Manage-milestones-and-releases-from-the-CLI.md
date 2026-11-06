---
id: TASK-8
title: Manage milestones and releases from the CLI
status: Done
assignee: []
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-04T19:08:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-7
  - TASK-22
references:
  - docs/requirements/business/REQ-03.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/specifications/milestone-and-release-management.md
  - docs/specifications/release-lifecycle.md
ordinal: 8000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver the `papyrus milestone` and `papyrus release` commands so a project maintainer can create, list, view, edit, archive, and delete milestones, and create, list, view, and edit releases, with stable IDs and text-file persistence inside the project directory. This implements REQ-03, REQ-03.001, and REQ-03.002, including the documented milestone/release relationship and the release lifecycle clarified by TASK-22. Milestones provide delivery containers that TASK-9 (tasks) and later tickets assign work to.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Users can create, list, view, and edit milestones with title, description, and optional release assignment using the documented flags; generated IDs follow the configured prefix and padding (default MS-001).
- [ ] #2 Users can create, list, view, and edit releases with title and description using the documented flags; generated IDs follow the configured prefix and padding (default REL-001).
- [ ] #3 A milestone can be assigned to zero or one release; a release may be referenced by multiple milestones; assigning requires the release to exist.
- [ ] #4 Milestone archive moves an active milestone to the archived store and hides it from list; deleting permanently removes it (active or archived), with --force skipping confirmation.
- [ ] #5 Release records have no archive, restore, delete, or status-transition command; release status is not represented as archived and no undocumented transition is inferred.
- [ ] #6 Milestone and release data are persisted as UTF-8 text files under the project backlog directory, with archived milestones under backlog/milestones/archived/.
- [ ] #7 All user-controlled input (title, description, release ID) is sanitized before use; values that cannot be normalized fail with exit code 4.
- [ ] #8 No reads or writes occur outside the canonicalized project directory (init remains the sole exception).
- [ ] #9 Standard CLI controls and exit statuses (0 success, 1 runtime/IO/boundary, 2 usage, 3 not found, 4 validation) behave as delivered by TASK-2, including --json output for all commands.
- [ ] #10 Unknown milestone or release IDs fail with exit code 3; validation and conflict cases fail with exit code 4.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 All acceptance criteria are verified and recorded in the ticket notes.
- [ ] #2 cargo build --release, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
- [ ] #3 Milestone and release commands are implemented within the existing single crate; no UI mockups or web UI changes are required.
- [ ] #4 Feature specification docs/specifications/milestone-and-release-management.md is created and referenced; release-lifecycle open questions remain recorded, not resolved by inference.
- [ ] #5 No requirements, business rules, ADRs, or packaging scripts are changed; any deviation is recorded in the ticket notes.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add `milestone` and `release` subcommands to the CLI (lib.rs Commands enum) and create modules `src/milestone.rs` and `src/release.rs` (or one combined module) following the req/nfreq module pattern.
2. Implement milestone storage helpers: active dir `<backlog>/milestones/`, archived dir `<backlog>/milestones/archived/`, scan-max for IDs across both folders, next ID generation with configured prefix/padding.
3. Implement release storage helpers: dir `<backlog>/releases/`, scan-max, next ID generation.
4. Implement create/list/view/edit for both commands with sanitization, validation, and JSON output.
5. Implement milestone-only archive (move active->archived) and delete (active or archived, --force skips stdin confirmation).
6. Implement optional release assignment on milestone create/edit with existence validation and mutual exclusion of --release/--no-release.
7. Add unit/integration tests covering IDs, relationship, archive/delete, sanitization, boundary, and exit codes.
8. Update docs if needed (none expected) and record verification evidence in ticket notes.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Quality review (gatekeeper): Ready to close. All AC #1-#10 pass; DoD #2-#5 pass. cargo build --release, cargo test (milestone_cli 3/3 + all suites), clippy -D warnings, fmt --check all pass on Rust 1.99.1. Scope: only src/milestone.rs, src/release.rs, src/lib.rs, tests/milestone_cli.rs, spec doc. NFR-02.002/NFR-04.001/NFR-04.002 addressed. Open items: release create -d flag vs docs mismatch; branch divergence from feedback-collection (resolved at merge).
<!-- SECTION:NOTES:END -->
