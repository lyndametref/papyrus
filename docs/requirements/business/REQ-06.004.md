---
id: REQ-06.004
title: "Use Front-Matter Metadata in CLI and Browser Views"
created_date: '2026-11-13T20:56:43Z'
updated_date: '2026-11-20T20:58:16Z'
labels: []
references:
  - RE-010
  - ADR-036
  - ADR-039
---
# REQ-06.004 Use Front-Matter Metadata in CLI and Browser Views

## User Story
As a project contributor, I want the command line and the browser to show record information from the front matter so that both views present the same values and I do not have to look inside the file.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
CLI list and view output, in plain text and in JSON, and the browser list and view pages read record metadata from the record's field map, which is built from the front matter and the descriptor (REQ-05.010, ADR-039). Both surfaces show the same metadata values for the same record.

**Field map.** One field map is built per loaded record from the document model and role: `SingleMarkdown` uses the type's top-level `fields` sequence with role `Single`; `GroupedMarkdown` uses `group.fields` with role `Group` for a group record and `member.fields` with role `Member` for a member record; `SingleHtmlMockup` has no field sequence and carries fixed metadata. The map holds the record envelope (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, body, version, path), the resolved fields in descriptor sequence order, and any undeclared stored front-matter keys as read-only extra metadata.

**Reading by `frontmatter_key`.** Every document front-matter key beyond the general keys is a declared field's `frontmatter_key` for the record's role. A field with only a `frontmatter_key` is metadata-only: CLI and browser read that key and it has no body section. A field with only a `heading` contributes no front-matter key and is read from its body section. A field with both reads the body section as the display value and does not merge the two locations; a divergence already on disk is displayed from the body section, left as stored, and never repaired. `frontmatter_key` may differ from the field `key`; for example the `audit` type stores its `audit_type` field under the front-matter key `type`.

Document front-matter keys follow field sequence order after the general keys. An undeclared key already on disk is preserved and shown read-only, so CLI and browser parity is not lost.

Both surfaces show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references` plus the declared fields of the record type and role. For a document record they show its front-matter `status` value as stored, including a custom status declared for its type (REQ-07.001, REQ-07.002); a configured status value is displayed verbatim and is never renamed or replaced by a display label.

**Edit and save.** The browser edit form renders the fixed title, the declared fields whose verbs allow edit, `labels`, `references`, read-only `created_date`, `updated_date`, and `status`, and the hidden version token. `title` is not a field, and `status` is not editable because it is lifecycle- and configuration-owned (REQ-05.011). A saved value is written to the field's storage locations: the `heading` section and the `frontmatter_key`, whichever the field declares. A browser save preserves the other front-matter fields and the rest of the file and does not copy metadata back into the document core.

A `SingleHtmlMockup` record carries no YAML front matter; its metadata is read from and written to its HTML metadata comment (REQ-05.008), and the same CLI/browser parity applies to those values.

## Acceptance Criteria
- CLI list and view output show field-map metadata values for the viewed or listed record, in plain text and in JSON.
- Browser list and view pages show the same field-map metadata values as the CLI for the same record.
- One field map is built per record from its document model and role, and both surfaces read only that map for document content.
- A `SingleMarkdown` record uses its top-level `fields`; a `GroupedMarkdown` group uses `group.fields` and a member uses `member.fields`; a `SingleHtmlMockup` record has no field sequence and uses its fixed metadata.
- Every document front-matter key beyond `id`, `title`, `status`, `labels`, `references`, `created_date`, and `updated_date` is a declared field's `frontmatter_key` for the record's role.
- A metadata-only field is read from its `frontmatter_key`; a body-only field is read from its `heading`; a field with both is read from its body section without merging the two locations.
- `frontmatter_key` may differ from the field `key`, and the `audit` type reads its `audit_type` field from the front-matter key `type`.
- A divergence between body and front matter already on disk is displayed from the body section, left as stored, and never repaired or merged.
- Undeclared front-matter keys already on disk are preserved and shown read-only.
- Document front-matter keys follow field sequence order after the general keys.
- Both surfaces show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and every declared field of the record type and role.
- Both surfaces show a document record's `status` value as stored, including a custom status declared for its type, and the value is identical in the CLI and browser.
- A configured custom status value is displayed verbatim; the view does not rename or map it.
- The browser edit form renders the fixed title, editable declared fields, `labels`, `references`, read-only `created_date`, `updated_date`, and `status`, and the hidden version token.
- `title` is not a field and is not edited through a field control; `status` is not editable.
- A browser save writes a submitted value to the field's declared `heading` section and/or `frontmatter_key`.
- A browser save preserves the other front-matter fields and the remainder of the file.
- A browser save does not copy metadata back into the document core.
- A `SingleHtmlMockup` record's metadata is read from its HTML metadata comment rather than YAML front matter, and the CLI and browser show the same values.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote and clarifications (verbatim in REQ-05.010): "update the browser view and the cli view to use those inftormation."
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D9 (record, JSON, and status-code equivalence; rendered HTML semantic), D10 (presentation facts live in the config seed), D14 (field sequence and `frontmatter_key`; `model`, `title`, `content`, and `id_shape` are not config keys).
- ADR-039 — One descriptor-driven document view and edit surface: decision 14 (model-neutral field map), decision 16 (deterministic value resolution and no merging), decision 18 (general metadata, fixed title, read-only status, undeclared keys as `extra_metadata`), decision 22 (one edit form builder), decision 29 (presentation facts from the seed).
- ADR-036 — Config-declared document field schema and one descriptor model: decisions 13-14 (`heading` and `frontmatter_key`, and a `frontmatter_key` that may differ from `key`), decision 20 (sequence position is order everywhere), decision 21 (both locations written, no merging), decisions 23-24 (reserved keys and fixed title), decisions 28-29 (seed fields and their storage).
- ADR-0033 — Uniform single-document browser view layout (as amended by ADR-039 decisions 6-7: the view consumes the shared projection and the field map).
- ADR-0009 — Shared record envelope and unified write path for CLI and browser (one projection for both views).
- ADR-0013 — Shared record envelope without a storage layer (single patch and write path).
- REQ-05.008 — Manage UI Mockups (HTML metadata, no YAML front matter).
- REQ-05.010 — Record Metadata in YAML Front Matter (field map, reserved keys, `frontmatter_key`).
- REQ-05.011 — Maintain Record Metadata Through All Commands (`status` not editable).
- REQ-06.012 — Define Single Record View Layout.
- REQ-07.001 — Organize Artifacts into Three Categories (document status values).
- REQ-07.002 — Configure Document Types and Slugs (`statuses`, `default_status`, field items).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (fields, roles, models).
- REQ-02.002 — View and Edit Tasks.
- docs/specifications/browser-*.md — existing browser persistence contract (metadata preserved on save).
