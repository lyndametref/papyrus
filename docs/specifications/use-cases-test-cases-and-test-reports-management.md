# Feature Specification — Use Cases, Test Cases, and Test Reports Management from the CLI

**Ticket:** TASK-78
**Milestone:** m-15 (Use Cases and Test Documentation)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only; browser UI for these record types is TASK-79/TASK-81)

## Purpose

Deliver three dedicated commands — `use-case`, `test-case`, and `test-report` — so a project contributor can create, list, view, edit, deprecate, and restore use-case, test-case, and test-report records, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.012, REQ-05.013, and REQ-05.014, the lifecycle part of REQ-05.007, NFR-02.002, NFR-04.001, and NFR-04.002, together with the documented command contract in BR-018, BR-019, and BR-020 and the configuration contract in BR-016. It provides the records that later tickets search (TASK-80) and expose in the browser (TASK-81).

This ticket resolves the managing-command conflict recorded in REQ-05.012/013/014 and BR-018/019/020: those requirements state that the three types "are managed with the `req` command", but BR-009 defines `req` as business-requirements only (REQ-05.001: `--group`, `--story`, requirement-to-group cardinality 1..1, groups contain business requirements only). Routing the three types through `req` would place two record kinds in one document type and one ID namespace. **Resolution (decided for implementation): manage each type with a dedicated command — `use-case`, `test-case`, and `test-report` — following the per-type command pattern of `tech-spec` (BR-017).** The `req` command remains business-requirements only. The requirements files are not edited by this refinement; their `req` wording is superseded by this decision for implementation, and any wording correction is left to the requirements owner.

## Scope

### In scope

- `papyrus use-case create <title> --description <text> [--applicability <text>]` (BR-018, REQ-05.012).
- `papyrus use-case list` (BR-018, REQ-05.012).
- `papyrus use-case view <id>` (BR-018, REQ-05.012).
- `papyrus use-case edit <id> [--title <title>] [--description <text>] [--applicability <text>] [--editor]` (BR-018, REQ-05.012).
- `papyrus use-case deprecate <id>` and `papyrus use-case restore <id>` (BR-018, REQ-05.007).
- `papyrus test-case create <title> --description <text> [--applicability <text>]` (BR-019, REQ-05.013).
- `papyrus test-case list` (BR-019, REQ-05.013).
- `papyrus test-case view <id>` (BR-019, REQ-05.013).
- `papyrus test-case edit <id> [--title <title>] [--description <text>] [--applicability <text>] [--editor]` (BR-019, REQ-05.013).
- `papyrus test-case deprecate <id>` and `papyrus test-case restore <id>` (BR-019, REQ-05.007).
- `papyrus test-report create <title> --description <text> [--applicability <text>]` (BR-020, REQ-05.014).
- `papyrus test-report list` (BR-020, REQ-05.014).
- `papyrus test-report view <id>` (BR-020, REQ-05.014).
- `papyrus test-report edit <id> [--title <title>] [--description <text>] [--applicability <text>] [--editor]` (BR-020, REQ-05.014).
- `papyrus test-report deprecate <id>` and `papyrus test-report restore <id>` (BR-020, REQ-05.007).
- Record identity: stable IDs of the form `<prefix>-<number>` (for example `UC-001`, `TC-001`, `TR-001`), using the configured `document_types` prefix and `id_padding` (REQ-05.012/013/014, BR-018/019/020, BR-016).
- Record fields: title and description (required), applicability (optional) (REQ-05.012/013/014, BR-018/019/020).
- Deprecation moving a record to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.007, BR-018/019/020).
- Restoration returning a deprecated record to its original folder with the same ID and content (REQ-05.007, BR-018/019/020).
- YAML front matter on every record per REQ-05.010, with general fields only (`id`, `title`, `created_date`, `updated_date`, `labels`, `references`) (REQ-05.010, ADR-0007, TASK-65).
- Front-matter metadata options `--label` and `--reference` on create and edit for all three types, per the expose spec (TASK-65).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search integration for these record types — TASK-80.
- Browser listing, viewing, and editing, including the browser design review — TASK-79 and TASK-81.
- Packaging and verification of the use-cases-and-test-documentation milestone — TASK-82.
- Deleting a record — BR-018/019/020 define no `delete` command.
- Changing the default directories for these types — they stay `use-cases/`, `test-cases/`, `test-reports/` per BR-016/BR-018/019/020.
- Any new top-level configuration key beyond `document_types.<type>` entries (BR-016).
- Validation that a referenced test-case ID exists in a test report — BR-020 leaves test-case references to the general `references` field; existence is not validated, consistent with the business-rule reference handling.
- Dedicated fields for pass/fail outcomes or test-case coverage in test reports — REQ-05.014 does not define them; such information is captured in the record body (description/applicability).
- Association of these records to tasks or milestones — reference targets accepted by tasks are fixed by REQ-02.001; adding one requires a requirement change.
- UI mockups or web UI — none for this ticket; CLI output only.

