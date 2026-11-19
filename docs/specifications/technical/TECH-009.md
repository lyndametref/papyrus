---
id: TECH-009
title: "Milestone and Release Management from the CLI"
created_date: '2026-11-16T17:46:43Z'
updated_date: '2026-11-17T20:40:44Z'
labels: []
references:
  - ADR-0001
  - ADR-0003
  - ADR-0011
  - ADR-0012
  - ADR-0013
  - ADR-0022
  - ADR-0023
  - ADR-032
  - BR-005
  - BR-006
  - BR-007
  - NFR-02.002
  - NFR-04.001
  - NFR-04.002
  - REQ-01.003
  - REQ-01.004
  - REQ-03
  - REQ-03.001
  - REQ-03.002
  - REQ-05.010
  - TASK-2
  - TASK-7
  - TASK-8
  - TASK-9
  - TASK-10
  - TASK-11
  - TASK-12
  - TASK-13
  - TASK-22
  - TASK-89
  - TASK-118
  - TASK-130
  - TASK-136
  - TASK-159
status: draft
---
# TECH-009 Milestone and Release Management from the CLI

## Description
## Purpose

Deliver the `milestone` and `release` commands so a project maintainer can create, list, view, edit, close, archive, and delete milestones, and create, list, view, edit, and close releases, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-03, REQ-03.001, and REQ-03.002, including the documented milestone/release relationship, the milestone lifecycle open -> closed -> archived (ADR-032), and the release lifecycle clarified by TASK-22. It provides the delivery containers that later tasks (TASK-9 onward) assign work to.

## Scope

### In scope

- `papyrus milestone create <title> [-d <description>] [--release <id>]` (BR-006, REQ-03.001).
- `papyrus milestone list`, `view <id>`, `edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]` (BR-006).
- `papyrus milestone close <id>`, `papyrus milestone archive <id>`, and `papyrus milestone delete <id> [--force]` (BR-006, REQ-03.001, ADR-032).
- `papyrus release create <title>`, `list`, `view <id>`, `edit <id> [--title <title>] [-d <description>]`, and `close <id>` (BR-007, REQ-03.002).
- Milestone identity: stable IDs of the form `<milestone_prefix>-<number>` (default `MS-001`) using configured `milestone_prefix` and `id_padding` (REQ-03.001, BR-006).
- Release identity: stable IDs of the form `<release_prefix>-<number>` (default `REL-001`) using configured `release_prefix` and `id_padding` (REQ-03.002, BR-007).
- Relationship: a milestone is assigned to zero or one release; a release may contain multiple milestones (REQ-03, REQ-03.001, REQ-03.002, BR-006, BR-007).
- Milestone lifecycle `open -> closed -> archived`: `close` is the only `open -> closed` transition; `archive` closes an open milestone if needed and then moves it to `backlog/archive/milestones/`, marking it `archived` (ADR-032, BR-006, REQ-03.001).
- Release lifecycle `planned -> released`, reached only through `papyrus release close <id>`; releases are not archived and have no archive, restore, delete, or `--status` command (REQ-03.002, BR-007, ADR-0023 decision 2, `docs/specifications/release-lifecycle.md`).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Task commands and task-to-milestone assignment — TASK-9 (BR-005).
- Search and browser access for milestones and releases — TASK-10 and TASK-11.
- Packaging and verification of the backlog-discovery milestone — TASK-12.
- Decision, glossary, UI-specification, business-rule, audit, and deployment record commands — TASK-13 onward.
- Reopening a milestone (`closed -> open` or `archived -> closed`) or a release (`released -> planned`), bulk close, `--status` on `milestone edit` or `release edit`, milestone restore, and any release archive, restore, or release-delete behavior (BR-006, BR-007, REQ-03.001, REQ-03.002, ADR-032).
- UI mockups or web UI — none for this ticket; CLI output only.
- New configuration keys beyond those already delivered by TASK-2 (`milestone_prefix`, `release_prefix`, `id_padding`, `backlog_directory`).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `milestone_prefix` (default `MS`), `release_prefix` (default `REL`), `id_padding` (default 3), and `backlog_directory` (default `backlog/`).
4. Active milestone records (`open` or `closed`) stay in `<backlog_directory>/milestones/`; archived milestones are moved to `<backlog_directory>/archive/milestones/`. Releases stay in `<backlog_directory>/releases/`. Milestone and release state is the front-matter `status`. `<backlog_directory>/archive/milestones/` is both the milestone lifecycle archive destination and the legacy read/import location: it remains readable by view, search, and browser resolution and participates in ID allocation. `<backlog_directory>/milestones/archived/` remains a legacy read alias. No milestone restore command exists; the one-way Backlog.md import remains a writer of archived milestone records there (ADR-0011, ADR-0022, ADR-0023, ADR-032).

