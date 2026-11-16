---
id: TASK-017
title: Manage remaining documentation records from the CLI
status: ToDo
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-05T18:48:00Z'
labels: []
milestone: MS-014
dependencies:
  - TASK-016
  - TASK-021
references:
  - docs/requirements/business/REQ-05.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
ordinal: 17000
---


## Description
Contributors need the remaining documented record types—glossary terms, UI specifications, business rules, and audit records—in the same project-local documentation store.

## Acceptance Criteria
- [ ] Users can create, list, view, and edit glossary terms, UI specifications, business rules, and audit records with documented fields and type rules.
- [ ] Document deprecation and restoration commands preserve documented identity, folder, and visibility behavior for each supported type.
- [ ] All record data remains in project-local text files, input is sanitized, and no paths outside the project directory are accessed.


## Notes
OBSOLETE: scope split into per-type CLI tasks (glossary, ui-spec, business rules, audit, deployment, technical spec, mockup) across milestones m-5..m-11. Retained for traceability; do not implement.
