---
id: TASK-106
title: Correct the stale stricter-than-requirement notes in ADR-0019
status: To Do
assignee:
  - architect
created_date: '2026-11-13T21:24:00Z'
labels:
  - architecture
milestone: m-17
dependencies: []
documentation:
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - docs/requirements/business/REQ-07.002.md
priority: high
ordinal: 41005
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0019 was written while three validation rules were owner proposals: case-insensitive reserved-name matching, prefix uniqueness across disabled entries, and rejection of duplicate directories among enabled entries. It still says these are deliberately stricter than the requirement and flagged for confirmation. REQ-07.002 is now validated and explicitly requires all three, so the ADR text misstates the contract and would let a reader treat required behavior as optional. This is a documentation correction only; no architecture decision changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Decision 3 in ADR-0019 states that REQ-07.002 requires case-insensitive matching of the reserved built-in command names, instead of saying it is stricter and awaiting confirmation.
- [ ] #2 Decision 5 states that prefix uniqueness across all configured document types, including disabled entries, is required by REQ-07.002.
- [ ] #3 Decision 6 states that rejecting duplicate directories among enabled entries is required by REQ-07.002.
- [ ] #4 The Consequences bullet about stricter rules and the two open questions that are now settled are rewritten or removed so no stale confirmation request remains.
- [ ] #5 No other decision, status, index entry, or ADR is changed; the ADR stays Accepted.
<!-- AC:END -->
