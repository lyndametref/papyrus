---
id: TASK-45
title: 'Design review: business rules browser UI mockup'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T07:54:00Z'
labels:
  - design-review
  - ui-design
milestone: m-8
dependencies:
  - TASK-43
references:
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html
  - docs/specifications/ui/browser-business-rules-ui.md
documentation:
  - docs/specifications/ui/browser-business-rules-ui.md
  - docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html
  - docs/specifications/business-rules-management.md
  - docs/specifications/browser-requirements.md
priority: medium
type: task
ordinal: 26200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Design review for the business-rules browser UI (TASK-25 depends on this ticket). The UI designer produces and obtains explicit user approval for the business-rules browser mockup package before browser implementation. Deliverable is a single clickable mockup package plus a UI description document under docs/specifications/ui/; no source code.

Record model (from docs/specifications/business-rules-management.md, TASK-43): id (e.g. RUL-001), title, description, rationale, applicability, and zero or more references to business requirements (requirement IDs). Lifecycle active -> deprecated; deprecated rules are hidden from list, search, and browser (REQ-05.005, REQ-05.007, BR-013).

Mockup package: MOCK-0007 (docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html), status draft pending explicit user approval. UI description: docs/specifications/ui/browser-business-rules-ui.md.

Scope: list, view, edit for business rules (title, description, rationale, applicability, requirement refs with tag add/remove), desktop + 320 px, error pages, sidebar entry "Business Rules". Deprecated rules excluded.

Out of scope: CLI behavior (TASK-43), search (TASK-44), browser implementation (TASK-25), packaging (TASK-46).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Mockup package MOCK-0007 created per ui-clickable-mockup: review landing plus clickable web prototype happy path (list/empty/long, view, view-after-save, edit/dirty/errors/conflict), responsive 320 px list and edit, error pages, tag add/remove interaction for requirement refs, sidebar entry "Business Rules".
- [ ] #2 UI description document present at docs/specifications/ui/browser-business-rules-ui.md referencing MOCK-0007 and recording resolved open questions (sidebar position, requirement-ref rendering).
- [ ] #3 Mockup registry row added for MOCK-0007 with status draft (pending explicit user approval); mockup not auto-approved.
- [ ] #4 Mockup explicitly approved by the user before TASK-25 implementation; approval gate blocks Refined status until approved.
- [ ] #5 Ticket and UI description reference the MOCK id, UI doc, business-rules-management.md, browser-requirements baseline, and relevant REQ/NFR/BR.
- [ ] #6 No source code changes; validate_mockups.sh reports no errors for MOCK-0007.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Mockup package exists and passes validate_mockups.sh (no errors for MOCK-0007).
- [ ] #2 UI description doc present and consistent with the mockup.
- [ ] #3 Registry updated and status accurate (draft until explicit approval).
- [ ] #4 User explicitly approves mockup before ticket moves to Refined/Done.
- [ ] #5 No source code changes introduced.
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @ui-designer
created: 2026-11-07T07:40:00Z

---

UI mockup package created: MOCK-0007 (docs/specifications/ui/mockups/MOCK-0007-business-rules-review.html), status draft, pending explicit user approval. UI description: docs/specifications/ui/browser-business-rules-ui.md.

---

created: 2026-11-07T07:52:00Z

---

MOCK-0007 approved by user 2026-11-07 (UTC); registry + UI description updated to approved. TASK-45 ready for Refined.

---

created: 2026-11-07T07:54:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (MOCK-0007 approved; non-blocking: pre-existing validate_mockups.sh errors on MOCK-0001/2 out of scope). Auto-closed under no-validation mode per user request.

---
<!-- COMMENTS:END -->
