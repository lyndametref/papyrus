---
id: TASK-249
title: "Add the seed table, derivation, and selector-path resolution"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-20T17:41:37Z'
labels: []
references:
  - ADR-036
  - REQ-05.001
  - REQ-05.003
  - REQ-05.005
  - REQ-05.006
  - REQ-05.008
  - REQ-07.004
  - TECH-071
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-248
ordinal: 60
documentation: []
type: implementation
---

## Description
Context: the seed is the only definition of built-in document types after the nine typed modules are deleted.

Scope: implement the seed with presentation facts for req, nfr, audit, glossary, rule, request, ui-mockup, ui-spec, tech-spec, use-case, test-case, and test-report.
- req and nfr: GroupedMarkdown, prefix REQ and NFR, group and member fields story plus description, slots [group], reference_visibility hidden, seed orders.
- audit: audit_type with heading "## Audit Type", frontmatter_key type, option audit-type; content with heading "## Content", required; title positional.
- glossary: definition with heading "## Definition", required; title uniform, no --term.
- rule: description required; rationale required with verbs [create]; applicability required with verbs [create]; requirements with heading "## Requirements", kind list, option requirement.
- request: minimal entry; default description field; hidden references.
- ui-mockup: SingleHtmlMockup with fixed pending, approved, archived metadata and supersede.
- Derive verbs, options, and help from the fields and model; expose selector paths document_types.<slug>.<path> including numeric [0] indexing.
Do not start until MS-031 governance is accepted.
