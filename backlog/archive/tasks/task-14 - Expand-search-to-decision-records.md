---
id: TASK-14
title: Expand search to decision records
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-05T18:42:00Z'
labels: []
milestone: m-3
dependencies:
  - TASK-13
  - TASK-10
references:
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-008-decision-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/search-decision-records.md
ordinal: 15000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the unified search command so decision records are searched alongside task, milestone, release, and doc records. Decision records live in the configured decision_directory (default decisions/), outside the backlog store; only active records of enabled decision types are searched. Query matching, quoted-phrase handling, type filters, ordering, result limits, text/JSON output, boundary enforcement, sanitization, and the read-only guarantee are reused unchanged from search-requirements.md and search-tasks-milestones-and-releases.md. Implements REQ-06.001 under REQ-04/REQ-04.001/REQ-04.002 and advances BR-015, BR-008, BR-016.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus search includes active decision records by default, alongside task, milestone, release, and doc records.
- [ ] #2 --type decision returns only matching decision records; other documented type values behave as before.
- [ ] #3 Disabled decision types (BR-016) and archived decision records are excluded from search.
- [ ] #4 Query matching, quoted-phrase handling, case-insensitivity, type filters, ordering by record ID, and --limit work across decision records and all other supported record types.
- [ ] #5 Decision records are searched in full (status, decision, context, consequences, and any body text); text and --json output use the decision type value.
- [ ] #6 Missing or unreadable decision directories and boundary violations behave as specified; search performs no writes.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles with cargo build.
- [ ] #2 Unit and integration tests for decision-record search pass.
- [ ] #3 CLI behavior matches README manpage and BR-015/BR-008.
- [ ] #4 Feature specification updated at docs/specifications/search-decision-records.md.
- [ ] #5 No reads or writes occur outside the canonical project directory.
- [ ] #6 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
<!-- DOD:END -->
