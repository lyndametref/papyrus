# Feature Specification — Expand Unified Search to UI Mockups

**Ticket:** TASK-39
**Milestone:** m-7 (UI Mockups)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Extend `papyrus search` to include UI mockups. Stored under `<doc_directory>/<document_types.ui-mockup.directory>/` (default `docs/specifications/ui/mockups/`). Searched under `--type doc`; no new type value. Match, ordering, output, exit codes per search-requirements.md. REQ-06.001 under REQ-05/REQ-05.009; advances BR-015.

## Scope

Active UI mockup records only: files directly inside the configured active directory matching `<prefix>-<digits>.md` (e.g., `MOCK-0001.md`). The `deprecated/` subfolder and non-record files (HTML review packages, registry) are never scanned. Disabled or absent `ui-mockup` type contributes no results and no error. Deprecated hidden. Default and `--type doc` include them.

## Behavior

Same as search-ui-specifications.md with `ui-mockup` / `UI`-prefix / `mockups/` directory. Record ID is file stem; title is the first heading text.

## Acceptance criteria mapping

Default search includes UI mockups; `--type doc`; other `--type` values never; ordering; limit; missing dir empty; symlink escape exit 1; no writes; build/test/clippy/fmt pass.

## Open questions

OQ-1 no dedicated `--type mockup` value (recommended no); OQ-2 whole-record text (recommended); OQ-3 enabled types only; OQ-4 fixed configured directory.

## Traceability

REQ-06.001, REQ-05, REQ-05.009; BR-015; ADR-0001, ADR-0004; TASK-38 (management), TASK-39.

Matching runs over the whole persisted record text, including front matter, so metadata values are searchable.
