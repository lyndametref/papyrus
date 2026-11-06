# Feature Specification — Business and Non-Functional Requirements Management from the CLI

**Ticket:** TASK-3
**Milestone:** m-0 (Rust CLI Foundation and Requirements)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `req` and `nfreq` commands so a project contributor can create, list, view, edit, deprecate, and restore business requirements and non-functional requirements, including their required groups, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.001 and REQ-05.002 and the documented lifecycle in REQ-05.007.

## Scope

### In scope

- `papyrus req create <title> [--title <title>] [--story <text>] [-d <description>] --group <id>` (BR-009, REQ-05.001).
- `papyrus req create-group --title <title> [--story <text>] [-d <description>]` (BR-009, REQ-05.001).
- `papyrus req list`, `view <id>`, `edit <id> [--title <title>] [--story <text>] [-d <description>] [--editor]`, `deprecate <id>`, `restore <id>` (BR-009).
- `papyrus nfreq create <title> [-d <description>] --group <id>` (BR-010, REQ-05.002).
- `papyrus nfreq create-group --title <title> [-d <description>]` (BR-010, REQ-05.002).
- `papyrus nfreq list`, `view <id>`, `edit <id> [--title <title>] [-d <description>] [--editor]`, `deprecate <id>`, `restore <id>` (BR-010).
- Requirement-to-group cardinality `1..1`: a requirement is created only with a required group, belongs to exactly one group, and a group contains requirements of one type only (REQ-05.001, REQ-05.002, BR-009, BR-010).
- Stable requirement IDs that are preserved across deprecation and restoration (REQ-05.001, REQ-05.002, REQ-05.007).
- Deprecation moving a requirement to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.001, REQ-05.002, REQ-05.007, BR-009, BR-010).
- Restoration returning a deprecated requirement to its original folder with the same ID (REQ-05.001, REQ-05.002, REQ-05.007, BR-009, BR-010).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search and browser access for requirements — TASK-5 and TASK-6.
- Task, milestone, release, decision, and other document-type commands — TASK-8 onward.
- Requirement reference expansion in tasks/decisions — later tickets.
- Editing requirement files by external editors beyond the documented `--editor` body editing inherited from the man page.
- UI mockups or web UI — none for this ticket.
- New configuration keys beyond those already delivered by TASK-2; this ticket consumes the `document_types.req` and `document_types.nfr` keys.

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.req.prefix` (default `REQ`), `document_types.req.directory` (default `requirements/business/`), and the corresponding `nfr` entries, plus `id_padding` and `doc_directory`.

### 2. Groups

1. `papyrus req create-group --title <title> [--story <text>] [-d <description>]` creates a business-requirement group.
2. `papyrus nfreq create-group --title <title> [-d <description>]` creates a non-functional-requirement group.
3. A group is persisted as a text file in its type's configured directory, using a generated ID of the form `<prefix>-<number>` (for example `REQ-001`, `NFR-001`), padded to `id_padding` digits.
4. The next group number is one greater than the highest existing group number found in both the active and `deprecated/` folders for that type, so IDs are not reused.
5. A group has a title (required), and the optional user story and/or description provided by the command.
6. A group contains requirements of its own type only; mixing types is not permitted (REQ-05.001, REQ-05.002).
7. Group creation prints the generated group ID (and, in `--json` mode, returns it as JSON).

### 3. Requirements

1. `papyrus req create <title> [--title <title>] [--story <text>] [-d <description>] --group <id>` creates a business requirement and requires `--group <id>`.
2. `papyrus nfreq create <title> [-d <description>] --group <id>` creates a non-functional requirement and requires `--group <id>`.
3. The positional `<title>` and `--title <title>` are equivalent; if both are supplied they must agree, otherwise the command fails with exit code 4.
4. Creating a requirement without `--group`, or with a group ID that does not exist or belongs to the other type, fails with exit code 3 (not found) or 4 (validation) as appropriate.
5. A requirement ID is `<group-id>.<sequence>`, where `<sequence>` is the next unused sequence number within that group, padded to `id_padding` digits (for example `REQ-001.001`).
6. A requirement belongs to exactly one group; its group reference is recorded in the persisted file and cannot be changed by `edit`.
7. A business requirement persists title, optional user story, and optional description.
8. A non-functional requirement persists title and optional description.
9. `--group` accepts only an active or deprecated group ID of the same type; a deprecated group may still own requirements but new requirements are normally created against active groups. (Assumption: creation against a deprecated group is rejected with exit code 4.)

### 4. List and view

1. `papyrus req list` lists active business-requirement groups and business requirements (not deprecated ones).
2. `papyrus nfreq list` lists active non-functional-requirement groups and non-functional requirements (not deprecated ones).
3. List output includes at least the ID and title of each requirement in a stable order (by ID).
4. `papyrus req view <id>` and `papyrus nfreq view <id>` print the full persisted text of the requirement.
5. Viewing a deprecated requirement by ID is allowed and prints its persisted text; viewing an unknown ID fails with exit code 3.
6. `--json` wraps list and view output as stable JSON.

### 5. Edit

1. `papyrus req edit <id> [--title <title>] [--story <text>] [-d <description>] [--editor]` updates the supplied fields.
2. `papyrus nfreq edit <id> [--title <title>] [-d <description>] [--editor]` updates the supplied fields.
3. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values.
4. `--editor` opens the document body in `$VISUAL` or `$EDITOR` (per effective configuration) for editing.
5. Editing a deprecated requirement is allowed and keeps it deprecated.
6. Editing an unknown ID fails with exit code 3.

### 6. Deprecation and restoration

1. `papyrus req deprecate <id>` moves the business requirement file into `requirements/business/deprecated/`.
2. `papyrus nfreq deprecate <id>` moves the non-functional requirement file into `requirements/non-functional/deprecated/`.
3. Deprecated requirements are hidden from `list`, search, and the web UI (REQ-05.001, REQ-05.002, REQ-05.007).
4. `restore <id>` moves a deprecated requirement back to its original folder and preserves its ID (REQ-05.001, REQ-05.002, REQ-05.007).
5. Deprecating or restoring an already-deprecated/active requirement respectively is a validation error (exit code 4) OR a no-op success; this ticket specifies it as a validation error to avoid silent state changes. (Assumption.)
6. A group itself may be deprecated; deprecating a group does not automatically deprecate its requirements. (Assumption.)
7. Deprecating a requirement does not change its group membership or ID.

### 7. Persistence and sanitization

1. Every requirement and group is stored as a UTF-8 Markdown text file named `<ID>.md` in its type's configured directory (or `deprecated/` when deprecated).
2. File content follows the repository's existing requirement-file style, excluding the `Criticality` section and any fields not supported by the documented command options: a top-level heading `# <ID> <title>`, then `## User Story` (when provided), `## Group` (for requirements, recording the owning group ID and title), and `## Description` (when provided). Empty optional fields are omitted.
3. All user-controlled input (title, user story, description, group ID) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory, except none for this ticket (init remains the sole exception, delivered by TASK-2) (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing or invalid usage (including missing `--group`, conflicting title sources) exits 2 when it is a usage error, or 4 when it is a validation error, following clap's classification for missing required arguments.
3. Unknown record or group ID exits 3.
4. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `req create-group` / `nfreq create-group`: inputs title, optional story, optional description; output is the generated group ID (text) or JSON.
- `req create` / `nfreq create`: inputs title (positional or `--title`), optional story/description, required `--group <id>`; output is the generated requirement ID (text) or JSON.
- `req list` / `nfreq list`: output is IDs and titles of active requirements (text table or JSON array).
- `req view <id>` / `nfreq view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `req edit <id>` / `nfreq edit <id>`: inputs optional fields and `--editor`; output minimal success text or JSON.
- `req deprecate <id>` / `nfreq deprecate <id>`: moves file; minimal success output.
- `req restore <id>` / `nfreq restore <id>`: moves file back; minimal success output.

## Validation and error behavior

- Missing required arguments (for example `--group`) → exit code 2 (usage).
- Unknown ID or group → exit code 3 (not found).
- Validation/conflict (type mismatch, deprecated/active state conflict, unnormalizable input, conflicting title sources) → exit code 4.
- IO failure or boundary violation → exit code 1.
- `--json` error output includes the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: single-crate layout is retained; req/nfreq modules may be added within the crate without a new crate or ADR.
- Assumption: IDs are generated by scanning active and deprecated folders for the highest existing number; the configured `id_padding` (default 3) applies to both group numbers and per-group sequence numbers.
- Assumption: group IDs and requirement IDs use the configured `document_types.req` / `document_types.nfr` prefix and directory, so renamed/disabled types are honored; a disabled type rejects creation.
- Assumption: `--editor` resolves `$VISUAL` then `$EDITOR` via effective configuration, as delivered by TASK-2.
- Assumption: requirement files use the Markdown shape described in section 7.2; tests verify ID, title, and field presence rather than byte-for-byte formatting.
- Assumption: deprecating or restoring an already-deprecated/active requirement is a validation error (exit code 4).
- Assumption: deprecating a group does not cascade to its requirements.
- Assumption: creation against a deprecated group is rejected (exit code 4).
- Assumption: list output order is by ID ascending.

## Open questions

- OQ-1: Resolved by user direction: generated files follow the repository's existing requirement-file style, excluding `Criticality` and unsupported fields (see section 7.2).
- OQ-2: Resolved by user direction: `req list`/`nfreq list` display both groups and requirements.
- OQ-3: Resolved by user direction: group membership is immutable after creation; edit does not support `--group`.
- OQ-4: The ID-padding convention for the child sequence (`.001`) is not explicitly documented; this specification applies `id_padding` to that sequence.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.001, REQ-05.002, REQ-05.007 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-009 (req), BR-010 (nfreq) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002) |
| Ticket | TASK-3 |
| UI mockups | none |
| Related specification | docs/specifications/project-initialization-and-configuration.md (TASK-2) |
path: docs/specifications/business-and-non-functional-requirements-management.md