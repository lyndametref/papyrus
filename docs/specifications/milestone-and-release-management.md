# Feature Specification — Milestone and Release Management from the CLI

**Ticket:** TASK-8
**Milestone:** m-2 (Backlog Discovery and Management)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `milestone` and `release` commands so a project maintainer can create, list, view, edit, archive, and delete milestones, and create, list, view, and edit releases, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-03, REQ-03.001, and REQ-03.002, including the documented milestone/release relationship and the release lifecycle clarified by TASK-22. It provides the delivery containers that later tasks (TASK-9 onward) assign work to.

## Scope

### In scope

- `papyrus milestone create <title> [-d <description>] [--release <id>]` (BR-006, REQ-03.001).
- `papyrus milestone list`, `view <id>`, `edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]` (BR-006).
- `papyrus milestone archive <id>` and `papyrus milestone delete <id> [--force]` (BR-006, REQ-03.001).
- `papyrus release create <title>`, `list`, `view <id>`, `edit <id> [--title <title>] [-d <description>]` (BR-007, REQ-03.002).
- Milestone identity: stable IDs of the form `<milestone_prefix>-<number>` (default `MS-001`) using configured `milestone_prefix` and `id_padding` (REQ-03.001, BR-006).
- Release identity: stable IDs of the form `<release_prefix>-<number>` (default `REL-001`) using configured `release_prefix` and `id_padding` (REQ-03.002, BR-007).
- Relationship: a milestone is assigned to zero or one release; a release may contain multiple milestones (REQ-03, REQ-03.001, REQ-03.002, BR-006, BR-007).
- Milestone lifecycle `active -> archived`: `archive` marks a milestone archived; `delete` permanently deletes it (REQ-03.001, BR-006).
- Release lifecycle `active -> released`: releases are not archived and have no documented archive, restore, delete, or status-transition command; this ticket does not infer one (REQ-03.002, BR-007, TASK-22, `docs/specifications/release-lifecycle.md`).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Task commands and task-to-milestone assignment — TASK-9 (BR-005).
- Search and browser access for milestones and releases — TASK-10 and TASK-11.
- Packaging and verification of the backlog-discovery milestone — TASK-12.
- Decision, glossary, UI-specification, business-rule, audit, and deployment record commands — TASK-13 onward.
- Any release status transition mechanism (for example an undocumented `--status`, `release` subcommand, or derivation from milestone/task completion) — not documented and not inferred; remains an open question in REQ-03.002 and `docs/specifications/release-lifecycle.md`.
- UI mockups or web UI — none for this ticket; CLI output only.
- New configuration keys beyond those already delivered by TASK-2 (`milestone_prefix`, `release_prefix`, `id_padding`, `backlog_directory`).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `milestone_prefix` (default `MS`), `release_prefix` (default `REL`), `id_padding` (default 3), and `backlog_directory` (default `backlog/`).
4. Milestone and release records are stored under the configured backlog directory: active milestones in `<backlog_directory>/milestones/`, archived milestones in `<backlog_directory>/milestones/archived/`, and releases in `<backlog_directory>/releases/`. These directories are created as needed; `init` already creates the active milestone and release directories.

### 2. Milestones — create

1. `papyrus milestone create <title> [-d <description>] [--release <id>]` creates a milestone.
2. `<title>` is required and is sanitized before use.
3. `-d, --description <text>` sets an optional description, sanitized before use.
4. `--release <id>` optionally assigns the milestone to a release. The release ID is sanitized and must identify an existing release; an unknown release ID fails with exit code 3.
5. The milestone ID is generated as `<milestone_prefix>-<number>`, padded to `id_padding` digits (default `MS-001`), where `<number>` is one greater than the highest existing milestone number found in both the active and `archived/` folders. IDs are never reused.
6. A created milestone is active and has no release assignment unless `--release` is supplied.
7. On success the generated milestone ID is printed (text mode) or returned as JSON.

### 3. Milestones — list

1. `papyrus milestone list` lists active milestones only; archived milestones are hidden.
2. Output includes at least the ID and title of each milestone in a stable order (by ID).
3. `--json` wraps the output as a stable JSON array.

### 4. Milestones — view

1. `papyrus milestone view <id>` prints the full persisted text of the milestone identified by `<id>`.
2. Viewing an archived milestone by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns the ID, path, and content.

### 5. Milestones — edit

1. `papyrus milestone edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]` updates only the supplied fields; omitted fields retain their previous values.
2. `--title <title>` updates the title.
3. `-d, --description <text>` updates the description.
4. `--release <id>` assigns the milestone to the specified release. The release must exist; an unknown release ID fails with exit code 3.
5. `--no-release [<id>]` removes a release assignment: with an ID, it removes that specific assignment (failing with exit code 4 if the milestone is not currently assigned to that release); without an ID, it removes any existing assignment.
6. `--release` and `--no-release` are mutually exclusive; supplying both fails with exit code 4.
7. Editing an archived milestone is allowed and keeps it archived.
8. Editing an unknown ID fails with exit code 3.

### 6. Milestones — archive and delete

1. `papyrus milestone archive <id>` moves the milestone file from the active folder to the `archived/` folder and marks it archived. Archiving an already-archived milestone fails with exit code 4.
2. Archiving an unknown ID fails with exit code 3.
3. `papyrus milestone delete <id> [--force]` permanently deletes the milestone file (from either the active or `archived/` folder).
4. Without `--force`, the command prompts for confirmation on standard input and aborts (exit 0, no change) unless the user confirms.
5. With `--force`, the confirmation prompt is skipped.
6. Deleting an unknown ID fails with exit code 3.
7. Deleting a milestone does not modify any release file; release-to-milestone relationships are derived by scanning milestone files.

