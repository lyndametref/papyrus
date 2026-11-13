# REQ-07.001 Organize Artifacts into Three Categories

## User Story
As a project contributor, I want every artifact to belong to a known category so that I know which lifecycle applies and what configuration can change.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
Papyrus manages exactly three artifact categories:

- **backlog** — tasks, milestones, and releases.
- **decision** — decision records.
- **document** — every document type declared under `document_types`, including the default entries and any entry added by the user.

Each record type belongs to exactly one category.

Status models differ by category. Task statuses are configurable through `statuses` and `default_status` (REQ-01.002). Within the backlog category, the milestone and release lifecycles are fixed: a milestone is `open` then `closed`, and a release is `planned` then `released`. Decision records use the fixed Nygard statuses `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded` (REQ-04.001). Document types have a **main status** plus optional **custom statuses**, described below.

**Main status.** Every document type has the main status `active` or `archived`, and these are the only statuses that affect storage location. An `active` record is stored in the document type's configured folder; an `archived` record is stored in the `archived/` subfolder of that folder and carries `status: archived` (REQ-05.007). For a front-matter-backed record, `archive` is the only operation that produces the `archived` state; UI mockups set the equivalent HTML state through their own lifecycle (REQ-05.008).

**Custom statuses.** A document type may declare additional statuses in `document_types.<slug>.statuses` (REQ-07.002), optionally with a configured initial value in `document_types.<slug>.default_status`. Custom statuses are front-matter values only: they cause no file move, define no folder per status, introduce no new lifecycle, and change nothing about which records are listed, whether a record can be archived or deleted, or where a record is stored. A record carrying a custom status is an active record in the type folder and remains archivable. When no `default_status` is configured, a new document record starts `active`.

**Status list invariants.** Every document type's declared status list includes `active` and `archived`. Those two values are fixed: configuration cannot remove, rename, or redefine them, and a custom status may not reuse either value. Custom statuses are available only for front-matter-backed document types. UI mockups keep their state in HTML metadata with `pending`, `approved`, and `archived` (REQ-05.008) and declare no custom statuses.

**Setting a status from the CLI.** `papyrus <document-slug> status <id> <value>` sets a record's front-matter `status` to a declared custom status or to `active`. It changes only the status value and refreshes `updated_date`. This command never archives: the value `archived` is rejected with a validation outcome (exit code 4, REQ-01.004) that names `archive` as the only way to archive a record, and no file is moved. An archived record is likewise rejected with a validation outcome (exit code 4) that names the record as archived; `status` changes nothing on it and never moves it back. UI mockups have no `status` command; their lifecycle is governed by REQ-05.008.

**Archiving a record with a custom status.** Archiving such a record sets `status` to `archived` and moves the file into the `archived/` subfolder (REQ-05.007). The previous custom status is not retained: no front-matter field or metadata stores the value that the record had before archiving, and documents have no restore command.

Configuration cannot add, remove, or rename a category, and cannot change a fixed lifecycle (milestone, release, or decision) or the document main statuses. Declaring document custom statuses is configuration of front-matter values, not a change to those fixed lifecycles.

The decision status `deprecated` and the document state `archived` are different terms for different categories: a decision is never `archived`, and a document is never `deprecated`.

## Acceptance Criteria
- Papyrus manages exactly three artifact categories: backlog, decision, and document.
- Tasks, milestones, and releases belong to the backlog category; decision records belong to the decision category; every document type declared under `document_types` belongs to the document category.
- Each record type belongs to exactly one category.
- Task statuses are configurable through `statuses` and `default_status` (REQ-01.002); document types may additionally declare custom statuses through `document_types.<slug>.statuses` and an optional `document_types.<slug>.default_status`.
- The milestone lifecycle is fixed to `open -> closed` and the release lifecycle is fixed to `planned -> released`; neither is affected by any `statuses` or `default_status` configuration.
- Decision records use only `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`.
- Every document type has the main status `active` or `archived`; an active record is stored in the type folder and an archived record is stored in that folder's `archived/` subfolder with `status: archived` (REQ-05.007).
- A document type's declared statuses always include `active` and `archived`; configuration cannot remove, rename, or redefine them, and a custom status cannot reuse either value.
- A custom status is a front-matter value only: setting it moves no file and creates no per-status folder.
- A record carrying a custom status remains in the active storage location, is still listed, and can still be archived and deleted.
- A new document record starts `active` when no `default_status` is configured, and starts at the configured `default_status` when one is.
- `papyrus <document-slug> status <id> <value>` sets a declared custom status or `active` in the front matter, changes only the status, and refreshes `updated_date`.
- `papyrus <document-slug> status <id> archived` is rejected with a validation outcome (exit code 4) that names `archive` as the only way to archive, and no file is moved.
- `papyrus <document-slug> status <id> <value>` on an archived record is rejected with a validation outcome (exit code 4) that names the record as archived; no status change and no file move occur.
- Archiving a record that carries a custom status sets `status: archived`, moves the file to the `archived/` subfolder, and does not retain the previous custom status value anywhere.
- UI mockups declare no custom statuses and have no `status` command; their state remains `pending`, `approved`, or `archived` in HTML metadata (REQ-05.008).
- Configuration cannot add, remove, or rename a category and cannot change a fixed lifecycle or the document main statuses.
- A decision is never described as archived, and a document is never described as deprecated.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "papyrus must have 3 categories of artifact: backlog (task. milestone and relsease), decision and documents. the first category has configurable status for tasks but milestone and releases have a fixed lifecycle. the decision follow the Nygard adr template and have all the same lifecycle (proposed, accepted, refused, deprecated, superseded) the document are active or archived."
- Project-owner decisions 2026-11-13 (refinement): categories are backlog, decision, and document; decisions keep the Nygard status `rejected` (not `refused`); documents use `archived` instead of `deprecated`; milestone lifecycle is `open -> closed` and release lifecycle is `planned -> released`.
- Project-owner decision 2026-11-13 (document status model): every document type has the main status `active` or `archived`, which alone governs storage location; additional custom statuses are declared per document type in configuration and are front-matter values only; `papyrus <document-slug> status <id> <value>` sets a custom status or `active` but never archives; archiving discards the custom value; `status` on an archived record is rejected. This supersedes the earlier statement that document types have the fixed lifecycle `active -> archived` with no configurable statuses.
- REQ-01.002 — Read and Edit Project Configuration (task statuses, document custom statuses)
- REQ-01.004 — Use Common CLI Controls and Outcomes (validation and not-found outcomes)
- REQ-02 — Task and Board Management
- REQ-03.001 — Manage Milestones, REQ-03.002 — Manage Releases
- REQ-04.001 — Create and Inspect Decisions
- REQ-04.002 — Update and Supersede Decisions
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.008 — Manage UI Mockups (HTML metadata exception, no custom statuses)
- REQ-05.010 — Record Metadata in YAML Front Matter (`status` values)
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` exemption from the option rule)
- REQ-07.002 — Configure Document Types and Slugs (status-list validation)
- REQ-07.003 — Dispatch Config-Defined Document Commands (`status` command)
