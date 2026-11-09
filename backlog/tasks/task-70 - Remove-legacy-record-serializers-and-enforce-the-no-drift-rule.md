---
id: TASK-70
title: Remove legacy record serializers and enforce the no-drift rule
status: Done
assignee: []
created_date: '2026-11-08T15:06:00Z'
updated_date: '2026-11-09T20:26:00Z'
labels:
  - front-matter
  - refactor
  - no-drift
milestone: m-12
dependencies:
  - TASK-67
  - TASK-69
references:
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-04.001.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-04.003.md
  - >-
    docs/architecture/adr/ADR-0008-adopt-backlog-md-on-disk-layout-for-tasks-and-milestones.md
documentation:
  - >-
    docs/specifications/remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md
priority: high
type: chore
ordinal: 30080
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver ADR-0009 Stage 5: remove all legacy per-module record parsers, renderers, and serializers and enforce the no-drift rule so parsing, identity, and writing each have one implementation under src/record/. Implements REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, NFR-04.003. Refactor only: no intended change to command behavior, output, or exit codes; existing tests keep expected results. Builds on TASK-63 through TASK-69. Backlog.md round-trip verification remains TASK-71.

Scope: remove module-local parsers/renderers/manual front-matter formatting from audit, decision, glossary, milestone, release, requirement, rule, task, tech_spec, ui_spec; replace browser parse_doc/apply_*_edit with shared patch operations; move all ID formatting/allocation to record::identity; move all record writes to record::store::write_atomic including UI mockup HTML; remove duplicate src/record/descriptors.rs; update search to the shared reader and search specifications to state metadata searchable; add no-drift checks to the test suite. Out of scope: new features, mockup metadata format changes, round-trip verification (TASK-71), browser UI (TASK-66/67/72), corpus migration, config/init writes.

Assumptions: src/record/descriptor.rs is canonical; UI mockup rendering remains the only non-front-matter serializer exception but its writes use the shared store; no corpus file is modified; no new dependency. Dependencies: TASK-67, TASK-69.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Legacy per-module parsers, renderers, and manual front-matter serializers are removed from all managed Markdown record modules; their read/write paths use record::document, record::frontmatter, and record::patch.
- [ ] #2 Browser records::parse_doc and apply_*_edit functions are removed or reduced to thin adapters over record::document, record::patch, and record::store; browser handlers build PatchOp lists.
- [ ] #3 No record path uses fs::write in production source outside the documented allowlist (config.rs, init.rs, tests); all record writes, including UI mockup HTML, use record::store::write_atomic.
- [ ] #4 No format!("---") or equivalent front-matter delimiter construction exists outside src/record/.
- [ ] #5 No ID parsing, formatting, padding, offset, or allocation logic exists outside src/record/identity.
- [ ] #6 Every record type reads and writes through the shared module, and existing unit and integration tests pass with unchanged behavior.
- [ ] #7 Search reads whole persisted record text, including front matter, and the per-type search specifications state that metadata values are searchable.
- [ ] #8 Temporary dual implementations are gone and the no-drift checks are part of the test suite and pass.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo test passes for the record module, browser, search, and affected integration tests.
- [ ] #2 No fs::write on a record path exists outside the documented allowlist.
- [ ] #3 No format!("---") or equivalent delimiter formatting exists outside src/record/.
- [ ] #4 No ID formatting exists outside src/record/identity.
- [ ] #5 Feature specification is updated and linked from the ticket.
- [ ] #6 No existing corpus record file is modified by this stage.
<!-- DOD:END -->
