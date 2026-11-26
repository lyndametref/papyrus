---
id: TECH-083
title: "Record Identifiers"
created_date: '2026-11-25T20:42:00Z'
updated_date: '2026-11-26T21:42:55Z'
labels: []
references:
  - REQ-001.005
  - REQ-007.004
status: draft
---
# TECH-083 Record Identifiers

## Description
Covers record identifier assignment, shape, padding, and the single identifier space. Metadata, front matter, field projection, write behavior, and browser forms are separate specifications.

## Identifier Assignment

- Papyrus generates record identifiers automatically; creating any managed record assigns one.
- An identifier never changes when a record moves between lifecycle states.
- Any record keeps its identifier across archive and restore.
- The display identifier and the stored identifier are the same value.

## Identifier Shape

- A document record's identifier shape follows its type's configured document model (REQ-007.004).
- A `simple-record` or `formatted-record` record has one plain identifier `<PREFIX>-<NUMBER>`.
- A `grouped-record` group has a plain identifier `<PREFIX>-<GROUP>`.
- A member of a `grouped-record` group has a dotted identifier `<PREFIX>-<GROUP>.<MEMBER>` under the same type prefix.
- `<PREFIX>` is the type's configured `prefix`.
- A newly configured `grouped-record` type produces grouped identifiers, and a newly configured `simple-record` type produces plain identifiers.

## Identifier Padding

- The shared `id_padding` determines the generated numeric width for tasks, milestones, releases, and every document type.
- `id_padding` is a whole number from 1 to 6 inclusive and defaults to 3.
- A value outside that range is rejected during configuration validation, with a message that names the permitted range.
- Padding is a minimum, never a maximum: when a number's digit count exceeds `id_padding`, the wider number is used, never truncated, and never an error.
- Existing files are not re-padded, and no migration is performed when `id_padding` changes.
- Existing identifiers are never truncated or migrated.

## Identifier Allocation

- Allocation is the highest observed number plus one across the type directory and its `archive/` subfolder, with no high-water mark.
- A deleted number is reusable only when it was the highest observed number at allocation time; lower deleted numbers remain gaps.

## Single Identifier Space

- The Papyrus identifier is the record identity everywhere: in the front-matter `id`, in command input, and in command-line and browser output.
- Papyrus keeps no second, tool-specific storage identifier and does not translate between identifier spaces.
