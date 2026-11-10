---
id: TASK-18
title: Complete unified search across all documented record types
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-10T18:13:00Z'
labels: []
milestone: m-12
dependencies:
  - TASK-14
  - TASK-31
  - TASK-35
  - TASK-39
  - TASK-44
  - TASK-48
  - TASK-56
  - TASK-68
references:
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.004.md
  - docs/requirements/business/REQ-05.005.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-02.003.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-04.001.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - docs/requirements/business/REQ-06.md
  - docs/requirements/business/REQ-05.008.md
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.011.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-04.003.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/business-rules/BR-008-decision-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/business-rules/BR-012-ui-spec-command-behavior.md
  - docs/business-rules/BR-013-rule-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - docs/specifications/search-complete-unified.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
ordinal: 30100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Final integration increment of papyrus search: one query searches every documented record type Papyrus manages (business and non-functional requirements and their groups, tasks, milestones, releases, decisions, glossary terms, UI specifications, UI mockups, business rules, audit records, and technical specifications). The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, sanitization, and read-only guarantee from docs/specifications/search-requirements.md are reused unchanged; no new --type value or option is added. Deprecated document records and archived task/milestone records are hidden; disabled document types are excluded; front-matter migration (ADR-0007/ADR-0009) must not narrow coverage. See docs/specifications/search-complete-unified.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A single default papyrus search <query> returns matching records from every documented record type: business requirements and their group documents, non-functional requirements and their group documents, tasks, milestones, releases, decisions, glossary terms, UI specifications, UI mockups, business rules, audit records, and technical specifications.
- [ ] #2 --type doc returns all document-type records (all of the above except task, milestone, release, decision) in one result set; --type task, --type milestone, --type release, and --type decision return only their own store.
- [ ] #3 Quoted phrases, case-insensitive substring matching (including over front-matter text), and --limit operate across the combined result set.
- [ ] #4 Deprecated document records never appear and restored records reappear; archived task and milestone records never appear; release records (no archived state) are all searched.
- [ ] #5 Disabled document types are excluded; absent store directories yield empty success without creation.
- [ ] #6 Results are ordered by record ID ascending across stores, ties broken by type then path; text output is '<id> <type> <title>' per line; JSON output is the ordered {id,type,title,path} array.
- [ ] #7 Boundary, sanitization, and read-only guarantees hold across every store: symlink escape exits 1, invalid query exits 4, missing query/invalid type/invalid limit exit 2, and search performs no writes.
- [ ] #8 Non-record files (registries, descriptions, notes, and files not matching the documented record filename pattern) are ignored in every store.
- [ ] #9 Lazy front-matter migration does not narrow coverage: migrated records remain searchable and their metadata values remain searchable.
- [ ] #10 Build, tests, clippy, and formatting pass with existing dependencies only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Cross-type integration test creates one record of every supported type and searches them in a single query (default and --type doc), covering ordering, --limit, quoted phrases, and deprecation/archival hiding.
- [ ] #3 Feature specification docs/specifications/search-complete-unified.md reviewed and aligned.
- [ ] #4 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #5 No direct backlog file edits; all changes via backlog CLI.
- [ ] #6 Search performs no writes and no reads outside the canonical project directory.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Review docs/specifications/search-complete-unified.md and the per-type search specs; confirm the combined store enumeration, filename eligibility, and visibility rules.
2. Ensure src/search.rs aggregates all documented stores (req, nfr, task, milestone, release, decision, glossary, ui-spec, ui-mockup, rule, audit, tech-spec) with the established contract, including front-matter text in searchable content.
3. Verify title extraction uses the heading for Markdown records and the front-matter title fallback for task/milestone/release records that lack a heading.
4. Add a cross-type integration test in tests/search_cli.rs (or a dedicated file) creating one record of every supported type with a shared phrase and asserting default/--type doc results, ordering, --limit, quoted phrases, and deprecation/archival hiding.
5. Run cargo build, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check; record evidence in the ticket.
<!-- SECTION:PLAN:END -->
