# Document-Type Configuration Validation Contract (TASK-109)

Companion to ADR-0019. Defines the implementation contract for fail-closed
validation and repair mode. No implementation code.

## 1. Config::validate

```rust
impl Config {
    pub fn validate(&self, project_dir: Option<&Path>) -> Result<(), String>;
}
```

- Single validator. Returns all diagnostics joined by \n (one per line), or Ok(()).
- Some(project_dir) enables symlink-aware containment (NFR-04.002);
  None is string-level only (FS-free callers, registry, tests).
- Diagnostics are collected across all fields/entries, not fail-fast.
- Order: top-level fields in struct order, then document_types in BTreeMap
  order. Within an entry: slug, reserved name, prefix, directory, statuses,
  default_status.

## 2. Slug / prefix / directory / status rules

- Slug: non-empty; chars [A-Za-z0-9_-]; first char != -; no whitespace,
  control, path separator, or .. . Invalid: document_types.<slug>: invalid slug.
- Reserved (case-insensitive vs cli::BUILTIN_COMMANDS):
  document_types.<slug>: slug conflicts with built-in type <builtin>.
- Prefix: non-empty; no whitespace/control/path separator/..;
  empty document_types.<slug>.prefix: must not be empty;
  bad chars document_types.<slug>.prefix: must not contain whitespace, control characters, path separators, or '..';
  duplicate (case-insensitive) document_types.<slug>.prefix: <prefix> is already used by <other>.
- Directory: non-empty; string-level relative/no traversal/backslash;
  any string/FS failure document_types.<slug>.directory: value must be relative and stay inside the project directory: <value>;
  duplicate among enabled entries document_types.<slug>.directory: directory is already used by <other>.
- FS containment checks Context::ensure_within(project_dir, project_dir.join(doc_directory).join(directory)).
- Statuses (when Some): non-empty; no duplicates; active and archived
  exactly once; every value follows the slug char rule; ui-mockup rejects any
  statuses/default_status.
  Diagnostics align with RegistryError Display (must not be empty,
  duplicate value <v>, "active"/"archived" must be declared exactly once,
  invalid status value <v>, not allowed for a type without a front-matter status).
- default_status: when present must be declared and not archived
  (<v> is not a declared status, must not be "archived").

## 3. Duplicate YAML keys

- yaml_serde::from_str already rejects duplicate mapping keys. No custom
  parser. Duplicate keys are fatal for every load, including repair commands.
- load_effective_value maps the parse error to AppError::Validation.

## 4. Loaders

```rust
pub fn load_effective_value(config_path: &Path) -> Result<yaml_serde::Value, AppError>;
pub fn load_effective_config(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
pub fn load_effective_config_tolerant(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
```

- load_effective_value: parse + wholesale merge defaults; no validation.
  document_types is replaced wholesale when present in the overlay.
- load_effective_config (strict): load_effective_value + browser YAML
  validation + deserialize + Config::validate(Some(project_dir)).
- load_effective_config_tolerant: same parse/deserialize/browser validation,
  but skips Config::validate (used by repair commands and folder relocation).
- merge_yaml: recursive for all keys except document_types, which is
  replaced when the overlay contains it.

## 5. Repair / tolerant commands

- config get|set|unset|list use load_effective_value/tolerant load and do
  not validate the pre-existing document types on entry.
- config set materializes before editing:
  1. Load raw file_val.
  2. If key starts with document_types. and file_val has no
     document_types, insert the default document_types map.
  3. yaml_set(file_val, key, parsed).
  4. Build candidate: default Config YAML + merge_yaml (wholesale) +
     browser YAML validation + deserialize + Config::validate(Some(project_dir)).
  5. If invalid, return error before relocation/write; file byte-identical.
  6. Folder relocation (if applicable) then write file_val.
- config unset removes the key, validates the resulting configuration with
  the same validator, refuses invalid writes.
- config list/get do not call Config::validate on the loaded config.

## 6. init --force

- Replaces the file with the generated default set even when invalid.
- Validates the generated config with Config::validate(Some(project_dir)).
- Never builds a registry or runs record operations. lib.rs keeps
  registry = None for init.

## 7. Error aggregation format

- Separator: \n (no trailing newline).
- Each line: <key>: <message> (document-type lines use
  document_types.<slug>.<field> or document_types.<slug>).
- Diagnostics are deterministic: top-level fields, then document_types in
  BTreeMap order.

## 8. Registry

- ArtifactRegistry::build keeps re-asserting its existing invariants.
  No change.

## Open questions

- Top-level field diagnostics retain their historical messages; document-type
  lines use the templates above.
- Empty document_types maps are valid.
- Prefix uniqueness is case-insensitive; directory duplication only among
  enabled entries.
- Status char validation applies to every declared value, including active
  and archived.
