---
id: TASK-120
title: >-
  Route UI-mockup archive, supersede, and edit-with-move through the shared
  archive write-then-unlink path
status: Done
assignee:
  - rust-dev
created_date: '2026-11-15T11:27:00Z'
updated_date: '2026-11-15T12:00:00Z'
labels:
  - rust
  - tests
milestone: MS-020
dependencies: []
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - REQ-05.008
  - REQ-05.007
  - docs/specifications/refactor-record-layer-for-document-archive-and-status.md
  - docs/specifications/ui-mockups-management.md
documentation:
  - src/ui_mockup.rs
  - src/record/store.rs
  - tests/ui_mockup_cli.rs
modified_files:
  - src/ui_mockup.rs
priority: high
ordinal: 41300
---


## Description
**Why.** `src/ui_mockup.rs` still implements the archive step as `record::store::move_record` (an `fs::rename`) followed by `record::store::write_atomic` on the destination. If the process dies or the write fails between the two steps, the mockup has already left the active folder but still carries `pending`/`approved` in its HTML metadata, and the source no longer exists. `supersede` and the `edit --superseded-by` move path use the same move-then-write order. ADR-0022 decisions 2 and 8 require the opposite: build the patched content, write the destination, then unlink the source, and supersede must reuse that sequence rather than introduce a second archive mechanism. ADR-0018 decision 3 keeps `ContentModel::HtmlMockup` archive/delete in the mockup module as a recorded exception, but not a second archive mechanism.

**What.** Route archive, supersede, and `edit --superseded-by` (edit-with-move) through `record::store::archive_content(ctx, source, destination, content)`: render the full destination HTML first, write the destination with the shared atomic write, then unlink the source. Delete on an active mockup keeps archiving first and then removes the archived file; it must never unlink the active file.

**Behavior contract.**
- `archive <id>` resolves the mockup from the active folder only. It renders the archived HTML (status `archived`, every other field preserved from the source) before any file operation, writes `<archived>/<same file name>`, and only then unlinks the active file. If the destination write fails, the active file keeps its bytes and the command fails; no source write occurs.
- `archive` on a mockup that resolves only from `archived/` is a no-op success: no write, no content change, no modification-time change, same success output as today. An id in neither folder stays not-found (exit 3).
- `supersede <old> <new>` keeps today's self-reference check and successor resolution. It renders `<old>` with status `archived` and the `superseded_by` back-reference, writes that destination, then unlinks the active source, so the back-reference is present in the destination before the source disappears.
- `edit <id> --superseded-by <new>` uses the same destination-write-then-unlink sequence as supersede. An edit without `--superseded-by` stays a single in-place atomic write and unlinks nothing. An edit-with-move that targets a mockup already in `archived/` writes in place and never unlinks, because source and destination are the same path.
- `delete <id>` (with `--force` or after confirmation): an active mockup runs the archive sequence, then removes the archived file; an archived mockup removes the archived file only. No delete path removes an active file directly, and a delete that fails during archiving removes nothing and does not fall back to unlinking the active file.
- Unchanged: the HTML template, parsed and rendered metadata fields, `superseded_by` semantics, status-transition rules and their validation errors, `--json` shapes, human-readable output, and exit codes 0/1/2/3/4.

**In scope.** `src/ui_mockup.rs`: archive, supersede, edit-with-move, and delete paths, including removal of the move-then-write order from all of them. Tests in `tests/ui_mockup_cli.rs` covering the new sequences, both failure points, the no-op case, and a source-level guard against `fs::rename`/`move_record`.

**Out of scope.** HTML metadata fields and the HTML template; status-transition rules; `--status` combined with `--superseded-by`; delete confirmation ordering and its `aborted` output; front matter for mockups (REQ-05.008 forbids it); `record::store::archive_content` itself; the mockup module placement recorded by ADR-0018 decision 3; specification corrections owned by TASK-127; any way to move a mockup out of `archived/`.

