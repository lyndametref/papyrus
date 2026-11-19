---
id: TASK-214
title: "Fix pre-existing characterization replay empty-decision failure (AUD-123)"
created_date: '2026-11-19T19:11:39Z'
updated_date: '2026-11-19T19:12:02Z'
labels: []
references:
  - AUD-123
  - MS-030
  - TASK-142
status: ToDo
assignee:
  - test-manager
milestone: null
dependencies: []
ordinal: "80"
documentation:
  - tests/list_characterization_replay.rs
type: implementation
---

## Description
Quality audit AUD-123 deferred low finding. Resolve the unrelated failure in tests list characterization replay every empty fixture replays the shared empty state where project-empty contains ADR-001. Not an MS-030 defect.

## Acceptance Criteria
- [ ] The pre-existing failure in tests list characterization replay every empty fixture replays the shared empty state is resolved.
- [ ] cargo test passes without the known unrelated failure.
- [ ] No MS-030 browser behavior is changed.