## Type parameters

| Type | Command | Prefix (default) | Directory (default) | Config key |
|---|---|---|---|---|
| Use case | `use-case` | `UC` | `use-cases/` | `document_types.use-case` |
| Test case | `test-case` | `TC` | `test-cases/` | `document_types.test-case` |
| Test report | `test-report` | `TR` | `test-reports/` | `document_types.test-report` |

All three share the same field set, command shape, lifecycle, persistence, and sanitization rules below; only the command name, prefix, directory, and config key differ.

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.<type>.prefix`, `document_types.<type>.directory`, `document_types.<type>.enabled`, plus `doc_directory` and `id_padding` (BR-016, BR-018/019/020).
4. The active directory is `<project>/<doc_directory>/<document_types.<type>.directory>`; the deprecated directory is `<active>/deprecated/`.
5. If `document_types.<type>.enabled` is false, `create` and `list` report a validation error (exit code 4) and no records are created or shown, matching the behavior delivered for the other document types (BR-016, BR-018/019/020).
6. The default configuration gains three entries, all enabled: `use-case: { prefix: UC, directory: use-cases/, enabled: true }`, `test-case: { prefix: TC, directory: test-cases/, enabled: true }`, `test-report: { prefix: TR, directory: test-reports/, enabled: true }`.

### 2. Create

1. `papyrus <type> create <title> --description <text> [--applicability <text>] [--label <label>]... [--reference <value>]...` creates a record of that type.
2. `<title>` and `--description` are required; `--applicability` is optional; a missing required argument is a usage error (exit code 2).
3. `title`, `description`, and `applicability` (when supplied) are sanitized before use (NFR-02.002).
4. Each `--label` and `--reference` value is sanitized before use; repeated values collapse to unique entries.
5. The ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `UC-001`), where `<number>` is one greater than the highest existing number found for that prefix in both the active and `deprecated/` folders. IDs are never reused (REQ-05.012/013/014, BR-018/019/020).
6. The active directory and its `deprecated/` subfolder are created if absent.
7. `created_date` and `updated_date` are set to the current UTC time at creation (TASK-65, REQ-05.010).
8. `create` prints the generated ID; in `--json` mode it returns `{"id": "<ID>"}`.
9. If any supplied value cannot be sanitized, creation fails with exit code 4 and no file is written.

### 3. List

1. `papyrus <type> list` lists active records of that type only (not deprecated).
2. Each row shows at least the ID and the title, ordered by ID ascending.
3. Only regular `*.md` files directly inside the active directory whose file stem matches `<prefix>-<digits>` are records. Subdirectories and files with other names are ignored.
4. Deprecated records are hidden from `list` (REQ-05.007, BR-018/019/020).
5. `--json` returns a JSON array of objects with `id` and `title`.

### 4. View

1. `papyrus <type> view <id>` prints the full persisted text of the record.
2. Viewing a deprecated record by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns an object with at least `id`, `path`, and `content`.

### 5. Edit

1. `papyrus <type> edit <id> [--title <title>] [--description <text>] [--applicability <text>] [--label <label>]... [--reference <value>]... [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values (REQ-05.012/013/014).
3. `--title` replaces the title (and the top-level heading).
4. `--description` replaces the description text.
5. `--applicability` replaces the applicability text; when the supplied value is empty after sanitization the `## Applicability` section is removed.
6. `--label` appends unique sanitized labels to the existing front-matter `labels` list; `--reference` appends unique sanitized values to the existing front-matter `references` list.
7. `--editor` opens the persisted file in `$VISUAL`, falling back to `$EDITOR` per effective configuration. Content saved by the editor is the new record content; the file name (ID) is never changed by an edit.
8. Editing a deprecated record is allowed and keeps it deprecated.
9. Editing an unknown ID fails with exit code 3.
10. If any supplied value cannot be sanitized, edit fails with exit code 4 and no content is changed.
11. If `--editor` is requested and no editor is configured, the command fails with exit code 1 and no content is changed.
12. `updated_date` is refreshed only when at least one byte changes (TASK-65). A no-op edit leaves the file byte-identical and does not refresh `updated_date`.
13. `--json` returns `{"status": "ok", "id": "<ID>"}`.

### 6. Deprecation and restoration

1. `papyrus <type> deprecate <id>` moves the record file from the active folder to `deprecated/`.
2. Deprecating an already-deprecated record fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecated records are hidden from lists, search, and the web UI (REQ-05.007, BR-018/019/020).
5. `papyrus <type> restore <id>` moves a deprecated record back to the active folder.
6. Restoring an already-active record fails with exit code 4.
7. Restoring an unknown ID fails with exit code 3.
8. Restoration preserves the record's ID and its file content (REQ-05.007, BR-018/019/020).
9. Deprecation and restoration do not rewrite file content and therefore do not refresh `updated_date`.
10. `--json` returns `{"status": "ok", "id": "<ID>"}` for both commands.

### 7. Persistence and sanitization

1. Records are UTF-8 Markdown files named `<ID>.md` in the active directory or its `deprecated/` subfolder (NFR-04.001).
2. File content uses this template: a top-level heading `# <ID> <title>`, then `## Description` containing the description text, then, only when applicability is provided, `## Applicability` containing the applicability text. `title` and `description` are required, so their sections are always present. The file ends with a single trailing newline.
3. `description` and `applicability` (when present) are stored as their section bodies; multi-line bodies are preserved when the file is edited through `--editor`.
4. All user-controlled input (`<title>`, `--description`, `--applicability`, `--label`, `--reference`, `<id>`) is normalized/escaped before use: values are trimmed, and empty values, control characters (NUL, CR, LF in single-line inputs), and path separators are rejected (NFR-02.002).
5. Values that cannot be normalized fail with exit code 4 and leave the filesystem unchanged (NFR-02.002).
6. `<id>` is resolved by exact, case-sensitive match of the file stem; request values are never concatenated into a path without resolution, and every read/write path is checked against the canonicalized project directory (NFR-04.002).
7. No binary data is written (NFR-04.001).
8. Every managed record write uses the shared envelope and patch-preserving store delivered by TASK-63/TASK-65: patch contains only changed keys; unmodified front-matter keys, key order, unknown keys, body lines, section order, and line endings are preserved byte-for-byte. Create renders the initial document through the shared envelope rather than ad-hoc string formatting.
9. Front matter carries `id`, `title`, `created_date`, `updated_date`, `labels`, and `references` (REQ-05.010). `description` and `applicability` are body fields, not front-matter keys.

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (missing title or `--description`) and unknown options exit 2 (usage).
3. Unknown record ID exits 3.
4. Disabled type, already-deprecated/already-active state conflict, unnormalizable input, or empty applicability-clearance conflict exit 4 (validation).
5. Filesystem/IO failure, missing editor, or project-directory boundary violation exits 1 (runtime).
6. A failed validation writes nothing; a failed `deprecate`/`restore` leaves the file in its current folder.
7. `--json` error output carries the exit-relevant error information, as delivered by TASK-2.

## Inputs and outputs

- `use-case|test-case|test-report create <title> --description <text> [--applicability <text>] [--label ...] [--reference ...]`: required title and description; output is the generated ID, or `{"id": "<ID>"}` in JSON mode.
- `use-case|test-case|test-report list`: output is ID and title rows of active records, or a JSON array of `{id, title}`.
- `use-case|test-case|test-report view <id>`: output is the full file text, or a JSON object with `id`, `path`, and `content`.
- `use-case|test-case|test-report edit <id> [--title] [--description] [--applicability] [--label ...] [--reference ...] [--editor]`: output is minimal success text, or `{"status": "ok", "id": "<ID>"}`.
- `use-case|test-case|test-report deprecate <id>` / `restore <id>`: move the file; minimal success output, or `{"status": "ok", "id": "<ID>"}`.

## Validation and error behavior

- Missing required argument or unknown option → exit code 2 (usage).
- Unknown ID → exit code 3 (not found).
- Disabled type, already-deprecated/already-active state conflict, unnormalizable input → exit code 4 (validation).
- IO failure, missing editor, or project-directory boundary violation → exit code 1 (runtime).
- A failed validation writes nothing; a failed `deprecate`/`restore` leaves the file in its current folder.
- `--json` error output carries the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001); the single-crate layout is retained (ADR-0004), so `use_case`, `test_case`, and `test_report` modules may be added within the crate without a new ADR.
- Assumption: IDs are generated by scanning the active and `deprecated/` folders for the highest existing number with the configured prefix; `id_padding` (default 3) applies. This mirrors TASK-30/TASK-55.
- Assumption: record file style follows the TASK-30/TASK-55 template pattern (`# <ID> <title>` plus `## ` sections). BR-018/019/020 do not fix section names, so `## Description` and `## Applicability` are used; tests assert field presence rather than byte-for-byte formatting.
- Assumption: `list`, `view`, `deprecate`, and `restore` do not recurse into subdirectories of the type directory.
- Assumption: `init` already creates the type directories and their `deprecated/` subfolders for enabled types, or this ticket ensures both exist on `create` and on `deprecate` if absent, matching TASK-55.
- Assumption: `--title`, `--description`, `--applicability`, `--label`, and `--reference` are single-line sanitized inputs; multi-line bodies are available through `--editor`.
- Assumption: deprecating an already-deprecated record or restoring an already-active one is a validation error (exit code 4), matching TASK-30.
- Assumption: `list` order is by ID ascending.
- Assumption: editing a deprecated record is allowed, including field changes, matching TASK-30.
- Assumption: `--applicability` supplied with an empty-after-sanitization value removes the `## Applicability` section; otherwise it replaces the section body.
- Assumption: the ID prefixes `UC`, `TC`, and `TR`, directories `use-cases/`, `test-cases/`, and `test-reports/`, and enabled default `true` are adopted from the BR-018/019/020 proposals.
- Assumption: test-case references in a test report, pass/fail outcomes, and coverage data are not modeled as dedicated fields; they are captured in the record body or in the general `references` list, per BR-020.
- Assumption: no association to tasks or milestones is added, per BR-018/019/020.
- Assumption: front-matter general fields only; `description` and `applicability` remain body fields, per REQ-05.010 and the expose spec (TASK-65).
- Assumption: `--label` and `--reference` manage the general front-matter lists and are independent of any body reference list (none exists for these types).

