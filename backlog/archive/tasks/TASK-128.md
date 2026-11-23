---
id: TASK-128
title: >-
  Correct REQ-001.005 ID stability to tasks only and remove the stale REQ-003.001
  BR-006 revision note
status: Done
assignee:
  - requirements-engineer
created_date: '2026-11-15T13:13:00Z'
updated_date: '2026-11-15T13:39:00Z'
labels:
  - docs
  - requirements
milestone: MS-020
dependencies: []
references:
  - docs/requirements/business/REQ-001.005.md
  - docs/requirements/business/REQ-003.001.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - TASK-130
  - TASK-127
documentation:
  - docs/requirements/business/index.html
priority: medium
ordinal: 41314
---


## Description
**Why.** REQ-001.005 still states that IDs remain stable `across archive/restore cycles for tasks and milestones`, and its second acceptance criterion says `archiving and restoring a task or milestone`. Two artifact lifecycles are fixed and have no archive verb: milestones are `open -> closed` through `papyrus milestone close <id>` (REQ-003.001, REQ-007.001, TASK-130, ADR-0022 decision 5) and releases are `planned -> released` through `papyrus release close <id>`, where REQ-003.002 states that `released` is delivery completion, distinct from archiving, and that releases are not archived. REQ-001.005 therefore describes removed operations and contradicts REQ-003.001 and REQ-003.002. REQ-003.001 traceability also still carries `BR-006 - milestone Command Behavior (needs revision: archive becomes close)` although BR-006 now documents the close-only lifecycle, so the note makes a settled rule look outstanding.

**What.** Correct REQ-001.005 so archive/restore ID stability covers tasks only and archiving covers documents only, state that milestones and releases are closed in place rather than archived, replace the stale BR-006 revision note in REQ-003.001 with a note recording that the close-only milestone lifecycle is delivered and documented, then regenerate the requirement indexes. No behavior change.

**In scope.**
- REQ-001.005 Description: IDs remain stable across archive/restore cycles for tasks only and across archiving for documents; the sentence must not claim archive/restore for milestones or releases and must state that `papyrus milestone close <id>` moves a milestone from `open` to `closed` in place (REQ-003.001) and that releases are never archived because `papyrus release close <id>` moves a release from `planned` to `released` (REQ-003.002).
- REQ-001.005 acceptance criterion 2: state that archiving and restoring a task does not change its ID and archiving a document does not change its ID; documents have no restore command, and milestones are never archived because `papyrus milestone close <id>` changes `open` to `closed` in place (REQ-003.001).
- REQ-001.005 traceability: cite REQ-003.001 for the milestone exclusion and REQ-003.002 for the release exclusion, and keep the existing references.
- REQ-003.001 traceability: remove only the parenthetical `(needs revision: archive becomes close)` and keep `BR-006 - milestone Command Behavior`, because BR-006 now documents the close-only lifecycle and remains the valid trace; record that TASK-130 delivered the close-only milestone lifecycle and TASK-127 corrected the specification pages, so no revision note remains.
- Regenerate `docs/requirements/business/index.html` and `docs/requirements/non-functional/index.html` with the `requirements-index-visualization` skill and its bundled `generate_index.sh`; report each `wrote ...` line and exit 0, and never hand-edit either index.

**Out of scope.** CLI behavior, code, tests, ADRs, specifications, business rules, the domain model, requirement IDs and criticality, and other requirement text - including the content of REQ-003.002 itself, which REQ-001.005 cites without being edited, and the stale BR notes on REQ-003.002 (BR-007), REQ-004.002 (BR-008), REQ-005.002 (BR-010), and REQ-005.009 (BR-016/BR-017), which this task does not touch.

**Assumptions.** This is a wording and traceability correction only. TASK-130 delivered the close-only milestone lifecycle and TASK-127 corrected the specification pages; they are the traceability context, not dependencies. The release lifecycle and the absence of a release archive command are read from REQ-003.002 and BR-007, not invented here.

