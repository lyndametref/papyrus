---
id: TASK-15
title: Extend browser access to decision records
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-05T18:28:00Z'
labels:
  - browser
  - decisions
milestone: m-3
dependencies:
  - TASK-11
  - TASK-13
references:
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/specifications/browser-decision-records.md
  - >-
    docs/specifications/ui/mockups/MOCK-0003-browser-decision-records-review.html
  - docs/business-rules/BR-008-decision-command-behavior.md
documentation:
  - docs/specifications/browser-decision-records.md
  - docs/specifications/ui/browser-decision-records-ui.md
type: feature
ordinal: 14000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the offline browser (TASK-6 baseline, ADR-0005/ADR-0006) to serve active decision records from the decisions store, with read and edit support, under the same transport, asset, rendering, security, and configuration baseline.

Scope:
- Categories: extend the persistent sidebar with a Decisions entry and the ?type= selector to decision (default req; any other value 400).
- List/view/edit active decision records; archived decisions hidden and unreachable (404).
- Decision editable fields: title, status, decision, context, consequences, superseded-by. Type and ID read-only. Supersession follows the CLI rule (mutually exclusive with status).
- JSON API endpoints for decisions and generic /api/doc/{id}.
- Reuse baseline escaping, ID resolution, boundary checks, Host/Origin validation, request limits, a11y.

Out of scope: create/archive/restore/delete from browser; other record types; search in browser; new deps/ADRs.

Spec: docs/specifications/browser-decision-records.md
UI: MOCK-0003 (pending) extends MOCK-0001/MOCK-0002.

Acceptance criteria mapping: AC-1..AC-5 in the spec; test scenarios T-15a..T-15j in spec section 10.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The local browser displays and edits active decision records in addition to all previously supported requirement and backlog content.
- [ ] #2 Category selection supports type=decision (default req); any other value, including all, returns 400.
- [ ] #3 Archived decisions are not listed and return 404 by URL.
- [ ] #4 Decision view shows ID, decision type, status, decision text, context, consequences, and supersession; the superseding-decision link resolves when possible.
- [ ] #5 An edit round-trip persists and is visible through both the browser and the CLI.
- [ ] #6 Field validation returns 422 and does not write for empty title, invalid status transition, unknown superseding decision, type mismatch, self-supersession, supplying both status and superseded-by, and lines starting with ## .
- [ ] #7 A stale version token returns 409 and does not write.
- [ ] #8 Writes are atomic, project-local, leave no temporary file, and symlink escapes return 404.
- [ ] #9 Browser operations remain offline-capable and use no paths outside the project directory except during init.
- [ ] #10 The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Unit and integration tests added/updated and pass.
- [ ] #2 cargo build, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
- [ ] #3 Feature spec and UI description updated with MOCK-0003 reference.
- [ ] #4 MOCK-0003 explicitly approved by the user before implementation.
- [ ] #5 No new dependencies, crates, or ADRs introduced.
- [ ] #6 Explicit test scenarios T-15a–T-15j (spec §10) implemented and passing.
<!-- DOD:END -->
