# Feature Specification — Audit Records Management from the CLI

**Ticket:** TASK-47
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only; browser UI for this record type is TASK-26/TASK-49)

## Purpose

Deliver the `audit` commands so a project contributor can create, list, view, edit, deprecate, and restore audit records, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.006, the lifecycle part of REQ-05.007, NFR-02.002, NFR-04.001, and NFR-04.002, together with the documented command contract in BR-014 and the configuration contract in BR-016. It provides the records that later tickets search (TASK-48) and expose in the browser (TASK-26).

## Scope

### In scope

- `papyrus audit create --title <title> --content <text> [--audit-type <type>]` (BR-014, REQ-05.006).
- `papyrus audit list` (BR-014, REQ-05.006).
- `papyrus audit view <id>` (BR-014, REQ-05.006).
- `papyrus audit edit <id> [--title <title>] [--content <text>] [--audit-type <type>] [--editor]` (BR-014, REQ-05.006).
- `papyrus audit deprecate <id>` (BR-014, REQ-05.006, REQ-05.007).
- `papyrus audit restore <id>` (BR-014, REQ-05.006, REQ-05.007).
- Audit-record identity: stable IDs of the form `<prefix>-<number>` (for example `AUD-001`), using the configured `document_types.audit` prefix and `id_padding` (REQ-05.006, BR-014, BR-016).
- Audit-record fields: title, content, and audit type (REQ-05.006, BR-014, domain glossary term "Audit Record").
- Audit type defaults to `security`; `licence` and `architecture` are the other built-in types, and additional types may be used (REQ-05.006, BR-014).
- Deprecation moving a record to its type's `deprecated/` subfolder (REQ-05.006, REQ-05.007, BR-014).
- Restoration returning a deprecated record to its original folder with the same ID and content (REQ-05.006, REQ-05.007, BR-014).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search integration for audit records — TASK-48.
- Browser listing, viewing, and editing of audit records, including the browser design review — TASK-26 and TASK-49.
- Packaging and verification of the audit-records milestone — TASK-50.
- Deleting an audit record — BR-014 defines no `delete` command.
- Changing the default directory for the `audit` type (it stays `audit-records/` per BR-016).
- Any new configuration key beyond those already delivered by TASK-2 (`document_types.audit`); in particular, no dedicated `audit_types` configuration key is introduced by this ticket.
- Validation that an audit type exists in a configured allow-list — BR-014 documents no such configuration key (see Open Questions).
- UI mockups or web UI — none for this ticket; CLI output only.

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.audit.prefix` (default `AUD`), `document_types.audit.directory` (default `audit-records/`), `document_types.audit.enabled`, plus `doc_directory` and `id_padding` (BR-016).
4. The active directory is `<project>/<doc_directory>/<document_types.audit.directory>`; the deprecated directory is `<active>/deprecated/`.
5. If `document_types.audit.enabled` is false, all commands report a validation error (exit code 4) and no records are created or shown, consistent with the other document types (BR-016).

### 2. Create

1. `papyrus audit create --title <title> --content <text> [--audit-type <type>]` creates an audit record.
2. `--title` and `--content` are both required; a missing argument is a usage error (exit code 2).
3. `title`, `content`, and `audit-type` are sanitized before use (NFR-02.002).
4. When `--audit-type` is omitted, the audit type defaults to `security`.
5. The audit type is stored as the supplied (or default) sanitized value. The built-in values are `security`, `licence`, and `architecture`; additional values may be used without a dedicated configuration key (see Open Questions).
6. The ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `AUD-001`), where `<number>` is one greater than the highest existing number found for that prefix in both the active and `deprecated/` folders. IDs are never reused (REQ-05.006, BR-014).
7. The active directory and its `deprecated/` subfolder are created if absent.
8. `create` prints the generated ID; in `--json` mode it returns `{"id": "<ID>"}`.
9. If any supplied value cannot be sanitized, creation fails with exit code 4 and no file is written.

### 3. List

1. `papyrus audit list` lists active audit records only (not deprecated).
2. Each row shows at least the ID and the title, ordered by ID ascending.
3. Only regular `*.md` files directly inside the active directory whose file stem matches `<prefix>-<digits>` are records. Subdirectories and files with other names are ignored.
4. Deprecated records are hidden from `list`.
5. `--json` returns a JSON array of objects with `id` and `title`.

### 4. View

1. `papyrus audit view <id>` prints the full persisted text of the audit record.
2. Viewing a deprecated record by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns an object with at least `id`, `path`, and `content`.

### 5. Edit

1. `papyrus audit edit <id> [--title <title>] [--content <text>] [--audit-type <type>] [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values (REQ-05.006).
3. `--title` replaces the title (and the top-level heading).
4. `--content` replaces the content text.
5. `--audit-type` sets the audit record type.
6. `--editor` opens the persisted file in `$VISUAL`, falling back to `$EDITOR` per effective configuration. Content saved by the editor is the new record content; the file name (ID) is never changed by an edit.
7. Editing a deprecated record is allowed and keeps it deprecated.
8. Editing an unknown ID fails with exit code 3.
9. If any supplied value cannot be sanitized, edit fails with exit code 4 and no content is changed.
10. If `--editor` is requested and no editor is configured, the command fails with exit code 1 and no content is changed.
11. `--json` returns `{"status": "ok", "id": "<ID>"}`.

### 6. Deprecation and restoration

1. `papyrus audit deprecate <id>` moves the audit record file from the active folder to `deprecated/`.
2. Deprecating an already-deprecated record fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecated records are moved out of the active folder, so they are not listed by `audit list`. The man page (BR-014) does not state whether deprecated audit records are hidden from search and the web UI; that visibility is resolved by TASK-48 and TASK-26 (recommended default: hide, consistent with the other document types).
5. `papyrus audit restore <id>` moves a deprecated record back to the active folder.
6. Restoring an already-active record fails with exit code 4.
7. Restoring an unknown ID fails with exit code 3.
8. Restoration preserves the record's ID and its file content (REQ-05.007, BR-014).
9. `--json` returns `{"status": "ok", "id": "<ID>"}` for both commands.

### 7. Persistence and sanitization

1. Records are UTF-8 Markdown files named `<ID>.md` in the active directory or its `deprecated/` subfolder (NFR-04.001).
2. File content uses this template: a top-level heading `# <ID> <title>`, then `## Audit Type`, then `## Content`. The file ends with a single trailing newline. `title`, `audit type`, and `content` are required, so their sections are always present.
3. `title`, `audit type`, and `content` are stored as their section bodies; multi-line bodies are preserved when the file is edited through `--editor`.
4. All user-controlled input (`<title>`, `--content`, `--audit-type`, `<id>`) is normalized/escaped before use: values are trimmed, and empty values, control characters (NUL, CR, LF in single-line inputs), and path separators are rejected (NFR-02.002).
5. Values that cannot be normalized fail with exit code 4 and leave the filesystem unchanged (NFR-02.002).
6. `<id>` is resolved by exact, case-sensitive match of the file stem; request values are never concatenated into a path without resolution, and every read/write path is checked against the canonicalized project directory (NFR-04.002).
7. No binary data is written (NFR-04.001).

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (missing title or `--content`) and unknown options exit 2 (usage).
3. Unknown audit record ID exits 3.
4. Filesystem/IO failure, missing editor, or project-directory boundary violation exits 1.

## Inputs and outputs

- `audit create --title <title> --content <text> [--audit-type <type>]`: `--title` and `--content` required; `--audit-type` defaults to `security`; output is the generated ID, or `{"id": "<ID>"}` in JSON mode.
- `audit list`: output is ID and title rows of active records, or a JSON array of `{id, title}`.
- `audit view <id>`: output is the full file text, or a JSON object with `id`, `path`, and `content`.
- `audit edit <id> [--title] [--content] [--audit-type] [--editor]`: output is minimal success text, or `{"status": "ok", "id": "<ID>"}`.
- `audit deprecate <id>` / `audit restore <id>`: move the file; minimal success output, or `{"status": "ok", "id": "<ID>"}`.

## Validation and error behavior

- Missing required argument or unknown option → exit code 2 (usage).
- Unknown ID → exit code 3 (not found).
- Disabled type, already-deprecated/already-active state conflict, unnormalizable input → exit code 4 (validation).
- IO failure, missing editor, or project-directory boundary violation → exit code 1 (runtime).
- A failed validation writes nothing; a failed `deprecate`/`restore` leaves the file in its current folder.
- `--json` error output carries the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001); the single-crate layout is retained (ADR-0004), so an `audit` module may be added within the crate without a new ADR.
- Assumption: IDs are generated by scanning the active and `deprecated/` folders for the highest existing number with the configured prefix; `id_padding` (default 3) applies. This mirrors TASK-30 and TASK-43.
- Assumption: record file style follows the TASK-30/TASK-43 template pattern (`# <ID> <title>` plus `## ` sections). BR-014 does not fix section names, so `## Audit Type` and `## Content` are used; tests assert field presence rather than byte-for-byte formatting.
- Assumption: `list`, `view`, `deprecate`, and `restore` do not recurse into subdirectories of the audit directory.
- Assumption: `init` already creates the audit directory and its `deprecated/` subfolder; this ticket ensures both exist on `create` and on `deprecate` if absent.
- Assumption: `--title`, `--content`, and `--audit-type` are single-line sanitized inputs; multi-line content is available through `--editor`. The audit type is a single-line sanitized value.
- Assumption: deprecating an already-deprecated record or restoring an already-active one is a validation error (exit code 4), matching TASK-30 and TASK-43.
- Assumption: `list` order is by ID ascending.
- Assumption: editing a deprecated record is allowed, including field changes, matching TASK-30 and TASK-43.
- Assumption: audit types are not validated against a configured allow-list because BR-016 documents no configuration key for them; the type value is stored as supplied after sanitization, with `security` as the default.

