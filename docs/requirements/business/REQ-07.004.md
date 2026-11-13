# REQ-07.004 Provide a Generic Content Model for Config-Defined Document Types

## User Story
As a project contributor, I want a document type added through configuration to have a usable content model so that I can create and edit its records without new code or a per-type schema.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
A document type that exists only because it was declared in `document_types` has no type-specific fields in this release. Its records use the same generic content model as technical specifications: a required title, an optional description that forms the document body, and the general front-matter fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and `status` (REQ-05.010). Labels and references are repeatable and are supplied through `--label` and `--reference` on create and edit.

The `status` of an active record is `active` or one of the type's declared custom statuses (REQ-07.001, REQ-07.002), and it becomes `archived` only through archiving. A new record starts at the type's configured `default_status`, or at `active` when none is configured. Status is not set by a create or edit option: it is lifecycle- and configuration-owned and changes only through `papyrus <document-slug> status` and `archive` (REQ-05.011).

Records are created with an ID built from the entry's `prefix` and `id_padding` (REQ-01.005), stored under `doc_directory` plus the entry's `directory`, and archived into the `archived/` subfolder of that folder (REQ-05.007). The generic model has no per-type field schema and no template file, so declaring the entry is the whole setup. A config-defined type has the baseline command family and no additional verbs (REQ-07.003).

The baseline command family is the same for every document type. Built-in default types keep their existing type-specific content, fields, and additional verbs, and the generic model adds no field to them and does not replace their documented fields (REQ-05.001 to REQ-05.014).

## Acceptance Criteria
- A document type added through `document_types` can create a record with a required title and optional description.
- Records of a config-defined type carry the general front-matter fields `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and `status`.
- A config-defined record contains no type-specific field beyond those general fields: no additional front-matter key and no type-specific body section appears.
- Labels and references are supplied through repeatable `--label` and `--reference` options on create and edit.
- Record IDs use the entry's `prefix` and the configured `id_padding`.
- Records are stored under `doc_directory` plus the entry's `directory`, and archive into that folder's `archived/` subfolder.
- A config-defined type is usable in create, list, view, edit, archive, and delete with only `prefix`, `directory`, and `enabled` declared (plus optional `statuses` and `default_status`); no field schema, template file, or other per-type artifact is needed.
- A new record starts at the type's configured `default_status`, or at `active` when none is configured.
- `status` is not accepted as a create or edit option; it changes only through `papyrus <document-slug> status` and `archive`.
- A config-defined type offers the baseline command family and no additional verbs.
- Default document types keep their existing type-specific fields and additional verbs; the generic model adds no field to them.

## Open Questions
- Whether a per-entry field schema or a template file should be added later. Current requirement: neither exists.
- Whether a config-defined type may reuse a built-in type's content schema. Current requirement: no.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "any type of document can be configured in the config file. a default set is proposed at init but editing the file allows to create any document type unless the name is already used by a buit in document type."
- Project-owner decision 2026-11-13 (refinement): config-defined types use a generic content model equivalent to technical specifications; no per-type schema and no template.
- Project-owner decision 2026-11-13 (D1): the baseline verb family is shared by every document type, and built-in typed types may add verbs; "same family for all" therefore describes the baseline, not the complete verb set of the built-in types.
- Project-owner decision 2026-11-13 (D5): `status` of a generic record is a declared custom status or `active`, starts at `default_status`, and is not supplied through create or edit options (REQ-05.011).
- REQ-01.005 — Assign Stable Record IDs
- REQ-05.007 — Manage Document Archive State and Deletion
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views
- REQ-07.001 — Organize Artifacts into Three Categories (document status model)
- REQ-07.002 — Configure Document Types and Slugs
- REQ-07.003 — Dispatch Config-Defined Document Commands
- BR-017 — Tech Spec Command Behavior (typed type whose content model the generic model mirrors)
