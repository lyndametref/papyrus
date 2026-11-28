---
id: TECH-022
title: "Record Write Concurrency and Conflict Detection"
created_date: '2026-11-27T20:35:41Z'
updated_date: '2026-11-28T08:25:50Z'
labels: []
references:
  - REQ-005.011
  - REQ-006.014
  - NFR-004.001
  - NFR-004.002
  - NFR-004.005
status: verified
---
# TECH-022 Record Write Concurrency and Conflict Detection

## Description
Covers the single record write path shared by the CLI and the browser, the version token, the conflict outcomes, the atomic write sequence, identifier resolution and the project-directory boundary checks, and the preservation guarantees a write gives. Front-matter key registry, field semantics, and read or render behavior are out of scope.

## Single write path

- `record::store::write_atomic` is the only writer of a record file. No module calls `fs::write` on a record path; every surface — CLI verbs, browser submission, import — expresses its write through it.
- Each operation builds one patch: `SetFrontMatter`, `ClearFrontMatter`, `SetSection`, `ClearSection`, `SetHeadingTitle`, `SetHtmlTitle`, and explicit section moves. The writer applies that patch and performs no other transformation.
- A patch that changes no byte performs no write, so a no-op does not refresh `updated_date`.
- One clock read per write supplies the timestamp. `created_date` is set at creation and never changes afterwards; `updated_date` is refreshed by every write, and on a decision record only when the status changes (REQ-005.011).
- Persistence adds no database, index, cache, lock file, sidecar, or mapping store (NFR-004.005). A record remains a UTF-8 text file (NFR-004.001).

## Version token

- Token format: `"<mtime-seconds>.<mtime-nanos>.<length>"` of the record file at read or render time. The token is opaque to clients.
- It is exposed as the `version` member of the JSON record and as a hidden `version` form field.
- The token always describes the live file: a `POST` reads the document from disk, derives the token from that read, patches that live document, and writes it.
- `POST` outcomes: a missing or malformed token is `400` with no write; a token that differs from the current on-disk token is `409 Conflict` with no write; a matching token proceeds to the write.
- On a stale token the form is rendered again with the submitted values and a conflict notice, and nothing is written (REQ-006.014). A JSON submission with a stale token returns `409`.
- The token is metadata, not a content hash. A concurrent change that preserves both the modification time and the byte length is not detected.
- The CLI does not issue or accept version tokens; it inherits atomicity and preservation only.

## Resolution and boundary

- A record identifier is resolved by exact, case-sensitive match against the active directory listing of its enabled type. A request value is never concatenated into a path.
- Every path a read or write touches — record directory, record file, temporary file — passes `Context::ensure_within` against the canonical project directory (NFR-004.002). A symlink escape is rejected before any content is read or written.
- An identifier that is unknown or of a type that is not configured produces the shared not-found page and no write. An archived record resolves for reads; a write to it is rejected with a validation outcome and writes nothing.

## Atomic write sequence

1. Create `<record-file>.tmp` in the record's own directory with `create_new(true)`; on collision retry with a numeric suffix, `<record-file>.tmp.<n>`.
2. Write the UTF-8 bytes of the patched document.
3. Call `sync_all` on the file.
4. Copy the original file's permissions.
5. Rename the temporary file over the target, within the same directory.
6. Sync the containing directory on a best-effort basis.
7. Remove the temporary file on every handled failure path, leaving the original file untouched.

- A crash between `create_new` and the rename can leave `<record-file>.tmp` or `<record-file>.tmp.<n>`. Such a file is inert: it is not indexed, not served, and not removed automatically.
- The rename is only as strong as the filesystem; network filesystems and unusual mount options are outside the contract. A read-only record directory fails the save rather than degrading.

## Save-then-reread

- A successful form save returns `303 See Other`; the redirect target re-reads the file from disk.
- A successful JSON save re-reads the record and returns the updated representation.
- No record content is cached between requests; the save is proven by a fresh read after the write.

## Preservation

- Front-matter keys and body content that the operation does not change are preserved.
- An undeclared front-matter key already on disk is preserved and is never repaired, merged, or removed.
- Unknown and editor-added `##` sections are preserved content-identically, in their original order.
- The body carries no generated title line: for a Markdown-backed record identity and title live in the front matter, and for a `formatted-record` the title is the HTML `<title>` element; a write never inserts a generated title line (REQ-005.010).
- For a `formatted-record`, the HTML body and the metadata-comment keys the operation does not change are preserved; a title edit rewrites the HTML `<title>` element, and the record has no front matter.
- The file ends with a single LF newline.
- A validation failure writes nothing.

## Residual limits

- The version check and the rename are not one atomic operation. Two writers can both pass the check before either renames, and the later rename wins. This is accepted for a single-user loopback interface and is not a multi-writer guarantee.
- Filesystem timestamp granularity can hide a same-length edit: a change that preserves both the modification time and the byte length is undetected until the next change.
- A failed write leaves no partially written record observable; the original file stays intact.
