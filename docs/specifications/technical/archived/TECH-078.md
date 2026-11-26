---
id: TECH-078
title: "Feature Roadmap"
created_date: '2026-11-16T21:03:55Z'
updated_date: '2026-11-19T21:33:23Z'
labels: []
references:
  - RE-002
  - RE-004
  - RE-005
  - REQ-003.001
  - REQ-006.012
  - REQ-006.018
  - RE-006
  - REQ-006.019
  - MS-028
  - MOCK-0016
  - RE-007
  - MS-029
  - RE-008
  - MS-030
  - REQ-006.020
  - REQ-006.021
  - REQ-006.022
status: archived
---
# TECH-078 Feature Roadmap

## Description
Ordered feature roadmap for Papyrus.

Entries:
1. RE-002 — milestone archive command (MS-025)
   - Source: REQ-003.001, RE-002, BR-006, BR-001, GLO-006, TECH-009
   - Rationale: CLI archive lifecycle.
2. RE-004 — uniform single-document browser layout (MS-026)
   - Source: REQ-006.012, RE-004
   - Rationale: consistent metadata layout.
3. RE-005 — remove browser sections (MS-027)
   - Source: REQ-006.018, RE-005
   - Rationale: focused browser navigation.
4. RE-007 — multiline input for create and update commands (MS-029)
   - Source: RE-007
   - Rationale: supported multiline body and content input on stdin; highest-priority open work.
5. RE-006 — remove reference list from document-type browser list views (MS-028)
   - Source: REQ-006.019, RE-006, MOCK-0016
   - Rationale: compact document-type list rows.
6. RE-008 — browser Backlog milestone and release ordering by lowest contained task ordinal ascending with lifecycle markers and cleanup (MS-030)
   - Source: REQ-006.020, REQ-006.021, REQ-006.022, RE-008, MOCK-0002, MOCK-0017
   - Rationale: earliest sequenced work first, plus open and closed markers and archive cleanup; planned in REL-001.

Priority order: delivered RE-002, then RE-004, then RE-005; open work RE-007 before RE-006; RE-008 appended after the existing entries without changing their relative priority, and its relative priority to the open work is not recorded.
