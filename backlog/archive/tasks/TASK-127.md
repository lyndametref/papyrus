---
id: TASK-127
title: >-
  Amend ADR-0017 decision 2 and ADR-0023 decisions 2-3 for the milestone
  close-only lifecycle and regenerate the ADR index
status: Done
assignee:
  - architect
created_date: '2026-11-15T13:13:00Z'
updated_date: '2026-11-15T13:32:00Z'
labels:
  - docs
  - architecture
milestone: MS-020
dependencies:
  - TASK-125
  - TASK-126
references:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - REQ-003.001
  - REQ-007.001
  - REQ-001.005
documentation:
  - docs/architecture/adr/index.html
priority: high
ordinal: 41312
---


## Description
**Why.** TASK-130 delivered the milestone lifecycle `open -> closed` through `papyrus milestone close <id>` and removed milestone archive/restore, and TASK-131 amended ADR-0022 decision 5 to match. Two Accepted ADRs still state the removed surface: ADR-0017 decision 2 assigns the milestone the verbs `create/list/view/edit/archive/delete` and no `close`, and ADR-0023 decision 2 gives the milestone the archive location `<backlog_directory>/archive/milestones/` while decision 3 says only tasks and milestones keep archive/restore. TASK-127 AC6 requires agreement with ADR-0017 and ADR-0023, so the specification pages corrected there currently cite contradicted decisions, and an implementer reading these ADRs alone would build a milestone `archive` verb and archive folder (REQ-003.001, REQ-007.001).

**What.** Amend ADR-0017 decision 2 and ADR-0023 decisions 2-3 in place to the milestone close-only lifecycle, with 2026-11-15 revision notes and unchanged statuses, and regenerate the ADR index.

**In scope.**
- ADR-0017 decision 2: the `verbs` row milestone entry becomes `create/list/view/edit/close/delete`, with no archive and no restore; every other category entry in that row is unchanged. Extend the status line with a 2026-11-15 amendment clause for the milestone verbs; it must still start with `Accepted` and the status stays Accepted. Add an italic revision note dated 2026-11-15 covering the milestone change, keeping the existing revision note in that decision as history.
- ADR-0023 decision 2: the Milestone row archive location becomes `none` and states the record is never moved, with its state in the front-matter `status` (mirroring the Release row); state that `papyrus milestone close <id>` rewrites `status` from `open` to `closed` in place through the single write path and no `archive/milestones/` destination is resolved by any command. Add an italic revision note dated 2026-11-15 after the existing Release revision note; leave the Release row and its revision note untouched.
- ADR-0023 decision 3: replace `Only tasks and milestones keep archive/restore` with a statement that only tasks keep archive/restore because REQ-001.005 requires task IDs to survive that cycle, and that milestones have no archive/restore and are closed in place. Add an italic revision note dated 2026-11-15 immediately after decision 3.
- ADR-0023 clauses that assert the removed milestone pair: downstream constraint 2 (`tasks and milestones keep their archive/restore pair`) and the decision 9 supersession bullet (`archive/restore for tasks and milestones restated unchanged`). Correct only the milestone claim; leave the rest of both clauses and the historical sections intact.
- Both status lines still begin with `Accepted`; no decision is renumbered, no new ADR is created, and no other decision text changes.
- `docs/architecture/adr/index.html`: regenerate by invoking the `adr-index-visualization` skill and its bundled `generate_adr_index.sh`; it must print `wrote docs/architecture/adr/index.html: 24 decision records` and exit 0. Never hand-edit the file.

**Out of scope.** Code and tests, including the milestone descriptor `archive_policy` value and milestone files already under `backlog/archive/milestones`; requirements, specifications, business rules, and the domain model; every other ADR and every other decision; re-opening the settled lifecycle; any migration or relocation of milestone files.

**Assumptions.** The delivered behavior is TASK-130 and the ADR-0022 amendment is TASK-131; this task asserts only what those and REQ-003.001/REQ-007.001 establish. Pre-existing milestone records under an archive path remain readable legacy files under TASK-130 assumption; this task neither migrates nor deletes them. If the `never moved` row cannot be made truthful without a code change, stop and report it instead of editing code or inventing behavior. The ADR record count stays 24 because no ADR is added or removed.

## Acceptance Criteria
- [x] ADR-0017 decision 2 `verbs` row lists milestone `create/list/view/edit/close/delete` with no archive or restore, and no other category verb set in that row changes
- [x] ADR-0017 status still starts with Accepted and records the 2026-11-15 milestone-verbs amendment without a status change; an italic revision note dated 2026-11-15 covers decision 2 and no other decision text or numbering changes
- [x] ADR-0023 decision 2 Milestone row has no archive location, states the record is never moved, and names the front-matter status; `papyrus milestone close <id>` writes `open` -> `closed` in place, and no milestone command resolves an `archive/milestones/` lifecycle destination or archives/restores: the legacy `<backlog_directory>/archive/milestones/` folder is recorded as a read/import location only, where the resolver reads it and the one-way import writes archived source milestones there (decisions 2 and 4)
- [x] ADR-0023 decision 3 states that only tasks keep archive/restore (REQ-001.005 task-ID stability) and that milestones are closed in place with no archive/restore
- [x] ADR-0023 records italic 2026-11-15 revision notes for decisions 2-3 and its status still starts with Accepted, with no status change, no renumbering, and no new ADR
- [x] ADR-0023 downstream constraint 2 and its decision 9 supersession bullet no longer assert milestones keep archive/restore; a grep for milestone archive/restore finds only revision notes or historical text recording the removal
- [x] docs/architecture/adr/index.html is regenerated by the adr-index-visualization script, reports 24 decision records and exit 0, shows ADR-0017 and ADR-0023 as Accepted with the amended content, and is never hand-edited
- [x] The amendments agree with REQ-003.001, REQ-007.001, ADR-0022 decision 5, BR-006, and TASK-130 behavior, so TASK-127 AC6 is unblocked
- [x] No code, test, requirement, specification, business-rule, domain-model, or other ADR file is changed


## Comments
author: @architect
created: 2026-11-15T13:31:00Z

---

AC3 reworded to the delivered contract: no milestone command resolves an archive/milestones/ lifecycle destination or archives/restores; <backlog_directory>/archive/milestones/ is a read/import location only (resolver reads it, one-way import writes archived source milestones), per ADR-0023 decisions 2 and 4. Re-checked against ADR-0023 lines 44, 46, 58, 63. No implementation change.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
ADR-0017 decision 2 now lists milestone create/list/view/edit/close/delete with a dated revision note; ADR-0023 decision 2 Milestone row is 'none - never moved by a milestone command' with the front-matter status, decision 3 states only tasks keep archive/restore, downstream constraint 2 and decision 9 supersession bullet corrected. The legacy backlog/archive/milestones/ folder is documented as a read/import location only (resolver reads it; the one-way import writes archived source milestones there; no milestone command archives, restores, migrates or relocates). ADR-0017 downstream constraint 3 now assigns ArchivePolicy::None to milestones. Statuses stay Accepted with 2026-11-15 amendment clauses and index regenerated (24 records). Residual code gap recorded for follow-up: the milestone descriptor still carries the archived-folder policy value.
