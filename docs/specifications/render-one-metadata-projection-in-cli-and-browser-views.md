# Feature Specification — Render One Metadata Projection in CLI and Browser Views

**Ticket:** TASK-67
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0010`](ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html) (approved) — UI description [`ui/browser-front-matter-and-lifecycle-ui.md`](ui/browser-front-matter-and-lifecycle-ui.md); extends approved baselines MOCK-0001..MOCK-0009. No new mockup is required for this ticket.

## Purpose

Deliver ADR-0009 decision 5: a single `RecordView` projection that is the only source for record metadata in every read surface, so that CLI list/view text and `--json`, browser list/detail JSON, and browser HTML present identical metadata values for the same record. This ticket implements REQ-06.004 (CLI and browser read front matter), REQ-05.010 (front-matter field map), REQ-05.011 (metadata maintained through all commands), and REQ-01.005 (display/storage ID split), on top of the shared envelope and identity translation delivered by TASK-63 and TASK-64 and the option exposure delivered by TASK-65. It does not migrate the corpus (TASK-68, TASK-69), remove legacy serializers (TASK-70), or add the lifecycle page (TASK-72).

## Scope

### In scope

- Definition of a single `RecordView` type (or equivalent shared projection) that exposes:
  - the record identity as the Papyrus display ID;
  - a `metadata` object that is the ordered map of the record's front-matter keys exactly as stored (general keys first, then type-specific keys), where `metadata.id` is the Backlog.md storage ID;
  - the typed document fields already rendered by each surface (title, description, status, milestone, assignee, labels, dependencies, acceptance criteria, comments, release, audit type, superseded_by, and the other type-specific fields).
- Refactoring every CLI `list`/`view` text renderer and every `--json` serializer for every managed record type to render from `RecordView` instead of re-parsing or re-formatting metadata.
- Refactoring every browser list/detail JSON serializer and every browser HTML renderer to render from the same `RecordView`.
- Adding the `metadata` object to CLI `--json` output and to browser list/detail JSON output, in canonical front-matter key order.
- Keeping the top-level identity field as the display ID in every surface; the storage ID appears only inside `metadata` and in labelled diagnostics.
- Keeping the browser editable field set unchanged; `created_date` and `updated_date` are rendered read-only (disabled inputs) in the browser edit form.
- Routing every browser save through the shared patcher/writer so that a metadata-backed edit writes the new value into front matter, preserves the other front-matter keys and the rest of the file, refreshes `updated_date`, and never copies metadata into the body.
- Parity, round-trip, read-only-timestamp, and identity-position tests.
- Correction of the three browser specification pages that currently describe front matter as content preserved content-identically (see section 7).

### Out of scope

- Lazy migration of document or task/milestone records — TASK-68, TASK-69.
- Removal of legacy per-module parsers/writers and no-drift grep enforcement — TASK-70.
- Backlog.md round-trip verification — TASK-71.
- The managed-artifacts-and-lifecycles browser page — TASK-72.
- Complete browser access for all content — TASK-19 (this ticket only supplies the projection it consumes).
- UI mockup changes — MOCK-0010 is already approved and is reused.
- New record types, new front-matter keys, or changes to record identity rules.
- Corpus migration or any write that changes an existing record body beyond the front-matter patch.

## Behavior

### 1. `RecordView` projection

1. A single `RecordView` type (or an equivalent shared projection) is defined in `src/record/` and is the only type used to carry record metadata from the reader to any renderer. No surface re-parses front matter or re-derives metadata after the reader returns.
2. `RecordView` contains at least:
   - `id: String` — the Papyrus display ID. This is the record identity on every surface.
   - `metadata: Vec<(String, Value)>` (or an equivalent ordered map) — the front-matter keys exactly as stored, in canonical key order (general keys first, then type-specific keys). `metadata.id` is the Backlog.md storage ID. Unknown foreign keys are retained in their stored position.
   - the typed document fields the surfaces already render for the record type (title, description, status, milestone, assignee, labels, dependencies, acceptance criteria, comments, release, audit type, superseded_by, documentation, ordinal, type, and the general fields). These fields are projections of the same parsed document, not a second parse.
3. `RecordView` also carries the ADR-0006 version token (`version: String`) for browser surfaces; CLI surfaces may ignore it. The version token is not part of `metadata`.
4. The reader builds one `RecordView` per record read, applying identity translation (TASK-64) so that `id` is the display ID and `metadata.id` is the storage ID. No other code path translates IDs for display.
5. A record that lacks front matter is still projected: the reader's legacy fallback supplies the metadata fields from the document body/heading, and the projection marks the record as unmigrated only through the existing migration path (TASK-68/TASK-69). This ticket does not add migration.

### 2. CLI rendering

1. `papyrus <type> list` and `papyrus <type> view` plain-text output is rendered from `RecordView`. The metadata values printed are taken from `RecordView`, not from a second parse of the file.
2. CLI list output continues to print at least the ID and title of each record; where the command already prints additional metadata (status, milestone, assignee, etc.) it prints the same values as before, now sourced from `RecordView`.
3. `papyrus <type> view` prints the full record. In addition to the existing body rendering, the metadata block (id, title, created_date, updated_date, labels, references, plus type-specific keys) is available to the renderer from `RecordView`. The exact plain-text layout of the metadata block follows the existing per-type view layout; this ticket does not redesign CLI text formatting.
4. `--json` output for every managed record type gains a `metadata` object whose keys are the front-matter keys in canonical order and whose values are the stored values (strings, lists, or null as stored). The top-level `id` field remains the display ID and is not replaced or duplicated by `metadata.id`.
5. The storage ID appears in `--json` only as `metadata.id` and in labelled diagnostics; it never appears as the top-level `id` or as any other top-level identity-like field.
6. `--json` remains a stable public contract: existing keys keep their meaning and type; `metadata` is additive only. Changes to `metadata` are additive only (ADR-0009 decision 5).
7. A no-op read (a record already carrying front matter) produces byte-identical file content and an identical `RecordView` on repeated reads.

### 3. Browser rendering

1. Browser list and detail JSON (`GET /api/...`) for every managed record type is rendered from `RecordView`. The JSON contains the same `metadata` object as the CLI `--json` output for the same record, plus the existing browser-specific fields (path, version, and the typed fields the browser form already uses).
2. Browser HTML list and detail pages render metadata from `RecordView`: list rows show at least id, title, created_date, updated_date, labels, references, and the type-specific keys in a metadata line beneath the primary link; detail pages show a metadata block (`dl`) with Record ID, Title, Created date, Updated date, Labels, References, plus type-specific keys (per MOCK-0010 section 4).
3. The browser editable field set is unchanged from the existing per-type edit forms. `created_date` and `updated_date` are rendered read-only (disabled inputs, clearly marked "(read-only)") in the edit form and are never accepted from the request.
4. Timestamps are rendered as the quoted ISO-8601 UTC strings stored in front matter, without reformatting, unless a human-readable secondary is added (MOCK-0010 OQ-4 default: as stored).
5. Labels and references are rendered as tags in view and as a tag add/remove interaction in edit, reusing the existing interaction (MOCK-0002/MOCK-0010).
6. The browser never copies any `metadata` value into the document body on render or on save.

### 4. Browser save through the shared writer

1. Every browser `POST` (HTML form and JSON API) for a metadata-backed record is handled by reading the record into a `RecordView`, building a patch through `record::patch` (front-matter `SetFrontMatter`/`ClearFrontMatter` for changed metadata fields; section patches for changed body fields), and writing through `record::store::write_atomic`.
2. A save that changes a metadata-backed field writes the new value into the front matter only. It does not rewrite the `# <ID> <title>` heading from metadata unless the title field itself changed, in which case both the front matter and, where the record type keeps one, the heading line are patched in the same write (ADR-0009 decision 3).
3. A save preserves every front-matter key the operation does not change, including unknown foreign keys, in their stored order and quoting.
4. A save preserves the body, marker comments, unknown sections, preamble, and line endings byte-for-byte apart from the patched sections.
5. A save refreshes `updated_date` to a single UTC timestamp obtained for that write (ADR-0009 decision 9). `created_date` is never changed by a save.
6. A no-op save (no field changed) performs no write and does not refresh `updated_date`.
7. A save never copies metadata back into the document core (no retired metadata section is created or updated).
8. Optimistic concurrency (ADR-0006) is unchanged: missing/malformed `version` → 400, stale token → 409 with "record changed on disk; reload the form", matching token proceeds. After a successful write the server keeps no cached copy and the redirect target re-reads the file (save-then-reread).
9. `created_date` and `updated_date` are never accepted from the request; a submitted value for either is ignored (not an error), matching the exemption in REQ-05.011.