## Open questions

- OQ-1 (non-blocking): BR-018/019/020 give no section names for the record body. `## Description` and `## Applicability` are adopted here for consistency with the other document types; a different layout would change the file template only, not the command contract.
- OQ-2 (non-blocking): clearing applicability requires `--applicability` with an empty value (which removes the section) or `--editor`; no separate `--no-applicability` flag is defined. If a clearer toggle is wanted, it is a command-surface addition, not a contract change.
- OQ-3 (non-blocking, for TASK-80/TASK-81): whether test reports should gain a dedicated test-case-reference or outcome field is undecided (BR-020); this spec deliberately keeps the field set minimal (title, description, optional applicability) and relies on body text plus general `references`.
- OQ-4 (non-blocking, requirements wording): REQ-05.012/013/014 state these types are managed with the `req` command, which conflicts with BR-009. This spec resolves the conflict in favor of dedicated commands. The requirements files are not edited by this refinement; the requirements owner may correct the wording separately.
- OQ-5 (non-blocking, configuration): the default `document_types` entries proposed here must be added to the configuration defaults and to README `papyrus-config(5)`; until then, projects without the entries cannot create records of these types.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.012, REQ-05.013, REQ-05.014, REQ-05.007, REQ-05.010 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-018, BR-019, BR-020, BR-016, BR-009, BR-017 |
| Domain | `docs/domain-model/business-glossary/` (use case, test case, test report terms if present) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (single-crate layout), ADR-0007 (YAML front matter), ADR-0009 (front-matter options) |
| Ticket | TASK-78 |
| UI mockups | none |
| Related specification | `docs/specifications/technical-specifications-management.md` (TASK-55 pattern), `docs/specifications/business-rules-management.md` (TASK-43 pattern), `docs/specifications/expose-front-matter-metadata-options-on-all-record-commands.md` (TASK-65) |
| Dependent tickets | TASK-79 (design review), TASK-80 (search), TASK-81 (browser), TASK-82 (packaging) |
