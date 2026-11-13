# Feature Specification — Remove Legacy Record Serializers and Enforce the No-Drift Rule

**Ticket:** TASK-97
**Milestone:** m-16 (Release 1.0)
**Status:** Implemented
**UI mockups:** none (refactor only; no user-visible change)

## Purpose

Deliver ADR-0013 Stage 7: delete every remaining legacy per-type record parser and writer and the
browser's private record write path, then make the no-drift rule mechanically checkable so parsing,
identity, projection, and writing each have exactly one implementation under `src/record/`. This is
the removal and enforcement half of NFR-04.004 and implements REQ-05.011, REQ-06.004, NFR-04.001,
and NFR-04.002 through ADR-0013 decisions 2–7.

The ticket is a refactor with no intended change to command output, JSON shapes, browser pages,
validation messages, HTTP status codes, or exit codes. It closes the release claim that Papyrus has
one record reader, one patcher, one ID implementation, and one writer. End-to-end verification and
packaging stay in TASK-98.

## Scope

### In scope

- Delete the browser's private record path in `src/browser/records.rs`: the `Doc`/`Section` document
  model, `parse_doc`, per-type `load_*` parsers, its own index/resolution and ID helpers, its own
  version token, the `apply_*_edit` serializers, and `records::write_atomic`.
- Rebuild browser reads and edits on `record::document`, `record::store`, `record::resolve` and the
  descriptors, `record::identity`, `record::frontmatter`, `record::patch`, and `record::view`.
- Rewire `src/browser/mod.rs` edit routes and `src/browser/render.rs` display to the shared reader,
  patcher, and projection.
- Remove the last module-local record parsing and formatting outside `src/record/`: the raw `## Release`
  section scan in `src/task.rs`, the duplicated decision-prefix matching in `src/decision.rs`, the
  padded-ID example formatting in `src/browser/render.rs`, and any equivalent helper found by the
  no-drift checks.
- Extend `tests/no_drift.rs` from three checks to the full ADR-0013 decision 7 set, including
  storage-ID symbols, offsets, storage prefixes, hardcoded record directories, Backlog-shaped stem
  detection, duplicate record writers, and duplicate section splitting.
- Update this specification and its traceability.

### Out of scope

- Any change to routes, pages, layout, JSON fields, wording, validation rules, or exit codes.
- The UI mockup HTML serializer (`src/ui_mockup.rs::render_record`) and mockup HTML editing, which
  ADR-0012 and ADR-0013 decision 2 exempt because mockups are HTML and carry no front matter. Mockup
  writes already use `record::store::write_atomic` and are verified, not rebuilt.
- The import's Backlog.md body mapping in `src/import/`, which ADR-0013 decision 1 explicitly allows
  and requires to be expressed through the shared writer.
- Search specification text: the `docs/specifications/search-*.md` files already state that matching
  runs over whole persisted text including front matter, and `src/search.rs` already reads through
  `record::document`/`record::resolve`.
- Corpus migration commands, configuration/init writes, and test-fixture writes.
- TASK-98 verification and release packaging.

## Delivered state (2026-11-13)

| Surface | State |
|---|---|
| CLI record modules | Create/edit paths use `record::frontmatter`, `record::document`, `record::patch`, and `record::store::write_atomic`. The raw `## Release` scan in `src/task.rs` is removed; `src/decision.rs` uses `record::identity` for prefix matching. |
| `src/browser/records.rs` | A render-only view model built from `record::document`, `record::view`, and the front matter. The local `Doc`/`Section` model, `parse_doc`, per-type `load_*` parsers, local index/ID helpers, version token, `apply_*_edit` serializers, and `write_atomic` are deleted. |
| `src/browser/mod.rs` | Every save builds one patch list through `src/record/edit.rs`, applies it through `record::patch::apply_patch`, and writes once through `record::store::write_atomic`. The `has_front_matter` dual branch is deleted. |
| `src/browser/render.rs` | Renders from the shared document and `RecordView`; the `id_padding` example is built through `record::identity`. |
| `tests/no_drift.rs` | Eight independent checks cover record writes and renames, delimiter construction, ID formatting and prefix parsing, storage-ID symbols, hardcoded directories, Backlog-shaped stems, a second writer, and a second section parser. Each check was proven able to fail with a reintroduced violation. |
| `src/record/*` | `document`, `frontmatter`, `patch`, `identity`, `store`, `resolve`, `descriptor`, `view`, `migrate`, and the browser patch builders in `edit` are the single implementations. |

## Behavior

### 1. Single parser

1. `record::document::parse` is the only parser of managed Markdown records used by CLI modules,
   browser handlers, search, and adoption.
