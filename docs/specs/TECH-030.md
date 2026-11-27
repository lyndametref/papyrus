---
id: TECH-030
title: "Configuration Management"
created_date: '2026-11-27T20:35:44Z'
updated_date: '2026-11-27T20:35:44Z'
labels: []
references:
  - REQ-001.002
  - REQ-001.004
  - REQ-004.001
  - REQ-007.002
  - NFR-004.001
  - NFR-004.002
status: verified
---
# TECH-030 Configuration Management

## Description
Covers the `config` command family, the configuration file model, selectors, value shapes, document- and decision-type entries, folder relocation, and validation and repair. The CLI context and command surface are separate specifications.

## Configuration File

- The configuration file is UTF-8 YAML (NFR-004.001).
- The effective configuration is the file's values plus the declared default for every key the file does not declare.
- An unknown attribute is accepted, ignored, and reported as one warning. It is never an error.
- An invalid value of a known attribute fails closed: no command runs on a partially valid configuration (REQ-007.002).

## Configuration File Format

- The configuration file is one YAML mapping at the top level.
- The defined top-level keys are:

| Key | Type | Rule |
|---|---|---|
| `project_name` | string | Project name written by `init` |
| `id_padding` | integer | 1 to 6 inclusive; default 3 |
| `backlog_directory` | relative path | Root folder of the backlog records |
| `decision_directory` | relative path | Root folder of the decision records |
| `doc_directory` | relative path | Root folder of the document records |
| `document_types` | map, slug to entry | Document-type entries; absent means none |
| `decision_types` | map, slug to entry | Decision-type entries; absent means none |

- An entry beneath `document_types` or `decision_types` follows the entry keys and defaults of the record-type specification.
- An unknown top-level attribute is accepted, ignored, and warned, like any unknown attribute.
- The key names for the task, milestone, and release prefixes and for the browser host, port, and automatic opening are not yet specified in the requirements; they are omitted from this table until named.

## Commands

- `papyrus config list` prints the effective configuration with every default resolved.
- `papyrus config get <selector>` prints one leaf's effective value.
- `papyrus config set <selector> <value>` changes a declared entry.
- `papyrus config unset <selector>` removes a key so that its default applies again.
- `get` and `set` accept exactly one selector.
- `config get` of an unknown key is a not-found outcome (exit 3).
- `config set` builds the resulting configuration, validates it, and writes only if it is valid. An invalid result is refused and the file is unchanged.

## Listing Order

- `config list` prints one leaf per line: the selector path and its effective value.
- Order is global keys, task statuses, `document_types` entries in configuration order, then `decision_types` entries in configuration order.
- Within an entry, keys follow the configuration order, including nested role and field blocks.
- Every default is resolved, so a leaf omitted from the file still appears with its effective value.

## Selectors

- A document-type leaf is addressed as `document_types.<slug>.<path>`.
- A field sequence element is addressed by key, for example `document_types.<slug>.<role>.fields[<key>].heading`, or by zero-based index, for example `document_types.<slug>.<role>.fields[<index>].required`.
- A decision-type leaf is addressed as `decision_types.<slug>.<path>`, where `<path>` is `prefix` or `directory`.
- `<key>` is any key the type declares; no field key is reserved by the selector grammar.
- The key form and the index form resolve to the same element.
- A selector that does not resolve to a leaf is rejected with a validation outcome. This covers an unknown key, a keyed selector matching no unique field, and an out-of-range index.

## Value Shapes

- A scalar key is parsed as a YAML scalar.
- A scalar value is persisted as a YAML scalar; a comma inside a scalar is part of the value.
- A list-valued key accepts a comma-separated value and is written as a YAML flow sequence with the same values in the same order.
- The list-valued keys are the task `statuses`, a document type's `statuses`, and `slots`.
- A comma is always a separator and has no escape form.
- An empty value for a list-valued key is rejected instead of written.
- `fields` and the role field sequences are sequences of mappings and are never written from a comma-separated value.

## Unset Semantics

- `config unset` of a scalar removes the key so that its default applies again.
- `config unset` of a flat collection — `statuses` or `slots` — removes it so that the collection default applies.
- For a document type, the applied default is the model-level default, never a value from a built-in table.

## Document and Decision Type Entries

- A document type is addressed beneath `document_types.<slug>`; a decision type is addressed beneath `decision_types.<slug>`.
- Adding or removing a document type, and adding, removing, or reordering a field, is a configuration file edit and not a `config set` operation.
- The configured task statuses are unique, and the configured default task status is one of them.
- A decision type entry declares a `prefix` and a `directory`; presence in the configuration makes the type available.
- The decision-type slug is the command name, follows the same character and reserved-name rules as a document-type slug, and the status vocabulary is shared by every decision type and is not configurable.
- A second decision type cannot be added to a project with exactly one type; removing a configured type and changing the prefix of a type that still has decisions are rejected, and no file moves.
- There is no `enabled` key; presence in the configuration is the only switch.

## Folder Relocation

- Changing a folder with `config set` moves the entire content of the original folder to the new one: every file and subfolder, not only managed records.
- The folder keys affected are a document type's `directory`, `backlog_directory`, `decision_directory`, and `doc_directory`.
- The move happens during the `set` command and is a move rather than a copy.
- The move is atomic. If any entry cannot be moved, for example because an entry with the same relative path exists at the destination, nothing is moved.
- On such a failure both folders keep their content, the new value is not persisted, and the outcome is a conflict.
- A cross-device move falls back to a copy-then-remove sequence and is reported as non-atomic when a single rename is impossible.
- On success the source folder is removed once empty.

## Validation and Repair

- The effective configuration is validated at load.
- While the configuration is invalid, every command fails closed with a validation outcome.
- The repair surfaces are `config get`, `config set`, `config unset`, `config list`, `init --force`, and `check`.
- `config` commands load an invalid configuration tolerantly, validate the configuration that would result from the requested change, and refuse a write that would still be invalid.
- `init --force` replaces an invalid configuration with the generated configuration.
- `check` reports the findings and runs the checks that do not depend on the document-type set.
- A broken configuration is therefore always repairable through Papyrus.
