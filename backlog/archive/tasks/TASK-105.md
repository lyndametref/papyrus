---
id: TASK-105
title: Implement the generic document handler for every document type
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
updated_date: '2026-11-14T09:27:00Z'
labels:
  - rust
milestone: MS-018
dependencies:
  - TASK-100
  - TASK-102
documentation:
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/requirements/business/REQ-007.004.md
  - docs/requirements/business/REQ-007.001.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - docs/requirements/business/REQ-007.002.md
  - docs/requirements/business/REQ-007.003.md
  - docs/requirements/business/REQ-005.007.md
  - docs/requirements/business/REQ-005.010.md
  - docs/requirements/business/REQ-005.011.md
  - docs/requirements/business/REQ-006.004.md
  - docs/requirements/business/REQ-006.014.md
priority: high
ordinal: 41020
---


## Description
Implement the single descriptor-parameterized document handler in `src/document.rs` for the baseline family (`create`, `list`, `view`, `edit`, `archive`, `delete`, `status`) used by every document type. It consumes the registry `TypeDescriptor` (TASK-105) and the record-layer primitives (TASK-107); no slug literals or per-type storage/ID logic.

Generic (config-defined) types use the seven-field content model of ADR-0018: `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, `status`, plus a generated heading and an optional `## Description`. Built-in typed types keep their existing field sets and sections for the baseline verbs, and delegate only `create-group`/`--group` and `supersede` to type-specific code. `status` is never a create/edit option.

Scope: add `src/document.rs` and its module declaration; define `DocumentInvocation` and `run(ctx, registry, invocation)`; implement ID allocation, storage, status handling, auto-relocate awareness, and typed-extension delegation. Out of scope: registry construction (TASK-105), command-tree cutover (TASK-108), validation (TASK-109), browser/search wiring (TASK-113/114), and per-type option metadata (`--view`, `--rationale`, etc., deferred to TASK-108/114).

## Acceptance Criteria
- [x] src/document.rs exposes DocumentInvocation and pub fn run(ctx, registry, invocation) for Create, List, View, Edit, Archive, Delete, and Status; the descriptor is resolved from the registry and no slug literals or slug match arms appear in the module.
- [x] Generic create requires a non-empty sanitized title, accepts an optional description and repeatable labels/references, allocates an ID as <prefix>-<zero-padded sequence> using cfg.id_padding across active and archived locations, and writes active/<id>.md with exactly the front-matter keys id, title, created_date, updated_date, labels, references, status plus the generated heading and an optional Description section.
- [x] A new generic record starts at the type's default_status or active; status is not accepted as a create or edit option and is never written by those paths.
- [x] Generic edit patches title in front matter and the heading, the Description section, and appended labels/references; preserves unknown sections and keys; supports --editor for the description only; refreshes updated_date only when something changes; performs no write when nothing changes.
- [x] Generic view reads an active record and returns its content and metadata; list reads active records only, sorted by id, and hides archived records.
- [x] Archive, status, and delete delegate to record::store::archive, record::store::set_status, and record::store::delete: archive sets archived and moves the file idempotently; status accepts active or a declared custom status, rejects archived and undeclared values, and moves no file; delete archives an active record first and then removes it.
- [x] All record reads and writes go through record::document, record::patch, record::identity, and record::store; src/document.rs adds no direct record file write or front-matter formatting, and record content is not re-rendered outside the patch path.
- [x] Generic records are stored under doc_directory plus the entry directory and archive into that folder's archived/ subfolder.
- [x] Built-in Typed descriptors keep their existing field sets, sections, and verbs for the baseline commands; the generic model adds no field to them, and only create-group/--group and supersede are delegated to type-specific code.
- [x] Typed baseline verbs reuse the same storage, ID, patch, archive, delete, and status flow through the descriptor and TypedSchema, with validation and section mapping supplied by the owning type module.
- [x] Read paths used by view, list, edit, archive, delete, and status auto-relocate an active-folder record whose status is archived.
- [x] An undeclared status value is preserved verbatim, flagged in diagnostics, and makes the record read-only for status writes until the value or configuration is fixed.
- [x] A record with a hand-added section or unknown front-matter key keeps it through create/edit/view.
- [x] Tests cover a user-added generic type end to end (create, list, view, edit, archive, delete, status), a custom status vocabulary and default, an undeclared status, a hand-added section, and the typed req/nfr/ui-mockup paths.