## Open questions

- OQ-1 (non-blocking, for the architect/product owner): BR-014 says additional audit types are "configurable", but BR-016 documents no configuration key for them (unlike `decision_types`). This specification treats the audit type as a sanitized free-form value and does not enforce an allow-list. If a dedicated configuration key is later added, only the validation step changes; the file template and commands remain the same.
- OQ-2 (non-blocking, for TASK-48 and TASK-26): BR-014 does not state whether deprecated audit records are hidden from search and the web UI. The CLI `list` excludes them by folder. The recommended default for search and browser is to hide deprecated audit records, consistent with the other document types, but that decision is owned by TASK-48/TASK-26.
- OQ-3 (non-blocking): BR-014 gives no section names for the record body. `## Audit Type` and `## Content` are adopted here for consistency with the other document types; a different layout would change the file template only, not the command contract.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.006, REQ-05.007 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-014, BR-016 |
| Domain | `docs/domain-model/business-glossary/audit-record.md` |
| ADR | ADR-0001 (Rust), ADR-0002 (amd64 .deb), ADR-0004 (single-crate layout) |
| Ticket | TASK-47 |
| UI mockups | none |
| Related specification | `docs/specifications/business-rules-management.md` (TASK-43 pattern), `docs/specifications/glossary-terms-management.md` (TASK-30 pattern), `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |
path: docs/specifications/audit-records-management.md