## Acceptance Criteria
- [x] REQ-001.005 Description states ID stability across archive/restore for tasks only (and across archiving for documents), and no longer claims archive/restore for milestones or releases
- [x] REQ-001.005 acceptance criterion 2 names only tasks for archive-and-restore, documents for archive-only, and states that milestones are never archived (REQ-003.001)
- [x] REQ-001.005 traceability cites REQ-003.001 for the milestone exclusion and REQ-003.002 for the release exclusion, and keeps the existing references
- [x] REQ-003.001 traceability no longer contains "needs revision: archive becomes close", keeps BR-006 cited as the milestone command behavior trace, and notes that TASK-130 delivered the close-only lifecycle and TASK-127 corrected the specification pages
- [x] No other requirement text, ID, criticality, acceptance criterion, or traceability line changes; the content of REQ-003.002 itself and the other stale BR notes are untouched
- [x] Both requirement indexes are regenerated by the requirements-index-visualization script after the edits, report success and exit 0, reflect the corrected REQ-001.005 and REQ-003.001, and are never hand-edited
- [x] A grep of REQ-001.005 and REQ-003.001 finds no claim that milestones or releases are archived or restored and no revision note about BR-006


## Notes
Retrofit authorization 2026-11-15. The three documentation edits were already implemented and are authorized as-is, not reverted and not re-implemented. Scope was widened to include the release exclusion and the TASK-130/TASK-127 trace note.

**Edits verified as factually accurate**
- `papyrus release close <id>` and "releases are not archived" come from REQ-003.002 description and AC 5/6 and README.md lines 334 and 355; BR-007 records the same command behavior.
- BR-006 documents the close-only milestone lifecycle (`close` is the only transition, no archive or restore verb, no `--status` transition option).
- TASK-130 delivered the close-only lifecycle (commit d34e152, merge of task/TASK-130); TASK-127 corrected the specification pages and BR-006 (commit c152cb9).

**Files changed (docs only)**
- `docs/requirements/business/REQ-001.005.md`: description now scopes archive/restore to tasks only and archiving to documents, and states that milestones (REQ-003.001) and releases (REQ-003.002) are closed in place and never archived; AC 2 rewritten accordingly; traceability gains REQ-003.001 and REQ-003.002 bullets, existing bullets kept.
- `docs/requirements/business/REQ-003.001.md`: `(needs revision: archive becomes close)` removed, `BR-006 - milestone Command Behavior` kept, plus a note that TASK-130 and TASK-127 delivered and documented the lifecycle.
- `docs/requirements/business/index.html` and `docs/requirements/non-functional/index.html` regenerated by the requirements-index-visualization script (wrote lines, exit 0); indexes embed the corrected REQ-001.005 description and contain no traceability text, so the traceability edits need no index change beyond regeneration.

**AC5 evidence**: `git status --porcelain -- docs/` lists only the two requirement files and the business index; REQ-003.002 is untouched.

**AC7 evidence**: grep for archive/restore near "milestone"/"release" in both files returns only negative statements ("never archived", "no restore command"); grep for "needs revision" returns no match in either file.

**Not changed**: code, CLI behavior, tests, ADRs, specifications, business rules, domain model, requirement IDs/criticality, and the stale BR notes on REQ-003.002, REQ-004.002, REQ-005.002, and REQ-005.009.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
REQ-001.005 corrected: IDs remain stable across archive/restore for tasks only, across archiving for documents (no restore); milestones are never archived (papyrus milestone close moves open -> closed in place, REQ-003.001) and releases likewise (papyrus release close, REQ-003.002). Traceability cites REQ-003.001 and REQ-003.002. REQ-003.001's stale 'needs revision: archive becomes close' parenthetical removed and replaced with a note that TASK-130 delivered the close-only lifecycle and TASK-127 corrected the specs and BR-006. Both requirement indexes regenerated (47 business / 15 non-functional, exit 0); REQ-003.002 content and the other stale BR notes untouched; no code/ADR/BR/spec edits.
