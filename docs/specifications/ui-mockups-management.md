# Feature Specification — UI Mockups Management from the CLI

**Ticket:** TASK-38
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none (CLI-only; browser UI for mockups is TASK-40/TASK-41)

## Purpose

Deliver the `ui-mockup` commands so a project contributor can create, list, view, edit, deprecate, restore, and supersede UI mockups, with stable `MOCK-xxxx` IDs and HTML-file persistence inside the project directory. This ticket implements REQ-05.008 and the lifecycle part of REQ-05.007. It provides the mockup records that later tickets search (TASK-39) and expose in the browser (TASK-40). No dedicated business rule exists for this command (see Open questions); BR-016 governs configuration.

Mockups are HTML review packages stored under `docs/specifications/ui/mockups/`. Their registry and status vocabulary are documented in `docs/specifications/ui/mockups/README.md` (`pending`/`approved`/`deprecated`; project notes may say `draft` for not-yet-approved mockups).

## Scope

### In scope

- `papyrus ui-mockup create <title> --task <task> --spec <spec-id>... [--ref <doc>...]` (REQ-05.008).
- `papyrus ui-mockup list` (REQ-05.008).
- `papyrus ui-mockup view <id>` (REQ-05.008).
- `papyrus ui-mockup edit <id> [--title <title>] [--task <task>] [--spec <spec-id>...] [--ref <doc>...] [--status <status>] [--superseded-by <id>] [--editor]` (REQ-05.008).
- `papyrus ui-mockup deprecate <id>` and `papyrus ui-mockup restore <id>` (REQ-05.007, REQ-05.008).
- Mockup identity: stable IDs of the form `MOCK-<number>` (for example `MOCK-001`), using the configured `document_types.ui-mockup` prefix and `id_padding` (REQ-05.008, BR-016).
- Mockup fields: title, associated task, references to other documents, and implementation reference(s) to UI specification(s) by spec id (REQ-05.008).
- Status vocabulary: `pending`, `approved`, `deprecated` (REQ-05.008).
- Supersession: a mockup may supersede an older mockup; the superseded mockup becomes deprecated and records a back-reference to its successor; status transitions are blocked while a mockup is superseded (REQ-05.008).
- Deprecation moving a mockup to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.007, REQ-05.008).
- Restoration returning a deprecated mockup to its original folder with the same ID and an updated status field in the HTML (REQ-05.007, REQ-05.008).
- Persistence as UTF-8 HTML text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search integration for mockups — TASK-39.
- Browser listing, viewing, and editing of mockups, including the browser design review — TASK-40 and TASK-41.
- Generating full visual review-package content (design deck, prototypes); this ticket allocates IDs/files and manages metadata only. Visual content is authored by `@ui-designer`.
- Packaging and verification of the UI-mockups milestone — TASK-42.
- Deleting a mockup — no `delete` command is defined.
- Changing the default directory for the `ui-mockup` type (it stays `specifications/ui/mockups/`).
- New configuration keys beyond `document_types.ui-mockup` (BR-016).
- Linking mockups to tickets beyond recording the ticket id in the HTML and referencing the mockup id from the ticket (REQ-05.008).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.ui-mockup.prefix` (default `MOCK`), `document_types.ui-mockup.directory` (default `specifications/ui/mockups/`), `document_types.ui-mockup.enabled`, plus `doc_directory` and `id_padding` (BR-016).
4. The active directory is `<project>/<doc_directory>/<document_types.ui-mockup.directory>`; the deprecated directory is `<active>/deprecated/`.
5. If `document_types.ui-mockup.enabled` is false, `create` and `list` report a validation error (exit code 4) and no records are created or shown, matching the other document types (BR-016).
6. The active directory also contains non-record files (`README.md`, `wireframe.css`) and the `mockups/` registry is shared with them; only top-level files matching `<prefix>-<digits>.html` are records. The `deprecated/` subfolder is scanned only for ID generation and resolution.

### 2. Create

1. `papyrus ui-mockup create <title> --task <task> --spec <spec-id>... [--ref <doc>...]` creates a mockup.
2. `<title>`, `--task`, and at least one `--spec` are required; a missing argument is a usage error (exit code 2).
3. `title`, `task`, each `--spec`, and each `--ref` are sanitized before use (NFR-02.002).
4. The ID is generated as `MOCK-<number>`, padded to `id_padding` digits (for example `MOCK-001`), where `<number>` is one greater than the highest existing number found for that prefix in both the active and `deprecated/` folders. IDs are never reused (REQ-05.008).
5. The active directory and its `deprecated/` subfolder are created if absent.
6. `create` writes a minimal HTML stub (section 9.2) with status `pending`, records the associated task and the spec id(s) and references in the HTML, and prints the generated ID; in `--json` mode it returns `{"id": "<ID>"}`.
7. If any supplied value cannot be sanitized, creation fails with exit code 4 and no file is written.

### 3. List

1. `papyrus ui-mockup list` lists active mockups only (not deprecated).
2. Each row shows at least the ID and the title, ordered by ID ascending.
3. Only regular `*.html` files directly inside the active directory whose file stem matches `<prefix>-<digits>` are records. Subdirectories (including `deprecated/` and any future subfolder) and non-matching files (`README.md`, `wireframe.css`, MOCK files without the numeric stem) are ignored.
4. Deprecated mockups are hidden from `list` (REQ-05.007, REQ-05.008).
5. `--json` returns a JSON array of objects with `id` and `title`.

### 4. View

1. `papyrus ui-mockup view <id>` prints the full persisted HTML text of the mockup.
2. Viewing a deprecated mockup by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` returns an object with at least `id`, `path`, and `content`.

