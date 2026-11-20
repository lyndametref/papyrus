---
id: TASK-240
title: "Build characterization fixtures and the behavior snapshot corpus"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-20T17:40:57Z'
labels: []
references:
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - TECH-069
status: ToDo
assignee:
  - test-manager
milestone: MS-032
dependencies:
  - TASK-239
ordinal: 30
documentation:
  - docs/test-cases
type: test-design
---

## Description
Context: every later milestone proves equivalence against this corpus. Missing coverage here becomes an undetectable regression.

Scope:
- Corpus: one type per DocumentModel; req and nfr groups with several members; ui-mockup in pending, approved, and archived metadata states; a user-defined SingleMarkdown type; records with resolved and unresolved references; archived records; a group with no members; a record at a zero-padding boundary.
- Per case capture: record file bytes, CLI stdout and exit code, JSON payload, HTTP status code, response headers, and rendered HTML.
- CLI help snapshot for every document command, including config subcommands.
- Byte capture of create and edit output per document type, including current title handling and glossary --term, so the intended title migration is measurable.
- Expected-delta list for deliberate changes: uniform title CLI, --term removal, help prose, asset headers, latency.
- Fixtures must be regenerable by one documented command and must not depend on wall-clock timestamps.
Do not start until MS-031 governance is accepted and TASK-239 is Done.
