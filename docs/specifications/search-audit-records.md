# Feature Specification — Expand Unified Search to Audit Records

**Ticket:** TASK-48
**Milestone:** m-9 (Audit Records)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only)

## Purpose

Extend `papyrus search` to include audit records. Audit records are document records: they are stored under the configured `document_types.audit` directory (default `docs/audit-records/`) and are searched whenever the documented `doc` type is selected (including the default all-types search). The command shape, option vocabulary, matching rules, ordering, output shapes, exit codes, boundary enforcement, and read-only guarantee established by `docs/specifications/search-requirements.md` are reused unchanged. This increment implements REQ-06.001 under REQ-05/REQ-05.006/REQ-05.007, and advances BR-015 and BR-014.

## Scope

Active audit-record records only: files directly inside the configured active directory matching `<prefix>-<digits>.md` (for example `AUD-001.md`). The `deprecated/` subfolder and non-record files are never scanned. Disabled or absent `audit` type contributes no results and no error. Deprecated audit records are hidden; restored audit records reappear.

## Behavior

Same as `search-business-rules.md` with `audit` / `AUD` prefix / `audit-records/` directory: no new `--type` value is added; audit records are returned whenever `doc` is selected. Record ID is the file stem; title is the text after the ID on the first heading line, or the ID when that text is empty. Matching is case-insensitive substring matching over the whole persisted record text, including front matter, so metadata values are searchable; quoted phrases are matched as one contiguous substring; results are ordered by record ID; `--limit` caps the combined result set; missing configured directory yields empty success; symlink escape exits 1; `search` performs no writes.

## Acceptance criteria mapping

- Default search includes active audit records; `--type doc` includes them; other `--type` values never do.
- Quoted phrases and `--limit` work across audit records and other supported record types.
- Matching is case-insensitive substring matching over the whole audit-record text.
- Deprecated audit records never appear; restored audit records appear again.
- Disabled `audit` document type excludes its audit records.
- Audit records are returned with type value `doc` and title equal to the record title.
- Results are ordered by record ID ascending across stores.
- Missing audit-records directory yields empty success without creation.
- Symlink escape exits 1 and does not expose outside content.
- `search` creates no files or directories.
- Build, tests, clippy, and formatting pass with existing dependencies only.

## Open questions

None of these block implementation; each has a specified default in this document. User confirmation is recommended before release.

- **OQ-1 (recommended default: no dedicated type filter)** — The documented `--type` vocabulary contains no audit-record value. This specification searches audit records under `--type doc`, consistent with other document records.
- **OQ-2 (recommended default: whole record text)** — The man page says "free-text search" without defining fields. This specification searches the whole persisted record text, consistent with prior search increments.
- **OQ-3 (recommended default: enabled types only)** — Disabled `audit` document types are excluded, consistent with `audit list` and BR-016.
- **OQ-4 (recommended default: fixed `document_types.audit` directory)** — The configured `document_types.audit` directory is searched, consistent with TASK-47.
- **OQ-5 (recommended default: hide deprecated audit records)** — BR-014 does not state whether deprecated audit records are hidden from search. This specification hides them, consistent with the other document types and the audit-management specification's recommendation. A future increment may add an option to include deprecated records.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.001, REQ-05, REQ-05.006, REQ-05.007 |
| Non-functional | NFR-02.002 (sanitize input), NFR-04.001 (text-only persistence), NFR-04.002 (project-directory boundary) |
| Business rule | BR-015 (search command behavior), BR-014 (audit command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0004 (repository-root single crate) |
| Ticket | TASK-48 |
| UI mockups | none |
| Domain model | `docs/domain-model/business-glossary/audit-record.md` |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/search-business-rules.md` (TASK-44) |
| Related specification | `docs/specifications/audit-records-management.md` (TASK-47) |
path: docs/specifications/search-audit-records.md