## Definition of Done
- [ ] #1 src/document.rs compiles and cargo build succeeds.
- [ ] #2 All acceptance criteria have automated tests; cargo test and cargo clippy are clean.
- [ ] #3 No slug literals or slug match arms in src/document.rs (review/grep check).
- [ ] #4 All record writes go through record::store; no direct fs::write of record content and no front-matter reformatting.
- [ ] #5 Unit and integration tests listed in the implementation plan exist and pass.
- [ ] #6 Open questions and deferred items are recorded in the task notes with owning tasks.


## Implementation Notes
# Implementation Plan

## 0. Scope and boundaries

- New module `src/document.rs` (plus `pub mod document;` in `src/lib.rs`). No slug literals and no `match` on a slug string anywhere in the module; every type-specific fact comes from the `TypeDescriptor` resolved through the registry.
- Consumes `ArtifactRegistry` and `TypeDescriptor` (TASK-105) and the record-layer primitives `record::document`, `record::patch`, `record::identity`, `record::store`, `record::resolve`, `record::view`, `record::migrate` (TASK-107). No per-type storage or ID logic is reimplemented.
- Implements the baseline verbs for every document descriptor. `ContentModel::Generic` gets the full seven-field model of ADR-0018. `ContentModel::Typed` / `HtmlMockup` keep their existing field sets and sections: their baseline verbs reuse the same storage, ID, patch, archive, delete, and status flow, but validation and section mapping come from the `TypedSchema` / owning type module (ADR-0017 decision 7). The generic model adds no field to a typed type.
- Extension verbs are not handled here: `create-group` / `--group` remain in `src/req.rs` / `src/nfreq.rs`-equivalent typed code and `supersede` remains in `src/ui_mockup.rs`. Those modules may call the shared record primitives, but they do not duplicate the generic create/edit/storage path.
- `status` is never a create or edit option and is never written by those paths. It changes only through `document::run` `Status` and `record::store::archive`.
- No template file, schema key, or `--field` escape hatch is introduced.

## 1. Module layout (`src/document.rs`)

Public surface:

```rust
pub enum DocumentVerb {
    Create, List, View, Edit, Archive, Delete, Status,
}

pub struct DocumentInvocation {
    pub slug: String,
    pub verb: DocumentVerb,
    pub id: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub labels: Vec<String>,
    pub references: Vec<String>,
    pub editor: bool,
    pub status_value: Option<String>,
}

pub fn run(
    ctx: &Context,
    registry: &ArtifactRegistry,
    invocation: DocumentInvocation,
) -> Result<CommandOutput, AppError>;
```

Private helpers (all take `&TypeDescriptor` and `&Config`, never a slug):

- `load_config`, `type_entry` (registry lookup + enabled check)
- `active_dir`, `archived_dir`, `read_dirs`
- `sanitize_field`
- `allocate_id`
- `create_generic`, `list_generic`, `view_generic`, `edit_generic`
- `archive`, `delete`, `status` (thin wrappers over `record::store`)
- `record_view` / `metadata_json` helpers

`run` resolves the descriptor once (`registry.descriptor(&invocation.slug)`), validates that it is a `Document` category descriptor, then dispatches by `invocation.verb`. A second resolution or slug comparison is not performed.

## 2. Dispatch off the registry descriptor

1. Look up `descriptor = registry.descriptor(&invocation.slug)`; missing/disabled -> `AppError::Validation` (exit 4) naming `document_types.<slug>`.
2. Match `descriptor.content_model`:
   - `Generic` -> the implementations in section 4.
   - `Typed(schema)` / `HtmlMockup(schema)` -> call the baseline implementation with `schema` for validation and section mapping; extension verbs are never routed here (they are handled by their owning module before `document::run` is reached).
3. Match `invocation.verb` to the handler. No arm matches on `slug`.

Directory, prefix, padding, dotted-ness, and status vocabulary are read from `descriptor` / `cfg.document_types[slug]` only.

## 3. Create / edit option mapping

General mapping (Generic model, ADR-0018 decision 1 and 7):

