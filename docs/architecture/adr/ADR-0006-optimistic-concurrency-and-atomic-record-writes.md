# Optimistic Concurrency and Atomic Record Writes for Browser Edits

## Status

Proposed

## Date

2026-11-03

## Context

The TASK-6 browser is the first Papyrus surface that writes requirement records while the CLI can also edit the same files. The browser has no session, no lock manager, and no database; records are plain UTF-8 Markdown files inside the project directory (NFR-04.001, NFR-04.002). The behavior contract is `docs/specifications/browser-requirements.md` sections 7.10–7.12.

The problem is twofold: a save must not silently overwrite a file changed after the edit form was rendered, and a crash or concurrent read must never observe a partially written record. The alternatives considered were advisory file locking, a single-writer queue or daemon, last-write-wins, a lock file, and moving records into a database. Locking and daemons add lifecycle and stale-lock failure modes; last-write-wins loses data; a database violates the text-file and single-crate constraints.

## Decision

- Use **optimistic concurrency without locking**. The version token is `"<mtime-seconds>.<mtime-nanos>.<length>"` of the record file at render/read time. It is opaque to clients and is exposed as the `version` member of the JSON record and as a hidden `version` form field.
- On `POST`, a missing or malformed version token is `400`; a token that differs from the current on-disk token is `409 Conflict` with no write; a matching token proceeds to the write.
- Write records **atomically within their own directory**: create `<id>.md.tmp` with `create_new(true)` (retrying with a numeric suffix on collision), write UTF-8 bytes, call `sync_all`, copy the original file's permissions, rename the temporary file over the target, sync the directory on a best-effort basis, and remove the temporary file on every failure path so the original file is left untouched.
- Accept and document the residual race: the version check and the rename are not one atomic operation. This is acceptable for a single-user loopback interface. It is not a multi-writer guarantee.
- Save-then-reread: a successful form save returns `303 See Other` and the follow-up `GET` re-reads the file from disk; a successful JSON save re-reads and returns the updated record. The server keeps no cached record content between requests.
- Resolve record IDs by exact, case-sensitive match against the active directory listing and never concatenate request values into a path. Route every read and write path (record directory, record file, temporary file) through `Context::ensure_within` against the canonical project directory. Symlink escapes are rejected before any content is read or written.
- Preserve unknown and editor-added `## ` sections content-identically, preserve the original section order, keep the first heading as `# <ID> <title>`, and terminate the file with a single LF newline.

## Consequences

**Positive:**

- No lock manager, lock file, daemon, or stale-lock recovery is needed; the CLI and browser can coexist without a shared process.
- A crash or failed write leaves the original file intact and removes the temporary file on the handled failure paths.
- Permissions, unknown sections, and section order survive an edit.
- The version token gives users a clear `409` recovery path instead of silent data loss.
- All writes remain project-local text files (NFR-04.001) and pass the project-directory boundary check (NFR-04.002).

**Negative:**

- The token is metadata-based (`mtime`, nanoseconds, length). A concurrent change that preserves both the timestamp and the length can go undetected; the token is not a content hash.
- The check-then-rename window is a real race. Two writers can both pass the check before either renames, and the later rename wins.
- There is no merge, conflict resolution, or three-way diff; a stale save is rejected and the user must reload and retype.
- Atomic rename is only as strong as the underlying filesystem; network filesystems and unusual mount options are not covered.
- The temporary file naming scheme assumes the record directory is writable; a read-only directory fails the save rather than degrading.

**Risks and mitigations:**

- *Token collision*: acceptable for the single-user loopback scope; if stronger guarantees are needed, a content hash or file-generation counter requires a new ADR.
- *Residual check-then-rename race*: explicitly documented in the specification and accepted for this increment; multi-writer or network-filesystem use is out of scope.
- *Partial writes*: `create_new` + `sync_all` + same-directory rename + cleanup on failure; tests in `tests/browser_cli.rs` cover atomicity, permission preservation, and temporary-file cleanup.
- *Path escape or traversal*: exact-stem resolution plus `Context::ensure_within` on every path; symlink-escape tests are part of the TASK-6 suite.
- *Lost unknown sections*: the parser/serializer preserves unknown `## ` sections and their order; tests cover this.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Optimistic concurrency with a metadata version token and atomic rename (this ADR) | Selected for TASK-6 | No locking lifecycle, works across CLI and browser, protects against partial writes, and is sufficient for a single-user loopback interface. |
| Advisory file locking (`flock`/lock file) | Rejected | Adds stale-lock handling, platform differences, and a second failure mode; unnecessary for the single-user scope. |
| Single-writer queue or daemon | Rejected | Adds a process, lifecycle, and IPC surface; conflicts with the dependency-free single-binary model. |
| Last-write-wins | Rejected | Silently loses concurrent edits. |
| Database-backed records | Rejected | Violates text-file persistence (NFR-04.001) and the single-crate/single-binary constraints. |
| Content-hash version token | Deferred | Stronger than metadata, but more work per request; can be adopted later via a new ADR if the token proves insufficient. |

## References

- `docs/specifications/browser-requirements.md` — TASK-6 sections 5 (JSON API), 7 (editing contract), and 9 (security).
- `docs/requirements/non-functional/NFR-04.001.md` — text-file persistence.
- `docs/requirements/non-functional/NFR-04.002.md` — project-directory boundary.
- `docs/requirements/non-functional/NFR-02.002.md` — input sanitization.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust and Cargo.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single-crate layout.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — server and UI context.
- Implementation: `src/browser/records.rs` (`version_token`, `valid_version_token`, `apply_edit`, `write_atomic`), `src/browser/mod.rs` (`form_post`, `api_post`), `src/context.rs` (`ensure_within`).
- Tests: `tests/browser_cli.rs` (edit round-trip, stale version, atomic write, permissions, symlink escape).
- Status note: implemented and verified in TASK-6/TASK-7; this ADR is `Proposed` pending explicit user acceptance.