**Assumptions.**
- Assumption: tests run as a non-root user. The unlink-failure test makes the active folder read-only (mode 0o555) so the unlink fails after the destination write succeeds. The test first probes whether a read-only folder actually blocks file creation and skips with a printed notice on a root/privileged run where permission bits do not apply.
- Assumption: `archive_content` already creates the destination parent directory and returns a runtime error naming both paths when the source unlink fails, so no store change is required.
- Assumption: `write_atomic`'s lazy front-matter migration never applies to `.html`, because `managed_descriptor` requires a `.md` extension, so HTML bytes are written verbatim.
- Assumption: the active and archived folders are distinct. A resolved source path equal to the destination is written in place with no unlink, so the only copy is never deleted.
- Assumption: delete keeps its current confirmation ordering (archive first, prompt after) and its `{"status":"aborted","id":...}` JSON shape.

**Dependencies.**
- `record::store::archive_content` exists from TASK-107 (completed) and is used unchanged.
- TASK-126 records the ADR-0018 decision-3 exception for mockup archive/delete. Documentation only; it does not block this ticket.

## Acceptance Criteria
- [x] archive writes the rendered archived HTML (status archived, every other field preserved from the source) to archived/<same file name> and unlinks the active source only after that write, using record::store::archive_content exactly once; no fs::rename or move_record remains in src/ui_mockup.rs production code
- [x] supersede sets status archived and the superseded_by back-reference and uses the same archive_content sequence, so the destination contains the back-reference before the active source is unlinked; a supersede whose source is already in archived/ writes in place and unlinks nothing
- [x] edit with --superseded-by uses the same destination-write-then-unlink sequence when a move is required; edit without --superseded-by performs one in-place write_atomic and never unlinks; edit-with-move of an already-archived mockup writes in place without unlinking
- [x] delete on an active mockup archives first and then removes the archived file; delete never calls remove_file on an active path; delete of an already-archived mockup removes only the archived file; a delete whose archive step fails removes nothing and exits 1
- [x] A failure between the destination write and the source unlink leaves the active source byte-identical with its original status, leaves the destination with status archived, and returns a runtime error (exit 1) whose message names both paths; the test injects the failure by making the active folder read-only and skips with a printed notice when a root/privileged run is detected by a permission probe
- [x] A destination-write failure leaves the active source byte-identical, writes no destination file and no leftover temporary file, and exits 1
- [x] Archiving an already-archived mockup is a no-op success: exit 0, unchanged bytes, unchanged modification time, no write, and the existing success JSON
- [x] HTML metadata parsing and rendering, superseded_by semantics, allowed status transitions with their errors, --json shapes, human output, and exit codes 0/1/2/3/4 are unchanged; existing ui-mockup, search, and browser tests pass unmodified
- [x] A source-level guard test asserts that src/ui_mockup.rs production code contains no fs::rename and no move_record call and references archive_content

## Definition of Done
- [x] #1 cargo fmt --check, cargo clippy --all-targets -- -D warnings, and cargo test pass; the new failure-injection tests pass, or skip only with the documented root/privileged notice
- [x] #2 No change to src/record/store.rs, the HTML template, parsed or rendered metadata, JSON shapes, output text, or exit codes; no new dependency and no new public item
- [x] #3 The final summary lists the four call sequences and the store call each one uses, both injected failure points with their observed evidence, and the no-op byte and modification-time evidence
- [x] #4 No mockup file is unlinked before its destination write succeeds, and no delete path removes an active file