- `title`: required, non-empty after `sanitize::sanitize_name`; stored as `title` front matter and reflected in the generated `# <id> <title>` heading.
- `description`: optional; stored in the `## Description` section. The section is written only when a description is supplied.
- `labels`, `references`: repeatable `--label` / `--reference`; sanitized and deduplicated; stored as YAML lists in front matter. They never move into body sections.
- `status`: not an option on either path. A new record starts at `descriptor`'s `default_status` when configured, otherwise `active`.
- `id`, `created_date`, `updated_date`: set automatically and are exempt from the option rule.

`create` writes exactly the seven front-matter keys `id, title, created_date, updated_date, labels, references, status`, the generated heading, and an optional `## Description`. No other key or section is emitted.

`edit` accepts `title`, `description`, `labels`, `references`, and `--editor`:
- `title` -> `SetFrontMatter("title")` + `SetHeadingTitle`.
- `description` -> `SetSection("## Description", ...)` (creates the section if absent).
- `labels` / `references` -> read the current list from front matter, append new sanitized values with deduplication, `SetFrontMatter`.
- `--editor` edits the description section text only: the file is written with the already-computed changes first, then `$VISUAL`/`$EDITOR` is launched on the file path. If no metadata changes occurred, no write happens before the editor.
- `updated_date` is refreshed only when at least one field changes. A no-op edit performs no write.
- `status` is never accepted or written.

Unknown sections and unknown front-matter keys are preserved by `record::patch` (byte-identical outside changed fields).

## 4. Verb handlers

- `create`: sanitize inputs; load config; resolve descriptor; ensure active dir exists; allocate ID; build body document; apply front-matter ops (including status default); `write_atomic` to `active/<id>.md`; return `id`.
- `list`: read active dir only; resolve records with `record::resolve::list_in_dir` using the descriptor prefix and `Plain` id shape; read each through `record::store::read_document` (so adoption and auto-relocate run); sort by id; render via `RecordView::lenient(...).list_line` and metadata JSON. Archived records are excluded.
- `view`: resolve the record in the active dir only (archived records are not viewable by id); `ensure_front_matter`; return content + metadata JSON.
- `edit`: resolve active record; `ensure_writable`; read document; apply the option mapping; write atomically only on change; run editor if requested.
- `archive`: delegate to `record::store::archive(ctx, descriptor, id)`; return empty text + JSON `{status,id}`.
- `delete`: delegate to `record::store::delete(ctx, descriptor, id)`; confirmation and `--force` are CLI concerns and are handled before this call. Return empty text + JSON `{status,id}`.
- `status`: delegate to `record::store::set_status(ctx, descriptor, id, value)`; return empty text + JSON `{status,id}`.

## 5. ID allocation

- Uses `record::identity::next_number(&[active_dir, archived_dir], prefix)` and `format_id(prefix, next, cfg.id_padding as usize)`.
- Scans both active and archived locations, so disabling/re-enabling a type cannot reuse a number and deleted highest numbers may be reused (documented behavior, ADR-0022 decision 12).
- Plain shape only for generic types; dotted/grouped remain properties of `req` / `nfr` descriptors and are not produced here.

## 6. Storage

- Active path: `descriptor.resolve_directory(cfg, project_dir)` = `doc_directory` + `entry.directory`.
- Archived path: `descriptor.resolve_archived_directory(...)` = active + `archived/`.
- Read paths: `descriptor.resolve_read_directories(...)` (active + archived) for archive/delete/status resolution.
- All writes go through `record::store::write_atomic`, `record::store::archive`, `record::store::delete`, or `record::store::set_status`. No `fs::write`, no front-matter reformatting, and no body re-rendering occurs in `src/document.rs`. Directory creation uses `fs::create_dir_all` on the active directory only.

## 7. Status model handling

- Generic records use `StatusModel::DocumentMain { custom, default }` from the registry. A new record starts at `default` (or `active`).
- `status <id> <value>` accepts `active` or a declared custom status, rejects `archived` with a validation error naming `archive`, and refuses archived records.
- Archiving sets `archived` and moves the file; the previous custom value is discarded.
- Undeclared status values are preserved on read, flagged in diagnostics, and make the record read-only for `status` writes until fixed.
- Archived-in-active records are auto-relocated on read through `record::store::read_document` / `resolve_in_dirs` paths.

