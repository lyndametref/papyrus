# REQ-01.002 Read and Edit Project Configuration

## User Story
As a project maintainer, I want to inspect and change configuration values so that Papyrus uses settings appropriate to my project.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Must have

## Description
The user can list effective configuration keys and defaults, get one effective value, persist a value, and unset a key to fall back to its default. Keys support dotted notation, and values supplied to `set` are parsed as YAML scalars.

## Acceptance Criteria
- `papyrus config list` prints effective keys including defaults.
- `papyrus config get <key>` prints that key's effective value.
- `papyrus config set <key> <value>` persists the value as a YAML scalar.
- `papyrus config unset <key>` removes the configured key and uses its default.
- A nested key can be addressed with dotted notation, such as `document_types.req.enabled`.
- The configured task statuses must be unique, and the configured default task status must be one of them.
- With default settings, new tasks start at `ToDo` and the ordered statuses are `ToDo`, `Initiated`, `Refined`, `Implemented`, `Validated`, and `Done`.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → config (lines 71–92)
- README.md, `papyrus-config(5)` → KEYS (lines 945–1026)
