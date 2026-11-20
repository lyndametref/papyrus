---
id: TASK-230
title: "Update BR-017 to BR-020 for config-declared SingleMarkdown document types"
created_date: '2026-11-20T17:39:27Z'
updated_date: '2026-11-20T17:39:27Z'
labels: []
references:
  - BR-017
  - BR-018
  - BR-019
  - BR-020
  - REQ-05.009
  - REQ-05.012
  - REQ-05.013
  - REQ-05.014
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-226
ordinal: 140
documentation:
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
type: requirements
---

## Description
Context: tech-spec, use-case, test-case, and test-report lose their dedicated modules and become SingleMarkdown entries in the seed. Their existing fields and command behavior must survive as configuration.

Scope:
- BR-017 (tech-spec), BR-018 (use-case), BR-019 (test-case), BR-020 (test-report): each becomes a SingleMarkdown document type whose fields are declared in the seed; baseline verbs plus status; no type-specific verb; title is the positional argument and --title on edit; front-matter keys come from each field's frontmatter_key; the existing body headings and field order are preserved in the seed sequence.
- Record the exact field list for each type in the rule so a later seed change is a requirement change, not a silent behavior change.
- State that these types are user-extensible: adding a field is a file edit and needs no code.
