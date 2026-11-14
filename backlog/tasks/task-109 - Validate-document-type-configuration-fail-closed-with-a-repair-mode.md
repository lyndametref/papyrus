---
id: TASK-109
title: Validate document-type configuration fail-closed with a repair mode
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
updated_date: '2026-11-14T13:56:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
  - TASK-108
documentation:
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - docs/requirements/business/REQ-07.002.md
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.004.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/architecture/document-type-config-validation-contract.md
priority: high
ordinal: 41040
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Validate the `document_types` configuration fail-closed and provide repair surfaces, per ADR-0019 and the TASK-109 implementation contract (`docs/architecture/document-type-config-validation-contract.md`). `Config::validate` becomes the single, aggregated validator for document-type entries (slug grammar, reserved names, prefix uniqueness, directory containment and uniqueness, status lists, default status, UI-mockup status prohibition) and runs for every command. `document_types` is replaced wholesale when present, so removal is expressible and defaults are not re-added. Strict loading applies to all commands; `config get|set|unset|list` load tolerantly, validate the resulting configuration, and refuse invalid writes; `init --force` regenerates the default set. `ArtifactRegistry::build` re-asserts the same invariants as defence in depth (TASK-105).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Config::validate(&self, project_dir: Option<&Path>) is the only validator for document types and aggregates every violation into one AppError::Validation (exit 4) message, lines joined by \n, in deterministic order: top-level fields in struct order, then document_types entries in BTreeMap order.
- [x] #2 Slug validation rejects empty values, unsupported characters, a leading -, whitespace, control characters, path separators, backslash, and .. without trimming or normalizing; reserved built-in names come from cli::BUILTIN_COMMANDS and are compared case-insensitively.
- [x] #3 Prefix validation rejects empty, whitespace, control characters, path separators, backslash, and traversal; a prefix must be unique case-insensitively across all configured entries including disabled ones.
- [x] #4 Each directory must be non-empty, relative, free of traversal and backslash; when project_dir is Some it must pass Context::ensure_within after symlink resolution; duplicate directories are rejected only among enabled entries.
- [x] #5 Status lists, when present, are rejected when empty, duplicated, missing active or archived, containing a value that violates the slug character rules, or declared on ui-mockup; default_status must be declared and must not be archived.
- [x] #6 A repeated YAML mapping key is detected by yaml_serde::from_str before defaults are merged and rejected as invalid configuration; document_types is replaced wholesale when present, so removing an entry is expressible and no default is re-added.
- [x] #7 Strict load (load_effective_config) validates with Config::validate(Some(project_dir)); tolerant load (load_effective_config_tolerant) skips Config::validate for config get/set/unset/list; config get/list do not validate pre-existing document types.
- [x] #8 config set materializes the full default document_types map when absent, applies the change, validates the resulting configuration with the same validator, and writes only a valid result; an invalid result is refused and the file is byte-identical.
- [x] #9 config unset validates the resulting configuration with the same validator and refuses an invalid write.
- [x] #10 init --force replaces the configuration with the generated default set even when the existing file is invalid; repair mode never executes record operations or builds a registry from an invalid set.
- [x] #11 ArtifactRegistry::build re-asserts the same invariants as defence in depth and converts failures to AppError::Validation (exit 4).
- [x] #12 Tests cover each rejection, aggregated diagnostics, tolerant-load versus validated-write, list-valued status writes, and a symlinked directory escape.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo build, cargo test, and cargo clippy are clean.
- [ ] #2 Config::validate and both loaders have unit tests for every rule in the implementation plan.
- [ ] #3 Symlink escape and tolerant/strict behavior are covered by tests.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
## Implementation Plan

Design authority: ADR-0019 (accepted), ADR-0016, ADR-0017, REQ-07.002, REQ-01.002, REQ-01.004, NFR-02.002, NFR-04.002, and `docs/architecture/document-type-config-validation-contract.md` (TASK-109 implementation contract).

### 1. Scope and out of scope

In scope:
- Make `Config::validate` the single, aggregated validator for document types, with a `project_dir` parameter for symlink-aware containment.
- Replace per-entry default merging with wholesale `document_types` replacement.
- Detect duplicate YAML keys (already a parse error) and map it to `AppError::Validation`.
- Add a tolerant loader for `config get|set|unset|list`.
- Materialize defaults before editing `document_types.*` in `config set`.
- Ensure `init --force` repairs an invalid file.
- Keep `ArtifactRegistry::build` as defence-in-depth (no change).

Out of scope:
- List-valued `statuses` parsing syntax details beyond validation of the resulting list (TASK-111).
- Updating init defaults / seeding statuses (TASK-112).
- Browser views, search, business rules, README (TASK-114+).

### 2. Config::validate signature and diagnostics

```rust
impl Config {
    pub fn validate(&self, project_dir: Option<&Path>) -> Result<(), String>;
}
```

- Single validator; returns all diagnostics joined by `\n` (no trailing newline), or `Ok(())`.
- `Some(project_dir)` enables containment checks via `Context::ensure_within`; `None` is string-level only and keeps registry/tests filesystem-free.
- Order: top-level fields in struct order, then `document_types` in `BTreeMap` order. Within an entry: slug, reserved name, prefix, directory, statuses, default_status.

### 3. Validation rule set

