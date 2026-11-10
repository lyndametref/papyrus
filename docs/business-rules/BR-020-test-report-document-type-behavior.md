# BR-020: Test-Report Document Type Behavior

## Description

A test report is a managed document type whose records capture test execution results, pass/fail outcomes, test-case coverage, and verification evidence in a required title and description plus an optional applicability, and can be created, listed, viewed, edited, deprecated, and restored with the ID preserved; records follow the Documents lifecycle `active -> deprecated`, and deprecation hides them from lists, search, and the web UI.

## Rationale

Based on REQ-05.014 — Manage Test Reports, with REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle and deprecation visibility), REQ-05.010 — Record Metadata in YAML Front Matter (front matter and body shape), and REQ-05.005 — Manage Business Rules (business rules may reference test reports), this rule exists because execution evidence is only auditable when results are stored as records with a stable ID under the same create, list, view, edit, deprecate, and restore contract as the other documentation types; without that contract test evidence cannot be found, hidden, or restored consistently across the CLI, search, and the web UI. README.md is the sole behavior source, so a `test-report` manpage entry is still to be added; until it exists, REQ-05.014 states the behavior and behavior beyond it is unspecified.

## Applicability

Applies to every test-report record and to the command invocations that create, list, view, edit, deprecate, or restore one, in projects where the test-report document type is enabled.

### Behavior (source: REQ-05.014 acceptance criteria)

- Create requires a title and a description; applicability is optional.
- List and view list and display test reports.
- Edit updates title and description.
- Deprecate moves the record to the type's `deprecated/` folder and hides it from lists, search, and the web UI.
- Restore returns a deprecated record to its original folder with the same ID.
- Lifecycle is `active -> deprecated` (REQ-05.007).

### IDs, directories, and configuration — not decided

- The document-type key is `test-report`; its `document_types` entry (prefix, directory, enabled) is not yet defined. Proposed, not decided: prefix `TR`, directory `test-reports/`, `enabled: true`.
- Records are stored under `<doc_directory>/<document_types.test-report.directory>/`, deprecated records under that type directory's `deprecated/` subfolder. IDs are padded to the configured `id_padding` and remain stable across deprecate/restore cycles (REQ-01.005, BR-016).
- Records carry YAML front matter with the general fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references` and a `# <ID> <title>` body heading generated from front-matter `id` and `title` (REQ-05.010). Whether `description` and `applicability` are front-matter fields or body content is not defined at type level by REQ-05.010 and remains a spec decision.

### Conflict: managing command

- REQ-05.014 states that test reports "are managed with the `req` command". BR-009 defines `req` as business requirements only (REQ-05.001: `--group`, `--story`, requirement-to-group cardinality 1..1, groups contain business requirements only), so routing test reports through `req` conflicts with the existing command contract and would place two record kinds in one document type and ID namespace.
- Recommendation (not decided): manage test reports with a dedicated `test-report` command, following the per-type command pattern of `tech-spec` (BR-017), and correct REQ-05.014 together with the README manpage entry.

### Open product decisions

- Managing command: `req` as written, or a dedicated `test-report` command per the per-type pattern. Recommendation: dedicated `test-report` command.
- ID prefix: proposal `TR`. The REQ-05.014 open-question mention of "prefix MOCK" is carried over from the UI-mockup open question and does not apply to test reports.
- Directory: proposal `test-reports/`.
- `enabled` default: proposal `true`, matching every current `document_types` default (BR-016).
- Visibility: proposal that active test reports appear in lists, search, and the web UI and deprecated ones are hidden, matching the other document types (REQ-05.007).
- Association to tickets or milestones: not stated. Proposal: no association, because the reference targets accepted by tasks are fixed by REQ-02.001 and adding one requires a requirement change.
- Test-case coverage: REQ-05.014 says a report captures "test case coverage" but defines no field for it. Proposal: record test-case references in the general front-matter `references` field (REQ-05.010) rather than adding a type-specific field; not decided.
- Pass/fail outcome storage: REQ-05.014 says outcomes are captured but defines neither a field nor allowed values. Undecided; no values are defined by this rule.
