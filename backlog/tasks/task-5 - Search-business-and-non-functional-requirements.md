---
id: TASK-5
title: Search business and non-functional requirements
status: Refined
assignee:
  - '@orchestrator'
created_date: '2026-10-31T22:25:00Z'
updated_date: '2026-11-02T20:14:00Z'
labels: []
milestone: m-1
dependencies:
  - TASK-4
references:
  - docs/requirements/business/REQ-06.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/specifications/search-requirements.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.002.md
ordinal: 5000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the first increment of `papyrus search`: a read-only, project-local free-text search over business requirements, non-functional requirements, and requirement group documents, using the documented CLI shape `papyrus search <query> [--type <type>]... [--limit <n>]` and the documented type vocabulary task, milestone, release, decision, doc (default all). Types with no record store yet (task, milestone, release, decision) are accepted and return no matches without error. Search must honor configured document directories and prefixes, hide deprecated and disabled records, sanitize input, stay inside the project directory, and never write. Feature specification: docs/specifications/search-requirements.md.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `papyrus search <query> [--type <type>]... [--limit <n>]` is available and accepts exactly the documented types task, milestone, release, decision, and doc, defaulting to all types.
- [ ] #2 Search returns matching active business and non-functional requirement records, including requirement group documents, from the configured document directories; deprecated records and disabled document types never appear.
- [ ] #3 Matching is case-insensitive substring matching over the record ID, title, and full body (user story, description, and other body sections); a quoted phrase is matched as one contiguous query and regex metacharacters are literal.
- [ ] #4 `--type` filters restrict results; task, milestone, release, and decision are accepted and return no matches without error in this increment.
- [ ] #5 `--limit <n>` caps results after ordering; non-integer, zero, and negative values exit 2; results are ordered by record ID ascending.
- [ ] #6 Text output is one `<id> <type> <title>` line per result and `--json` output is an ordered array of `{id,type,title,path}`; zero matches exits 0 with empty output or `[]`.
- [ ] #7 Empty or whitespace-only queries and control characters exit 4; missing query, unknown type, and invalid limit exit 2; IO or boundary violations exit 1; search never writes and never exits 3.
- [ ] #8 Search reads only within the canonical project directory, never follows a symlink outside it, and adds no dependencies.
- [ ] #9 Automated tests cover matching and quoted phrases, type filters including unimplemented types, limit and ordering, deprecation and disabled-type exclusion, JSON output, and the exit-code matrix.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 `papyrus search` is implemented in the existing single crate with existing dependencies only; no new crate, application, stack, or dependency is added.
- [ ] #2 `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass with Rust 1.99.1.
- [ ] #3 Integration tests in `tests/` exercise the acceptance-criteria checks against the built binary.
- [ ] #4 The implementation matches `docs/specifications/search-requirements.md`; any deviation is recorded and the specification updated.
- [ ] #5 Search performs no writes and no reads outside the canonical project directory.
- [ ] #6 `--json` works for both results and errors, following the existing error shape.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Add the `search` subcommand (query, repeatable `--type`, `--limit`) to the existing CLI with the documented type vocabulary and validation.
2. Add a search module that loads the effective config, enumerates enabled `req` and `nfr` active directories, reads `.md` records with boundary checks, and skips `deprecated/`.
3. Implement case-insensitive whole-record matching, deterministic ID ordering, and limit application.
4. Emit `<id> <type> <title>` text and the `{id,type,title,path}` JSON array, reusing existing error and exit-code conventions.
5. Add integration tests in `tests/search_cli.rs` covering the acceptance-criteria matrix, then run build/test/clippy/fmt.
<!-- SECTION:PLAN:END -->