### 2. Milestones — create

1. `papyrus milestone create <title> [-d <description>] [--release <id>]` creates a milestone.
2. `<title>` is required and is sanitized before use.
3. `-d, --description <text>` sets an optional description, sanitized before use.
4. `--release <id>` optionally assigns the milestone to a release. The release ID is sanitized and must identify an existing release; an unknown release ID fails with exit code 3.
5. The milestone ID is generated as `<milestone_prefix>-<number>`, padded to `id_padding` digits (default `MS-001`), where `<number>` is one greater than the highest existing milestone number found in every read location — the active folder, `<backlog_directory>/archive/milestones/`, and `<backlog_directory>/milestones/archived/` (ADR-0023 decision 4, ADR-032). IDs are never reused.
6. A created milestone starts `open` and has no release assignment unless `--release` is supplied.
7. On success the generated milestone ID is printed (text mode) or returned as JSON.

### 3. Milestones — list

1. `papyrus milestone list` lists the milestones in the active folder, including those whose `status` is `open` or `closed`. Archived milestones are hidden. Files in the legacy read locations are not listed.
2. Output includes at least the ID and title of each milestone in a stable order (by ID).
3. `--json` wraps the output as a stable JSON array.

### 4. Milestones — view

1. `papyrus milestone view <id>` prints the full persisted text of the milestone identified by `<id>`.
2. A milestone ID resolves in the active folder and in the legacy read locations (`<backlog_directory>/archive/milestones/` and `<backlog_directory>/milestones/archived/`); a file found there is printed as legacy data and is never moved.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns the ID, path, and content.

### 5. Milestones — edit

1. `papyrus milestone edit <id> [--title <title>] [-d <description>] [--release <id>] [--no-release [<id>]]` updates only the supplied fields; omitted fields retain their previous values.
2. `--title <title>` updates the title.
3. `-d, --description <text>` updates the description.
4. `--release <id>` assigns the milestone to the specified release. The release must exist; an unknown release ID fails with exit code 3.
5. `--no-release [<id>]` removes a release assignment: with an ID, it removes that specific assignment (failing with exit code 4 if the milestone is not currently assigned to that release); without an ID, it removes any existing assignment.
6. `--release` and `--no-release` are mutually exclusive; supplying both fails with exit code 4.
7. Editing a milestone whose file sits in a legacy read location or archive folder writes the change in place and does not move it; an archived milestone stays archived (no unarchive).
8. Editing an unknown ID fails with exit code 3.

### 6. Milestones — close, archive, and delete

#### Close

1. `papyrus milestone close <id>` is the only `open -> closed` transition. It sets the front-matter `status` to `closed` and refreshes `updated_date`, preserving title, description, labels, references, release assignment, and body, and writes the milestone in place: it performs no move and creates no folder (REQ-03.001, ADR-0022 decision 5, ADR-0023 decision 2).
2. A missing or empty front-matter `status` is read as `open`, so imported and legacy milestone records remain closable.
3. Closing a milestone whose stored status is anything other than `open` fails with exit code 4, names the current status, and writes nothing; the file stays byte-identical.
4. Closing an unknown ID fails with exit code 3.
5. The write goes through the shared atomic record writer (ADR-0013); no direct file write is added in the milestone path.
6. Text mode prints the milestone ID; `--json` emits one object with `id` and `status: closed`.
7. There is no `--status` transition option, and editing a closed milestone does not reopen it.

#### Archive

1. `papyrus milestone archive <id>` is the only transition to `archived`. If the stored status is `open`, it first closes the milestone (status `closed`) and then sets status to `archived`; if the stored status is `closed`, it sets status to `archived` directly. The final status is `archived`, `updated_date` is refreshed once, and title, description, labels, references, and release assignment are preserved.
2. The milestone file is moved to `<backlog_directory>/archive/milestones/` using the shared atomic record writer (ADR-0013, ADR-032).
3. If the stored status is already `archived`, the command is a no-op success: it exits 0 and writes or moves nothing.
4. Archive is refused for any stored status other than `open`, `closed`, or `archived`: it fails with exit code 4, names the current status, and writes nothing.
5. An unknown ID fails with exit code 3.
6. Filesystem/IO failure or project-directory boundary violation fails with exit code 1.
7. Text mode prints the milestone ID; `--json` emits one object with `id` and `status: archived`.
8. Archiving does not modify any release file; release-to-milestone relationships are derived by scanning milestone files.
9. An archived milestone remains readable by ID and editable in place as legacy data; it is not restored by any command.

