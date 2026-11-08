---
id: TASK-44
title: Search business rules
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T07:30:00Z'
labels:
  - search
  - documentation
milestone: m-8
dependencies:
  - TASK-43
references:
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/search-business-rules.md
priority: medium
type: task
ordinal: 26100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend unified search to include business rules.

Business rules are document records stored under the configured document_types.rule directory (default docs/business-rules/). They are searched whenever doc is selected, including the default all-types search. Only active records matching <prefix>-<digits>.md (for example RUL-001.md) are searched; the deprecated/ subfolder and non-record files such as canonical BR-*.md documents are never scanned. A disabled or absent rule type contributes no results and no error. Deprecated business rules are hidden; restored rules reappear.

Matching, ordering, output shapes, exit codes, boundary enforcement, and the read-only guarantee are reused unchanged from search-requirements.md. The type filter value remains doc; no dedicated --type rule value is added.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Search returns active business rules by default and with --type doc, and never returns them for task, milestone, release, or decision; deprecated business rules are excluded.
- [x] #2 Quoted phrases and --limit work across business rules and other supported record types, with deterministic record-ID ordering and the documented text/JSON output.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover active business rules in default and --type doc search, exclusion of other type filters, quoted phrases, --limit, deprecated exclusion, disabled type, missing directory, symlink escape, and read-only behavior.
- [ ] #3 Feature specification docs/specifications/search-business-rules.md reviewed and aligned.
- [ ] #4 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #5 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T07:30:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass (2/2 AC); cargo test 30 search_cli passed; auto-closed under no-validation mode per user request.

---
<!-- COMMENTS:END -->