### 5. Edit

1. `papyrus ui-mockup edit <id> [--title <title>] [--task <task>] [--spec <spec-id>...] [--ref <doc>...] [--status <status>] [--superseded-by <id>] [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values (REQ-05.008).
3. `--title` updates the title (and the `<title>` element and the `<h1>`). `--task` updates the associated task. `--spec` replaces the full set of referenced UI specification ids. `--ref` replaces the full set of references to other documents.
4. `--status <status>` sets the status to one of `pending`, `approved`, `deprecated`, subject to the transition rules in section 8 and the supersede constraint in section 7.
5. `--superseded-by <id>` marks this mockup as superseded by the specified mockup (section 7).
6. `--editor` opens the persisted file in `$VISUAL`, falling back to `$EDITOR`. Content saved by the editor is the new record content; the file name (ID) is never changed by an edit.
7. `--superseded-by` cannot be combined with `--status` (usage error, exit code 2).
8. Editing a deprecated mockup is allowed and keeps it deprecated, unless `--status` requests a permitted transition (section 8).
9. Editing an unknown ID fails with exit code 3.
10. If any supplied value cannot be sanitized, or no editor is configured while `--editor` is requested, the command fails with exit code 4 (sanitization) or exit code 1 (no configured editor), and no content is changed.
11. `--json` returns `{"status": "ok", "id": "<ID>"}`.

### 6. Deprecation and restoration

1. `papyrus ui-mockup deprecate <id>` moves the mockup file from the active folder to `deprecated/`.
2. Deprecating an already-deprecated mockup fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecating a superseded mockup fails with exit code 4 (status transitions blocked, section 7).
5. Deprecated mockups are hidden from lists, search, and the web UI (REQ-05.007, REQ-05.008).
6. `papyrus ui-mockup restore <id>` moves a deprecated mockup back to the active folder.
7. Restoring an already-active mockup fails with exit code 4.
8. Restoring an unknown ID fails with exit code 3.
9. Restoring a superseded mockup fails with exit code 4 (status transitions blocked, section 7).
10. Restoration preserves the mockup's ID and updates its status field in the HTML (REQ-05.008). The restored status becomes `pending` (OQ-6).
11. `--json` returns `{"status": "ok", "id": "<ID>"}` for both commands.

### 7. Supersession

1. A mockup may supersede an older mockup. The superseded mockup becomes deprecated and records a back-reference to its successor (REQ-05.008).
2. `papyrus ui-mockup edit <id> --superseded-by <successor-id>` marks `<id>` as superseded by `<successor-id>`. It sets `<id>`'s status to `deprecated`, moves `<id>` to the `deprecated/` folder, and writes the back-reference `Superseded by: <successor-id>` into its HTML (both the HTML comment and a visible line).
3. The successor must exist (active or deprecated); an unknown successor exits 3.
4. A mockup cannot supersede itself; self-reference exits 4.
5. If `<id>` is already superseded, overwriting with a new `--superseded-by` value is allowed (consistent with TASK-13).
6. Status transitions are blocked for a mockup that has been superseded: `--status`, `deprecate`, and `restore` on a superseded mockup exit 4. The back-reference is never removed by these commands.
7. `--superseded-by` cannot be combined with `--status` (exit code 2).

### 8. Status transitions

The full status-transition table is not specified by REQ-05.008 (OQ-5). The following minimal table is adopted:

| From | To | Allowed via |
|---|---|---|
| `pending` | `approved` | `edit --status approved` |
| `pending` | `deprecated` | `deprecate` or `edit --status deprecated` |
| `approved` | `deprecated` | `deprecate` or `edit --status deprecated` |
| `deprecated` | `pending` | `restore` (status updated to `pending`) |

Any other transition is a validation error (exit code 4). Setting the current status again is a no-op success. A superseded mockup (section 7) permits no transitions.

### 9. Persistence and sanitization

1. Records are UTF-8 HTML files named `<ID>.html` in the active directory or its `deprecated/` subfolder (NFR-04.001). No binary data is written.
2. Created files use this minimal template (field presence is what tests verify; exact byte formatting is not contractually fixed beyond the single trailing newline):

```html
<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>MOCK-001 — <title></title>
</head>
<body>
<!-- mockup-id: MOCK-001 -->
<!-- status: pending -->
<!-- task: TASK-41 -->
<!-- specs: UI-001 -->
<!-- refs:  -->
<h1>MOCK-001 <title></h1>
<p>Status: pending</p>
<p>Task: TASK-41</p>
<p>UI specifications: UI-001</p>
<p>References: </p>
<p>Superseded by: </p>
</body>
</html>
```

   When a field has no value, its line is still emitted with an empty value. When supersession applies, `Superseded by: <id>` is populated and the file is moved to `deprecated/`.
3. `title`, `task`, `--spec`, and `--ref` values are stored in the HTML comment lines and in the corresponding visible lines; multi-line bodies are available through `--editor`.
4. All user-controlled input (`<title>`, `--task`, `--spec`, `--ref`, `<id>`, successor id) is normalized/escaped before use: values are trimmed, and empty values, control characters, and path separators are rejected (NFR-02.002). A value that cannot be normalized fails with exit code 4 and writes nothing.
5. `<id>` is resolved by exact, case-sensitive match of the file stem; request values are never concatenated into a path without resolution, and every read/write path is checked against the canonicalized project directory (NFR-04.002).
6. Status updates rewrite the status token in both the HTML comment (`<!-- status: ... -->`) and the visible `Status: ...` line. `--editor` rewrites the whole file without changing the ID or folder.
7. The registry `docs/specifications/ui/mockups/README.md` is not rewritten by these commands; it remains a manually maintained index.

### 10. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (missing title, `--task`, or `--spec`) and unknown options exit 2 (usage).
3. Unknown mockup ID or successor ID exits 3.
4. Disabled type, already-deprecated/already-active state conflict, unnormalizable input, blocked transition, self-reference, or superseded-mockup conflict exits 4 (validation).
5. IO failure, missing editor, or project-directory boundary violation exits 1.

## Inputs and outputs

- `ui-mockup create <title> --task <task> --spec <spec>... [--ref <doc>...]`: required title/task/at least one spec; output is the generated ID, or `{"id": "<ID>"}` in JSON mode.
- `ui-mockup list`: output is ID and title rows of active mockups, or a JSON array of `{id, title}`.
- `ui-mockup view <id>`: output is the full HTML text, or a JSON object with `id`, `path`, and `content`.
- `ui-mockup edit <id> [...]`: output is minimal success text, or `{"status": "ok", "id": "<ID>"}`.
- `ui-mockup deprecate <id>` / `ui-mockup restore <id>`: move the file; minimal success output, or `{"status": "ok", "id": "<ID>"}`.

## Validation and error behavior

- Missing required argument or unknown option → exit code 2 (usage).
- Unknown ID or successor ID → exit code 3 (not found).
- Disabled type, already-deprecated/already-active conflict, unnormalizable input, blocked transition, self-reference, superseded-mockup conflict → exit code 4 (validation).
- IO failure, missing editor, or project-directory boundary violation → exit code 1 (runtime).
- A failed validation writes nothing; a failed `deprecate`/`restore`/supersede leaves the file in its current folder.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001); the single-crate layout is retained (ADR-0004), so a `ui_mockup` module may be added without a new ADR.
- Assumption: IDs are generated by scanning the active and `deprecated/` folders for the highest existing number with the configured prefix; `id_padding` (default 3) applies. This mirrors TASK-30/TASK-34.
- Assumption: record files are HTML (`*.html`) rather than Markdown; `list`/`view`/`deprecate`/`restore` do not recurse, so sibling files (`README.md`, `wireframe.css`) and the `deprecated/` subfolder are never treated as records at the top level.
- Assumption: `init` creates the `specifications/ui/mockups/` directory and its `deprecated/` subfolder; this ticket ensures both exist on `create` and on `deprecate` if absent.
- Assumption: `title`, `task`, `--spec`, and `--ref` are single-line sanitized inputs; multi-line bodies are available through `--editor`.
- Assumption: deprecating/restoring an already-deprecated/active mockup is a validation error (exit code 4), matching TASK-30/TASK-34.
- Assumption: `list` order is by ID ascending.
- Assumption: editing a deprecated mockup is allowed, including field and status changes permitted by section 8.
- Assumption: the mockup is not versioned by Papyrus beyond git; no version field is maintained.

## Open questions

- OQ-1 (product/CLI): no command name or config key is documented for mockups in README.md or any BR. `ui-mockup` / `ui-mockup` (key) with prefix `MOCK` and directory `specifications/ui/mockups/` are proposed. A different name changes the command and key only.
- OQ-2 (product): no dedicated business rule exists for this command. BR-016 covers configuration. Whether to author a new rule (proposed BR-013) is open; until then this specification is the command contract.
- OQ-3 (product): the status field location and machine-readable format in the HTML are unspecified by REQ-05.008. The proposed HTML-comment plus visible-line representation (section 9.2) is an implementation choice; tests should assert presence rather than exact markup.
- OQ-4 (product): the supersede mechanism is unspecified (command vs. flag vs. both). `edit --superseded-by <id>` is proposed, consistent with TASK-13.
- OQ-5 (product): the full status-transition table beyond the supersede constraint is unspecified. The minimal table in section 8 is proposed.
- OQ-6 (product): the status assigned on restore is unspecified. `pending` is proposed.
- OQ-7 (product): whether `--task` is a task ID or free text, and whether `--ref` values are document ids or paths, is unspecified. Task IDs and document ids are proposed.
- OQ-8 (product): whether `create` requires at least one `--spec` or allows zero is unspecified. At least one is proposed (REQ-05.008 calls the spec reference required).
- OQ-9 (non-blocking, for TASK-40/TASK-41): whether the browser mockup for mockup management should be produced is tracked separately; no UI mockup is created by this ticket.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.007, REQ-05.008 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | none dedicated (BR-016 for configuration) |
| Domain | `docs/specifications/ui/mockups/README.md` (registry; status values pending/approved/deprecated) |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002), ADR-0004 (single-crate layout) |
| Ticket | TASK-38 |
| UI mockups | none |
| Related specification | `docs/specifications/ui-specifications-management.md` (TASK-34 pattern), `docs/specifications/glossary-terms-management.md` (TASK-30 pattern), `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |