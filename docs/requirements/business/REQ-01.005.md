---
id: REQ-01.005
title: "Assign Stable Record IDs"
created_date: '2026-11-15T13:39:01Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-01.005 Assign Stable Record IDs

## User Story
As a project user, I want Papyrus to assign and preserve record identifiers so that records can be referred to consistently.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus generates record IDs automatically. IDs remain stable across archive/restore cycles for tasks only, and across archiving for documents. Milestones are never archived: `papyrus milestone close <id>` moves a milestone from `open` to `closed` in place (REQ-03.001). Releases are likewise never archived; `papyrus release close <id>` moves a release from `planned` to `released` (REQ-03.002). Configurable prefixes and padding determine the generated ID format: task, milestone, and release IDs, and records of a document type that uses the generic content model, whose IDs use that type's `prefix` with the shared `id_padding` (REQ-07.004). Document types with type-specific content keep their documented ID forms, such as the hierarchical `REQ-XX.XXX` of business requirements (REQ-05.001). The man page gives `TASK-001`, `MS-001`, and `REL-001` as default examples.

Papyrus keeps a single ID space. The Papyrus ID is the record identity everywhere: in the front-matter `id`, in the CLI and browser output, and as command input. Papyrus does not maintain a second, tool-specific storage ID and does not translate between ID spaces, because file-level interoperability with Backlog.md is renounced (NFR-04.004); the on-disk file-name convention is an architecture decision. Records brought into Papyrus by the one-way Backlog.md import script (NFR-04.004) receive Papyrus IDs on import, and no Backlog.md ID is accepted as command input.

The `id_padding` configuration is constrained to 1-6 characters (REQ-01.007). Values outside this range error out during configuration validation. Display padding is a minimum: when a record's number digit count exceeds `id_padding`, the ID uses the wider number (REQ-01.007, ADR-0008).

## Acceptance Criteria
- Creating a supported record automatically assigns it an ID.
- Archiving and restoring a task does not change its ID, and archiving a document does not change its ID; documents have no restore command (REQ-05.007), and milestones are never archived because `papyrus milestone close <id>` changes `open` to `closed` in place (REQ-03.001).
- With default configuration, task, milestone, and release IDs use the documented prefixes and three-digit examples (`TASK-001`, `MS-001`, `REL-001`); the configured prefix and padding control their format.
- A record of a document type that uses the generic content model is identified by its type's configured `prefix` and the configured `id_padding`, following the same padding rules (REQ-07.004).
- Papyrus uses a single ID space: the same Papyrus ID is written to the front-matter `id` and displayed by CLI and browser output, and it is the only accepted command input form for that record.
- No Backlog.md storage ID (for example `task-18` or `m-0`) is treated as a Papyrus record identity or accepted as command input.
- Records imported from a Backlog.md store receive Papyrus IDs; the import does not create or preserve a parallel Papyrus-visible ID space.
- `id_padding` must be between 1 and 6 inclusive; values outside this range are rejected with clear error messages (REQ-01.007).
- When a record's number digit count exceeds `id_padding`, the displayed ID uses the wider number; padding is a minimum, never truncated or errored (REQ-01.007).

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION (line 33), command record sections and FILES (lines 109–844)
- README.md, `papyrus-config(5)` → General and Documents keys (lines 949–962, 1007–1026)
- NFR-04.004 — Renounce Backlog.md Compatibility and Provide Import (single Papyrus ID space, one-way import)
- Maintainer decision 2026-11-11: file-level Backlog.md compatibility is renounced, so Papyrus IDs are the only record identity on disk and in commands. This supersedes the 2026-11-08 dual-ID-space and CLI-ID-input decisions and ADR-0008's storage-ID translation; ADR-0008 is still referenced only for the display-width resolution (padding is a minimum).
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (display-width resolution only)
- REQ-05.010 — Record Metadata in YAML Front Matter (`id` in front matter)
- REQ-05.001 — Manage Business Requirements (hierarchical type-specific ID form)
- REQ-05.007 — Manage Document Archive State and Deletion (archive, no restore)
- REQ-03.001 — Manage Milestones (fixed `open -> closed` lifecycle, `close` in place; milestones are never archived, so archive/restore ID stability does not apply)
- REQ-03.002 — Manage Releases (releases are never archived)
- REQ-07.004 — Provide a Generic Content Model for Config-Defined Document Types (`prefix` + `id_padding`)
- REQ-01.007 — ID Padding Constraint for File IDs
