---
id: REQ-05.010
title: "Record Metadata in YAML Front Matter"
created_date: '2026-11-13T20:58:56Z'
updated_date: '2026-11-15T15:47:28Z'
labels: []
references: []
---
# REQ-05.010 Record Metadata in YAML Front Matter

## User Story
As a project contributor, I want every managed file to begin with structured record metadata so that records can be identified, related, filtered, and displayed consistently without reading the document body.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
Every Markdown file managed by Papyrus begins with a YAML front-matter block. The block carries the general fields `id`, `title`, `created_date`, `updated_date`, `labels`, and `references` for every managed record type, followed by the type-specific fields. Task front matter carries `id`, `title`, `status`, `assignee`, `created_date`, `updated_date`, `labels`, `milestone`, `dependencies`, `references`, `documentation`, `type`, and `ordinal`; `assignee` is a YAML list. Milestone front matter carries the general fields plus `release` and `status` (`open` or `closed`). Release front matter carries the general fields plus `status` (`planned` or `released`). Front-matter-backed document types carry `status` with the value `active`, `archived`, or a custom status declared for that document type (REQ-07.001, REQ-07.002); UI mockups keep that state in their HTML content and carry no front matter (REQ-05.008). A decision's front matter may carry `superseded_by`, and front matter is additive for decisions: the existing `## Date` and `## References` sections remain part of the body, `created_date` is the front-matter timestamp and does not replace or duplicate the ADR `## Date` line, and neither section becomes an editable metadata source. Decision `status` stays in the document core and decision `type` (`adr`/`bdr`) stays derived from the record ID prefix; neither moves to front matter. Audit records carry `type`.

The front-matter `id` and every reference value (`milestone`, `dependencies`, `references`, `superseded_by`) use the Papyrus ID (REQ-01.005). There is no separate storage ID and no translation at a storage boundary (NFR-04.004).

Scope is Markdown records. UI mockups are excluded: their status, associated task, and references remain stored in the mockup's HTML content (REQ-05.008), which is the single source for mockup metadata, and mockup HTML files carry no front matter. Backlog.md `docs` and `drafts` directories are not managed record types and are out of scope.

Front matter is authoritative for the fields it carries and replaces the equivalent information in the document core for records Papyrus writes, so no retired metadata section stores a field that front matter carries; frozen decision bodies are the exception described above. Key order and YAML quoting follow the documented serialization, with general keys preceding type-specific keys. Task and milestone records have no `# <ID> <title>` body title line; their bodies use the record's section layout. Every other Markdown record type keeps a `# <ID> <title>` body title line that the writer generates from the front-matter `id` and `title`, so the two cannot diverge; frozen ADR bodies keep their existing heading and gain front matter only. Timestamps are quoted ISO-8601 UTC strings, for example `'2026-10-31T22:26:00Z'`. Adding front matter to an existing record does not change its body. Records brought in by the one-way Backlog.md import script (NFR-04.004) are written in the Papyrus record format, with this front-matter block, on import.

## Acceptance Criteria
- Every managed Markdown record file created by Papyrus begins with a YAML front-matter block at byte 0, delimited by `---` lines.
- Every managed Markdown record type carries `id`, `title`, `created_date`, `updated_date`, `labels`, and `references` in its front matter.
- Task front matter carries `id`, `title`, `status`, `assignee`, `created_date`, `updated_date`, `labels`, `milestone`, `dependencies`, `references`, `documentation`, `type`, and `ordinal`; `assignee` is a YAML list.
- Milestone front matter carries the general fields plus `release` and `status`; `status` is `open` or `closed`.
- Release front matter carries the general fields plus `status`; `status` is `planned` or `released`.
- Front-matter-backed document types carry `status`; its value is `active`, `archived`, or a custom status declared for that type, and Papyrus writes `archived` only when it moves the record into the type's `archived/` subfolder.
- A decision's front matter may carry `superseded_by`; adding front matter to a decision leaves `## Date` and `## References` intact and does not move decision `status` or decision `type` into front matter.
- An audit record's front matter carries `type`.
- The front-matter `id` and every reference value carry the Papyrus ID; no second, tool-specific ID appears in front matter.
- Tasks and milestones have no `# <ID> <title>` body title line; every other Markdown record type keeps a body title line generated by the writer from the front-matter `id` and `title`.
- A frozen ADR body keeps its existing heading and sections; the added front-matter block is the only change made to an existing decision file.
- UI mockup HTML files carry no YAML front matter; mockup metadata stays in the HTML content (REQ-05.008).
- Backlog.md `docs` and `drafts` files are not managed by Papyrus front matter.
- `created_date` and `updated_date` are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.
- No front-matter field is also stored in a retired metadata section of the document core.
- Adding front matter to an existing record leaves the record body unchanged.
- Records imported by the one-way Backlog.md import script carry this front-matter block in the Papyrus record format.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote:
  - "Add at the begining of each file a front matter with at least id, title, created_date, updated_date, labels, references and for tasks dependencies, assignee, ordinal, milestone and for milestones release. for decision a superseded_by is optional. It replaces the information in the core of the document. all command lines must insure to maintain those information. created_at and updated_at are automatically set by the command line tool. the other information must be passable as option at creation and update (check that there is an option for those). update the browser view and the cli view to use those inftormation."
  - Clarification: "for audit records a type is also part of the fronf matter"
  - Field map: all files `id`, `title`, `created_date`, `updated_date`, `labels`, `references`; tasks add `dependencies`, `assignee`, `ordinal`, `milestone`; milestones add `release`; decisions add optional `superseded_by`; audit records add `type`.
- User decision 2026-11-08 (front-matter scope): UI mockups are excluded and REQ-05.008 HTML metadata remains their single source.
- User decision 2026-11-08 (heading policy, decision mapping, release key, decision status/type): tasks and milestones have no `# <ID> <title>` body title line, other document records keep a writer-generated one, frozen ADR bodies gain front matter only; decision front matter is additive and `## Date`/`## References` stay intact; releases carry the general keys only; decision `status`/`type` are unchanged by this requirement.
- Maintainer decision 2026-11-11: the front-matter `id` and reference values use the Papyrus ID only (single ID space); the Backlog.md key-set, storage-ID, and serialization clauses are removed per NFR-04.004.
- ADR-0007 — YAML Front Matter as the Single Record Metadata Source (field map, canonical key order, heading policy, scope exclusions).
- Project-owner decision 2026-11-13: milestone and release records store `status` in front matter (`open`/`closed`, `planned`/`released`), and front-matter-backed documents store `status` (`active`/`archived`); UI mockups remain HTML-only (REQ-05.008). This supersedes the earlier "release records carry general fields only" clause.
- Project-owner decision 2026-11-13 (D5): a front-matter-backed document's `status` may also hold a custom status declared for its type (`document_types.<slug>.statuses`), while `archived` remains reserved for records stored in the `archived/` subfolder (REQ-07.001).
- REQ-01.005 — Assign Stable Record IDs
- REQ-03.002 — Manage Releases
- REQ-05.007 — Manage Documentation Record Lifecycle
- REQ-05.008 — Manage UI Mockups (mockup metadata remains in HTML)
- NFR-04.001 — Persist Application Data Only in Text Files
- NFR-04.004 — Renounce Backlog.md Compatibility and Provide Import
- docs/specifications/browser-*.md — browser persistence contract already refers to preserving "front matter"
