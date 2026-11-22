---
id: TASK-269
title: "Delete the remaining per-type browser branches and prove semantic equivalence"
created_date: '2026-11-20T17:42:52Z'
updated_date: '2026-11-22T12:56:37Z'
labels: []
references:
  - ADR-039
  - TECH-068
  - REQ-06.014
  - ADR-042
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-268
ordinal: 80
documentation: []
type: implementation
---

## Description
Context: after the generic list, view, edit, conflict, and not-found surfaces exist, the per-type branches are dead but still compiled.

Scope: delete every per-type document browser branch, including the code-behind of the audited conflict and not-found renderers. Zero document TypedHandler variants must remain. No document slug dispatch may exist outside the seed; tests and docs are exempt. Delete the eight document edit ops builders that MS-034 TASK-259 deliberately retained: generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, and use_case_ops. Replay the MS-032 HTML fixtures and compare semantically: selectors, text, navigation, form fields, and status codes identical; byte differences allowed only where attributed to the uniform title block or seed presentation. Backlog and Decision browser output must be byte-identical.
## Comments
- **planner**: MS-034 TASK-259 deferral (ADR-042): TASK-259 retains the eight document edit ops builders and the document TypedHandler variants because the browser consumes them. Delete the eight builders here together with the remaining per-type browser branches after TASK-266 lands the generic edit form and form_post. Builders: generic_document_ops, requirement_ops, glossary_ops, ui_spec_ops, business_rule_ops, audit_ops, tech_spec_ops, use_case_ops.
