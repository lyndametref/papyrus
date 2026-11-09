# Feature Specification — Remove Legacy Record Serializers and Enforce the No-Drift Rule

**Ticket:** TASK-70
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready)
**UI mockups:** none

## Purpose

Deliver ADR-0009 Stage 5: remove every legacy per-module record parser, renderer, and serializer and enforce the no-drift rule so that parsing, identity, and writing each have exactly one implementation under `src/record/`. This ticket implements REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, and NFR-04.003. It is a refactor with no intended change to command behavior, output, or exit codes; existing unit and integration tests keep their expected results. Backlog.md round-trip verification remains TASK-71.

## Scope

### In scope

- Remove module-local record parsers and renderers from every managed Markdown record module: `audit`, `decision`, `glossary`, `milestone`, `release`, `requirement`, `rule`, `task`, `tech_spec`, `ui_spec`, and any other module that still splits, parses, or renders a managed record body.
- Remove manual front-matter string formatting from all record creation and edit paths; record creation and edits build their output through `record::frontmatter`, `record::document`, and `record::patch`.
- Remove browser-local `records::parse_doc` parsing logic and all `records::apply_*_edit` functions; browser edit handlers build `PatchOp` lists and persist through `record::patch` and `record::store`.
- Move all remaining ID parsing, formatting, allocation, and translation into `record::identity`; no other module formats an ID, strips or pads a number, applies an offset, or allocates a storage number.
- Move all remaining record-file writes to `record::store::write_atomic`, including UI mockup HTML record files. `fs::write` remains permitted only for non-record files such as configuration, init output, and test fixtures.
- Remove the duplicate `src/record/descriptors.rs` module; `src/record/descriptor.rs` is the sole descriptor source.
- Update search code to obtain record text and titles through the shared reader/envelope instead of module-local parsers.
- Update the per-type search specifications under `docs/specifications/search-*.md` to state that matching runs over the whole persisted record text, including front matter, so metadata values are searchable.
- Add no-drift checks to the test suite for the three ADR-0009 decision 7 conditions.
- Preserve all existing command behavior, JSON shapes, validation messages, and exit codes.

### Out of scope

- New record types, fields, commands, or user-visible features.
- UI mockup metadata format changes; mockups remain HTML-comment based (REQ-05.008) and carry no front matter.
- Backlog.md round-trip verification — TASK-71.
- Browser UI redesign or metadata projection changes — TASK-66, TASK-67, TASK-72.
- Corpus migration or any write to an existing record file; convergence stays lazy.
- Configuration and init file writes.
- Test-fixture writes in `#[cfg(test)]` modules.

## Behavior

### 1. Single owner of parsing

1. `src/record/document::parse` is the only parser used for managed Markdown records by CLI modules, browser handlers, search, and migration.
2. No module outside `src/record/` detects a leading YAML block, parses YAML, splits `## ` sections, or extracts a record heading for a managed record.
3. `record::frontmatter` remains the only YAML implementation.
4. `record::document` remains the only lossless envelope implementation.
5. Search and other read-only consumers may call `record::store::read` or `record::store::read_document`; they must not reimplement parsing.

### 2. Single owner of mutation and rendering

1. No managed Markdown record module keeps a `render_*` function that assembles a whole file.
2. All field changes are expressed as `PatchOp` values applied through `record::patch::apply_patch`.
3. Browser route handlers no longer call `records::apply_*_edit`; they construct the same patch operations (directly or through helpers located under `src/record/`) and persist with `record::store::write_atomic`.
4. UI mockup HTML body rendering remains the only non-front-matter serializer exception because mockups are excluded from front matter by REQ-05.010 and ADR-0007 decision 11. It must not format or parse a `---` delimiter.
5. A write that changes no byte does not happen, so `updated_date` is not refreshed by a no-op.

### 3. Single owner of identity

1. `src/record/identity` is the only location of ID parsing, normalization, formatting, allocation, and display/storage translation.
2. Record modules no longer format IDs with `format!`, strip prefixes, pad numbers, apply offsets, or scan for the next number locally. They call `record::identity` allocators and formatters.
3. Path construction that appends a storage filename (for example `format!("{}.md", id)`) is permitted only after the ID has been produced by `record::identity`; it is path construction, not ID formatting.
4. No CLI, browser, JSON, or URL surface presents a storage ID as the record identity.

### 4. Single write path

1. Every create, edit, archive, restore, deprecate, supersede, and delete operation on a managed Markdown record uses `record::store::write_atomic`.
2. UI mockup HTML record writes use `record::store::write_atomic` as well.
3. No production code outside the allowlist calls `fs::write` on a record path.
4. Atomic write failures leave the original file byte-identical and remove temporary files.