Slug (`document_types` key):
- non-empty; chars `[A-Za-z0-9_-]`; first char != `-`; no whitespace, control, path separator (`/` or `\`), or `..`.
- no trimming or normalization.
- reserved names from `cli::BUILTIN_COMMANDS`, compared case-insensitively: `document_types.<slug>: slug conflicts with built-in type <builtin>`.

Prefix:
- non-empty; no whitespace, control, path separator, or `..`.
- unique case-insensitively across all configured entries, including disabled: `document_types.<slug>.prefix: <prefix> is already used by <other>`.

Directory:
- non-empty; relative; no traversal segments or backslashes.
- when `project_dir` is `Some`, `Context::ensure_within(project_dir, project_dir.join(doc_directory).join(directory))` must succeed (symlink resolution).
- duplicate among enabled entries rejected: `document_types.<slug>.directory: directory is already used by <other>`.

Statuses (when `Some`):
- non-empty; no duplicates; `active` and `archived` exactly once.
- every declared value follows the slug character rule.
- `ui-mockup` rejects any `statuses` or `default_status`.

default_status (when `Some`):
- must be a declared status and must not be `archived`.

Aggregation: one `AppError::Validation` message, lines joined by `\n`.

### 4. Loaders and merge semantics

```rust
pub fn load_effective_value(config_path: &Path) -> Result<yaml_serde::Value, AppError>;
pub fn load_effective_config(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
pub fn load_effective_config_tolerant(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
```

- `load_effective_value`: parse raw file; on parse error (including duplicate keys) return `AppError::Validation`; merge defaults with `merge_yaml`, replacing `document_types` wholesale when present in the overlay; no validation.
- `load_effective_config` (strict): browser YAML validation + deserialize + `Config::validate(Some(project_dir))`. Used by every command except repair surfaces and init.
- `load_effective_config_tolerant`: same parse/deserialize/browser validation, but skips `Config::validate`. Used by `config get|set|unset|list` and folder relocation.
- `merge_yaml`: recursive for all keys except `document_types`, which is replaced when the overlay contains it.

Duplicate keys: `yaml_serde::from_str` already rejects them; map to `AppError::Validation`. Fatal for all loads, including repair commands.

### 5. Repair surfaces

config get / list:
- use tolerant load; never validate pre-existing document types; read values from the parsed/merged YAML.

config set:
1. Read raw `file_val` (or empty mapping).
2. If the key starts with `document_types.` and `file_val` has no `document_types`, insert the default `document_types` map.
3. Parse and sanitize the value as today.
4. `yaml_set(file_val, key, parsed)`.
5. Build candidate: default Config YAML + `merge_yaml` (wholesale) + browser validation + deserialize + `Config::validate(Some(project_dir))`.
6. If invalid, return `AppError::Validation` before relocation/write; file remains byte-identical.
7. Folder relocation (if applicable) runs, then write `file_val`.

config unset:
- remove the key, build candidate with the same validation, refuse invalid writes.

init --force:
- replace the file with the generated default set even when the existing file is invalid; validate the generated config with `Config::validate(Some(project_dir))`; never build a registry or execute record operations.

### 6. Call-site updates

- `load_effective_config` callers already pass `project_dir`; keep strict.
- `Config::validate()` call sites in tests and `init.rs` update to `validate(Some(project_dir))` or `validate(None)`.
- `ArtifactRegistry::build` unchanged; it uses `validate`-equivalent invariants via `RegistryError`.

### 7. Test plan

Unit tests (src/config.rs):
- `validate(None)` and `validate(Some(project_dir))` for each rule.
- Slug: empty, bad chars, leading `-`, whitespace, control, separator, `..`, reserved (case-insensitive).
- Prefix: empty, bad chars, duplicate (including disabled), case-insensitivity.
- Directory: empty, absolute, traversal, backslash, duplicate among enabled only, symlink escape (with temp project dir).
- Statuses: empty, duplicate, missing active/archived, repeated main, invalid char, ui-mockup rejection.
- default_status: undeclared, archived.
- Aggregated diagnostics: multiple violations in one message, deterministic order.
- Wholesale merge: absent keeps defaults; present uses exactly listed entries; removing an entry does not re-add it; `deploy` not re-added.

Loader tests (src/commands.rs):
- Strict load rejects invalid document types (exit 4).
- Tolerant load allows reading an invalid config.
- Duplicate key parse error maps to validation.
- `config set` materializes defaults then edits; resulting invalid config refused and file byte-identical.
- `config set` valid status list writes YAML flow sequence and round-trips.
- `config unset` validates resulting config.
- `init --force` replaces invalid config with defaults.

Registry tests (src/registry.rs, existing):
- `ArtifactRegistry::build` re-asserts invariants; unchanged.

Integration/CLI:
- invalid config blocks every command except repair surfaces with exit 4.
- `config get|set|unset|list` repair an invalid slug/status.
- symlinked directory escape rejected at load and at resolution.

### 8. Assumptions

- `Config::validate` may perform filesystem access only when `project_dir` is `Some`; callers in FS-free contexts pass `None`.
- Top-level field diagnostics retain their historical messages; document-type lines use the templates in section 3.
- Empty `document_types` maps are valid.
- Prefix uniqueness is case-insensitive; directory duplication only among enabled entries.
- Status character validation applies to every declared value, including `active` and `archived`.
- Duplicate YAML keys are fatal for all commands, including repair commands, because parsing fails before merge.

### 9. Open questions

- Whether top-level field diagnostics should be normalized to the same `<key>: <message>` form as document-type lines.
- Whether `config set` materialization should persist the full default `document_types` map or only the `document_types` section. Current contract: full `document_types` map.
- Whether a file with duplicate YAML keys can be repaired through `config set` at all, given parsing fails before merge (current contract: repair via `init --force` or manual edit).
<!-- SECTION:PLAN:END -->
