---
id: TASK-48
title: Search audit records
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-07T09:15:00Z'
labels:
  - search
  - documentation
milestone: m-9
dependencies:
  - TASK-47
references:
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-015-search-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/search-audit-records.md
priority: medium
type: task
ordinal: 27100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend unified search to include audit records.

Audit records are document records stored under the configured document_types.audit directory (default docs/audit-records/). They are searched whenever doc is selected, including the default all-types search. Only active records matching <prefix>-<digits>.md (for example AUD-001.md) are searched; the deprecated/ subfolder and non-record files are never scanned. A disabled or absent audit type contributes no results and no error. Deprecated audit records are hidden; restored audit records reappear.

Matching, ordering, output shapes, exit codes, boundary enforcement, and the read-only guarantee are reused unchanged from search-requirements.md. The type filter value remains doc; no dedicated --type audit value is added.

Assumptions:
- Assumption: Rust 1.99.1 and the existing single-crate layout are retained (ADR-0001, ADR-0004); no new ADR or dependency is required.
- Assumption: deprecated audit records are hidden from search, consistent with other document types and BR-014/REQ-05.007.
- Assumption: the searchable text is the whole persisted record text, so editor-added body sections are searchable.
- Assumption: text output uses doc as the type value, not the audit type or prefix.
- Assumption: record IDs come from file stems, so configured prefixes are honored without hardcoding AUD.
- Assumption: audit records live in the configured document_types.audit directory (default audit-records/), gated by document_types like other document records.
- Assumption: unreadable individual record files are skipped; an unreadable configured directory or a boundary violation fails the command (exit 1).
- Assumption: search performs no writes at all, including not creating missing configured directories.
- Assumption: no UI mockups are needed because the increment is CLI-only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Search returns active audit records by default and with --type doc, and never returns them for task, milestone, release, or decision; deprecated audit records are excluded.
- [x] #2 Quoted phrases and --limit work across audit records and other supported record types, with deterministic record-ID ordering and the documented text/JSON output.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover active audit records in default and --type doc search, exclusion of other type filters, quoted phrases, --limit, deprecated exclusion, disabled type, missing directory, symlink escape, and read-only behavior.
- [ ] #3 Feature specification docs/specifications/search-audit-records.md reviewed and aligned.
- [ ] #4 Build, tests, clippy, and formatting pass with existing dependencies only.
- [ ] #5 No direct backlog file edits; all changes via backlog CLI.
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T09:15:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass (2/2 AC); cargo test search_cli 39 passed; auto-closed under no-validation mode.

---
<!-- COMMENTS:END -->
