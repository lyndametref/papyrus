---
id: TASK-22
title: Research and define the release lifecycle
status: Done
assignee: []
created_date: '2026-10-31T22:37:00Z'
updated_date: '2026-11-02T18:30:00Z'
labels: []
milestone: m-0
dependencies:
  - TASK-1
references:
  - README.md
  - docs/requirements/business/REQ-03.002.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/specifications/release-lifecycle.md
priority: high
ordinal: 2500
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Research and define the Papyrus release lifecycle before release behavior is implemented.

Scope:
- Audit the README manpage (papyrus(1)) release command entry and STATUSES -> Releases, contrasting with milestones, which do document an archive command.
- Reconcile REQ-03.002, BR-007, the release glossary term, and the business object model with the clarified lifecycle.
- Record the product decision that releases are not archived: a release's terminal delivery state is 'released', distinct from archiving (record management).
- Update the README STATUSES -> Releases line to 'active -> released' and keep BR-007's verbatim excerpt aligned.
- Record any unresolved mechanism (how a release becomes 'released'; stored vs. derived state) as an open question in REQ-03.002 rather than inferring an undocumented command or transition.

Out of scope:
- Implementing or changing papyrus release commands, storage, or status transitions.
- Inventing a release archive/restore/delete command or a --status option not documented in the README.
- Editing other tasks (e.g., TASK-8) beyond noting consistency expectations.

Assumptions:
- Assumption: README.md is the sole behavior source for papyrus release (BR-007); no command or transition is inferred beyond it.
- Assumption: 'released' is a delivery-completion state, not a record-management/archive state.
- Assumption: Requirements file edits (REQ-03.002) are owned by requirements-engineer and must be routed to/verified by that role during implementation.
- Assumption: The README manpage may be updated for the STATUSES -> Releases line because the user confirmed this.
- Assumption: No UI is involved; no mockups are required.

Dependencies: TASK-1 (completed). Downstream: TASK-8 (Manage milestones and releases from the CLI) depends on this task and must keep its acceptance criterion #3 consistent (no undocumented archive command for releases).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Research the README manpage release command entry and STATUSES -> Releases line, contrasting with milestones, and record the exact source references (README lines) used.
- [ ] #2 Define the release lifecycle as active -> released (delivery completion), distinct from archiving (record management), and state that releases have no documented archive/restore/delete command.
- [ ] #3 Update REQ-03.002 to reflect the clarified lifecycle and distinguish release completion from archiving.
- [ ] #4 Record in REQ-03.002 any unresolved mechanism for marking a release as released (stored vs. derived state) as an open question instead of inferring an undocumented command or transition.
- [ ] #5 Update the README STATUSES -> Releases line to active -> released and keep the BR-007 verbatim manpage excerpt aligned with README.
- [ ] #6 Update affected lifecycle documentation: docs/domain-model/business-glossary/release.md and the Release entry in docs/domain-model/object-model/business-object-model.md.
- [ ] #7 Maintain traceability: REQ-03.002 and docs/specifications/release-lifecycle.md reference README lines, BR-007, the glossary term, and the BOM entry.
- [ ] #8 Do not introduce or document any release archive, restore, delete, or status-edit command beyond those in the README manpage.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Requirements edits to REQ-03.002 are made/verified through requirements-engineer (REQ files are owned by that role).
- [ ] #2 README, BR-007, glossary, BOM, and the release-lifecycle specification are consistent with each other.
- [ ] #3 Open questions are explicitly recorded in REQ-03.002; no undocumented inference is presented as confirmed behavior.
- [ ] #4 No implementation code or CLI behavior is changed by this task.
- [ ] #5 No UI mockups required (none).
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read README manpage release entry and STATUSES sections; record exact source lines.
2. Read REQ-03.002, BR-007, release glossary term, and BOM release description.
3. Route REQ-03.002 edits to requirements-engineer: update lifecycle wording to active -> released, distinguish completion from archiving, and add open questions for unresolved mechanism.
4. Update README STATUSES -> Releases to active -> released and align BR-007 verbatim excerpt.
5. Update release glossary term and BOM release description lifecycle wording.
6. Verify traceability and consistency with TASK-8 acceptance criterion #3.
7. Run documentation consistency checks; no code changes.
<!-- SECTION:PLAN:END -->
