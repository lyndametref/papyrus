---
id: TASK-29
title: Review proposed ADR-0006 optimistic concurrency and atomic record writes
status: To Do
assignee:
  - '@architect'
created_date: '2026-11-03T21:11:00Z'
labels:
  - architecture
  - adr
  - review
milestone: m-2
dependencies: []
references:
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - docs/specifications/browser-requirements.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - TASK-6
  - TASK-7
type: task
ordinal: 7900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0006 was written during the TASK-6 architecture checkpoint with status **Proposed**. It fixes how browser edits detect lost updates and how records are written without partial-file exposure, and every later writing surface (TASK-11, TASK-15, TASK-23 … TASK-27) inherits it. It must be decided before m-2 implementation starts.

Scope of this review: verify the ADR against the code actually delivered in m-1 (TASK-6, TASK-7) and against the specifications and requirements it cites. Decide accepted, rejected, or amend. If accepted or amended, update the ADR and regenerate the ADR index.

Review must check, at minimum:
- Decision vs. delivered code: version token composition, 400/409 handling, `create_new` temporary file with numeric suffix retry, `sync_all`, permission copy, rename, best-effort directory sync, failure-path cleanup, save-then-reread, exact ID matching, `Context::ensure_within` on every path, unknown-section preservation, section order, and trailing-newline rule.
- Context accuracy: the claim that the CLI and browser can write the same files, and the constraints attributed to NFR-04.001 and NFR-04.002.
- Consequences and residual risks: the documented check-then-rename race is stated honestly; any residual risk not recorded (for example per-record config re-reads on write, duplicate record parsing between read and write paths, temporary-file leftovers after crash).
- Checkpoint follow-ups the review must either fold into the ADR or explicitly defer: per-record config re-reads, duplicate record parsing, test temp-directory hygiene for write tests, and the manpage/CLI write-path claims that overlap record writes.

Follow-ups: this task may spawn separate follow-up tasks for any accepted gap. Do not create implementation tasks as part of this review unless the decision makes one necessary; capture them as follow-up items instead.

Acceptance of the ADR content is an explicit user decision: the ticket may only record `accepted` after the user has confirmed it through the orchestrator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The ADR and the delivered m-1 implementation (TASK-6/7 code, tests, packaging) have been read and the review evidence is cited in the ticket.
- [ ] #2 Decision, context, consequences, and residual risks are verified point by point against the code and the cited specifications/requirements, with discrepancies listed.
- [ ] #3 A decision of accepted, rejected, or amend is recorded in the ticket with rationale and evidence.
- [ ] #4 If accepted or amended: ADR-0006 Status is updated (and content amended where needed) and docs/architecture/adr/index.html is regenerated per the architecture-decision-record and adr-index-visualization skills.
- [ ] #5 The user has explicitly confirmed the acceptance decision through the orchestrator, and that confirmation is recorded in the ticket.
- [ ] #6 Follow-up items (including the checkpoint follow-ups named in the description) are captured in the ticket as follow-up notes or new tickets, without being implemented here.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Review evidence recorded in the ticket, each claim tied to a file path, test, or spec section.
- [ ] #2 ADR-0006 status and content match the recorded decision; a rejection states the reason and what replaces the decision.
- [ ] #3 docs/architecture/adr/index.html regenerated and consistent with ADR-0006 when the ADR changed.
- [ ] #4 Follow-ups captured for rejected or amended decisions, with owners implied by the affected area.
- [ ] #5 Explicit user acceptance recorded; no task closure without it.
- [ ] #6 No changes outside the ADR files and the ADR index.
<!-- DOD:END -->