2. No module outside `src/record/` splits a record into `## ` sections, detects a leading YAML block,
   parses YAML, or extracts a heading from a managed record. The browser has no second document model.
3. Typed per-type fields (for example `description`, `rationale`, `requirements`, acceptance criteria,
   comments) are extracted from `record::document::Document` or `record::frontmatter`; extraction
   helpers live in the record module or in the owning type module and never re-render a whole file.

### 2. Single mutation and rendering

1. Every browser metadata change is a front-matter `PatchOp`; every free-text change is a section
   `PatchOp`; all changes in one save are applied by one `record::patch::apply_patch` call.
2. A title edit patches the front-matter `title` and, for record types that keep one, the body heading
   line in the same write, so the two cannot diverge.
3. No managed record module or browser handler keeps a `render_*`/`apply_*_edit` function that
   assembles a whole record file. Patch construction helpers, where needed, live under `src/record/`.
4. UI mockup HTML rendering and mockup HTML editing are the only non-front-matter serializer
   exception; they must not construct or parse a `---` delimiter.
5. A save whose patches change no byte performs no write and does not refresh `updated_date`.

### 3. Single identity

1. `record::identity` is the only location of ID parsing, validation, formatting, and allocation.
2. The browser accepts and produces Papyrus IDs only. Routes, links, form values, JSON, and redirects
   carry the canonical Papyrus ID; no prefix stripping, digit validation, zero-padding, or form
   detection is reimplemented in the browser. Discovery and resolution use the record descriptors and
   `record::resolve`.
3. Path construction that appends `.md` after an ID produced by `record::identity` is allowed; it is
   path construction, not identity formatting.

### 4. Single writer

1. Every create, edit, archive, restore, deprecate, supersede, delete, and mockup write on a managed
   record uses `record::store::write_atomic`.
2. No production code outside `record::store` writes a record file or performs a record-file
   temporary-file/rename sequence. `fs::write` remains permitted only for non-record files in
   `src/config.rs` and `src/init.rs`, and for test fixtures under `#[cfg(test)]`.
3. Atomic write failure leaves the original file byte-identical and removes temporary files.

### 5. Browser parity and concurrency

1. Browser list/detail rendering, edit forms, redirects, JSON, and HTML keep their current field set,
   wording, and layout; the change is internal.
2. A browser save preserves the file byte-for-byte except for the fields the save changes: unknown
   sections, preamble, front-matter key order, quoting, and unrelated body text are preserved.
3. The ADR-0006 contract is unchanged: the version token comes from `record::store::version_token`;
   missing or malformed `version` is `400` before any write; a stale token is `409` with
   "record changed on disk; reload the form" and no write; field validation is `422`; a path escape is
   `404`; `POST` success is a redirect whose target re-reads the file.
4. Line endings are normalized to LF and the file ends with one trailing newline, as the browser
   specifications already require.
5. Reading or saving a managed record that lacks front matter adopts it through the shared writer
   (REQ-05.011, ADR-0012 decision 9): the body is unchanged apart from the added block, adoption uses
   the existing `record::migrate` semantics, and a subsequent read or save is idempotent. This
   replaces the browser's current non-migrating branch; an adoption write failure is reported and
   leaves the file byte-identical.

### 6. One projection

1. Browser list/detail JSON and CLI `--json` obtain metadata from `RecordView`; `metadata.id` is the
   Papyrus ID and there is no storage-ID field, display/storage translation pair, or second identity
   on any surface.
2. Display fields that the browser renderer needs are derived from the shared document and front
   matter; the browser does not re-parse metadata from the body.

### 7. No-drift checks

`tests/no_drift.rs` scans production `.rs` files under `src/` (excluding `#[cfg(test)]` modules) and
fails the default `cargo test` run when any of the following is found. Each allowlist is documented in
the test and is the minimum required:

| # | Condition | Allowlist |
|---|---|---|
| 1 | `fs::write` or `std::fs::write` on a record path | `src/config.rs`, `src/init.rs` |
| 2 | `format!("---` or equivalent delimiter construction | `src/record/` |
| 3 | ID formatting: prefix plus zero-padding/width (`"{}-{:0`, `"{}.{:0`, `{:0width`), and any `format!("{}-", <prefix>)` used to form or match a record ID | `src/record/identity.rs` |
| 4 | Storage-ID translation or storage-layer symbols: `to_storage_id`, `to_display_id`, `storage_prefix`, `storage_id`, storage offset constants, `backlog/config.yml` | none (symbols must not exist) |
| 5 | Hardcoded record-directory literals (`backlog/tasks`, `backlog/milestones`, `backlog/releases`, `archive/tasks`, `archive/milestones`, `tasks/archived`, `milestones/archived`, `decisions/archived`, `deprecated/`) | `src/record/descriptor.rs`, `src/config.rs` defaults, `src/init.rs` output, `src/import.rs` source reading |
| 6 | Reimplemented Backlog-shaped stem detection or storage prefix construction (`is_backlog_shaped`, lowercase-prefix comparison, storage prefix literals) | `src/record/identity.rs`, `src/record/descriptor.rs` |
| 7 | A second record writer: `fn write_atomic` outside the record store, or a record-file `.tmp` + `rename` sequence | `src/record/store.rs` |
| 8 | A second parser: splitting a record on `## `, `strip_prefix("## ")`, or detecting front-matter delimiters | `src/record/`, `src/import.rs` (Backlog.md body mapping) |

