---
id: TASK-152
title: "Refine the Backlog.md import metadata fix"
created_date: '2026-11-16T19:34:34Z'
updated_date: '2026-11-17T10:38:30Z'
labels:
  - bug
references:
  - RE-001
  - NFR-04.003
  - REQ-05.010
  - REQ-05.011
  - NFR-04.004
status: Done
assignee:
  - task-refiner
milestone: MS-023
dependencies:
  - TASK-151
ordinal: 20
documentation:
  - src/import.rs
  - src/record/frontmatter.rs
type: refinement
---

## Description
Outcome: confirm all information for implementing the import metadata fix is available and complete the task refinement fields. Root cause is format_scalar not quoting at-prefixed scalars; repair import write path and affected task files TASK-144 and TASK-145. Scope assumptions and acceptance criteria are in the refinement comment.
## Acceptance Criteria
- [ ] The defect and required code locations are identified: record frontmatter format_scalar and import write_one assignee handling.
- [ ] The repair set is identified: task files TASK-144 and TASK-145.
- [ ] The change preserves existing behavior for non-at scalars and empty lists.
- [ ] Traceability to TC-001 through TC-006, REQ-05.010, REQ-05.011, NFR-04.004, and request RE-001 is recorded.
- [ ] No open material questions remain.
## Comments
- **task-refiner**: Line one. Line two with slash? no.
- **task-refiner**: Refinement complete. Root cause: format_scalar does not quote at-prefixed scalars, so import write_one emits invalid YAML for assignee entries. Required fix: quote at-prefixed scalars as single-quoted values in record frontmatter format_scalar, preserving import handling and empty lists. Repair affected task files TASK-144 and TASK-145. Traceability: TC-001 to TC-006, REQ-05.010, REQ-05.011, NFR-04.004, RE-001. No open material questions.
