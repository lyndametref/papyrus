---
id: TASK-259
title: "Delete the nine typed document modules and ten per-type ops builders"
created_date: '2026-11-20T17:42:17Z'
updated_date: '2026-11-20T17:42:17Z'
labels: []
references:
  - ADR-036
  - TECH-070
  - TECH-066
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-256
  - TASK-257
  - TASK-258
ordinal: 70
documentation: []
type: implementation
---

## Description
Context: once the generic handler covers every document type, the per-type code is unreachable duplication.

Scope:
- Delete src/test_case.rs, test_report.rs, use_case.rs, ui_spec.rs, glossary.rs, tech_spec.rs, requirement.rs, rule.rs, audit.rs.
- Keep ui_mockup.rs as the SingleHtmlMockup surface.
- Remove the ten per-type ops builders from record/edit.rs.
- Keep the backlog and decision modules untouched.
- Confirm no document type loses a command, option, field, or status behavior: the seed must already carry every per-type field and presentation fact.
- Record every behavior found missing from the seed before deleting the module that provided it.
