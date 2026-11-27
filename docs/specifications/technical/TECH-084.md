---
id: TECH-084
title: "Record Persistence and Layout"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-27T16:54:39Z'
labels: []
references:
  - REQ-005.007
  - REQ-005.008
  - NFR-004
  - NFR-004.001
  - NFR-004.002
  - NFR-004.005
status: verified
---
# TECH-084 Record Persistence and Layout

## Description
Covers the storage format, the closed store set, the project-directory boundary, and the on-disk folder layout of the three record categories. Archive, restore, delete, and lifecycle transitions are separate specifications.

## Storage Format

- Every file Papyrus saves is UTF-8 text and contains no binary data.
- A `formatted-record` record is HTML.
- Every other managed record is Markdown: backlog records, decision records, and the two Markdown document models.
- The project configuration is YAML.

## Store Set

- Application state persists only in managed record files and the project configuration file. These are the only stores.
- Papyrus creates no database, index, cache, lock file, sidecar, or mapping store.
- Output that a command is explicitly asked to produce is not application state.
- A temporary file used for an atomic write is removed when the command completes.

## Project-Directory Boundary

- No operation reads or writes a path outside the project directory, except `init`.
- Every path is canonicalized and symbolic links are resolved before the path is used.
- A path that escapes the canonical project root after resolution is rejected.
- `init` may write outside the project directory only to create the project directory itself.

## Record Layout

- A backlog record lives in the backlog folder for its record type, and an archived backlog record lives in that folder's `archive/` subfolder.
- With exactly one configured decision type, a decision record lives directly in the main decision folder and an archived decision lives in its `archive/` subfolder.
- With two or more configured decision types, a decision record lives in its type's directory under the main decision folder, and an archived decision lives in that directory's `archive/` subfolder.
- A document record lives in the configured directory of its document type, and an archived document lives in that directory's `archive/` subfolder.
- A record is archived exactly when its file is in an `archive/` subfolder.
- A Markdown-backed record that is not archived carries `status: active` or one of the custom workflow statuses its type declares.
- An archived record keeps the status it had before archiving; archiving changes no status.
- A custom workflow status never moves a file and creates no per-status folder.
- `archive/` is the only recognised archive storage. No other subfolder is read as archived, nor is migrated. No other subfolder is inventoried by any command, including `papyrus check`.

## Document Models

- The layout rules are identical for `simple-record`, `grouped-record`, and `formatted-record`.
- They do not depend on a type's field sequence or role field sets.
