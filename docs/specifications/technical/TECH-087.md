---
id: TECH-087
title: "Content Models and Field Schema"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-25T20:42:10Z'
labels: []
references:
  - REQ-005
  - REQ-005.001
  - REQ-005.008
  - REQ-005.015
  - REQ-007.004
status: draft
---
# TECH-087 Content Models and Field Schema

## Description
Covers the three document models, the identifier shape and field schema of each
model, field storage locations, kinds, required and verb rules, sequence order,
engine-owned keys, and the single-writer rule. Excludes configuration entry
keys and defaults, the Seed, and command dispatch.

## One Model per Type

- Every document type declares exactly one model: `SingleMarkdown`,
  `GroupedMarkdown`, or `SingleHtmlMockup`.
- An omitted model is `SingleMarkdown`; an unknown value is rejected.
- Any configured type may declare any of the three models.
- The model fixes the identifier shape, the number of records, the body format,
  where state is stored, and whether role field sets apply.
- The model adds, removes, renames, and reshapes no status and introduces no
  per-type lifecycle.

## SingleMarkdown

- One record per plain identifier.
- The body is Markdown and consists of the type's field sequence, which may be
  empty.
- A record carries the general front matter; `status` is front-matter data.
- The type declares one top-level field sequence and no role block.

## GroupedMarkdown

- A group record has a plain identifier; a member record has a dotted
  identifier that names the owning group's sequence.
- A member belongs to exactly one group, and a group holds only members of its
  own document type.
- The type declares a `group` role and a `member` role, each with its own field
  sequence and independent order.
- A top-level field sequence on such a type is rejected.
- A record carries the fields of its own role.
- The owning group is the reserved front-matter key `group`. The engine alone
  writes and maintains it; it is not a field, no option sets it, and a member's
  group link cannot change after creation.
- Archiving or deleting a group changes no member's file, status, or
  identifier.

## SingleHtmlMockup

- One record per plain identifier, with an HTML body.
- The type declares no field sequence and the record has no YAML front matter.
- Metadata lives in an HTML metadata comment and carries exactly these keys:
  `status`, `task`, `specs`, `refs`, `references`, `superseded_by`.
- All six keys are engine-owned record data; none is a field, a CLI field
  option, or a configuration key.
- `task` is the associated task identifier, `specs` the UI specification
  identifiers the mockup realises, `refs` the references to other records,
  `references` the record's references shown read-only, and `superseded_by`
  the back-reference to the superseding mockup.
- The states are exactly `pending`, `approved`, and `archived`, in that linear
  order; configuration cannot add, remove, or rename a state.
- A new record starts `pending`. A backwards or skipping transition is
  rejected, and a superseded record admits no transition.
- `archive` is the only path to `archived`.
- Supersession sets the superseded record to `archived` and stores the
  superseding identifier in `superseded_by`; the superseding record is
  unchanged.
- Papyrus does not version a record.

## Field Specification

A field declares:

- `key` — stable identifier of the field.
- `heading` and/or `frontmatter_key` — at least one storage location.
- `option` — the CLI option, defaulting to the key in kebab case.
- `label` and `help` — presentation text.
- `kind` — `text`, `multiline`, or `list`.
- `required` — whether the field must be supplied.
- `verbs` — a subset of `create` and `edit`.

A field has no per-field order; sequence position is the only order.

## Storage and Kinds

- A field is stored under a body heading, at a front-matter key, or at both.
- A body field occupies one heading section of the record body.
- A front-matter field occupies one key of the record's YAML front matter.
- A `multiline` field accepts `-` to read its value from standard input.
- A `list` field holds a list; each command-line occurrence appends one item.

## Required and Verb Rules

- A required field must be supplied on create and may be changed, but not
  cleared, on edit.
- An optional field may be supplied or cleared.
- A create that omits a required field, and an edit that clears one, is
  rejected with one validation outcome and writes no file.
- A field contributes an option only on the verbs it declares. A field whose
  verbs exclude `edit` is never writable after creation; a field whose verbs
  exclude `create` has no create option.
- Field values are applied by the engine; no field sets `status`.

## Sequence Order

- A field's position in its sequence is its order in the record body, in the
  command line and help, and in the browser form.
- A `GroupedMarkdown` type has two independent orders, one per role.
- Adding or changing a field is a configuration file edit, not a code change.

## Engine-Owned Keys

- `group` — owning-group link of a member record.
- `title` — record title, never a field.
- `status` — lifecycle state, written only by the status and archive paths.
- The `SingleHtmlMockup` metadata keys listed in section 5.
- General metadata such as labels and references is carried by the record and
  is not a field.

## Single Writer

- Content changes are computed by the engine as operations: set a front-matter
  key, set or clear a body section, set the heading title, or change the
  model-specific group, metadata, and supersession state.
- One writer applies those operations atomically.
- A patch that changes nothing writes no file.
- No per-type module, table, or code path defines a document type's model,
  fields, or verbs.