### 5. Display/storage ID handling

1. The top-level identity on every surface is the display ID. The storage ID never appears in the identity position of any CLI text line, CLI `--json` top-level field, browser JSON top-level field, or browser HTML heading/primary link.
2. `metadata.id` is the storage ID and is the only place the storage ID may appear in structured output. Labelled diagnostics may also print it with an explicit `storage id:` label and the file path.
3. ID-typed metadata fields that store storage IDs (`milestone`, `dependencies`, `release`, `superseded_by`) are rendered to users as display IDs in CLI text and browser HTML. In the raw `metadata` object (CLI `--json` and browser JSON) they remain in storage form, consistent with `metadata.id`. A renderer that needs the display form applies the same translation uniformly.
4. Reference resolution (whether a milestone or dependency resolves) uses the storage form internally and displays the resolved record's display ID.

### 6. Read-only timestamps and field exemption

1. `created_date` is set at creation and is never changed by any path, including browser save, migration, or edit.
2. `updated_date` is refreshed on every real write, including a browser save of a metadata-backed field, and is not refreshed by a no-op.
3. `id`, `created_date`, and `updated_date` are exempt from the option rule: no command or form accepts them as input, and no save applies a submitted value.
4. Timestamps are quoted ISO-8601 UTC strings of the form `'YYYY-MM-DDTHH:MM:SSZ'` on disk and are rendered verbatim (without quotes in JSON strings) in CLI/browser output.