### 5. Browser

1. `browser::records::parse_doc` is removed or reduced to a thin adapter over `record::document::parse` with no duplicated parsing logic.
2. The ten `apply_*_edit` functions are removed from `browser::records`.
3. Browser edit handlers continue to use the `RecordView` projection from TASK-67 for display and `record::patch` for mutation.
4. Browser routes, form values, and links continue to carry display IDs; `record::store` translates before filesystem access.

### 6. Search

1. Search obtains record content through the shared reader and does not parse records with module-local parsers.
2. Title extraction uses the shared envelope or front matter, with the same fallback behavior as before.
3. Search performs no writes.
4. The per-type search specifications state that matching runs over the whole persisted record text, including front matter, so metadata values are searchable.

### 7. No-drift checks in the test suite

The test suite includes automated checks that fail the build if any of the following is found in production source under `src/`:

1. `fs::write` on a record path outside the allowlist (`src/config.rs`, `src/init.rs`, and `#[cfg(test)]` modules). UI mockup writes must already have moved to `record::store::write_atomic`.
2. `format!("---` or an equivalent front-matter delimiter construction outside `src/record/`.
3. ID formatting outside `src/record/identity`, defined as any `format!` that combines an ID prefix literal with zero-padding, width, or offset logic. Path construction of the form `format!("{}.md", id)` is excluded.

The checks may be implemented as a test that shells out to `grep`/`rg` with documented patterns and allowlists. Their patterns and allowlists are documented in the test itself.

### 8. Behavior preservation

1. All existing unit and integration tests pass without changing their expected command output, JSON, or exit codes.
2. Lazy migration remains idempotent: a record carrying front matter is not re-migrated and its bytes are unchanged by a read or write.
3. Existing corpus files are not modified by this ticket.

## Inputs and outputs

- Input: a project `Context`, record paths, and existing command arguments.
- Output: unchanged command output and JSON shapes; migration reports only when a lazy migration actually changes a file.
- No new command, option, or user-facing field is added.

## Validation and error behavior

- Existing validation, not-found, and runtime error behavior is unchanged.
- No-drift check failures fail the test suite with a non-zero exit status.
- Atomic write and path-safety failures retain their existing exit codes and messages.

## Assumptions

- Assumption: `src/record/descriptors.rs` is dead duplicate code and can be removed without changing behavior; `src/record/descriptor.rs` is canonical.
- Assumption: UI mockup HTML rendering remains an exception to "no serializer outside `src/record/`" because mockups are excluded from front matter, but mockup file writes move to `record::store::write_atomic`.
- Assumption: `parse_heading` callers can migrate to `record::document` or front matter without changing title extraction behavior.
- Assumption: no corpus record file is modified by this ticket; convergence remains lazy.
- Assumption: no new dependency is added.
- Assumption: no-drift grep checks exclude `#[cfg(test)]` modules; production source is the enforcement target.

## Open questions

- OQ-1: Should browser type-specific edit helpers live under `src/record/` as one module per record type or as a single `record::edit` module? Either is acceptable provided browser no longer owns them.
- OQ-2: Should UI mockup rendering eventually move into `src/record/` despite HTML being out of front-matter scope, or remain an exception? This ticket changes only the mockup write path, not rendering.
- OQ-3: Should no-drift checks also scan `#[cfg(test)]` modules, requiring test fixtures to use temporary helpers instead of `fs::write`? The default assumption excludes tests; a stricter scan would expand scope.

## Traceability

| Artifact | Reference |
|---|---|
| Ticket | TASK-70 |
| Requirements | REQ-05.010, REQ-05.011, REQ-06.004, NFR-04.001, NFR-04.002, NFR-04.003 |
| ADRs | ADR-0006, ADR-0007, ADR-0008, ADR-0009 |
| Related tickets | TASK-63, TASK-64, TASK-65, TASK-66, TASK-67, TASK-68, TASK-69, TASK-71 |
| Source | `src/audit.rs`, `src/decision.rs`, `src/glossary.rs`, `src/milestone.rs`, `src/release.rs`, `src/requirement.rs`, `src/rule.rs`, `src/task.rs`, `src/tech_spec.rs`, `src/ui_spec.rs`, `src/browser/records.rs`, `src/search.rs`, `src/record/*` |
| Tests | `tests/*.rs`, `src/record/*` unit tests |
| Documentation | `docs/specifications/search-*.md` |
