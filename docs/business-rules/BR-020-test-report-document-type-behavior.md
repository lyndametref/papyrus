# BR-020: Test-Report Document Type Behavior

## Description

A test report is a managed document type whose records capture test execution results, pass/fail outcomes, test-case coverage, and verification evidence in a required title and description plus an optional applicability, and can be created, listed, viewed, edited, deprecated, and restored with the ID preserved; records follow the Documents lifecycle `active -> deprecated`, and deprecation hides them from lists, search, and the web UI.

## Rationale

Based on REQ-05.014 — Manage Test Reports, with REQ-05.007 — Manage Documentation Record Lifecycle (lifecycle and deprecation visibility), REQ-05.010 — Record Metadata in YAML Front Matter (front matter and body shape), and REQ-05.005 — Manage Business Rules (business rules may reference test reports), this rule exists because execution evidence is only auditable when results are stored as records with a stable ID under the same create, list, view, edit, deprecate, and restore contract as the other documentation types; without that contract test evidence cannot be found, hidden, or restored consistently across the CLI, search, and the web UI. README.md is the sole behavior source; the `test-report` manpage entry is present, and behavior beyond it is unspecified.

## Applicability

Applies to every test-report record and to the command invocations that create, list, view, edit, deprecate, or restore one, in projects where the test-report document type is enabled.

### Behavior (source: REQ-05.014 acceptance criteria)

- Create requires a title and a description; applicability is optional.
- List and view list and display test reports.
- Edit updates title and description.
- Deprecate moves the record to the type's `deprecated/` folder and hides it from lists, search, and the web UI.
- Restore returns a deprecated record to its original folder with the same ID.
- Lifecycle is `active -> deprecated` (REQ-05.007).

### IDs, directories, and configuration (decided)

- The document-type key is `test-report`; `document_types.test-report: { prefix: TR, directory: test-reports/, enabled: true }` (decided).
- Records are stored under `<doc_directory>/<document_types.test-report.directory>/`, deprecated records under that type directory's `deprecated/` subfolder. IDs are padded to the configured `id_padding` and remain stable across deprecate/restore cycles (REQ-01.005, BR-016).
- Records carry YAML front matter with the general fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references` and a `# <ID> <title>` body heading generated from front-matter `id` and `title` (REQ-05.010). `description` and `applicability` are body fields, not front-matter keys (REQ-05.010, expose spec TASK-65).

### Managing command (resolved)

- REQ-05.014 states that test reports "are managed with the `req` command"; this wording is superseded by the decided resolution for implementation. BR-009 defines `req` as business requirements only (REQ-05.001: `--group`, `--story`, requirement-to-group cardinality 1..1, groups contain business requirements only), so routing test reports through `req` conflicts with that contract and would place two record kinds in one document type and ID namespace.
- Resolution (decided): manage test reports with a dedicated `test-report` command, following the per-type command pattern of `tech-spec` (BR-017). The `req` command remains business-requirements only (BR-009). Any REQ-05.014 wording correction is left to the requirements owner.

### Decided defaults

- Managing command: dedicated `test-report` command (resolved; see "Managing command (resolved)").
- ID prefix: `TR` (decided). The REQ-05.014 open-question mention of "prefix MOCK" is carried over from the UI-mockup open question and does not apply to test reports.
- Directory: `test-reports/` (decided).
- `enabled` default: `true`, matching every current `document_types` default (BR-016) (decided).
- Visibility: active test reports appear in lists, search, and the web UI; deprecated ones are hidden (REQ-05.007) (decided).
- Association to tickets or milestones: none; reference targets accepted by tasks are fixed by REQ-02.001 and adding one requires a requirement change.
- Test-case coverage: REQ-05.014 says a report captures "test case coverage" but defines no dedicated field; test-case references are recorded in the general front-matter `references` field (REQ-05.010) rather than a type-specific field (not defined by this rule).
- Pass/fail outcome storage: REQ-05.014 says outcomes are captured but defines neither a field nor allowed values; not defined by this rule, no values are defined here.
```