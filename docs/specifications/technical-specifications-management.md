# Feature Specification — Technical Specifications Management from the CLI

**Ticket:** TASK-55
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only; browser UI for this record type is TASK-57/TASK-58)

## Purpose

Deliver the `tech-spec` commands so a project contributor can create, list, view, edit, deprecate, and restore technical specifications, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.009, the lifecycle part of REQ-05.007, NFR-02.002, NFR-04.001, and NFR-04.002, together with the documented command contract in BR-017 and the configuration contract in BR-016. It provides the records that later tickets search (TASK-56) and expose in the browser (TASK-57).

## Scope

### In scope

- `papyrus tech-spec create <title> --description <text>` (BR-017, REQ-05.009).
- `papyrus tech-spec list` (BR-017, REQ-05.009).
- `papyrus tech-spec view <id>` (BR-017, REQ-05.009).
- `papyrus tech-spec edit <id> [--title <title>] [--description <text>] [--editor]` (BR-017, REQ-05.009).
- `papyrus tech-spec deprecate <id>` and `papyrus tech-spec restore <id>` (BR-017, REQ-05.007).
- Technical-specification identity: stable IDs of the form `<prefix>-<number>` (for example `TECH-001`), using the configured `document_types.tech-spec` prefix and `id_padding` (REQ-05.009, BR-017, BR-016).
- Technical-specification fields: title and description (REQ-05.009, BR-017, domain glossary term "Technical Specification").
- Deprecation moving a specification to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.007, BR-017).
- Restoration returning a deprecated specification to its original folder with the same ID (REQ-05.007, BR-017).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search integration for technical specifications — TASK-56.
- Browser listing, viewing, and editing of technical specifications, including the browser design review — TASK-57 and TASK-58.
- Packaging and verification of the technical-specifications milestone — TASK-59.
- Deleting a technical specification — BR-017 defines no `delete` command.
- Dedicated CLI flags for referencing non-functional requirements or tasks from a technical specification — BR-017 defines none; such references, when present, appear as literal text in the title or description (consistent with REQ-05.009, which only states that NFRs may be referenced by technical specifications and that tasks may reference technical specifications).
- Changing the default directory for the `tech-spec` type (it stays `technical/` per BR-016).
- Any new configuration key beyond those already delivered by TASK-2 (`document_types.tech-spec`).
- UI mockups or web UI — none for this ticket; CLI output only.

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.tech-spec.prefix` (default `TECH`), `document_types.tech-spec.directory` (default `technical/`), `document_types.tech-spec.enabled`, plus `doc_directory` and `id_padding` (BR-016, BR-017).
4. The active directory is `<project>/<doc_directory>/<document_types.tech-spec.directory>`; the deprecated directory is `<active>/deprecated/`.
5. If `document_types.tech-spec.enabled` is false, `create` and `list` report a validation error (exit code 4) and no records are created or shown, matching the behavior delivered for the other document types (BR-016, BR-017).

### 2. Create

1. `papyrus tech-spec create <title> --description <text>` creates a technical specification.
2. `<title>` and `--description` are both required; a missing argument is a usage error (exit code 2).
3. `title` and `description` are sanitized before use (NFR-02.002).
4. The ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `TECH-001`), where `<number>` is one greater than the highest existing number found for that prefix in both the active and `deprecated/` folders. IDs are never reused (REQ-05.009, BR-017).
5. The active directory and its `deprecated/` subfolder are created if absent.
6. `create` prints the generated ID; in `--json` mode it returns `{"id": "<ID>"}`.
7. If any supplied value cannot be sanitized, creation fails with exit code 4 and no file is written.

### 3. List

1. `papyrus tech-spec list` lists active technical specifications only (not deprecated).
2. Each row shows at least the ID and the title, ordered by ID ascending.
3. Only regular `*.md` files directly inside the active directory whose file stem matches `<prefix>-<digits>` are records. Subdirectories and files with other names are ignored.
4. Deprecated specifications are hidden from `list` (REQ-05.007, BR-017).
5. `--json` returns a JSON array of objects with `id` and `title`.

### 4. View

1. `papyrus tech-spec view <id>` prints the full persisted text of the technical specification.
2. Viewing a deprecated specification by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns an object with at least `id`, `path`, and `content`.

### 5. Edit

1. `papyrus tech-spec edit <id> [--title <title>] [--description <text>] [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values (REQ-05.009).
3. `--title` replaces the title (and the top-level heading).
4. `--description` replaces the description text.
5. `--editor` opens the persisted file in `$VISUAL`, falling back to `$EDITOR` per effective configuration. Content saved by the editor is the new record content; the file name (ID) is never changed by an edit.
6. Editing a deprecated specification is allowed and keeps it deprecated.
7. Editing an unknown ID fails with exit code 3.
8. If any supplied value cannot be sanitized, edit fails with exit code 4 and no content is changed.
9. If `--editor` is requested and no editor is configured, the command fails with exit code 1 and no content is changed.
10. `--json` returns `{"status": "ok", "id": "<ID>"}`.

### 6. Deprecation and restoration

1. `papyrus tech-spec deprecate <id>` moves the specification file from the active folder to `deprecated/`.
2. Deprecating an already-deprecated specification fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecated specifications are hidden from lists, search, and the web UI (REQ-05.007, BR-017).
5. `papyrus tech-spec restore <id>` moves a deprecated specification back to the active folder.
6. Restoring an already-active specification fails with exit code 4.
7. Restoring an unknown ID fails with exit code 3.
8. Restoration preserves the specification's ID and its file content (REQ-05.007, BR-017).
9. `--json` returns `{"status": "ok", "id": "<ID>"}` for both commands.

### 7. Persistence and sanitization

1. Records are UTF-8 Markdown files named `<ID>.md` in the active directory or its `deprecated/` subfolder (NFR-04.001).
2. File content uses this template: a top-level heading `# <ID> <title>`, then `## Description` containing the description text. Both fields are required, so both are always present. The file ends with a single trailing newline.
3. `title` and `description` are stored as their section bodies; multi-line bodies are preserved when the file is edited through `--editor`.
4. All user-controlled input (`<title>`, `--description`, `<id>`) is normalized/escaped before use: values are trimmed, and empty values, control characters (NUL, CR, LF in single-line inputs), and path separators are rejected (NFR-02.002).
5. Values that cannot be normalized fail with exit code 4 and leave the filesystem unchanged (NFR-02.002).
6. `<id>` is resolved by exact, case-sensitive match of the file stem; request values are never concatenated into a path without resolution, and every read/write path is checked against the canonicalized project directory (NFR-04.002).
7. No binary data is written (NFR-04.001).

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (missing title or `--description`) and unknown options exit 2 (usage).
3. Unknown technical specification ID exits 3.
4. Filesystem/IO failure, missing editor, or project-directory boundary violation exits 1.

## Inputs and outputs

- `tech-spec create <title> --description <text>`: both required; output is the generated ID, or `{"id": "<ID>"}` in JSON mode.
- `tech-spec list`: output is ID and title rows of active specifications, or a JSON array of `{id, title}`.
- `tech-spec view <id>`: output is the full file text, or a JSON object with `id`, `path`, and `content`.
- `tech-spec edit <id> [--title] [--description] [--editor]`: output is minimal success text, or `{"status": "ok", "id": "<ID>"}`.
- `tech-spec deprecate <id>` / `tech-spec restore <id>`: move the file; minimal success output, or `{"status": "ok", "id": "<ID>"}`.

## Validation and error behavior

- Missing required argument or unknown option → exit code 2 (usage).
- Unknown ID → exit code 3 (not found).
- Disabled type, already-deprecated/already-active state conflict, unnormalizable input → exit code 4 (validation).
- IO failure, missing editor, or project-directory boundary violation → exit code 1 (runtime).
- A failed validation writes nothing; a failed `deprecate`/`restore` leaves the file in its current folder.
- `--json` error output carries the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001); the single-crate layout is retained (ADR-0004), so a `tech_spec` module may be added within the crate without a new ADR.
- Assumption: IDs are generated by scanning the active and `deprecated/` folders for the highest existing number with the configured prefix; `id_padding` (default 3) applies. This mirrors TASK-30.
- Assumption: record file style follows the TASK-30 template pattern (`# <ID> <title>` plus a `## Description` section). BR-017 does not fix section names, so `## Description` is used; tests assert field presence rather than byte-for-byte formatting.
- Assumption: `list`, `view`, `deprecate`, and `restore` do not recurse into subdirectories of the tech-spec directory.
- Assumption: the default `tech-spec` directory `technical/` is used only for technical specifications; relocation is a configuration decision, not part of this ticket.
- Assumption: `init` already creates the tech-spec directory and its `deprecated/` subfolder; this ticket ensures both exist on `create` and on `deprecate` if absent.
- Assumption: `--title` and `--description` are single-line sanitized inputs; multi-line bodies are available through `--editor`.
- Assumption: deprecating an already-deprecated specification or restoring an already-active one is a validation error (exit code 4), matching TASK-30.
- Assumption: `list` order is by ID ascending.
- Assumption: editing a deprecated specification is allowed, including field changes, matching TASK-30.

## Open questions

- OQ-1 (non-blocking): BR-017 gives no section names for the record body. `## Description` is adopted here for consistency with the other document types; a different layout would change the file template only, not the command contract.
- OQ-2 (non-blocking, for the browser ticket TASK-57/TASK-58): whether a technical specification may reference MOCK ids or other records is unspecified and explicitly out of scope for this ticket.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.009, REQ-05.007 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-017, BR-016 |
| Domain | `docs/domain-model/business-glossary/technical-specification.md` |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (single-crate layout) |
| Ticket | TASK-55 |
| UI mockups | none |
| Related specification | `docs/specifications/ui-specifications-management.md` (TASK-34 pattern), `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |
