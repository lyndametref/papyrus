# REQ-05.011 Maintain Record Metadata Through All Commands

## User Story
As a project contributor, I want Papyrus commands to set and maintain record metadata automatically so that front matter stays accurate without manual editing.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
`created_date` is set automatically when a record is created and is never changed afterwards. `updated_date` is set automatically at creation and refreshed by every write path, including a browser save. `id` is generated and stable (REQ-01.005), and `id`, `created_date`, and `updated_date` are exempt from the option rule; every other front-matter field can be supplied as a command option at creation and at update, and a command that lacks an option for an applicable field gains one. `ordinal` is an opaque non-negative integer supplied through a create/update option and defaulted from the current maximum. Every command that writes a record file keeps the existing front matter intact and changes only the fields that the operation changes; a browser save follows the same contract.

Existing records are migrated lazily, per record: each Markdown record file claimed by a Papyrus managed record type receives front matter on its first read and on its first write, and no bulk migration command is required. The managed record types are task, milestone, release, decision, business requirement, non-functional requirement, glossary term, UI specification, technical specification, business rule, and audit record. Prose Markdown that no managed record type claims, such as files under `docs/specifications`, `docs/domain-model`, and `docs/business-processes` or `README` files, is out of scope and is not migrated. UI mockups and other HTML files are excluded from the migration. Frozen ADR bodies are not edited beyond the added front-matter block.

The byte-identical rule applies to task and milestone files that are already in the Backlog.md storage shape: their bodies, including marker comments, are preserved byte-identically apart from the front-matter block, on both first read and first write. Task and milestone files still in the legacy Papyrus shape (`# <ID> <title>` heading, `## Status` and other retired metadata sections, no marker comments) cannot keep a byte-identical body and are converted to the Backlog.md shape at the time of their first read or first write: the heading is dropped, the retired metadata sections are removed after their values move to front matter, and `## Description`/`## Acceptance Criteria` are mapped into the Backlog.md marker sections. Every unknown section and free-text block is preserved verbatim and in file order, and each converted file is reported with a before/after diff. Migration is idempotent: once a record carries front matter, neither its first read nor its first write re-runs the conversion. Timestamps are quoted ISO-8601 UTC strings, for example `'2026-10-31T22:26:00Z'`, the same format Backlog.md writes.

## Acceptance Criteria
- `created_date` and `updated_date` are set automatically when a record is created.
- `updated_date` is refreshed automatically on every write, including a browser save of an edit; `created_date` is never changed.
- `id`, `created_date`, and `updated_date` are exempt from the option rule; for every record type, each other front-matter field can be supplied as an option on creation and as an option on update.
- `ordinal` is an opaque non-negative integer with a create option and an update option; a new record's default is derived from the current maximum.
- Every command that writes a record (create, edit, archive/restore, deprecate/restore, supersede, delete) preserves the front-matter fields that the operation does not change.
- A browser save preserves the front-matter fields and body content the save does not change.
- An existing command that has no option for an applicable front-matter field provides that option.
- Each managed Markdown record file is migrated lazily: front matter is added on its first read and on its first write; after either, the record carries front matter.
- Migration is idempotent: a record that already carries front matter is not re-migrated on subsequent reads or writes.
- No bulk migration command is required; the corpus converges record by record as records are read or written.
- Legacy Papyrus-shape task and milestone files are converted to the Backlog.md shape on their first read or first write, with unknown sections and free text preserved verbatim and a before/after diff reported per converted file.
- The migration covers only existing files claimed by a managed record type; prose Markdown that is not a managed record (for example under `docs/specifications`, `docs/domain-model`, `docs/business-processes`, or `README` files) is not modified.
- The migration excludes HTML files, including UI mockups.
- The migration changes nothing in a frozen ADR body except adding the front-matter block.
- For task and milestone files already in the Backlog.md storage shape, the migration preserves bodies and marker comments byte-identically apart from the front-matter block.
- Task and milestone files still in the legacy Papyrus shape are converted to the Backlog.md shape on their first read or first write; every unknown section and free-text block is preserved verbatim, and each converted file is reported with a before/after diff.
- `created_date` and `updated_date` are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote and clarifications (verbatim in REQ-05.010).
- User decision 2026-11-08: browser saves refresh `updated_date`; `id`/`created_date`/`updated_date` are exempt from the option rule; `ordinal` is an opaque non-negative integer defaulted from the current maximum; timestamps are quoted ISO-8601 UTC.
- User decision 2026-11-08 (migration, revised): migration is lazy and per-record — each managed Markdown record gains front matter on its first read and on its first write; no bulk migration command is required. Excluding UI mockups/HTML; frozen ADR bodies gain front matter only; Backlog.md bodies and marker comments stay byte-identical apart from the front-matter block.
- User decision 2026-11-08 (migration scope and legacy shape, revised): the byte-identical rule applies only to files already in Backlog.md shape; legacy Papyrus-shape task/milestone files are converted on first read or first write with unknown sections and free text preserved verbatim and a before/after diff per converted file; migration covers only files claimed by a managed record type, and non-record prose Markdown is out of scope; migration is idempotent.
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source (timestamp format, patch contract, option exemption, lazy migration decision).
- ADR-0009 — Shared Record Envelope and Unified Write Path for CLI and Browser (one write path for CLI and browser).
- REQ-01.002 — Read and Edit Project Configuration (configurable statuses)
- REQ-01.005 — Assign Stable Record IDs
- REQ-02.001 — Create and Describe Tasks
- REQ-02.002 — View and Edit Tasks
- REQ-04.002 — Update, Supersede, and Archive Decisions
- REQ-05.010 — Record Metadata in YAML Front Matter
- NFR-04.003 — Keep Task and Milestone Files Interoperable with Backlog.md
