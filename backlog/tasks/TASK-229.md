---
id: TASK-229
title: "Update BR-009 to BR-014 for the unified document engine"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T17:39:27Z'
labels: []
references:
  - BR-009
  - BR-010
  - BR-011
  - BR-012
  - BR-013
  - BR-014
  - REQ-05.001
  - REQ-05.003
  - REQ-05.005
  - REQ-05.006
  - REQ-07.003
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-226
  - TASK-225
ordinal: 130
documentation:
  - docs/business-rules/BR-009-req-command-behavior.md
  - docs/business-rules/BR-010-nfr-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
type: requirements
---

## Description
Context: six per-type command behavior rules describe hand-written command surfaces. After the refactor these commands are generated, so each rule must describe the generated behavior without losing any user-visible invariant.

Scope:
- BR-009 (req) and BR-010 (nfr): create-group creates a group record with a plain ID; create requires an existing active group of the same type; a cross-type group and an archived group are rejected; member IDs are dotted; membership is immutable after creation; archive or delete cascades to nothing; merged list ordering is preserved; references are hidden.
- BR-011 (glossary): title is the positional argument; --term is removed; definition is required with heading "## Definition"; existing term values survive as titles.
- BR-012 (ui-spec): SingleMarkdown baseline; fields come from the seed.
- BR-013 (rule): description required; rationale and applicability are create-only; requirements is a list field with option requirement.
- BR-014 (audit): title is positional; audit_type is a text field with option audit-type and frontmatter_key type; content is required.
