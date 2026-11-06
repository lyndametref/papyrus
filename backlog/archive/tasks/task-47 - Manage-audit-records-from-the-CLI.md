---
id: TASK-47
title: Manage audit records from the CLI
status: Done
assignee: []
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T07:45:00Z'
labels:
  - cli
  - documentation
milestone: m-4
dependencies:
  - TASK-13
references:
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/audit-records-management.md
priority: medium
type: feature
ordinal: 17400
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Deliver the `audit` commands so a project contributor can create, list, view, edit, deprecate, and restore audit records (AUD-xxx), with stable IDs and text-file persistence inside the project directory.

Implements REQ-05.006, REQ-05.007 (lifecycle), NFR-02.002, NFR-04.001, NFR-04.002, BR-014, BR-016. Full behavior, scope, validation, and acceptance details: docs/specifications/audit-records-management.md.

Audit types: `security` (default), `licence`, `architecture`; additional types are usable as sanitized values. Deprecation moves a record to its type directory's `deprecated/` subfolder.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 `papyrus audit create --title <title> --content <text> [--audit-type <type>]` creates a record and prints its AUD-xxx ID; `--title` and `--content` are required (missing either exits 2).
- [ ] #2 When `--audit-type` is omitted the record type defaults to `security`; supplied types (`licence`, `architecture`, or any sanitized value) are stored.
- [ ] #3 `papyrus audit list` lists only active records (ID and title), ordered by ID ascending; deprecated records are excluded.
- [ ] #4 `papyrus audit view <id>` prints the full persisted record, including deprecated records; an unknown ID exits 3.
- [ ] #5 `papyrus audit edit <id>` updates only the supplied fields (`--title`, `--content`, `--audit-type`); `--editor` opens the record in `$VISUAL`/`$EDITOR`; editing a deprecated record keeps it deprecated.
- [ ] #6 `papyrus audit deprecate <id>` moves the record to `deprecated/`; deprecating an already-deprecated or unknown ID exits 4 or 3 respectively.
- [ ] #7 `papyrus audit restore <id>` moves a deprecated record back to the active folder with its ID and content unchanged; restoring an already-active or unknown ID exits 4 or 3 respectively.
- [ ] #8 Records are stored as UTF-8 Markdown `AUD-xxx.md` files under `<doc_directory>/<document_types.audit.directory>` (active) or its `deprecated/` subfolder, with heading `# <ID> <title>` and `## Audit Type` / `## Content` sections.
- [ ] #9 All user-controlled input is sanitized before use; unnormalizable values exit 4 and leave the filesystem unchanged.
- [ ] #10 All reads/writes remain within the canonicalized project directory; no binary data is written; a disabled `document_types.audit` makes every command exit 4.
- [ ] #11 Common CLI controls (`--json`, `--no-color`/`NO_COLOR`, help/version) and exit statuses 0/1/2/3/4 behave as delivered by TASK-2.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Implementation compiles and unit/integration tests pass.
<!-- DOD:END -->
