# Feature Specification — Expand Unified Search to Business Rules

**Ticket:** TASK-44
**Milestone:** m-8 (Business Rules)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Extend `papyrus search` to include business rules. Business rules are document records: they are stored under the configured `document_types.rule` directory (default `docs/business-rules/`) and are searched whenever the documented `doc` type is selected (including the default all-types search). The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee established by `docs/specifications/search-requirements.md` are reused unchanged. This increment implements REQ-06.001 under REQ-05/REQ-05.005/REQ-05.007, and advances BR-015 and BR-013.

## Scope

Active business-rule records only: files directly inside the configured active directory matching `<prefix>-<digits>.md` (for example `RUL-001.md`). The `deprecated/` subfolder and non-record files (including canonical `BR-*.md` documents that share the same directory in this repository) are never scanned. Disabled or absent `rule` type contributes no results and no error. Deprecated business rules are hidden. Default and `--type doc` include active business rules.

## Behavior

Same as `search-ui-mockups.md`/`search-ui-specifications.md` with `rule` / `RUL` prefix / `business-rules/` directory: no new `--type` value is added; business rules are returned whenever `doc` is selected. Record ID is the file stem; title is the text after the ID on the first heading line, or the ID when that text is empty. Matching is case-insensitive substring matching over the whole persisted record text; quoted phrases are matched as one contiguous substring; results are ordered by record ID; `--limit` caps the combined result set; missing configured directory yields empty success; symlink escape exits 1; `search` performs no writes.

## Acceptance criteria mapping

- Default search includes active business rules; `--type doc` includes them; other `--type` values never do.
- Quoted phrases and `--limit` work across business rules and other supported record types.
- Matching is case-insensitive substring matching over the whole business-rule text.
- Deprecated business rules never appear; restored business rules appear again.
- Disabled `rule` document type excludes its business rules.
- Business rules are returned with type value `doc` and title equal to the rule title.
- Results are ordered by record ID ascending across stores.
- Missing business-rules directory yields empty success without creation.
- Symlink escape exits 1 and does not expose outside content.
- `search` creates no files or directories.
- Build, tests, clippy, and formatting pass with existing dependencies only.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: no dedicated type filter)** — The documented `--type` vocabulary contains no business-rule value. This specification searches business rules under `--type doc`, consistent with other document records.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text, consistent with prior search increments.
- **OQ-3 (recommended default: enabled types only)** — Disabled `rule` document types are excluded, consistent with `rule list` and BR-016.
- **OQ-4 (recommended default: fixed `document_types.rule` directory)** — The configured `document_types.rule` directory is searched, consistent with TASK-43.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05, REQ-05.005, REQ-05.007 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior), BR-013 (rule command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-44 |
| UI mockups | none |
| Domain model | `docs/domain-model/business-glossary/business-rule.md` |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/search-ui-mockups.md` (TASK-39) |
| Related specification | `docs/specifications/business-rules-management.md` (TASK-43) |
