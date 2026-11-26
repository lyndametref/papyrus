---
id: TECH-084
title: "Record Persistence, Layout, and Lifecycle"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-26T20:18:43Z'
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
# TECH-084 Record Persistence, Layout, and Lifecycle

## Description
Covers the storage format, the closed store set, the project-directory boundary, and the on-disk layout and lifecycle of document records: archive, delete, and relocation of misplaced records. Applies to all three document models. Does not cover configuration syntax or validation, command dispatch, field content, or the Seed.

## Storage Format

- Every file Papyrus saves is UTF-8 text and contains no binary data.
- Managed records other than UI mockups are Markdown.
- UI mockup records are HTML.
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

- A record file lives in the configured directory of its document type.
- An archived record file lives in the `archive/` subfolder of that directory.
- A record is archived exactly when its file is in `archive/`.
- A record that is not archived lives in the type directory and carries `status: active` or one of the custom workflow statuses its type declares.
- A custom workflow status never moves a file and creates no per-status folder.
- `archive/` is the only recognised archive storage. No other subfolder is read as archived, nor is migrated. No other subfolder are inventoried by any command, including `papyrus check`.

## Archive Lifecycle

- `papyrus <slug> archive <ID>` sets the record's status to `archived` and moves the file into the `archive/` subfolder of the type's configured directory.
- For a record that is not superseded, `archive` is the only operation that writes `archived`.
- `supersede` on a `SingleHtmlMockup` also archives only the mockup it supersedes (REQ-005.008).
- The command name is `archive`, the value it writes is `archived`, and the destination is `archive/`.
- Archiving preserves the record identifier.
- Archiving discards the previous workflow status; that value is not retained.
- For a `SingleHtmlMockup` record, the archived state is written to the HTML metadata and the file moves the same way.
- Archived records are hidden from lists, search, and the browser, except that a search may explicitly include archived records. The browser shows the shared not-found page for them.
- No command returns an archived record to a non-archived state, and no restore command exists.

## Delete Lifecycle

- `papyrus <slug> delete <ID>` permanently removes a record.
- A record that is not archived is archived first and then removed.
- Deletion asks for confirmation, and `--force` skips the prompt.
- A deleted record leaves no file.

## Misplaced Records

- A record whose stored status is `archived` while its file is outside `archive/` — after a hand edit or an interrupted write — is relocated into `archive/` before it is returned.
- Relocation applies on every read, list, view, search, and browser path.
- Relocation uses the same write-then-remove sequence as the `archive` command and reports the relocation in diagnostics.
- If relocation fails, the record is returned read-only with a validation outcome, and writes to it are refused until the conflict is resolved.

## GroupedMarkdown Records

- Archive and delete apply to a single record.
- Archiving or deleting a group record changes no member's file, status, or identifier.

## Model Independence

- The rules in this specification are identical for `SingleMarkdown`, `GroupedMarkdown`, and `SingleHtmlMockup`.
- They do not depend on a type's field sequence or role field sets.
