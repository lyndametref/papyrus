# Feature Specification — Glossary Terms Management from the CLI

**Ticket:** TASK-30
**Milestone:** m-4 (CLI)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver the `glossary` commands so a project contributor can create, list, view, edit, deprecate, and restore glossary terms, with stable IDs and text-file persistence inside the project directory. This ticket implements REQ-05.003 and REQ-05.007, together with the documented command contract in BR-011. It provides the glossary terms that later tickets search (TASK-31) and expose in the browser (TASK-23).

## Scope

### In scope

- `papyrus glossary create --term <term> --definition <text>` (BR-011, REQ-05.003).
- `papyrus glossary list` (BR-011, REQ-05.003).
- `papyrus glossary view <id>` (BR-011, REQ-05.003).
- `papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor]` (BR-011, REQ-05.003).
- `papyrus glossary deprecate <id>` (BR-011, REQ-05.007).
- `papyrus glossary restore <id>` (BR-011, REQ-05.007).
- Glossary identity: stable IDs of the form `<prefix>-<number>` (for example `GLO-001`), using the configured `document_types.glossary` prefix and `id_padding` (REQ-05.003, BR-011, BR-016).
- Glossary fields: term (title) and definition (REQ-05.003, BR-011).
- Deprecation moving a term to its type's `deprecated/` subfolder and hiding it from lists, search, and the web UI (REQ-05.003, REQ-05.007, BR-011).
- Restoration returning a deprecated term to its original folder with the same ID (REQ-05.003, REQ-05.007, BR-011).
- Persistence as UTF-8 text only inside the project directory (NFR-04.001, NFR-04.002).
- Input sanitization by normalization/escaping before use (NFR-02.002).
- Common CLI controls and outcomes inherited from TASK-2: `--json`, `--no-color`/`NO_COLOR`, help/version, exit statuses 0/1/2/3/4 (REQ-01.004).

### Out of scope

- Search and browser access for glossary terms — TASK-31 and TASK-23.
- Packaging and verification of the glossary milestone — TASK-33.
- Deleting a glossary term — BR-011 defines no `delete` command.
- UI mockups or web UI — none for this ticket; CLI output only.
- New configuration keys beyond those already delivered by TASK-2 (`document_types.glossary`).

## Behavior

### 1. Project and configuration context

1. Project directory and configuration file selection follow the precedence and canonicalization rules delivered by TASK-2 (REQ-01.003, NFR-04.002).
2. For every command in this ticket, reads and writes stay within the canonicalized project directory (NFR-04.002).
3. The effective configuration supplies `document_types.glossary.prefix` (default `GLO`), `document_types.glossary.directory` (default `glossary/`), `document_types.glossary.enabled`, plus `doc_directory` and `id_padding`.
4. A disabled glossary type is rejected by `create` (exit code 4) and its terms are hidden from `list` (BR-016).

### 2. Create

1. `papyrus glossary create --term <term> --definition <text>` creates a glossary term.
2. Both `--term` and `--definition` are required; missing either is a usage error (exit code 2).
3. The `--term` value is required and is sanitized before use; it becomes the term text and the top-level heading title.
4. The `--definition` value is required and is sanitized before use.
5. The glossary ID is generated as `<prefix>-<number>`, padded to `id_padding` digits (for example `GLO-001`), where `<number>` is one greater than the highest existing number found in both the active and `deprecated/` folders for that prefix. IDs are never reused.
6. `create` prints the generated glossary ID (and, in `--json` mode, returns it as JSON).
7. If any user-controlled input cannot be sanitized, creation fails with exit code 4.

### 3. List

1. `papyrus glossary list` lists active glossary terms (not deprecated).
2. List output includes at least the ID and term of each term in a stable order (by ID ascending).
3. Deprecated terms are hidden from `list`.
4. `--json` wraps the output as a JSON array.

### 4. View

1. `papyrus glossary view <id>` prints the full persisted text of the glossary term.
2. Viewing a deprecated term by ID is allowed and prints its persisted text.
3. Viewing an unknown ID fails with exit code 3.
4. `--json` wraps the output as a JSON object containing at least ID, path, and content.

### 5. Edit

1. `papyrus glossary edit <id> [--term <term>] [--definition <text>] [--editor]` updates the supplied fields.
2. Only fields explicitly supplied on the command line are changed; omitted fields retain their previous values.
3. `--term` updates the term text (and the top-level heading).
4. `--definition` updates the definition text.
5. `--editor` opens the persisted file in `$VISUAL` or `$EDITOR` (per effective configuration) for editing.
6. Editing a deprecated term is allowed and keeps it deprecated.
7. Editing an unknown ID fails with exit code 3.
8. If any supplied value cannot be sanitized, edit fails with exit code 4.