Rules for the checks:

1. Checks run as ordinary `#[test]` functions with no feature flag and are not `#[ignore]`d.
2. Violations report `file:line: text` so the fix is obvious.
3. Each check is validated by temporarily reintroducing a representative violation and observing the
   suite fail; the proof is recorded in implementation notes and does not remain in the tree.
4. The checks stay independent of the number of record types; adding a record type must not require
   changing them.

## Inputs and outputs

- Input: project `Context`, resolved record paths, and existing CLI/browser command and form values.
- Output: unchanged command output, JSON shapes, HTML, HTTP status codes, and exit codes.
- No new command, option, route, form field, or user-facing field.

## Validation and error behavior

- Existing validation, not-found, conflict, and runtime error behavior is unchanged.
- No-drift check failures fail `cargo test` with a non-zero exit status and name every violation.
- Atomic write and path-safety failures keep their existing messages and exit codes.

## Assumptions

- Assumption: the browser's legacy non-migrating write branch exists only for pre-front-matter files
  and can be removed; routing saves through `record::store` makes browser saves adopt front matter as
  REQ-05.011 requires. Adoption on browser read is therefore a documented behavior change from the
  current code, not a new requirement.
- Assumption: the browser view model may keep a `Record`-shaped struct as a render-only struct, but it
  must be populated from the shared document, front matter, and projection and must not contain
  parsing, ID, path, version, or write logic.
- Assumption: patch helpers for type-specific browser edits may live in one `src/record/edit` module
  rather than one module per type; either layout is acceptable if no handler owns serialization.
- Assumption: UI mockup HTML rendering and editing stay outside `src/record/` and are excluded from
  the no-drift parser and serializer checks.
- Assumption: `src/record/descriptors.rs` (mentioned by the superseded version of this specification)
  does not exist in the current tree; `src/record/descriptor.rs` is the sole descriptor source.
- Assumption: no new dependency is added.
- Assumption: the existing behavior tests are the compatibility contract; only tests that assert
  removed internal APIs are rewritten.

## Open questions

None blocking. OQ-1 (edit helper layout) and OQ-2 (mockup renderer placement) from the superseded
version are resolved by the assumptions above and by ADR-0013 decision 2. OQ-3 (scanning `#[cfg(test)]`)
is resolved: test modules stay excluded, as documented in the test allowlist.

## Verification

1. `cargo test` passes, including the browser integration suite (`tests/browser_cli.rs`), migration,
   import, search, and `tests/no_drift.rs`.
2. `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
3. A manual grep audit finds no `fs::write`/`fs::rename` on record paths outside `src/record/`, no
   `format!("---` outside `src/record/`, no `apply_*_edit`/`parse_doc`/`records::write_atomic` symbols,
   and no storage-ID or translation symbols.
4. Reintroducing one sample violation per no-drift check fails the suite.
5. `git status` shows no modified record corpus file after the test run; adopting a fixture legacy
   record through the browser adds front matter and leaves the body byte-identical on the next read.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-97 |
| Requirements | REQ-05.010, REQ-05.011, REQ-06.004, REQ-01.005, NFR-04.001, NFR-04.002, NFR-04.004 |
| ADRs | ADR-0006, ADR-0012, ADR-0013 (Stage 7, decisions 2–7) |
| Specifications | `browser-*.md` persistence and ID contracts (revised under TASK-86), `papyrus-native-record-layout-and-single-id-space.md`, `import-backlog-md-records.md` |
| Related tickets | TASK-88, TASK-89, TASK-92, TASK-93, TASK-94, TASK-95, TASK-96, TASK-98 |
| Source | `src/browser/records.rs`, `src/browser/mod.rs`, `src/browser/render.rs`, `src/task.rs`, `src/decision.rs`, `src/record/*`, `tests/no_drift.rs` |
| Documentation history | Supersedes the TASK-70 version of this specification (ADR-0009); revised 2026-11-13 for ADR-0013 Stage 7 and TASK-97 |
