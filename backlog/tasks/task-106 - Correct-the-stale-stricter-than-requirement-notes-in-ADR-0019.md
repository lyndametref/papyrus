---
id: TASK-106
title: Correct stale ADR wording in ADR-0019 and ADR-0020
status: Initiated
assignee:
  - architect
created_date: '2026-11-13T21:24:00Z'
updated_date: '2026-11-14T07:43:00Z'
labels:
  - architecture
milestone: m-17
dependencies: []
documentation:
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - docs/requirements/business/REQ-07.002.md
  - >-
    docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
priority: high
ordinal: 41005
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Two accepted ADRs still state behavior that the validated requirements and the registry encoding have moved past, so a reader can mistake required behavior for optional or read a contradictory id shape. ADR-0019 was written while three validation rules were owner proposals: case-insensitive reserved-name matching, prefix uniqueness across disabled entries, and rejection of duplicate directories among enabled entries. It still says these are deliberately stricter than the requirement and flagged for confirmation, but REQ-07.002 is now validated and explicitly requires all three. ADR-0020 decision 4 says `req`/`nfr` use `id_shape: Dotted`, while ADR-0018 decision 5 pairs `Dotted`/`Grouped` as their id shapes and TASK-105 encodes `Grouped` (group records plus dotted member records, with `Dotted` reserved for the member-only shape). The implementation is authoritative here and the ADR wording must match. Both corrections are documentation only; no architecture decision, requirement text, or other ADR changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Decision 3 in ADR-0019 states that REQ-07.002 requires case-insensitive matching of the reserved built-in command names, instead of saying it is stricter and awaiting confirmation.
- [ ] #2 Decision 5 states that prefix uniqueness across all configured document types, including disabled entries, is required by REQ-07.002.
- [ ] #3 Decision 6 states that rejecting duplicate directories among enabled entries is required by REQ-07.002.
- [ ] #4 The Consequences bullet about stricter rules and the two open questions that are now settled are rewritten or removed so no stale confirmation request remains in ADR-0019.
- [ ] #5 ADR-0020 decision 4 declares req and nfr as id_shape: Grouped (group records plus dotted member records), keeps Dotted described as the member-only shape, and agrees with ADR-0018 decision 5 and TASK-105 encoding decision D1.
- [ ] #6 Only ADR-0019 and ADR-0020 change; both stay Accepted, and no other decision, status, index entry, ADR, or requirement text is edited.
<!-- AC:END -->
