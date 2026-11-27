---
id: TECH-111
title: "Record Metadata, Front Matter, and Projection"
created_date: '2026-11-26T18:13:50Z'
updated_date: '2026-11-27T16:54:39Z'
labels: []
references:
  - REQ-001.005
  - REQ-005.008
  - REQ-005.010
  - REQ-005.011
  - REQ-006.004
status: verified
---
# TECH-111 Record Metadata, Front Matter, and Projection

## Description
Covers the front-matter contract, general, reserved, and document keys, serialization, automatic metadata values, adoption and import, and CLI/browser metadata parity. Identifier allocation is a separate specification; field schema and browser forms are also separate.

## Front-Matter Block

- Every Markdown record that Papyrus manages begins with a YAML front-matter block at byte 0, delimited by `---` lines.
- Front matter is authoritative for the keys it carries and replaces the equivalent information in the record body for records Papyrus writes.
- Records of a `formatted-record` type carry no front matter; their metadata is kept in their HTML metadata comment.
- Prose Markdown that no document type claims is not managed and is out of scope.

## General Keys

- Every managed Markdown record carries `id`, `title`, `status`, `labels`, `references`, `created_date`, and `updated_date`.
- A member record of a `grouped-record` type also carries `group`.
- Papyrus writes and maintains these keys.

## Reserved Keys

- The reserved keys are `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`, `version`, `kind`, `form`, `path`, and `metadata`.
- No document type declares a reserved key, and no field uses one as its key or as its front-matter key; such a declaration is rejected.
- `title` is fixed and is not a field.
- `status` is not a field; it is written only by the commands or options defined for the record's category.
- `group` is the engine-owned link of a member record: the write engine sets and maintains it from the member's creation context; it is not a field, not user-editable, and membership is immutable.
- `version` is reserved because it is the read/write concurrency token exposed in the JSON record and the edit form; it is not a stored front-matter key.

## Document Keys

- Beyond the general keys, a record's front-matter keys are exactly the front-matter keys of its type's declared fields for the record's role.
- A field is stored under exactly one location: a body heading or a front-matter key.
- A field stored under a body heading contributes no front-matter key; its value lives in the body.
- A field stored under a front-matter key is metadata-only and has no body section.
- A metadata-only field's front-matter key is its field key.
- Document keys follow field-sequence order after the general keys.
- An undeclared key already present on disk is preserved as stored and displayed read-only.

## Serialization

- `created_date` and `updated_date` are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'`.
- The `id` value and every reference value use the record identifier; no second identifier space exists.

## Automatic Values

- `created_date` is set when a record is created and never changes afterwards.
- `updated_date` is set at creation and refreshed by every write, including a browser save.
- For a decision record, `updated_date` is refreshed only when the status changes.
- `id`, `created_date`, and `updated_date` have no command option.
- For a Markdown-backed document record, `status` has no field-derived create or edit option; a new record starts at its type's `default_status`, or at `active` when none is configured, and changes only through the `status` verb; archiving changes no status.
- For a `formatted-record`, `status` has no field-derived create or edit option; `edit --status approved` moves `draft` to `approved`, and archiving changes no status.
- The status of a task, milestone, release, or decision keeps the behaviour of its own category.

## Adoption and Import

- An existing managed Markdown record that does not yet carry a front-matter block gains one on its first read and on its first write.
- Adoption is idempotent and requires no bulk migration command.
- Adoption covers only files claimed by a configured document type or by another managed record category; prose Markdown and HTML records are excluded.
- Adoption changes nothing in the body, except that a value still held in a legacy body section moves to front matter when its record is written, so no value is stored twice.
- Records brought in by the one-way Backlog.md import are written in this format.

## CLI and Browser Parity

- The command line in plain text and JSON and the browser single-record view display the same metadata values for the same record.
- All surfaces read the record's stored metadata and project the declared fields of the record's type and role; neither derives a value from the body when the metadata defines it.
- Neither surface renames a stored value for display.
- The command line and browser single-record view show at least the identifier, the title, the created and updated dates, the labels, the references, and every declared field of the record's type and role.
- A record's `status` is shown as stored, including a custom status declared for its type, and is identical in the command line and the browser.
