# Feature Specification — Business Rules Management from the CLI

**Ticket:** TASK-43
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only; browser UI for this record type is TASK-25/TASK-45)

## Purpose

Deliver the `rule` commands so a project contributor can create, list, view, edit, deprecate, and restore business rules, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.005 and the lifecycle part of REQ-05.007, together with the documented command contract in BR-013. It provides the records that later tickets search (TASK-44) and expose in the browser (TASK-25).

## Scope

### In scope

- `papyrus rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]` (BR-013, REQ-05.005).
- `papyrus rule list` (BR-013, REQ-05.005).
- `papyrus rule view <id>` (BR-013, REQ-05.005).
- `papyrus rule edit <id> [--title <title>] [-d <description>] [--requirement <id>...] [--no-requirement [<id>]] [--editor]` (BR-013, REQ-05.005).
- `papyrus rule deprecate <id>` and `papyrus rule restore <id>` (BR-013, REQ-05.007).
- Business-rule identity: stable IDs of the form `<prefix>-<number>` (for example `RUL-001`), using the configured `document_types.rule` prefix and `id_padding` (REQ-05.005, BR-013, BR-016).
- Business-rule fields: title, description, rationale, applicability, and zero or more references to business requirements (REQ-05.005, BR-013, domain glossary term "Business Rule").
- Deprecation moving a rule to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.005, REQ-05.007, BR-013).
- Restoration returning a deprecated rule to its original folder with the same ID and content (REQ-05.005, REQ-05.007, BR-013).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search integration for business rules — TASK-44.
- Browser listing, viewing, and editing of business rules, including the browser design review — TASK-25 and TASK-45.
- Packaging and verification of the business-rules milestone — TASK-46.
- Deleting a business rule — BR-013 defines no `delete` command.
- Changing the default directory for the `rule` type (it stays `business-rules/` per BR-016).
- Any new configuration key beyond those already delivered by TASK-2 (`document_types.rule`).
- Validation that a referenced business-requirement ID exists — BR-013 does not specify it; references are stored as sanitized IDs.

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.rule.prefix` (default `RUL`), `document_types.rule.directory` (default `business-rules/`), `document_types.rule.enabled`, plus `doc_directory` and `id_padding` (BR-016).
4. The active directory is `<project>/<doc_directory>/<document_types.rule.directory>`; the deprecated directory is `<active>/deprecated/`.
5. If `document_types.rule.enabled` is false, all commands report a validation error (exit code 4) and no records are created or shown, consistent with the other document types (BR-016).

### 2. Create

1. `papyrus rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]` creates a business rule.
2. `<title>`, `--description`, `--rationale`, and `--applicability` are all required; a missing argument is a usage error (exit code 2).
3. `title`, `description`, `rationale`, and `applicability` are sanitized before use (NFR-02.002).
4. Each `--requirement <id>` is sanitized before use and added to the rule's requirement-reference list. Zero or more may be supplied; duplicates are collapsed so each ID appears once.
5. The ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `RUL-001`), where `<number>` is one greater than the highest existing number found for that prefix in both the active and `deprecated/` folders. IDs are never reused (REQ-05.005, BR-013).
6. The active directory and its `deprecated/` subfolder are created if absent.
7. `create` prints the generated ID; in `--json` mode it returns `{"id": "<ID>"}`.
8. If any supplied value cannot be sanitized, creation fails with exit code 4 and no file is written.

### 3. List

1. `papyrus rule list` lists active business rules only (not deprecated).
2. Each row shows at least the ID and the title, ordered by ID ascending.
3. Only regular `*.md` files directly inside the active directory whose file stem matches `<prefix>-<digits>` are records. Subdirectories and files with other names (for example the canonical `BR-*.md` files in this repository) are ignored.
4. Deprecated rules are hidden from `list` (REQ-05.005, REQ-05.007, BR-013).
5. `--json` returns a JSON array of objects with `id` and `title`.

### 4. View

1. `papyrus rule view <id>` prints the full persisted text of the rule.
2. Viewing a deprecated rule by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns an object with at least `id`, `path`, and `content`.

### 5. Edit

1. `papyrus rule edit <id> [--title <title>] [-d <description>] [--requirement <id>...] [--no-requirement [<id>]] [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values (REQ-05.005).
3. `--title` replaces the title (and the top-level heading).
4. `-d`/`--description` replaces the description text.
5. `--requirement <id>` adds the supplied business-requirement ID to the existing reference list (repeatable). An ID already present is not duplicated.
6. `--no-requirement <id>` removes the specified business-requirement ID from the reference list. `--no-requirement` without an ID removes all requirement references.
7. `--requirement` and `--no-requirement` may both be used in one invocation; additions are applied before removals, and the resulting list contains no duplicates.
8. `--editor` opens the persisted file in `$VISUAL`, falling back to `$EDITOR` per effective configuration. Content saved by the editor is the new record content; the file name (ID) is never changed by an edit.
9. The rationale and applicability have no dedicated options; they are only changeable through `--editor`, which rewrites the file without renaming it.
10. Editing a deprecated rule is allowed and keeps it deprecated.
11. Editing an unknown ID fails with exit code 3.
12. If any supplied value cannot be sanitized, or no editor is configured while `--editor` is requested, the command fails with exit code 4 (sanitization) or exit code 1 (no configured editor), and no content is changed.
13. `--json` returns `{"status": "ok", "id": "<ID>"}`.

### 6. Deprecation and restoration

1. `papyrus rule deprecate <id>` moves the rule file from the active folder to `deprecated/`.
2. Deprecating an already-deprecated rule fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecated rules are hidden from lists, search, and the web UI (REQ-05.005, REQ-05.007, BR-013).
5. `papyrus rule restore <id>` moves a deprecated rule back to the active folder.
6. Restoring an already-active rule fails with exit code 4.
7. Restoring an unknown ID fails with exit code 3.
8. Restoration preserves the rule's ID and its file content (REQ-05.007, BR-013).
9. `--json` returns `{"status": "ok", "id": "<ID>"}` for both commands.

### 7. Persistence and sanitization

1. Records are UTF-8 Markdown files named `<ID>.md` in the active directory or its `deprecated/` subfolder (NFR-04.001).
2. File content uses this template: a top-level heading `# <ID> <title>`, then `## Description`, then `## Rationale`, then `## Applicability`, then, only when at least one requirement reference exists, `## Requirements` containing one bullet `- <id>` per referenced business requirement. All of `title`, `description`, `rationale`, and `applicability` are required, so their sections are always present. The file ends with a single trailing newline.
3. `description`, `rationale`, `applicability`, and each requirement reference are stored as their section bodies; multi-line bodies are preserved when the file is edited through `--editor`.
4. All user-controlled input (`<title>`, `--description`, `--rationale`, `--applicability`, `--requirement`, `<id>`) is normalized/escaped before use: values are trimmed, and empty values, control characters (NUL, CR, LF in single-line inputs), and path separators are rejected (NFR-02.002).
5. Values that cannot be normalized fail with exit code 4 and leave the filesystem unchanged (NFR-02.002).
6. `<id>` is resolved by exact, case-sensitive match of the file stem; request values are never concatenated into a path without resolution, and every read/write path is checked against the canonicalized project directory (NFR-04.002).
7. No binary data is written (NFR-04.001).

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (missing title, `--description`, `--rationale`, or `--applicability`) and unknown options exit 2 (usage).
3. Unknown rule ID exits 3.
4. Filesystem/IO failure, missing editor, or project-directory boundary violation exits 1.

## Inputs and outputs

- `rule create <title> --description <text> --rationale <text> --applicability <text> [--requirement <id>...]`: all four core fields required; output is the generated ID, or `{"id": "<ID>"}` in JSON mode.
- `rule list`: output is ID and title rows of active rules, or a JSON array of `{id, title}`.
- `rule view <id>`: output is the full file text, or a JSON object with `id`, `path`, and `content`.
- `rule edit <id> [--title] [-d] [--requirement...] [--no-requirement [id]] [--editor]`: output is minimal success text, or `{"status": "ok", "id": "<ID>"}`.
- `rule deprecate <id>` / `rule restore <id>`: move the file; minimal success output, or `{"status": "ok", "id": "<ID>"}`.

## Validation and error behavior

- Missing required argument or unknown option → exit code 2 (usage).
- Unknown ID → exit code 3 (not found).
- Disabled type, already-deprecated/already-active state conflict, unnormalizable input → exit code 4 (validation).
- IO failure, missing editor, or project-directory boundary violation → exit code 1 (runtime).
- A failed validation writes nothing; a failed `deprecate`/`restore` leaves the file in its current folder.
- `--json` error output carries the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001); the single-crate layout is retained (ADR-0004), so a `rule` module may be added within the crate without a new ADR.
- Assumption: IDs are generated by scanning the active and `deprecated/` folders for the highest existing number with the configured prefix; `id_padding` (default 3) applies. This mirrors TASK-30 and TASK-34.
- Assumption: record file style follows the TASK-30/TASK-34 template pattern (`# <ID> <title>` plus `## ` sections). BR-013 does not fix section names, so `## Description`, `## Rationale`, `## Applicability`, and `## Requirements` are used; tests assert field presence rather than byte-for-byte formatting.
- Assumption: `list`, `view`, `deprecate`, and `restore` do not recurse into subdirectories of the rule directory, so sibling files (including canonical `BR-*.md` documents) are never treated as records.
- Assumption: the default `rule` directory `business-rules/` is shared in this repository with canonical business-rule documents; the record set is limited to `<prefix>-<digits>.md` files at the top level. Relocating the type directory is a configuration decision, not part of this ticket.
- Assumption: `init` already creates the rule directory and its `deprecated/` subfolder; this ticket ensures both exist on `create` and on `deprecate` if absent.
- Assumption: `--description`, `--rationale`, `--applicability`, and `<title>` are single-line sanitized inputs; multi-line bodies are available through `--editor`. Requirement references are single-line sanitized IDs.
- Assumption: deprecating an already-deprecated rule or restoring an already-active one is a validation error (exit code 4), matching TASK-30 and TASK-34.
- Assumption: `list` order is by ID ascending.
- Assumption: editing a deprecated rule is allowed, including field changes, matching TASK-30 and TASK-34.
- Assumption: requirement references are accepted as sanitized IDs; existence of the referenced business requirement is not verified by this command, because BR-013 does not specify such validation.
- Assumption: duplicate requirement references are collapsed on write.

## Open questions

- OQ-1 (non-blocking, for the architect/product owner): BR-013 gives no section names for the record body. `## Description`, `## Rationale`, `## Applicability`, and `## Requirements` are adopted here for consistency with the other document types; a different layout would change the file template only, not the command contract.
- OQ-2 (non-blocking, for the browser ticket TASK-25/TASK-45): whether the browser renders requirement references as links is unspecified and out of scope for this ticket.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.005, REQ-05.007 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-013, BR-016 |
| Domain | `docs/domain-model/business-glossary/business-rule.md`, `docs/domain-model/object-model/business-object-model.md` |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (single-crate layout) |
| Ticket | TASK-43 |
| UI mockups | none |
| Related specification | `docs/specifications/glossary-terms-management.md` (TASK-30 pattern), `docs/specifications/ui-specifications-management.md` (TASK-34 pattern), `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |
