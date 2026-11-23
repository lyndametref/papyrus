---
id: TASK-130
title: >-
  Rewrite REQ-006.013 archived-milestone clauses to the close-only milestone
  lifecycle
status: Done
assignee:
  - @requirements-engineer
created_date: '2026-11-15T13:59:00Z'
updated_date: '2026-11-15T14:19:00Z'
labels:
  - docs
  - requirements
milestone: MS-021
dependencies:
  - TASK-129
references:
  - docs/requirements/business/REQ-006.013.md
  - docs/requirements/business/REQ-003.001.md
  - docs/requirements/business/index.html
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - TASK-132
priority: medium
ordinal: 41410
---


## Description
**Why.** `docs/requirements/business/REQ-006.013.md` still says "Archived tasks and milestones remain excluded from the board" in its Description and "Archived tasks and milestones are not shown on the board" in its acceptance criteria. Milestones have no archived lifecycle: their fixed lifecycle is `open -> closed` and `papyrus milestone close <id>` writes the status in place (REQ-003.001, ADR-0022 decision 5, ADR-0023 decision 2). Only tasks keep archive/restore. REQ-006.013 therefore describes a milestone state that does not exist and contradicts REQ-003.001 and the ADRs amended in m-19 (TASK-132). This is residual text drift found by the m-19 checkpoint, not a behavior change.

**What.** Rewrite the two archived-milestone clauses so they state the close-only milestone lifecycle: archived **tasks** stay excluded from the board, while milestones have no archived lifecycle - a closed milestone remains in `backlog/milestones/` and remains shown in its release or unassigned region. Then regenerate the business requirement index.

**In scope.**
- Description, final sentence: replace "Archived tasks and milestones remain excluded from the board, as in the rest of the browser backlog views." with wording that (a) archived tasks are excluded, (b) milestones have no archived lifecycle and are never hidden by an archive state, and (c) a closed milestone stays in the active folder and remains visible on the board. Cite REQ-003.001 for the `open -> closed` lifecycle.
- Acceptance criterion 12: replace "Archived tasks and milestones are not shown on the board." with criteria that state archived tasks are not shown, every milestone is shown in its release or unassigned region including closed milestones, and no milestone archive state exists.
- Traceability: add REQ-003.001 as the source of the milestone lifecycle claim; keep every existing reference.
- Regenerate `docs/requirements/business/index.html` with the `requirements-index-visualization` skill and its bundled `generate_index.sh`; report each `wrote ...` line and exit 0. Never hand-edit the index.

**Out of scope.** The text of REQ-003.001 and any other requirement, requirement IDs, criticality, user story, group, the other acceptance criteria, `docs/specifications/milestone-and-release-management.md` (TASK-136), code and browser behavior (TASK-134), ADRs, business rules, and the non-functional index.

**Assumptions.** The browser backlog view lists milestones from the active folder only and a closed milestone remains in that folder, so "closed milestones remain visible" is the delivered behavior; TASK-134 removes the last code path that could archive a milestone. This task changes text and the generated index only.

## Acceptance Criteria
- [x] REQ-006.013 Description no longer says archived milestones are excluded and states that milestones have no archived lifecycle, that papyrus milestone close <id> moves open -> closed in place (REQ-003.001), and that a closed milestone remains visible.
- [x] REQ-006.013 acceptance criterion 12 no longer says archived milestones are not shown; it states that archived tasks are excluded, that every milestone appears in its release or unassigned region including closed milestones, and that no archived milestone state exists.
- [x] The new milestone-lifecycle wording cites REQ-003.001, and REQ-003.001 appears in REQ-006.013 Traceability while every existing reference is kept.
- [x] No other requirement text, ID, criticality, user story, group, acceptance criterion, or traceability line changes.
- [x] docs/requirements/business/index.html is regenerated with the requirements-index-visualization skill; the run reports its wrote ... line(s) and exits 0, and the index is not hand-edited.
- [x] No code, specification, ADR, or business-rule file changes.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
REQ-006.013 rewritten to the close-only milestone lifecycle: archived tasks remain excluded from the board, every milestone is shown in its release or unassigned region including a closed one, and no archived milestone state exists (no archive/restore command; papyrus milestone close writes open -> closed in place per REQ-003.001). Traceability adds REQ-003.001 and cites ADR-0022 decision 5 / ADR-0023 decision 2 / TASK-130; all prior references kept. Business and non-functional indexes regenerated (47/15, exit 0, byte-identical reproductions); no other requirement, code, spec, or business-rule change. Evidence: milestone_cli 4/4, browser board tests 6/6.
