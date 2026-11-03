---
id: TASK-28
title: >-
  Review proposed ADR-0005 embedded dependency-free HTTP server and
  server-rendered UI
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
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - docs/specifications/browser-requirements.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - TASK-5
  - TASK-6
  - TASK-7
type: task
ordinal: 7800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0005 was written during the TASK-6 architecture checkpoint with status **Proposed**. It fixes the browser transport, asset delivery, rendering, module boundaries, and security posture that every later browser increment (TASK-11, TASK-15, TASK-19, TASK-23 … TASK-27) inherits, so it must be decided before m-2 implementation starts.

Scope of this review: verify the ADR against the code actually delivered in m-1 (TASK-5, TASK-6, TASK-7) and against the specifications and requirements it cites. Decide accepted, rejected, or amend. If accepted or amended, update the ADR and regenerate the ADR index.

Review must check, at minimum:
- Decision vs. delivered code: blocking `std::net` HTTP/1.1 server, one request per connection, connection/thread cap, timeouts, method and header rejection rules, fixed route table, ID resolution, embedded assets, escaping, JSON API.
- Context accuracy: the constraints attributed to ADR-0001, ADR-0004 and the TASK-6 specification.
- Consequences and residual risks: stated risks still match reality; any risk not recorded in the ADR.
- Module boundaries claimed in the ADR (`browser::http`, `browser::records`, `browser::render`, `browser::mod`, `config`, `context`) versus the actual layout.
- Checkpoint follow-ups the review must either fold into the ADR or explicitly defer: config loading out of `commands`, per-record config re-reads, duplicate record parsing, template placeholder hardening, test temp-directory hygiene, manpage over-promising, NFR-01.002 scope, and NFR-05.001 real-browser verification.

Follow-ups: this task may spawn separate follow-up tasks for any accepted gap. Do not create implementation tasks as part of this review unless the decision makes one necessary; capture them as follow-up items instead.

Acceptance of the ADR content is an explicit user decision: the ticket may only record `accepted` after the user has confirmed it through the orchestrator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The ADR and the delivered m-1 implementation (TASK-5/6/7 code, tests, packaging) have been read and the review evidence is cited in the ticket.
- [ ] #2 Decision, context, consequences, and residual risks are verified point by point against the code and the cited specifications/requirements, with discrepancies listed.
- [ ] #3 A decision of accepted, rejected, or amend is recorded in the ticket with rationale and evidence.
- [ ] #4 If accepted or amended: ADR-0005 Status is updated (and content amended where needed) and docs/architecture/adr/index.html is regenerated per the architecture-decision-record and adr-index-visualization skills.
- [ ] #5 The user has explicitly confirmed the acceptance decision through the orchestrator, and that confirmation is recorded in the ticket.
- [ ] #6 Follow-up items (including the checkpoint follow-ups named in the description) are captured in the ticket as follow-up notes or new tickets, without being implemented here.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Review evidence recorded in the ticket, each claim tied to a file path, test, or spec section.
- [ ] #2 ADR-0005 status and content match the recorded decision; a rejection states the reason and what replaces the decision.
- [ ] #3 docs/architecture/adr/index.html regenerated and consistent with ADR-0005 when the ADR changed.
- [ ] #4 Follow-ups captured for rejected or amended decisions, with owners implied by the affected area.
- [ ] #5 Explicit user acceptance recorded; no task closure without it.
- [ ] #6 No changes outside the ADR files and the ADR index.
<!-- DOD:END -->
