---
id: TASK-256
title: "Implement GroupedMarkdown create-group, group semantics, and HTML mockup supersede"
created_date: '2026-11-20T17:41:57Z'
updated_date: '2026-11-20T17:41:57Z'
labels: []
references:
  - ADR-036
  - REQ-05.001
  - REQ-05.002
  - REQ-05.008
  - BR-009
  - BR-010
status: ToDo
assignee:
  - rust-dev
milestone: MS-034
dependencies:
  - TASK-255
ordinal: 40
documentation: []
type: implementation
---

## Description
Context: req and nfr are GroupedMarkdown; ui-mockup is SingleHtmlMockup. Both models are seed-only and have verbs the other models do not.

Scope:
- create-group for GroupedMarkdown with a plain group ID and group fields; --group on member create; member IDs are dotted under the group prefix; the group link is engine-owned and not a field.
- Membership is immutable after creation. Preserve the required existing active group of the same type, cross-type rejection, archived-group rejection, and no cascade on archive or delete.
- Merged list ordering across groups and members is preserved.
- supersede exists only for SingleHtmlMockup; ui-mockup has no status verb and keeps fixed pending, approved, archived metadata; the HTML file stays Papyrus-owned.