## Implementation Notes
1. `archive(ctx, scope, id)`: keep the current resolution order. (a) `find_record(active)` returns None and `find_record(archived)` finds the id: return the existing success output without reading or writing. (b) Both None: keep NotFound. (c) Active found: read the source once, parse it, keep the existing superseded-mockup validation, set `status = archived`, and render the record once. Resolve `<archived>/<source file name>` with `Context::ensure_within`. If that destination equals the source path, use `write_atomic` only; otherwise call `record::store::archive_content(ctx, &source, &destination, &rendered)` exactly once. Delete the move-then-write block and the post-move re-read/re-parse. Output is unchanged.
2. `supersede(ctx, scope, old, new)`: keep the current self-reference check, successor resolution, and resolution order for `old`. Parse `old` once, keep the existing id/title/status defaults, set status `archived` and `superseded_by = new`, render once. Use `archive_content` when `old_path != destination` and `write_atomic` when they are equal (old already archived). Output is unchanged.
3. `edit(...)` with `--superseded-by`: keep the current field edits, status validation, and successor resolution. Render the final record once. Replace the conditional `move_record` followed by `write_atomic` with: `archive_content` when a move is required and `path != destination`; otherwise a single `write_atomic` (covers a plain edit and an edit-with-move of an already-archived mockup). Run the editor afterwards against the destination path, as today. A plain edit never calls `archive_content`.
4. `delete(ctx, scope, id, force)`: resolve the active record; when present, call `archive` first and then resolve the archived path with `find_record(archived)` and remove that resolved path (not a joined `<id>.html` guess). When only the archived record exists, remove the resolved archived path. Remove nothing when either step fails; keep the confirmation prompt, its ordering, and both JSON shapes.
5. After the four paths, `src/ui_mockup.rs` production code must contain no `move_record` call and no `fs::rename`; all file removal is limited to a path that was already written by `archive_content`.
6. Tests in `tests/ui_mockup_cli.rs`, reusing the existing `tmpdir`/`init`/`run` helpers:
   a. Sequence success: archive, supersede, and edit-with-move each leave only the archived file, with `<!-- status: archived -->` and the expected `Superseded by:` line where applicable; plain edit leaves the record in place and removes nothing.
   b. Destination-write failure: pre-create `<archived>/MOCK-001.html` as a directory, run archive; assert exit 1, the active source byte-identical, no destination file, and no `.tmp` leftover in `archived/`.
   c. Unlink failure: create a permission probe (temp dir at 0o555, attempt a file create); if the probe still allows creation, print a skip notice and return. Otherwise chmod the active mockups folder to 0o555, run each of archive, supersede, edit-with-move, and delete `--force`; restore the mode immediately; assert exit 1, stderr naming both the source and the destination path, the source byte-identical with its original status, the destination present with `<!-- status: archived -->`, and for delete no file removed.
   d. No-op: archive an already-archived mockup, assert exit 0, the success JSON `{"status":"ok","id":...}`, and identical bytes plus unchanged modification time before and after.
   e. Delete: active delete removes both the active path and the archived copy; archived delete removes only the archived copy; deleting an unknown id stays exit 3.
   f. Guard: read `src/ui_mockup.rs` with `include_str!` and assert the production text contains no `move_record` and no `fs::rename`, and that `archive_content` is referenced.
7. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, the targeted `cargo test --test ui_mockup_cli`, and the full `cargo test` to confirm the existing ui-mockup, search, browser, and no-drift suites still pass.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
UI-mockup archive-class moves now use record::store::archive_content (write destination, then unlink source); no fs::rename/move_record remains in src/ui_mockup.rs.

Call sequences: archive — resolve active only, archived hit returns no-op success with no read/write/mtime change, both missing NotFound, parse once, status archived, render once, destination archived/<same file name>, one archive_content call (in-place write_atomic only when source == destination). supersede — same sequence with status archived and superseded_by present in the destination before the unlink; already-archived source written in place. edit --superseded-by — archive_content when a move is required, plain edit a single write_atomic with no unlink, editor runs against the destination; archived edit-with-move writes in place. delete — active archives first, then removes the resolved archived path; archived-only removes that path; never unlinks an active path; archive failure removes nothing.

Failure injection: destination-write failure (archived path pre-created as a directory) → exit 1, source byte-identical, no destination, no temp leftovers, for archive/supersede/edit/delete. Unlink failure (active dir chmod 0o555) → exit 1, stderr names both canonical paths, source intact with original status, destination archived, no temp leftovers.

No-op archive: exit 0, same success JSON, bytes and mtime identical.

Preserved: HTML parsing/rendering/template, superseded_by semantics, transitions/errors, --json shapes, text output, exit codes 0/1/2/3/4. Evidence: ui_mockup_cli 18/18 (6 new), full suite 842 passed / 0 failed / 22 ignored; clippy and fmt clean; store.rs untouched.