## 8. Typed extensions

- Typed descriptors keep their existing sections, required fields, ID shape, and verbs. `document::run` does not add `status` handling beyond what the descriptor already declares (front-matter-backed typed types keep `status` as a declared key).
- Extensions (`create-group`, `--group`, `supersede`) are dispatched by the CLI to their owning modules. Those modules reuse `record::store` and `record::identity` and do not call `document::run`, so the generic path stays single-purpose.
- A conformance test (section 9) runs create/edit/view for one generic and one typed type against the same baseline assertions to catch drift.

## 9. Test plan

Unit tests (in `src/document.rs` `#[cfg(test)]` and/or `tests/`):

1. `generic_create_writes_seven_keys_and_optional_description`
2. `generic_create_requires_title`
3. `generic_create_starts_at_default_or_active`
4. `generic_create_allocates_across_active_and_archived`
5. `generic_list_hides_archived`
6. `generic_view_returns_metadata`
7. `generic_edit_patches_title_heading_description`
8. `generic_edit_appends_labels_and_references`
9. `generic_edit_preserves_unknown_section_and_key`
10. `generic_edit_editor_touches_description_only`
11. `generic_edit_no_write_when_unchanged` (byte-identical)
12. `generic_archive_round_trip_and_idempotent`
13. `generic_status_accepts_custom_and_rejects_archived`
14. `generic_delete_archives_then_removes`
15. `generic_archived_in_active_auto_relocates`
16. `generic_undeclared_status_preserved_and_write_refused`
17. `generic_hand_added_section_preserved`
18. `typed_req_nfr_ui_mockup_baseline_still_works`
19. `typed_extensions_create_group_and_supersede_still_work`

Integration / black-box tests:

20. Config-defined type with only `prefix`, `directory`, `enabled` runs all baseline verbs.
21. Custom status vocabulary and default.
22. No template/schema/extra file is created.
23. Conformance: one generic and one typed type satisfy the same create/edit/view assertions.
24. `cargo test`, `cargo clippy` clean; grep check: no slug literals in `src/document.rs`.

Definition of Done:

- `cargo build`, `cargo test`, `cargo clippy` pass.
- `src/document.rs` contains no slug literals or slug match arms (verified by review/grep).
- All record writes go through `record::store`; no `fs::write` of record content.
- Unit and integration tests listed above exist and pass.
- Open questions in section 10 are recorded; deferred items name their owning task.


## Notes
# Open questions

1. Per-type option metadata (`--view`, `--rationale`, `--applicability`, `--type`) for typed baseline verbs: whether `document::run` should expose them through a generic option set or whether typed modules retain their own create/edit option sets. Deferred to TASK-108/114 per user instruction; this task's generic path maps only title, description, labels, and references.
2. Whether a generic `list` should gain a status filter to match the browser filter of REQ-006.014. Current requirement defines no document-list filters; none is added here.
3. Whether `--editor` should refresh `updated_date` when it changes the description file. Current behavior (tech-spec) does not refresh through the editor path; kept as-is.
4. Whether `view` should resolve an archived record by id. Proposed: no, per ADR-0022 decision 10; archived records are not viewable by id. Confirm during TASK-108.
5. Whether delete confirmation and `--force` live in the CLI or the handler. Proposed: CLI gates, handler performs removal, matching REQ-005.007.

# Risks

- Typed baseline verbs must not regress while moving storage/ID logic into the generic handler; mitigation is the conformance test plus characterization tests before cutover (TASK-108).
- Generic handler could accidentally become a second writer if it formats front matter; mitigated by restricting all writes to record::store and review/grep.
- Auto-relocate writes during read paths can surprise callers; tests cover failure handling.
- Status vocabulary shrink can leave records read-only; handled by tolerant read + diagnostics, not silently rewritten.

TASK-110 quality follow-up: added eprintln diagnostics in record::store::relocate_if_archived for successful auto-relocation and relocation failure; list_records/status_record read paths surface these via existing resolve/list flow. Read-only-on-relocation-failure implemented interpretation: destination-write failure returns Err(Validation); source-unlink failure returns Err(Runtime) naming both paths; operation fails loudly rather than returning a read-only handle. No read API restructure.
