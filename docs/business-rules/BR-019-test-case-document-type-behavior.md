# BR-019: Test-Case Document Type Behavior

## Description

A test case is a managed document type whose records capture test scenarios, expected outcomes, and verification steps in a required title and description plus an optional applicability, and can be created, listed, viewed, edited, deprecated, and restored with the ID preserved; records follow the Documents lifecycle `active -> deprecated`, and deprecation hides them from lists, search, and the web UI.

## Rationale

Based on REQ-05.013 — Manage Test Cases, with REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle and deprecation visibility), REQ-05.010 — Record Metadata in YAML Front Matter (front matter and body shape), and REQ-05.005 — Manage Business Rules (business rules may reference test cases), this rule exists because test scenarios are only traceable when they are first-class stored records with a stable ID and the same create, list, view, edit, deprecate, and restore contract as the other documentation types; without that contract test cases cannot be found, hidden, or restored consistently across the CLI, search, and the web UI. README.md is the sole behavior source, so a `test-case` manpage entry is still to be added; until it exists, REQ-05.013 states the behavior and behavior beyond it is unspecified.

## Applicability

Applies to every test-case record and to the command invocations that create, list, view, edit, deprecate, or restore one, in projects where the test-case document type is enabled.

### Behavior (source: REQ-05.013 acceptance criteria)

- Create requires a title and a description; applicability is optional.
- List and view list and display test cases.
- Edit updates title and description.
- Deprecate moves the record to the type's `deprecated/` folder and hides it from lists, search, and the web UI.
- Restore returns a deprecated record to its original folder with the same ID.
- Lifecycle is `active -> deprecated` (REQ-05.007).

### IDs, directories, and configuration — not decided

- The document-type key is `test-case`; its `document_types` entry (prefix, directory, enabled) is not yet defined. Proposed, not decided: prefix `TC`, directory `test-cases/`, `enabled: true`.
- Records are stored under `<doc_directory>/<document_types.test-case.directory>/`, deprecated records under that type directory's `deprecated/` subfolder. IDs are padded to the configured `id_padding` and remain stable across deprecate/restore cycles (REQ-01.005, BR-016).
- Records carry YAML front matter with the general fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references` and a `# <ID> <title>` body heading generated from front-matter `id` and `title` (REQ-05.010). Whether `description` and `applicability` are front-matter fields or body content is not defined at type level by REQ-05.010 and remains a spec decision.

### Conflict: managing command

- REQ-05.013 states that test cases "are managed with the `req` command". BR-009 defines `req` as business requirements only (REQ-05.001: `--group`, `--story`, requirement-to-group cardinality 1..1, groups contain business requirements only), so routing test cases through `req` conflicts with the existing command contract and would place two record kinds in one document type and ID namespace.
- Recommendation (not decided): manage test cases with a dedicated `test-case` command, following the per-type command pattern of `tech-spec` (BR-017), and correct REQ-05.013 together with the README manpage entry.

### Open product decisions

- Managing command: `req` as written, or a dedicated `test-case` command per the per-type pattern. Recommendation: dedicated `test-case` command.
- ID prefix: proposal `TC`. The REQ-05.013 open-question mention of "prefix MOCK" is carried over from the UI-mockup open question and does not apply to test cases.
- Directory: proposal `test-cases/`.
- `enabled` default: proposal `true`, matching every current `document_types` default (BR-016).
- Visibility: proposal that active test cases appear in lists, search, and the web UI and deprecated ones are hidden, matching the other document types (REQ-05.007).
- Association to tickets or milestones: not stated. Proposal: no association, because the reference targets accepted by tasks are fixed by REQ-02.001 and adding one requires a requirement change.
- Relationship to test reports: whether a test case is referenced by test-report records is not stated here; see BR-020.
