---
id: TASK-10
title: Expand unified search to task, milestone, and release records
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-04T21:09:00Z'
labels: []
milestone: m-2
dependencies:
  - TASK-9
  - TASK-5
references:
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.003.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/specifications/search-tasks-milestones-and-releases.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-006-milestone-command-behavior.md
  - docs/business-rules/BR-007-release-command-behavior.md
ordinal: 11000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend `papyrus search` so task, milestone, and release records are searched in addition to business and non-functional requirements. Active task records from `<backlog_directory>/tasks/`, active milestone records from `<backlog_directory>/milestones/`, and all release records from `<backlog_directory>/releases/` become searchable; archived task/milestone records remain hidden. Type filters `task`, `milestone`, `release` return matching records with type labels `task`, `milestone`, `release`; default search includes them alongside `doc`. Matching, ordering, limit, output, boundary, sanitization, and read-only guarantees are unchanged from docs/specifications/search-requirements.md and are specified in docs/specifications/search-tasks-milestones-and-releases.md (TASK-10). CLI-only; no UI mockups. Decision records remain out of scope (TASK-14); other document types remain out of scope (TASK-18).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 papyrus search returns active task, milestone, and release records in addition to business and non-functional requirement records; archived task and milestone records never appear.
- [ ] #2 Type filters task, milestone, and release restrict results to their respective stores; combined filters work; unimplemented decision still returns empty success.
- [ ] #3 Matching is case-insensitive substring matching over the whole persisted record text; quoted phrases are matched as one contiguous substring; regex metacharacters are literal.
- [ ] #4 Results are ordered by record ID ascending; --limit <n> caps results after ordering; non-integer, zero, or negative limit values exit 2.
- [ ] #5 Text output is one '<id> <type> <title>' line per result with type values task, milestone, or release; --json outputs the ordered array of {id,type,title,path}; zero matches exits 0 with empty output or [].
- [ ] #6 Missing store directories yield no results and no error; search creates no files or directories.
- [ ] #7 Symlink escape exits 1 and never exposes outside content; unreadable non-UTF-8 files are skipped; an unreadable directory exits 1.
- [ ] #8 Empty, whitespace-only, or control-character queries exit 4; missing query, unknown type, or invalid limit exits 2; IO or boundary violations exit 1; search never writes and never exits 3.
- [ ] #9 Automated tests cover matching and quoted phrases across all stores, type filters, archive hiding, ordering, limit, JSON output, and the exit-code matrix.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles with cargo build.
- [ ] #2 Unit and integration tests for search pass.
- [ ] #3 CLI behavior matches README manpage and BR-015.
- [ ] #4 Feature specification updated at docs/specifications/search-tasks-milestones-and-releases.md.
- [ ] #5 No reads or writes occur outside the canonical project directory.
- [ ] #6 cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass with Rust 1.99.1.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Extend search::run to collect records for selected SearchType::Task, Milestone, and Release values.
2. Add collection helpers for active task, milestone, and release store directories under the configured backlog_directory, scanning only direct .md files and honoring boundary checks.
3. Map each hit type to task, milestone, or release; parse title from the first heading with ID fallback.
4. Reuse existing sanitization, ordering by ID, limit truncation, text/JSON rendering, and exit-code behavior.
5. Add unit and integration tests in tests/search_cli.rs covering the new stores, type filters, archive hiding, quoted phrases, ordering, limit, JSON, and exit codes.
6. Run cargo build, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Quality review (gatekeeper): Ready to close. All AC #1-#9 pass; DoD pass. cargo build/test/clippy/fmt all pass. 8 search_backlog_cli tests pass. Scope: src/search.rs + tests. NFR-02.002/NFR-04.001/NFR-04.002 addressed.
<!-- SECTION:NOTES:END -->
