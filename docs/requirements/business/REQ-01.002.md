# REQ-01.002 Read and Edit Project Configuration

## User Story
As a project maintainer, I want to inspect and change configuration values so that Papyrus uses settings appropriate to my project.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
The user can list effective configuration keys and defaults, get one effective value, persist a value, and unset a key to fall back to its default. Keys support dotted notation, and values supplied to `set` are parsed as YAML scalars.

When an artifact's folder is changed with `papyrus config set` — a document type's `directory`, or `backlog_directory`, `decision_directory`, or `doc_directory` — the entire content of the original folder is moved to the new folder. The move covers every file and subfolder, not only Papyrus-managed records. It happens during the `set` command (not later, not lazily), it is a move rather than a copy, and it is atomic: if any entry cannot be moved, for example because an entry with the same relative path already exists at the destination, nothing is moved, both folders are left as they were, the configuration value is not persisted, and the command reports a conflict (REQ-01.004). After a successful move the original folder no longer holds the moved content.

## Acceptance Criteria
- `papyrus config list` prints effective keys including defaults.
- `papyrus config get <key>` prints that key's effective value.
- `papyrus config set <key> <value>` persists the value as a YAML scalar.
- `papyrus config unset <key>` removes the configured key and uses its default.
- A nested key can be addressed with dotted notation, such as `document_types.req.enabled`.
- When an artifact's folder is changed via `papyrus config set`, every file and subfolder of the original folder is moved to the new folder.
- The move is performed as part of the `set` command, not lazily; afterwards the original folder no longer holds the moved content and is removed once empty.
- If the move cannot complete (destination entry conflict or write failure), the command reports a conflict, persists no configuration change, and leaves both folders unchanged.
- The configured task statuses must be unique, and the configured default task status must be one of them.
- With default settings, new tasks start at `ToDo` and the ordered statuses are `ToDo`, `Initiated`, `Refined`, `Implemented`, `Validated`, and `Done`.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → config (lines 71–92)
- README.md, `papyrus-config(5)` → KEYS (lines 945–1026)
- REQ-01.004 — Use Common CLI Controls and Outcomes (conflict outcome for a failed move)
- REQ-06.008 — Provide a Browser Configuration View (folder moves must be performed by `config set`)
- Maintainer decision 2026-11-11: a folder change moves all files of the original folder (not only managed records), synchronously, as an atomic move with conflict failure and no partial move.
