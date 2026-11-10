---
id: TASK-78
title: Manage use cases, test cases, and test reports from the CLI
status: Implemented
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-10T20:49:00Z'
labels:
  - cli
  - documentation
milestone: m-15
dependencies:
  - TASK-20
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.010.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
priority: medium
type: feature
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver CLI management for three new managed document types — use cases (UC), test cases (TC), and test reports (TR) — following the per-type command pattern (see BR-017 for tech-spec). Each record carries a required title and description plus optional applicability; supports create, list, view, edit, deprecate, and restore with ID preserved; lifecycle active -> deprecated; front-matter required (REQ-05.010). Open question: REQ-05.012/013/014 state management via the `req` command, which conflicts with BR-009 (req is business-requirements only); a dedicated per-type command is recommended (BR-018/019/020). Resolve before implementation.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A use case, test case, and test report can each be created with required title and description and optional applicability.
- [ ] #2 Each type can be listed, viewed, and edited for title and description.
- [ ] #3 Each type can be deprecated and restored with its ID preserved; deprecated records are hidden from lists, search, and the web UI.
- [ ] #4 Records carry YAML front matter per REQ-05.010 and follow the active -> deprecated lifecycle (REQ-05.007).
- [ ] #5 The managing command conflict (`req` vs dedicated per-type command) is resolved and documented before implementation.
<!-- AC:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implementation is on branch task/TASK-78 (WIP commit 167223f), NOT merged to main. Included: src/use_case.rs, src/test_case.rs, src/test_report.rs, CLI wiring in src/lib.rs/src/config.rs, browser render update, README manpage, tests/browser_cli.rs update, spec docs/specifications/use-cases-test-cases-and-test-reports-management.md. Next session: checkout task/TASK-78, run quality-gatekeeper first (no re-refinement, no re-implementation), then merge.
<!-- SECTION:NOTES:END -->