### 7. Browser specification page corrections (prerequisite)

The following browser specification pages currently state that "front matter" is preserved content-identically on a browser save. After this ticket those statements are inaccurate, because front matter is now the authoritative, patched metadata store rather than opaque preserved content. They must be corrected so that the preservation contract names the patched front-matter keys and the rest of the file, not "front matter" as a blob:

- `docs/specifications/browser-audit-records.md`
- `docs/specifications/browser-business-rules.md`
- `docs/specifications/browser-technical-specifications.md`

Correction is owned by `business-analyst`/`tech-writer` and is a prerequisite for implementation. The corrected wording states that a save patches the front-matter keys it changes, preserves the other front-matter keys and their order/quoting, preserves the body and unknown sections, and never copies metadata into the body. No other browser spec page claims front-matter preservation.

### 8. Error behavior

1. A missing record produces a not-found error (CLI exit code 3; browser 404) consistent with existing commands.
2. A malformed front-matter block produces a validation error (CLI exit code 4; browser 422/400) rather than a panic.
3. A stale version token on browser save produces 409 with "record changed on disk; reload the form" and no write.
4. A browser save that fails atomically leaves the original file byte-identical and removes any temporary file.
5. A path that escapes the project directory is rejected before any read or write (exit code 1 / 404).

## Inputs and outputs

- Input: a project `Context` and a record path or descriptor-claimed location; a `RecordView` built by the shared reader.
- Output: CLI text and `--json` (with `metadata` object); browser list/detail JSON (with `metadata` object); browser HTML rendered from `RecordView`; patched file text on save.
- No new command or route is added by this ticket.

## Validation and error behavior

- Missing record → exit code 3 (CLI) / 404 (browser).
- Malformed front matter → exit code 4 (CLI) / 400/422 (browser).
- Stale version → 409 (browser), no write.
- Atomic write failure → exit code 1 (CLI) / 500 (browser); original file unchanged.
- Path escape → exit code 1 (CLI) / 404 (browser).

## Assumptions

- Assumption: `RecordView` is added to `src/record/` (likely `src/record/view.rs` or as part of `src/record/document.rs`/`mod.rs`) and is the only projection type; no parallel projection is introduced.
- Assumption: the shared envelope, patcher, store, and identity modules from TASK-63/TASK-64 are available and are the only read/write path used; no surface re-parses front matter.
- Assumption: option exposure from TASK-65 is complete, so every metadata-backed field that can change already has a CLI option and a browser form field where applicable.
- Assumption: MOCK-0010 is the approved UI source for metadata display and is reused unchanged; no new mockup is required.
- Assumption: the three browser spec pages listed in section 7 are corrected by `business-analyst`/`tech-writer` before implementation; this ticket's tests reference the corrected wording.
- Assumption: `--json` `metadata` is a stable public contract; changes are additive only (ADR-0009 decision 5, resolved 2026-11-08).
- Assumption: no corpus migration occurs in this ticket; unmigrated records are projected through the legacy fallback and migration is delivered by TASK-68/TASK-69.

## Open questions

- OQ-1: Whether `RecordView` is a generic struct with an `extra: BTreeMap` for type-specific fields or a per-type enum variant. Either is acceptable provided there is exactly one projection type and no surface re-parses metadata. (Implementation choice; not blocking.)
- OQ-2: Whether the CLI plain-text metadata block is printed as a separate labelled section or inline with the existing per-type fields. The existing per-type view layout is retained; only the data source changes. (Not blocking.)
- OQ-3: Whether browser list rows show all metadata keys or the subset named in MOCK-0010 section 4 ("at least id, title, created_date, updated_date, labels, references, plus type-specific keys"). MOCK-0010 requires at least those; showing more is allowed. (Not blocking.)
- OQ-4: Whether ID-typed metadata fields in the raw `metadata` object should be accompanied by a display-form alias (for example `metadata_display`). ADR-0009 decision 5 forbids a second top-level identity field; an alias inside `metadata` would be a contract change and is deferred unless a consumer requires it. (Not blocking; default: no alias.)

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-67 |
| Requirements | REQ-06.004, REQ-05.010, REQ-05.011, REQ-01.005, REQ-06.002, REQ-06.003 |
| ADRs | ADR-0006, ADR-0007, ADR-0008, ADR-0009 (decision 5) |
| Related tickets | TASK-63, TASK-64, TASK-65, TASK-66, TASK-68, TASK-69, TASK-70, TASK-72, TASK-19 |
| Source | `src/record/` (`frontmatter`, `document`, `patch`, `store`, `identity`), `src/browser/records.rs`, per-type CLI modules, `src/browser/mod.rs` |
| UI | `docs/specifications/ui/browser-front-matter-and-lifecycle-ui.md`, `docs/specifications/ui/mockups/MOCK-0010-browser-front-matter-and-lifecycle-review.html` |
| Tests | `tests/browser_cli.rs`, module unit tests in `src/record/` and `src/browser/records.rs` |
