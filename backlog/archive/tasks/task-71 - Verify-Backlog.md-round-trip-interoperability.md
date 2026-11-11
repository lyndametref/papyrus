---
id: TASK-71
title: Verify Backlog.md round-trip interoperability
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-09T20:53:00Z'
labels:
  - verification
  - backlog-interop
milestone: m-12
dependencies:
  - TASK-69
  - TASK-70
references:
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-05.010.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
documentation:
  - docs/specifications/verify-backlog-md-round-trip-interoperability.md
priority: high
type: chore
ordinal: 30085
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Empirically verify NFR-04.003: Papyrus task and milestone files remain readable, listable, and updatable by Backlog.md in both directions, including translated identity (REQ-01.005) and the additive milestone general keys and release (REQ-05.010), on newly created records and on records migrated lazily on first read and on first write from both input shapes. This is the Backlog.md round-trip gate and a verification task only: no new product behavior is added. Scope, procedure, acceptance criteria, and blocker recording are defined in docs/specifications/verify-backlog-md-round-trip-interoperability.md. Verification uses a scratch project under ./.tmp/ and read-only checks against the repository backlog; any defect found is recorded as a blocker with exact command and output and is not resolved by this ticket.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Verification evidence shows a Backlog.md-created task is listed by Papyrus under its translated display ID and papyrus task view <ID> resolves with matching title, status, assignee, labels, milestone, dependencies, references, documentation, type, and ordinal (or any mismatch is recorded as a blocker).
- [ ] #2 Verification evidence shows a Papyrus-created or migrated task is listed by Backlog.md with the same title and that Backlog.md view/edit preserve or are documented to default the required fields (or any dropped field is recorded as a blocker).
- [ ] #3 Verification evidence shows milestone m-12 maps to MS-013, papyrus milestone view MS-013 resolves, and a dependency stored as TASK-31 displays as TASK-031.
- [ ] #4 Verification evidence shows Backlog.md list/view/update commands accept a milestone file carrying the general keys plus release and preserve those keys (or any dropped key is recorded as a blocker with exact command and output).
- [ ] #5 Verification covers newly created records and records migrated from both input shapes, with idempotency demonstrated by byte-identical output on a second pass.
- [ ] #6 Any field rejected, rewritten, or dropped by either tool is recorded as a blocker with exact command and output; TASK-71 is not Done while such a blocker exists.
- [ ] #7 Verification evidence records environment details (Papyrus binary, Backlog.md version, date/time in UTC) and the scratch project location.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Verification evidence is recorded in the TASK-71 ticket notes with commands, observed output, and file content where relevant.
- [ ] #2 No unresolved blocker remains, or each blocker is linked to a follow-up task.
- [ ] #3 No repository record file is modified except read-only identity checks; scratch projects are removed after verification.
- [ ] #4 No product code is changed by this ticket; any required fix is tracked separately.
<!-- DOD:END -->