### 7. Releases — create

1. `papyrus release create <title>` creates a release.
2. `<title>` is required and is sanitized before use.
3. The release ID is generated as `<release_prefix>-<number>`, padded to `id_padding` digits (default `REL-001`), where `<number>` is one greater than the highest existing release number found in the releases folder. IDs are never reused.
4. A created release is active.
5. On success the generated release ID is printed (text mode) or returned as JSON.

### 8. Releases — list

1. `papyrus release list` lists all releases.
2. Output includes at least the ID and title of each release in a stable order (by ID).
3. `--json` wraps the output as a stable JSON array.

### 9. Releases — view

1. `papyrus release view <id>` prints the full persisted text of the release.
2. Viewing an unknown ID fails with exit code 3.
3. `--json` returns the ID, path, and content.

### 10. Releases — edit

1. `papyrus release edit <id> [--title <title>] [-d <description>]` updates only the supplied fields; omitted fields retain their previous values.
2. `--title <title>` updates the title.
3. `-d, --description <text>` updates the description.
4. No release status, archive, restore, or delete field or command is provided by this ticket; `released` is not settable here (REQ-03.002, BR-007, TASK-22).
5. Editing an unknown ID fails with exit code 3.

### 11. Relationship and consistency

1. A milestone is assigned to at most one release at any time.
2. A release may be referenced by multiple milestones.
3. Assigning a milestone to a release requires that release to exist.
4. Removing or deleting a milestone does not delete the release it referenced.
5. Deleting a release is not supported by this ticket; if a referenced release is absent, the milestone's assignment is treated as dangling and reported as such on view (Assumption: dangling assignments are surfaced, not silently cleared).
6. Release status is never represented as archived; releases have no archive command.

### 12. Persistence and sanitization

1. Milestones and releases are stored as UTF-8 Markdown text files named `<ID>.md` in their respective folders (NFR-04.001).
2. File content follows a simple style: a top-level heading `# <ID> <title>`, then `## Description` (when provided), and for milestones `## Release` (when assigned) recording the owning release ID. Empty optional fields are omitted.
3. All user-controlled input (title, description, release ID, milestone ID) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 13. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing or invalid usage (including mutually exclusive flags, missing required title) exits 2 when it is a usage error, or 4 when it is a validation error, following clap's classification.
3. Unknown record or release ID exits 3.
4. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `milestone create`: inputs title, optional description, optional release ID; output is the generated milestone ID (text) or JSON.
- `milestone list`: output is IDs and titles of active milestones (text table or JSON array).
- `milestone view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `milestone edit <id>`: inputs optional fields and flags; output minimal success text or JSON.
- `milestone archive <id>`: moves file; minimal success output.
- `milestone delete <id> [--force]`: deletes file; minimal success output (or confirmation prompt without `--force`).
- `release create`: input title; output is the generated release ID (text) or JSON.
- `release list`: output is IDs and titles of releases (text table or JSON array).
- `release view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `release edit <id>`: inputs optional title/description; output minimal success text or JSON.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Unknown milestone or release ID → exit code 3.
- Validation/conflict (unnormalizable input, mutually exclusive flags, already-archived milestone, dangling/unknown release assignment where applicable, title conflicts) → exit code 4.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `init` already creates `backlog/milestones/` and `backlog/releases/`; this ticket creates `backlog/milestones/archived/` on first archive if absent.
- Assumption: archive moves the milestone file into `backlog/milestones/archived/` rather than recording status in place, consistent with the deprecation pattern used for requirements (TASK-3) and keeping archived milestones out of `list`.
- Assumption: release records have no status field and are implicitly active; no transition to `released` is implemented because no mechanism is documented (REQ-03.002, TASK-22). A future ticket may define the transition.
- Assumption: milestone `delete` prompts on standard input unless `--force` is given; in non-interactive contexts callers should pass `--force`.
- Assumption: a milestone assigned to a missing release is reported as dangling on view rather than silently cleared.
- Assumption: relationship data is stored only on the milestone (`## Release`), so releases do not need updating when milestones are created, edited, archived, or deleted.
- Assumption: no UI mockups are required; this is a CLI-only ticket.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained; new modules `milestone` and `release` (or a combined `backlog` module) may be added within the crate without a new ADR.

## Open questions

- OQ-1: How a release transitions from `active` to `released` is not documented (no status option on `papyrus release edit` and no dedicated command). Whether the state is set explicitly or derived from milestone/task completion remains unresolved; recorded in REQ-03.002 and `docs/specifications/release-lifecycle.md`. This ticket does not implement a transition.
- OQ-2: Whether `released` is a stored release attribute or derived from the completion of its milestones remains unresolved.
- OQ-3: The exact archive folder name and whether archived milestones remain editable are implementation choices documented as assumptions above; if the project later standardizes on a different folder, the spec must be updated.
- OQ-4: Whether deleting a release (not documented) should ever be supported is out of scope; if required, it needs a new requirement and command.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-03, REQ-03.001, REQ-03.002 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-006 (milestone command behavior), BR-007 (release command behavior) |
| Specification | `docs/specifications/release-lifecycle.md` (TASK-22) |
| Task | TASK-22 (release lifecycle research), TASK-7 (packaging baseline) |
| ADR | ADR-0001 (Rust), ADR-0003 (Forky-only) |
| Ticket | TASK-8 |
| UI mockups | none |
