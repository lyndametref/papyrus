---
id: TECH-096
title: "Browser Editing and Metadata Maintenance"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T19:13:49Z'
labels: []
references:
  - REQ-005.011
  - REQ-006.004
  - REQ-006.014
  - NFR-004.005
  - NFR-007.002
status: verified
---
# TECH-096 Browser Editing and Metadata Maintenance

## Description
Covers the browser edit surface for a document record: the controls it derives from the record's declared fields, the write path a save uses, the metadata a save maintains, the preservation guarantees of a write, and record-only persistence. It does not cover list or view layout, backlog records, or the Backlog board. The internal version mechanism — version tokens, entity tags, and locking — is outside this specification.

## Form Construction

- The form offers the fixed title control and one control per declared field of the record's type and role whose verbs allow edit (REQ-006.004).
- The form offers the `labels` and `references` controls (REQ-006.004).
- `status`, the created date, and the updated date are shown read-only (REQ-006.004).
- `title` is not a field: it is edited through the fixed title control and not through a field control (REQ-006.004, REQ-005.011).
- `status` is not editable because it is lifecycle- and configuration-owned (REQ-006.004).
- A field whose verbs exclude `edit` has no control on the form (REQ-006.004).
- The form's controls and their presentation derive from the declared fields of the record's type and role; the form is not authored per type (REQ-006.004).

## Save and Write Path

- A save submits the form through the same write path that a command-line edit uses (REQ-005.011).
- One save produces one patch of operations — set a front-matter key, set or clear a body section, set the heading title, and the model-specific group link, HTML-metadata, and supersession changes — and a single writer applies the patch atomically (REQ-005.011).
- A patch with no change performs no write; a failed validation leaves the store untouched (REQ-005.011).
- Each submitted value is written to the storage locations its field declares (REQ-006.004).
- The metadata and content that the save does not change are preserved (REQ-006.004, REQ-005.011).
- A front-matter key on disk that the type does not declare is kept as stored; a save never repairs, merges, or removes it (REQ-005.011).
- A `formatted-record` record carries no YAML front matter: its metadata is read from and written to its HTML metadata comment, with the same parity (REQ-006.004).
- A record still carrying a value in a legacy body section moves that value to front matter when the record is written, so no value is stored twice (REQ-005.011).

## Submission and Conflict

- The form is submitted through the shared document surface, not a per-type handler (REQ-006.014).
- A submission carrying a stale record version renders the same form with the submitted values and a conflict notice, and writes nothing (REQ-006.014).
- A valid submission saves the record and leaves the edit surface (REQ-006.014, REQ-005.011).

## Metadata Maintained by the Browser

- `created_date` is set when the record is created and never changes afterwards (REQ-005.011).
- `updated_date` is refreshed by every write, including a browser save (REQ-005.011).
- `id` is generated and stable (REQ-005.011).
- `id`, `created_date`, and `updated_date` have no form control (REQ-005.011).
- For a Markdown-backed document record, `status` has no edit control; it changes only through the `status` verb, and archiving changes no status. A `formatted-record` state changes only through `edit --status approved` (REQ-005.011).
- `labels` and `references` are general repeatable metadata, not fields (REQ-005.011).
- Adding front matter to a record that lacks it leaves the body unchanged (REQ-005.011).

## Record-Only Persistence

- Application state is persisted only in managed record files and the project configuration file (NFR-004.005).
- No database, index, cache, lock file, sidecar, or mapping store is created (NFR-004.005).
- Output that a command is explicitly asked to produce is not application state (NFR-004.005).
- A temporary file used for an atomic write is removed when the write completes (NFR-004.005).

