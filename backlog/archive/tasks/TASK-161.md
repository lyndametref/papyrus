---
id: TASK-161
title: "Refine milestone archive command implementation"
created_date: '2026-11-16T21:03:05Z'
updated_date: '2026-11-19T10:53:20Z'
labels: []
references:
  - REQ-003.001
  - RE-002
  - TC-015
  - ADR-032
  - BR-006
  - BR-001
  - GLO-006
  - TECH-009
  - TC-016
  - TC-017
  - TC-018
  - TC-019
  - TC-020
  - TC-021
  - TC-022
  - TC-023
status: Done
assignee:
  - task-refiner
milestone: MS-025
dependencies:
  - TASK-160
ordinal: 30
documentation:
  - src/registry.rs
  - src/milestone.rs
  - src/record/descriptor.rs
type: refinement
---

## Description
Outcome: confirm all information for implementing `papyrus milestone archive <id>` is available and complete the refinement fields. `archive` is the only transition to `archived`: it closes an open milestone if needed, then sets `status: archived`, refreshes `updated_date` once, preserves title, description, labels, references, and release assignment, and moves the milestone file to the backlog archive milestones folder using the shared atomic record writer. An already archived milestone is a no-op success. Archive is refused for any status other than `open`, `closed`, or `archived`. No restore command exists; the lifecycle is fixed `open -> closed -> archived`. Scope: implement the `archive` verb on the built-in `milestone` command and its supporting descriptor and registry changes. Close behavior (TASK-130, TASK-136) and list, view, edit, delete behavior remain unchanged. Archived milestones stay hidden from `papyrus milestone list` but remain viewable by ID and editable in place as legacy data. Assumptions: prerequisites are met (architecture impact assessed and user-verified with ADR-032 accepted, TECH-009 conformance reviewed in TASK-159 Done, test cases TC-015 through TC-023 approved by the user in TASK-160 Done). The archive destination is the existing read and import folder backlog archive milestones; no new configuration key is introduced and the folder is created if absent. A missing or empty stored status is treated as open, so imported and legacy records remain archivable; the close-then-archive sequence collapses to a single archived write with one updated_date refresh. Release assignment is stored only in milestone front matter; archiving never reads or writes a release file. Archived milestones remain readable and editable in place as legacy data; no restore command is added. Implementation must update the milestone descriptor and registry (states, verbs, archive location) and the tests that currently assert milestones have no lifecycle archive location. No UI mockup is required; this is a CLI-only change. Rust 1.99.1 and Cargo are available (ADR-0001). Definition of done: implementation and tests in TASK-162 satisfy all acceptance criteria; cargo test passes; no open material questions remain.
## Acceptance Criteria
- [ ] Implement `papyrus milestone archive <id>` as the only transition to `archived`: close first when status is `open`, set `archived` when status is `closed`, preserving title, description, labels, references, and release assignment (TC-015, TC-016, TC-023).
- [ ] Move the milestone file to the backlog archive milestones folder using the shared atomic record writer, refreshing `updated_date` exactly once (TC-015, TC-016, TC-020).
- [ ] Treat an already archived milestone as a no-op success: exit 0 with no write and no move (TC-017).
- [ ] Refuse archive for any status other than `open`, `closed`, or `archived` with exit code 4, naming the current status and writing nothing (TC-018).
- [ ] Fail with exit code 3 when the milestone ID is unknown (TC-019).
- [ ] Fail with exit code 1 on filesystem or IO failure or project-directory boundary violation (TC-020).
- [ ] Hide archived milestones from `papyrus milestone list` (TC-021).
- [ ] Keep an archived milestone viewable by ID and editable in place as legacy data; no restore command exists (TC-022, ADR-032).
- [ ] Preserve the milestone release assignment and modify no release file (TC-023).
- [ ] Emit text output printing the milestone ID and `--json` output with id and status archived (TECH-009 section 6).
- [ ] Add the `Archive` variant to `MilestoneCmd` and dispatch it in `run` in the milestone module.
- [ ] Update the registry module: add `archived` to milestone states, add the archive verb to milestone verbs, set the milestone template archive policy to the front-matter archived folder, and update comments.
- [ ] Update the descriptor module so milestone resolves the archive and archived directories to the backlog archive milestones folder, and update the descriptor tests accordingly.
- [ ] Update help and dispatch tests to list `archive` among milestone verbs.
- [ ] Traceability is recorded: REQ-003.001, RE-002, BR-006, BR-001, GLO-006, TECH-009, ADR-032, TC-015 through TC-023, MS-025.
- [ ] No open material questions remain.
## Comments
- **task-refiner**: Refinement complete. Scope: implement `papyrus milestone archive <id>` as the only transition to `archived` (close first if open, then set archived; no-op if already archived; refuse other statuses with exit 4; unknown ID exit 3; IO or boundary failure exit 1). Assumptions: archive destination is the existing backlog archive milestones folder (created if absent); missing or empty status is read as open; release assignment is preserved and no release file is touched; archived milestones stay viewable and editable in place as legacy data with no restore command. Code locations: milestone module (MilestoneCmd Archive variant and archive function), registry module (states, verbs, archive policy), descriptor module (archive and archived directories), help and dispatch tests. Traceability: REQ-003.001, RE-002, BR-006, BR-001, GLO-006, TECH-009, ADR-032, TC-015 through TC-023, MS-025. No open material questions.
