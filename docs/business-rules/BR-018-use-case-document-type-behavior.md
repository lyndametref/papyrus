# BR-018: Use-Case Document Type Behavior

## Description

A use case is a managed document type whose records carry a required title and description plus an optional applicability and can be created, listed, viewed, edited, deprecated, and restored with the ID preserved; records follow the Documents lifecycle `active -> deprecated`, and deprecation hides them from lists, search, and the web UI.

## Rationale

Based on REQ-05.012 — Manage Use Cases, with REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle and deprecation visibility), REQ-05.010 — Record Metadata in YAML Front Matter (front matter and body shape), and REQ-05.005 — Manage Business Rules (business rules may reference use cases), this rule exists because use cases are only usable for behavioral documentation and traceability when the same create, list, view, edit, deprecate, and restore contract that the other documentation types follow applies to them; without that contract the type cannot be stored, found, hidden, or restored consistently across the CLI, search, and the web UI. README.md is the sole behavior source; the `use-case` manpage entry is present, and behavior beyond it is unspecified.

## Applicability

Applies to every use-case record and to the command invocations that create, list, view, edit, deprecate, or restore one, in projects where the use-case document type is enabled.

### Behavior (source: REQ-05.012 acceptance criteria)

- Create requires a title and a description; applicability is optional.
- List and view list and display use cases.
- Edit updates title and description.
- Deprecate moves the record to the type's `deprecated/` folder and hides it from lists, search, and the web UI.
- Restore returns a deprecated record to its original folder with the same ID.
- Lifecycle is `active -> deprecated` (REQ-05.007).

### IDs, directories, and configuration (decided)

- The document-type key is `use-case`; `document_types.use-case: { prefix: UC, directory: use-cases/, enabled: true }` (decided).
- Records are stored under `<doc_directory>/<document_types.use-case.directory>/`, deprecated records under that type directory's `deprecated/` subfolder. IDs are padded to the configured `id_padding` and remain stable across deprecate/restore cycles (REQ-01.005, BR-016).
- Records carry YAML front matter with the general fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references` and a `# <ID> <title>` body heading generated from front-matter `id` and `title` (REQ-05.010). `description` and `applicability` are body fields, not front-matter keys (REQ-05.010, expose spec TASK-65).

### Managing command (resolved)

- REQ-05.012 states that use cases "are managed with the `req` command"; this wording is superseded by the decided resolution for implementation. BR-009 defines `req` as business requirements only (REQ-05.001: `--group`, `--story`, requirement-to-group cardinality 1..1, groups contain business requirements only), so routing use cases through `req` conflicts with that contract and would place two record kinds in one document type and ID namespace.
- Resolution (decided): manage use cases with a dedicated `use-case` command, following the per-type command pattern of `tech-spec` (BR-017). The `req` command remains business-requirements only (BR-009). Any REQ-05.012 wording correction is left to the requirements owner.

### Decided defaults

- Managing command: dedicated `use-case` command (resolved; see "Managing command (resolved)").
- ID prefix: `UC` (decided). The REQ-05.012 open-question mention of "prefix MOCK" is carried over from the UI-mockup open question and does not apply to use cases.
- Directory: `use-cases/` (decided).
- `enabled` default: `true`, matching every current `document_types` default (BR-016) (decided).
- Visibility: active use cases appear in lists, search, and the web UI; deprecated ones are hidden (REQ-05.007) (decided).
- Association to tickets or milestones: none; reference targets accepted by tasks are fixed by REQ-02.001 and adding one requires a requirement change.
```