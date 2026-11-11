---
id: TASK-80
title: Search use cases, test cases, and test reports
status: Done
assignee: []
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T08:48:00Z'
labels:
  - search
  - documentation
milestone: m-15
dependencies:
  - TASK-78
references:
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
  - docs/specifications/search-use-cases-test-cases-and-test-reports.md
priority: medium
type: task
ordinal: 35100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend papyrus search to include use cases, test cases, and test reports (document types use-case, test-case, test-report). Records are stored under their configured document_types directories (use-cases/, test-cases/, test-reports/). Only active records whose file stem matches <prefix>-<digits> (UC-xxx, TC-xxx, TR-xxx) are searched; deprecated/ subfolders and non-record files are never scanned. A disabled or absent type contributes no results and no error. Deprecated records are hidden; restored records reappear. Matching, ordering, output shapes, exit codes, boundary enforcement, sanitization, and the read-only guarantee are reused from docs/specifications/search-requirements.md; the type filter value remains doc (no dedicated per-type filter is added). Search operates on the dedicated-type records managed by the use-case/test-case/test-report commands (BR-018/BR-019/BR-020); the req wording in REQ-05.012/013/014 is superseded for implementation. See docs/specifications/search-use-cases-test-cases-and-test-reports.md.

In scope:
- Searching active use-case, test-case, and test-report records through the existing papyrus search <query> [--type <type>]... [--limit <n>] shape.
- --type doc returns all three types together with the other document records; default search includes them.
- Case-insensitive substring matching, quoted phrases, deterministic ordering by record ID, text and --json output, --limit.
- Deprecation hiding and restoration reappearance for all three types.
- Exclusion of disabled or absent types.
- Boundary enforcement, sanitization, read-only guarantee.
- Unit and integration tests.

Out of scope:
- A new --type value for any of the three types.
- Searching deprecated records.
- Browser/web UI access (TASK-81).
- Packaging/verification (TASK-82).
- Configuration changes, indexes, caches.
- Fuzzy/boolean/regex matching.
- Modifying, moving, creating, deprecating, restoring, or deleting records.
- UI mockups (CLI-only).

Assumptions:
- Assumption: Rust single-crate layout retained; no new crate/stack/dependency/ADR.
- Assumption: Deprecated records of all three types are hidden from search; a future increment may add an option to include them.
- Assumption: Searchable text is the whole persisted record text, including front matter and editor-added sections.
- Assumption: Records are included only when their document type is present and enabled (BR-016).
- Assumption: Record IDs come from file stems, so configured prefixes are honored without hardcoding UC/TC/TR.
- Assumption: Unreadable individual files are skipped; an unreadable directory or boundary violation fails with exit 1.
- Assumption: search performs no writes, including not creating missing directories.
- Assumption: No UI mockups are needed (CLI-only).

Dependencies: TASK-78 (completed).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Unified search returns active use-case, test-case, and test-report records (default and --type doc).
- [x] #2 --type doc includes all three types together with other document records; --type task/milestone/release/decision never returns them.
- [x] #3 Deprecated records of all three types never appear; restored records appear again.
- [x] #4 Disabled or absent use-case, test-case, or test-report type contributes no results and no error.
- [x] #5 Matching is case-insensitive substring matching over the whole persisted record text, including front matter.
- [x] #6 A quoted phrase is matched as one contiguous substring across all three types.
- [x] #7 Results are ordered by record ID ascending across stores; ties broken by type then path.
- [x] #8 Each hit reports type value doc and title equal to the record title.
- [x] #9 --limit caps the combined result set after ordering.
- [x] #10 Only files directly in the configured active directory matching <prefix>-<digits>.md are candidates; deprecated/ and non-record files are never scanned.
- [x] #11 Missing type directory yields empty success without creation.
- [x] #12 Symlink escape exits 1 and does not expose outside content.
- [x] #13 search performs no writes.
- [x] #14 Build, tests, clippy, and formatting pass with existing dependencies only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Cross-type integration test creates one record of each of the three types and searches them in a single query (default and --type doc), covering ordering, --limit, quoted phrases, and deprecation hiding.
- [ ] #3 Feature specification docs/specifications/search-use-cases-test-cases-and-test-reports.md reviewed and aligned.
- [ ] #4 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #5 No direct backlog file edits; all changes via backlog CLI.
- [ ] #6 Search performs no writes and no reads outside the canonical project directory.
<!-- DOD:END -->