#### Delete

1. `papyrus milestone delete <id> [--force]` permanently deletes the milestone file (from the active folder or, for legacy data, from a legacy read location).
2. Without `--force`, the command prompts for confirmation on standard input and aborts (exit 0, no change) unless the user confirms.
3. With `--force`, the confirmation prompt is skipped.
4. Deleting an unknown ID fails with exit code 3.
5. Deleting a milestone does not modify any release file; release-to-milestone relationships are derived by scanning milestone files.

### 7. Releases — create

1. `papyrus release create <title>` creates a release.
2. `<title>` is required and is sanitized before use.
3. The release ID is generated as `<release_prefix>-<number>`, padded to `id_padding` digits (default `REL-001`), where `<number>` is one greater than the highest existing release number found in the releases folder. IDs are never reused.
4. A created release starts `planned`; `create` stores `status: planned` in the front matter.
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
4. `--status` is not provided; the only lifecycle transition is `papyrus release close <id>`. No release archive, restore, or delete command exists (REQ-03.002, BR-007).
5. Editing an unknown ID fails with exit code 3.

### 11. Releases — close

1. `papyrus release close <id>` moves a release from `planned` to `released`.
2. A missing or empty front-matter `status` is read as `planned`, so imported and legacy release records remain closable.
3. Closing a release whose stored status is anything other than `planned` fails with exit code 4, names the current status, and writes nothing; the file stays byte-identical.
4. Closing an unknown ID fails with exit code 3.
5. On success the command sets `status: released` and refreshes `updated_date`, preserving title, description, labels, references, and body.
6. The write goes through the shared atomic record writer (ADR-0013); no direct file write is added in the release path.
7. Text mode prints the release ID; `--json` emits one object with `id` and `status: released`.
8. Reopening, bulk close, `--status`, archive, restore, and delete are not provided.

### 12. Relationship and consistency

1. A milestone is assigned to at most one release at any time.
2. A release may be referenced by multiple milestones.
3. Assigning a milestone to a release requires that release to exist.
4. Removing or deleting a milestone does not delete the release it referenced.
5. Deleting a release is not supported by this ticket; if a referenced release is absent, the milestone's assignment is treated as dangling and reported as such on view (Assumption: dangling assignments are surfaced, not silently cleared).
6. Release status is never represented as archived; releases have no archive command.
7. Archiving a milestone does not remove or alter its release assignment.

### 13. Persistence and sanitization

1. Milestones and releases are stored as UTF-8 Markdown text files named `<ID>.md` in their respective folders (NFR-04.001).
2. File content begins with a YAML front-matter block (REQ-05.010, ADR-0012): the general keys `id`, `title`, `created_date`, `updated_date`, `labels`, `references` in canonical order, followed by `release` and `status` (`open`, `closed`, or `archived`) for a milestone, and `status` (`planned` or `released`) for a release (ADR-0023 decision 5, ADR-032). Milestones have no `# <ID> <title>` heading; their body contains only `## Description` (when provided). Metadata is never mirrored into sections. A write to a legacy milestone moves its value to front matter and retires the title heading and the `## Release` section (ADR-0012 decisions 4 and 5); read adoption adds the block only and leaves the legacy body byte-identical (ADR-0012 decision 9). A release keeps its `# <ID> <title>` heading and `## Description` body; `create` seeds `status: planned`.
3. Open and closed milestones are stored in `<backlog_directory>/milestones/`; archived milestones are stored in `<backlog_directory>/archive/milestones/`. All user-controlled input (title, description, release ID, milestone ID) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 14. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing or invalid usage (including mutually exclusive flags, missing required title) exits 2 when it is a usage error, or 4 when it is a validation error, following clap's classification.
3. Unknown record or release ID exits 3.
4. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `milestone create`: inputs title, optional description, optional release ID; output is the generated milestone ID (text) or JSON.
- `milestone list`: output is IDs and titles of the milestones in the active folder, including `open` and `closed` ones (text table or JSON array).
- `milestone view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `milestone edit <id>`: inputs optional fields and flags; output minimal success text or JSON.
- `milestone close <id>`: sets `status: closed` and refreshes `updated_date` in place; output is the milestone ID (text) or one JSON object with `id` and `status: closed`.
- `milestone archive <id>`: closes an open milestone if needed, sets `status: archived`, refreshes `updated_date`, and moves the file to `backlog/archive/milestones/`; an already archived milestone is a no-op success. Output is the milestone ID (text) or one JSON object with `id` and `status: archived`.
- `milestone delete <id> [--force]`: deletes file; minimal success output (or confirmation prompt without `--force`).
- `release create`: input title; output is the generated release ID (text) or JSON.
- `release list`: output is IDs and titles of releases (text table or JSON array).
- `release view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `release edit <id>`: inputs optional title/description; output minimal success text or JSON.
- `release close <id>`: output is the release ID (text) or one JSON object with `id` and `status: released`.

