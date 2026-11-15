---
id: REQ-05.007
title: "Manage Document Archive State and Deletion"
created_date: '2026-11-14T09:00:40Z'
updated_date: '2026-11-15T15:57:53Z'
labels: []
references: []
---
# REQ-05.007 Manage Document Archive State and Deletion

## User Story
As a project maintainer, I want to archive or delete document records so that inactive documents are out of the way while identity is preserved until I deliberately delete them.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
Every document type declared under `document_types` — the default entries and any type added by configuration — has the main status `active` or `archived`, and those two values are the only statuses that affect storage location. A front-matter-backed record starts `active` (or at its type's configured `default_status`, REQ-07.002) and its main status is the front-matter `status` field (REQ-05.010). A type may declare custom statuses (REQ-07.001): they are front-matter values only, cause no file move, and define no folder per status. UI mockups add the approval states `pending` and `approved` before `archived` and keep the state in their HTML metadata (REQ-05.008).

`papyrus <document-slug> archive <id>` sets the record to `archived` and moves its file into the `archived/` subfolder of that document type's configured directory. Archiving preserves the record ID (REQ-01.005). If the record carried a custom status, archiving replaces it with `archived` and the previous value is not retained anywhere (REQ-07.001). Archived records are hidden from lists, search, and the browser.

There is no restore command for documents in this release: no Papyrus command returns an archived record to `active`. This is a deliberate product decision, not a missing requirement. Only the `archived/` subfolder is recognized as archive storage. A `deprecated/` subfolder left by an earlier version is not read as archived, is not migrated, and is not inventoried by any command; it stays inert on disk and can be recovered only by manual file and metadata changes outside Papyrus. `papyrus check` in particular never reads, scans, or inventories a `deprecated/` folder: files there are not candidates for any finding and produce no report entry (REQ-01.008).

`papyrus <document-slug> delete <id>` permanently removes a record. An active record is archived first and then deleted. Deletion asks for confirmation and `--force` skips the prompt. A deleted record keeps no file, and its ID is not reused under the current allocator: allocation is max(observed number) + 1 over active and archived locations with no high-water mark (REQ-01.005), so deleted numbers generally remain gaps and deleting the highest record makes its number available again. No change to the allocator is made.

Auto-relocation on read: if a record's front-matter `status` is `archived` while its file still resides in the active folder (for example a hand-edited file or an interrupted write), Papyrus treats the file as misplaced and relocates it to the type's `archived/` subfolder on any read, list, view, search, or browser path, using the same atomic write-destination-then-unlink sequence as `archive`, before returning the record. The relocation is reported in diagnostics. If the relocation fails, the record is returned read-only with a validation outcome and writes to it are refused until the conflict is resolved.

The same rules apply to every document type, including technical specifications, business requirements, non-functional requirements, glossary terms, UI specifications, UI mockups, business rules, audit records, use cases, test cases, test reports, and config-defined types.

## Acceptance Criteria
- Every front-matter-backed document type configured under `document_types` has the main status `active` or `archived`, and may declare custom statuses that never affect storage location; UI mockups add `pending` and `approved` before `archived`.
- A new front-matter-backed document record starts `active`, or at its type's configured `default_status` when one is set (REQ-07.002).
- A custom status causes no file move and creates no per-status folder; a record carrying a custom status stays in the type folder.
- `papyrus <document-slug> archive <id>` sets `status` to `archived` and moves the file to the `archived/` subfolder of that type's configured directory.
- Archiving a record that carries a custom status sets `status` to `archived` and does not retain the previous custom status value anywhere.
- Archiving preserves the record ID.
- Archived records are hidden from lists, search, and the browser.
- On any read, list, view, search, or browser path, a record whose front-matter `status` is `archived` while its file is in the active folder is relocated to the type's `archived/` subfolder using the archive write-destination-then-unlink sequence before it is returned; the relocation is reported in diagnostics.
- If that relocation fails, the record is returned read-only with a validation outcome and writes to it are refused until the conflict is resolved.
- Orphaned `deprecated/` folders are inert: no command reads, migrates, or inventories them, and `papyrus check` reports no finding for files in them.
- No Papyrus command returns an archived document to `active`; no `restore` command exists for documents.
- Only an `archived/` subfolder is recognized as archive storage; an existing `deprecated/` subfolder is not read as archived and is not migrated.
- `papyrus <document-slug> delete <id>` permanently removes the record.
- Deleting an active record archives it first, then removes it.
- Deletion requires confirmation, and `--force` skips the confirmation prompt.
- A deleted record's ID is not reused.
- The rules apply to every configured document type, including config-defined types.

## Open Questions
- Whether a future release should add a restore command. Current requirement: none.
- Deleted-ID allocation, auto-relocation on read, and the inert `deprecated/` folder are settled as documented in the Description and Acceptance Criteria (no high-water mark; no `deprecated/` inventory, including by `papyrus check`). No further product decision is needed for this release.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "the document are active or archived."
- Project-owner decisions 2026-11-13 (refinement): `archived` replaces `deprecated` for documents; the state is stored in front matter and the file moves to an `archived/` subfolder; there is no restore command; existing `deprecated/` folders are not migrated; `delete` archives an active record first, confirms, and supports `--force`.
- Project-owner decision 2026-11-13: decision records are out of scope here; they use Nygard statuses and have no archive command (REQ-04.002).
- Project-owner decision 2026-11-13 (D5): custom document statuses are front-matter values only and add no storage state; archiving a record with a custom status sets `archived` and discards the previous value (REQ-07.001).
- Project-owner decision 2026-11-14: a record whose front-matter `status` is `archived` while its file remains in the active folder is auto-relocated to `archived/` on any read/list/view/search/browser path, using the archive write-destination-then-unlink sequence; the move is reported in diagnostics; on failure the record is returned read-only with a validation outcome and writes are refused. Deleted-ID allocation keeps the existing max-plus-one behavior (no high-water mark) and orphaned `deprecated/` folders stay inert: no command inventories them, including `papyrus check`.
- ADR-0022 — Document Lifecycle State and Archive Location (archive sequence, no restore, visibility); auto-relocation on read updates the prior "refuse writes and report" disagreement handling.
- Current state: documents are deprecated and restored via a `deprecated/` subfolder.
- REQ-01.005 — Assign Stable Record IDs
- REQ-01.008 — Run Project Diagnostics (`check` never reads or inventories a `deprecated/` folder)
- REQ-01.002 — Read and Edit Project Configuration (type directory, default status)
- REQ-04.002 — Update and Supersede Decisions
- REQ-05.008 — Manage UI Mockups (HTML metadata exception)
- REQ-05.010 — Record Metadata in YAML Front Matter (`status` field)
- REQ-05.011 — Maintain Record Metadata Through All Commands
- REQ-06.003 — Describe Managed Artifacts and Their Lifecycles in the Browser
- REQ-07.001 — Organize Artifacts into Three Categories
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.003 — Dispatch Config-Defined Document Commands
