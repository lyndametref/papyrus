---
id: TASK-78
title: Manage use cases, test cases, and test reports from the CLI
status: Refined
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T06:59:00Z'
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
  - docs/specifications/use-cases-test-cases-and-test-reports-management.md
priority: medium
type: feature
ordinal: 35000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver dedicated CLI commands `use-case`, `test-case`, and `test-report` for managing use-case, test-case, and test-report records (REQ-05.012/013/014), following the tech-spec per-type pattern (BR-017/018/019/020). Each record has required title and description plus optional applicability; supports create, list, view, edit, deprecate, restore with ID preserved; lifecycle active->deprecated; front matter per REQ-05.010. Resolves the req vs dedicated command conflict in favor of dedicated commands (BR-009/018/019/020). Specification: docs/specifications/use-cases-test-cases-and-test-reports-management.md.

Assumptions:
- Prefixes/directories/enabled adopted from BR proposals: UC/use-cases, TC/test-cases, TR/test-reports, enabled true.
- description and applicability are body fields; front matter carries general fields only (id, title, created_date, updated_date, labels, references).
- No association to tasks/milestones; no dedicated outcome/coverage fields in test reports (captured in body/references).
- req command remains business-requirements only; REQ wording correction left to requirements owner.

Out of scope: search (TASK-80), browser (TASK-81), mockups (TASK-79), packaging (TASK-82).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Dedicated commands `use-case`, `test-case`, and `test-report` each support create, list, view, edit, deprecate, and restore subcommands.
- [ ] #2 Each type can be created with required title and description and optional applicability; missing required arguments exit 2.
- [ ] #3 Each type can be listed (active only, ID and title, ID ascending), viewed (full persisted text; deprecated viewable), and edited for title, description, and applicability.
- [ ] #4 Each type can be deprecated and restored with its ID preserved; deprecated records are hidden from lists, search, and the web UI; already-deprecated/already-active operations exit 4; unknown ID exits 3.
- [ ] #5 Records carry YAML front matter with id, title, created_date, updated_date, labels, references; description and applicability are body fields (REQ-05.010).
- [ ] #6 Record files use the template `# <ID> <title>`, then `## Description`, then optional `## Applicability`, ending with a single trailing newline.
- [ ] #7 IDs are generated as <prefix>-<number> padded to id_padding using UC/TC/TR and directories use-cases/test-cases/test-reports; IDs are never reused.
- [ ] #8 Create and edit expose --label and --reference for front-matter lists; created_date/updated_date are automatic; updated_date refreshes only on a content-changing write.
- [ ] #9 All user input is sanitized (NFR-02.002); a disabled type reports a validation error (exit 4); common CLI controls and exit codes 0/1/2/3/4 behave as TASK-2.
- [ ] #10 The req-vs-dedicated-command conflict is resolved and documented: dedicated commands are used and req remains business-requirements only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Unit and integration tests cover create, list, view, edit, deprecate, and restore for all three types.
- [ ] #2 Sanitization, ID generation, deprecation visibility, and front-matter preservation tests pass.
- [ ] #3 README manpage entries and configuration defaults updated for the three new types.
- [ ] #4 Acceptance criteria reviewed and signed off.
<!-- DOD:END -->
