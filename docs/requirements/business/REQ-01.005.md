# REQ-01.005 Assign Stable Record IDs

## User Story
As a project user, I want Papyrus to assign and preserve record identifiers so that records can be referred to consistently.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
Papyrus generates record IDs automatically. IDs remain stable across archive/restore and deprecate/restore cycles. Configurable prefixes and padding determine the format for task, milestone, and release IDs; the man page gives `TASK-001`, `MS-001`, and `REL-001` as default examples.

Papyrus keeps this documented ID format and treats it as the ID space it displays and accepts, even where Backlog.md stores the same record under its own ID format. The two ID spaces are: Papyrus IDs (`TASK-001`, `MS-001`, configured prefix with `id_padding`) and Backlog.md IDs (for example `task-18`, `m-0`, and front-matter `id: TASK-18`, unpadded and with Backlog.md's own casing). Papyrus maps between them at the Backlog.md storage boundary: the record file and its front matter carry the Backlog.md ID, while CLI and browser output show the Papyrus ID. Command input uses only the Papyrus display format: a Backlog.md storage ID is internal to the `backlog/` store, is translated at that boundary, and is rejected as command input with a hint naming the equivalent Papyrus ID. The mapping is bidirectional and bijective over the record sequence, so a record keeps one identity in both tools; it normalizes prefix casing and zero-padding and reconciles the different numbering start of each tool (Backlog.md numbers milestones from `m-0`, Papyrus milestone IDs start at `MS-001`). Reference values in `milestone:` and `dependencies:` are translated with the same rule, so a milestone or dependency reference resolves in both Papyrus and Backlog.md after translation.

The `id_padding` configuration is constrained to 1-6 characters (REQ-09). Values outside this range error out during configuration validation.

## Acceptance Criteria
- Creating a supported record automatically assigns it an ID.
- Archiving and restoring a record, or deprecating and restoring a document, does not change its ID.
- With default configuration, task, milestone, and release IDs use the documented prefixes and three-digit examples (`TASK-001`, `MS-001`, `REL-001`); the configured prefix and padding control their format.
- Papyrus commands accept only the Papyrus display ID as record input; a Backlog.md storage ID (for example `task-18` or `m-0`) is rejected as command input with a hint that names the equivalent Papyrus ID.
- Papyrus displays and accepts Papyrus IDs, and CLI/browser output never presents the Backlog.md storage ID as the record identity.
- A task or milestone file written by Papyrus carries the Backlog.md ID in its filename and front-matter `id` (for example `task-18`, `m-0`) while the same record is shown by Papyrus as `TASK-001` or `MS-001`.
- The bidirectional mapping is stable: translating a Papyrus ID to the Backlog.md ID and back yields the original Papyrus ID for every existing record.
- `milestone:` and `dependencies:` reference values are translated in both directions and resolve in both Papyrus and Backlog.md.
- `id_padding` must be between 1 and 6 inclusive; values outside this range are rejected with clear error messages (REQ-09).

## Traceability
- README.md, `papyrus(1)` → DESCRIPTION (line 33), command record sections and FILES (lines 109–844)
- README.md, `papyrus-config(5)` → General and Documents keys (lines 949–962, 1007–1026)
- User decision 2026-11-08 (ID spaces): Papyrus keeps its documented ID format and translates to/from Backlog.md IDs at the storage boundary; both ID spaces, the bidirectional mapping, and reference translation are stated here. This supersedes ADR-0008's proposal to adopt Backlog.md task/milestone IDs directly.
- User decision 2026-11-08 (CLI ID input): commands accept only the Papyrus display format (`TASK-018`, `MS-013`); Backlog.md storage IDs (`task-18`, `m-0`) are internal to the `backlog/` store, translated at that boundary, and rejected as command input with a hint.
- ADR-0008 — Adopt the Backlog.md On-Disk Layout for Task and Milestone Records (Backlog.md filename, stem, and ID conventions)
- REQ-05.010 — Record Metadata in YAML Front Matter (`id` in front matter)
- REQ-09 — ID Padding Constraint for File IDs
