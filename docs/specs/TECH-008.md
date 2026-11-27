---
id: TECH-008
title: Seed
created_date: '2026-11-27T20:35:37Z'
updated_date: '2026-11-27T20:35:37Z'
labels: []
references:
  - REQ-001.006
  - REQ-004.001
  - REQ-005.016
status: verified
---
# TECH-008 Seed

## Description
Defines the Seed as a compile-time configuration of the system: one configuration file in the source tree, compiled into the binary at build time, supplying both the document-type and the decision-type family. Covers the file's role, validation, and `init` usage. The per-type content is specified by REQ-005.016 and REQ-004.001.

## Seed Source

- The Seed is one configuration file kept in the source tree; it is not a table or a built-in list in code.
- The file is a configuration document in the shape of `papyrus.config.yml`; its `document_types` map is the document-type Seed and its `decision_types` map is the decision-type Seed.
- Because the file is a valid configuration document, it also serves as the example configuration.
- The build compiles the file into the binary; no seed file is read at runtime, and no path, environment variable, or project file replaces it.
- The file is the only Seed source; changing the Seed is a source change that requires a rebuild.
- The file must pass the configuration validation of REQ-007.002 and REQ-004.001; a Seed that fails validation fails the build.

## Families

- The document-type Seed is the file's `document_types` map and contains the entries of REQ-005.016.
- The decision-type Seed is the file's `decision_types` map and contains exactly these two entries:

| Slug | Prefix | Directory |
|---|---|---|
| `adr` | `ADR` | `adr` |
| `bdr` | `BDR` | `bdr` |

- A decision-type entry declares no criticality and no status list; every decision type uses the fixed decision status vocabulary (REQ-004.002).
- A document-type entry carries its model, field sequence, identifier prefix, directory, custom workflow status set, list slots, and field defaults.
- A decision-type entry carries its prefix and directory; the status vocabulary is shared and fixed and is never an entry value.
- Criticality is not part of the file: it is a requirement attribute of the built-in type, not a configuration key.
- The document-type entries are not restated here; the file's contents, bounded by REQ-005.016, are the definition.

## Init Lifecycle

- `init` is the only command that reads the Seed.
- The walkthrough presents the Seed for selection during `init`.
- For each selected document or decision type, `init` prompts for the prefix and the storage directory, using the file's values as the defaults.
- A selected entry is written into the project configuration fully expanded; an unselected entry is not written at all.
- A run without an interactive terminal writes every entry with the file's prefixes and directories and without prompting.
- `init` never copies the Seed file as-is; it writes the selected entries into the project configuration.
- After `init` the Seed is irrelevant: no command, view, search, validation, or default resolution reads it.
- The project configuration is then the only source of document and decision types, and a type exists if and only if the configuration declares it.
- There is no `enabled` key: presence in the configuration is the only switch, and removing an entry removes the type.

## Runtime Boundary

- The shared core library carries the Seed, statically linked into both front-end binaries; the project tree does not.
- No command writes the Seed back, and no command regenerates it from a project configuration.
- The Seed is fixed for the life of a binary; two projects initialized by the same binary start from the same Seed.