## Validation and error behavior

- Missing/invalid command usage → exit code 2.
- Filesystem/IO failure or project-directory boundary violation → exit code 1.
- Unknown milestone or release ID → exit code 3.
- Validation/conflict (unnormalizable input, mutually exclusive flags, closing or archiving a milestone that is not in the allowed state, archive refused for statuses other than `open`/`closed`/`archived`, dangling/unknown release assignment where applicable, title conflicts) → exit code 4.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: `init` already creates `backlog/milestones/` and `backlog/releases/`; `backlog/archive/milestones/` is created by archive or by the one-way Backlog.md import (ADR-0011, ADR-0023, ADR-032).
- Assumption: an open or closed milestone stays in `backlog/milestones/` and remains listed; `close` rewrites `status` and `updated_date` in front matter in place. `archive` moves the file to `backlog/archive/milestones/` and marks it `archived`; archived milestones are hidden from active listings but remain viewable by ID and editable in place as legacy data. No milestone restore command exists (ADR-032).
- Assumption: release status is stored in front matter as `planned` or `released`; `create` seeds `planned`, and only `release close` performs the fixed `planned -> released` transition (REQ-03.002, BR-007). A release is never archived.
- Assumption: milestone `delete` prompts on standard input unless `--force` is given; in non-interactive contexts callers should pass `--force`.
- Assumption: a milestone assigned to a missing release is reported as dangling on view rather than silently cleared.
- Assumption: relationship data is stored only in the milestone front-matter `release` key, so releases do not need updating when milestones are created, edited, closed, archived, or deleted.
- Assumption: no UI mockups are required; this is a CLI-only ticket.
- Assumption: single-crate layout (`src/main.rs` + `src/lib.rs`) is retained; new modules `milestone` and `release` (or a combined `backlog` module) may be added within the crate without a new ADR.

## Open questions

- OQ-1 (resolved by TASK-118): the transition is `papyrus release close <id>`; it is implemented and covered by CLI tests.
- OQ-2 (resolved by TASK-118): `released` is a stored front-matter `status` value, not derived from milestone completion.
- OQ-3 (resolved by ADR-032): `<backlog_directory>/archive/milestones/` is both the milestone lifecycle archive destination and the legacy read/import location. Imported and legacy archived milestone files stay readable and editable in place, and no milestone restore command exists (REQ-03.001, BR-006).
- OQ-4: Whether deleting a release (not documented) should ever be supported is out of scope; if required, it needs a new requirement and command.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-03, REQ-03.001, REQ-03.002 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-006 (milestone command behavior), BR-007 (release command behavior) |
| Specification | `docs/specifications/release-lifecycle.md` (TASK-22) |
| Task | TASK-2, TASK-7, TASK-8, TASK-9, TASK-10, TASK-11, TASK-12, TASK-13, TASK-22, TASK-89, TASK-118, TASK-130, TASK-136, TASK-159 |
| ADR | ADR-0001 (Rust), ADR-0003 (Forky-only), ADR-0011 (backlog layout), ADR-0012 (YAML front matter), ADR-0013 (shared record envelope), ADR-0022 (milestone close), ADR-0023 (layout and read/import legacy location), ADR-032 (milestone archive state) |
| Ticket | TASK-8; TASK-118 (`release close`); TASK-130 (milestone `close`); TASK-136 (specification correction); TASK-159 (milestone archive) |
| UI mockups | none |