### 6. Deprecation and restoration

1. `papyrus glossary deprecate <id>` moves the glossary term file from the active folder to the `deprecated/` folder.
2. Deprecating an already-deprecated term fails with exit code 4.
3. Deprecating an unknown ID fails with exit code 3.
4. Deprecated terms are hidden from `list`, search, and the web UI (REQ-05.003, REQ-05.007).
5. `papyrus glossary restore <id>` moves a deprecated term back to its original folder.
6. Restoring an already-active term fails with exit code 4.
7. Restoring an unknown ID fails with exit code 3.
8. Restoration preserves the term's ID.

### 7. Persistence and sanitization

1. Glossary terms are stored as UTF-8 Markdown text files named `<ID>.md` in the configured glossary directory (active) or its `deprecated/` subfolder (deprecated). The `deprecated/` subfolder is created on first deprecation if absent.
2. File content follows this template: a top-level heading `# <ID> <term>`, then a `## Definition` section containing the definition text. Empty optional fields are omitted; `term` and `definition` are required.
3. All user-controlled input (term, definition, ID) is sanitized by normalization/escaping before use (NFR-02.002).
4. Values that cannot be normalized into a valid form fail with exit code 4 (NFR-02.002).
5. No reads or writes occur outside the canonicalized project directory (NFR-04.002).
6. No binary data is written (NFR-04.001).

### 8. Common CLI controls and outcomes

1. `--version`, `-h`/`--help`, `--json`, `--no-color`/`NO_COLOR`, and exit statuses 0/1/2/3/4 behave as delivered by TASK-2 (REQ-01.004).
2. Missing required arguments (for example missing `--term` or `--definition`) exits 2 (usage).
3. Unknown glossary ID exits 3.
4. Filesystem/IO failure or project-directory boundary violation exits 1.

## Inputs and outputs

- `glossary create`: inputs `--term <term>` and `--definition <text>` (both required); output is the generated glossary ID (text) or JSON.
- `glossary list`: output is IDs and terms of active glossary terms (text table or JSON array).
- `glossary view <id>`: output is the full file text (text) or JSON object containing ID, path, and content.
- `glossary edit <id>`: inputs optional `--term`, `--definition`, and `--editor`; output minimal success text or JSON.
- `glossary deprecate <id>`: moves file; minimal success output.
- `glossary restore <id>`: moves file back; minimal success output.

## Validation and error behavior

- Missing required arguments (`--term` or `--definition`) → exit code 2 (usage).
- Unknown ID → exit code 3 (not found).
- Validation/conflict (disabled type, already-deprecated/active state conflict, unnormalizable input) → exit code 4.
- IO failure or boundary violation → exit code 1.
- `--json` error output includes the exit-relevant error information, as delivered by TASK-2.

## Assumptions

- Assumption: Rust 1.99.1 via rustup and Cargo are available (ADR-0001).
- Assumption: single-crate layout is retained; a `glossary` module may be added within the crate without a new crate or ADR.
- Assumption: IDs are generated by scanning active and `deprecated/` folders for the highest existing number for the configured prefix; `id_padding` (default 3) applies.
- Assumption: `init` already creates the glossary directory and its `deprecated/` subfolder; this ticket ensures the `deprecated/` subfolder exists on first deprecation if absent.
- Assumption: term text is stored as the top-level heading title and the `## Definition` section holds the definition; tests verify field presence rather than byte-for-byte formatting.
- Assumption: `--term` and `--definition` values are single-line sanitized inputs; multi-line editing is available through `--editor`.
- Assumption: deprecating or restoring an already-deprecated/active term is a validation error (exit code 4).
- Assumption: list output order is by ID ascending.

## Open questions

- OQ-1: The exact Markdown section order and omission of empty fields follow the template listed in section 7.2; tests verify field presence rather than byte-for-byte formatting.
- OQ-2: Whether a deprecated term's `--term`/`--definition` may be edited is unspecified; this specification allows edits to deprecated terms, including field changes, consistent with the milestone pattern.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-05, REQ-05.003, REQ-05.007 |
| Non-functional | NFR-02.002, NFR-04.001, NFR-04.002 |
| Business rule | BR-011 |
| ADR | ADR-0001 (Rust), ADR-0003 (Debian Forky only; supersedes ADR-0002) |
| Ticket | TASK-30 |
| UI mockups | none |
| Related specification | docs/specifications/project-initialization-and-configuration.md (TASK-2) |
