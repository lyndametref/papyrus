---
id: TECH-042
title: "Expose Front-Matter Metadata Options on All Record Commands"
created_date: '2026-11-16T17:46:48Z'
updated_date: '2026-11-21T08:11:34Z'
labels: []
references:
  - ADR-0013
  - ADR-0016
  - ADR-0023
  - ADR-0036
  - ADR-0039
  - BR-001
  - NFR-04.004
  - REQ-01.004
  - REQ-05.010
  - REQ-05.011
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
status: verified
---
# TECH-042 Expose Front-Matter Metadata Options on All Record Commands

## Description

The option surface of a generated document command is a function of the
descriptor's field sequence and each field's verb set. No hand-written table,
per-type module, or per-type help text supplies an option.

## Behavior

### 1. Option source

1. Every declared field of a document type contributes exactly one long option,
   for the record's role, to the commands of that type. There is no separate
   option table.
2. The option set is derived per invocation from the effective configuration
   descriptor, so adding, removing, reordering, or re-pointing a field changes
   the options with no code change and no rebuild.
3. A field option is named by the field's `option` when present, otherwise by
   its `key` rendered in kebab case.
4. The option carries the field's `label` as its display label and its `help` as
   its help text.

### 2. Value form by kind

| Kind | CLI value form |
|---|---|
| `text` | one single-line value |
| `multiline` | one multi-line value; `-` reads standard input |
| `list` | a repeatable option; each occurrence appends one sanitized item |

1. A `list` option appends unique values in submission order and never removes
   an existing item. Clearing a whole value is offered only where the command
   already provides a clear flag, such as the milestone and release clear flags.
2. A `text` or `multiline` value replaces the stored value.

### 3. Option availability follows the field's verbs

1. A field declares a subset of the `create` and `edit` verbs; the default is
   both.
2. A metadata option exists on a command only when the field's verbs include
   that command: the option is offered on `create` only when the verbs include
   `create`, and on `edit` only when the verbs include `edit`.
3. A field whose verbs exclude `edit` is never writable after creation; a field
   whose verbs exclude `create` has no create option and is first supplied by an
   edit.
4. No command offers an option for a field its record role does not declare.
5. In a grouped type the create and edit options come from the role's own
   sequence: `group.fields` for a group record and `member.fields` for a member
   record.

### 4. Option order

1. Options follow field sequence order in parsing and in help, including the
   role-appropriate sequence of a grouped type.
2. Global options, the general metadata options, and the title arguments keep
   their documented positions around the generated block; a field sequence
   change never renames, removes, or reorders an existing positional argument.

### 5. Title

1. Title is not a field and never appears in the generated option set: it
   carries no field label, help, or kind and is not derived from the sequence.
2. The title is the first positional `TITLE` on `create` and `create-group`.
3. `--title` is a fixed command flag, not a metadata option: it is accepted on
   `create` and `create-group` as an alternative to the positional title, and on
   `edit` to change the title.
4. The title is required on create. Supplying both the positional `TITLE` and
   `--title` in one invocation is rejected as a conflicting title source and
   nothing is written.
5. The title is single-line and non-empty; an empty or multi-line title is
   rejected, and an edit can never clear it.
6. No document command accepts `--term`. A term-shaped type stores its term as
   the record title.

### 6. Keys that are never field options

| Key | Treatment |
|---|---|
| `id` | generated and stable; never an option |
| `created_date` | set at creation; never an option or editable |
| `updated_date` | maintained by the writer; never an option or editable |
| `status` | reserved; written only by the status and archive verbs; no create, edit, or field option sets it |
| `group` | engine-owned member link, written at member create only; never an option |
| `title` | fixed record data; positional title argument and `--title`, never a field option |
| `labels` | general front-matter metadata; repeatable `--label` on every front-matter-backed record type |
| `references` | general front-matter metadata; repeatable `--reference` on every front-matter-backed record type |

1. `--label` and `--reference` remain general front-matter metadata options on
   every front-matter-backed record type and are not fields of any document
   type. A model without front matter carries neither: a `SingleHtmlMockup` has
   no front-matter labels or references, and its fixed metadata is written
   through the mockup metadata controls instead.
2. A field whose `key` or `frontmatter_key` equals a reserved key is rejected by
   configuration validation, so a reserved key can never reach the option
   builder.

### 7. Required, optional, and defaults

1. A required field must be supplied on create. A create that omits one or more
   required fields produces a single validation outcome that lists every missing
   key, and no file is written.
2. On edit a required field may be changed but not cleared; clearing it is a
   validation outcome and writes nothing.
3. An optional field may be cleared on edit.
4. The status of a new record is its type's configured default status, or
   `active` when the type declares none, and it is never supplied by an option. A
   `SingleHtmlMockup` declares no default status and starts `pending`.
5. `ordinal` remains an opaque non-negative integer backlog field with a create
   and an edit option; a document type does not gain an ordinal option.

### 8. Write contract

1. A create and an edit each build one patch from the supplied values and apply
   it through the single record writer; no command formats or writes a body.
2. A patch that changes no byte performs no write, and a validation failure
   writes nothing.
3. A write preserves every front-matter key, body section, comment, and line
   ending that the operation does not change, including undeclared keys.
4. `created_date` is written once at creation and never changes.
   `updated_date` is refreshed exactly once per write that changes at least one
   byte, including an archive, a delete, and a status move, and never by a no-op
   write.

### 9. Help output

1. Help for one effective configuration is identical across runs, machines,
   working directories, terminal widths, and locales, and contains no absolute
   path, host name, timestamp, process identifier, or memory address.
2. Enabled document types appear in seed `order` order with ties broken by slug.
3. Per-option help prose and option order come from the field `label`, `help`,
   and sequence position; a seed or configuration field change may legitimately
   change them.
4. The root about line, the per-type about line, each type's help line, and each
   verb's about line are fixed.

### 10. Validation and error behavior

1. An unknown, archived, or other-type group reference is a validation outcome
   (exit code 4) and writes nothing.
2. An unknown referenced record for a validated key is a not-found outcome
   (exit code 3).
3. A negative or non-integer `ordinal` is a validation outcome (exit code 4).
4. An empty or invalid sanitized value is a validation outcome (exit code 4).
5. Mutually exclusive flags supplied together are a validation outcome (exit
   code 4).
6. A no-op write exits successfully and leaves the file byte-identical.
7. An atomic-write failure leaves the original file byte-identical and returns a
   runtime error (exit code 1).

## Inputs and outputs

- Input: a project context, an effective configuration descriptor, and the
  supplied option values.
- Output: a created or edited record whose front matter and body contain every
  supplied value, with all unmodified content preserved byte-for-byte.

## Assumptions

- Assumption: option flag names follow the existing Papyrus conventions, and a
  newly applicable field gains the option its `option` or kebab-case `key`
  names.
- Assumption: a repeatable list option appends unique values, matching the
  existing task behaviour for labels and dependencies.
- Assumption: the general `references` list and documentation paths are stored
  after sanitization without existence validation, while `milestone`,
  `dependencies`, `release`, `group`, and `superseded_by` are validated.

## Open questions

- Whether a newly exposed list option should gain replace and remove variants
  in addition to append.
- Whether `documentation` paths and general `references` values should be
  validated for existence.
