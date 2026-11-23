---
id: TECH-077
title: "Document-Type Configuration Validation Contract"
created_date: '2026-11-16T17:46:56Z'
updated_date: '2026-11-22T12:12:48Z'
labels: []
references:
  - ADR-0019
  - ADR-036
  - ADR-041
  - NFR-004.002
  - NFR-002.002
  - REQ-007.002
  - REQ-007.004
  - BR-016
status: verified
---
# TECH-077 Document-Type Configuration Validation Contract

## Description
Companion to ADR-0019 and ADR-036. Defines the fail-closed validation and
resolution contract for the nested `document_types` entry, its field sequence,
and the seed. No implementation code.

## 1. Config::validate

```rust
impl Config {
    pub fn validate(&self, project_dir: Option<&Path>) -> Result<(), String>;
}
```

- Single validator. Returns all diagnostics joined by \n (one per line), or Ok(()).
- Some(project_dir) enables symlink-aware containment (NFR-004.002);
  None is string-level only (FS-free callers, registry, tests).
- Diagnostics are collected across all entries and all field sequences, not
  fail-fast. An invalid entry is never skipped, cleaned, or normalized.
- Order: top-level fields in struct order, then document_types in BTreeMap
  order. Within an entry: entry-level keys, then the role blocks, then their
  field items in sequence order.

## 2. Entry shape and slug / prefix / directory / status rules

- An entry is a mapping. Its top-level keys are `prefix`, `directory`,
  `enabled`, `label`, `plural`, `order`, `summary`, `type_alias`,
  `reference_visibility`, `statuses`, `default_status`, `slots`, `api_aliases`,
  `fields`, and the role blocks `group` and `member`.
- A single-role type (`SingleMarkdown` or `SingleHtmlMockup`) declares one
  top-level `fields` sequence and no role block. `fields: []` is valid and
  declares no field.
- A grouped type (`GroupedMarkdown`) declares `group` and `member` blocks, each
  with a `fields` sequence and an optional `reference_visibility`. A top-level
  `fields` on a grouped type is rejected.
- `model`, `title`, `content`, and `id_shape` are not entry keys and are
  rejected wherever they appear.
- Slug: non-empty; chars [A-Za-z0-9_-]; first char != -; no whitespace,
  control, path separator, or .. . Invalid: document_types.<slug>: invalid slug.
- Reserved (case-insensitive vs cli::BUILTIN_COMMANDS):
  document_types.<slug>: slug conflicts with built-in type <builtin>.
- Prefix: non-empty; no whitespace/control/path separator/..;
  empty document_types.<slug>.prefix: must not be empty;
  bad chars document_types.<slug>.prefix: must not contain whitespace, control characters, path separators, or '..';
  duplicate (case-insensitive) document_types.<slug>.prefix: <prefix> is already used by <other>.
  `prefix` is required on every entry, including an entry for a seed slug, and
  is never filled from the seed.
- Directory: non-empty; string-level relative/no traversal/backslash;
  any string/FS failure document_types.<slug>.directory: value must be relative and stay inside the project directory: <value>;
  duplicate among enabled entries document_types.<slug>.directory: directory is already used by <other>.
  `directory` is required on every entry and is never filled from the seed.
- FS containment checks Context::ensure_within(project_dir, project_dir.join(doc_directory).join(directory)).
- Statuses (when Some): non-empty; no duplicates; active and archived
  exactly once; every value follows the slug char rule; a statuses or
  default_status declaration on a `SingleHtmlMockup` type is rejected.
  Diagnostics align with RegistryError Display (must not be empty,
  duplicate value <v>, "active"/"archived" must be declared exactly once,
  invalid status value <v>, not allowed for a type without a front-matter status).
- default_status: when present must be declared and not archived
  (<v> is not a declared status, must not be "archived").
- A repeated `document_types` slug key is invalid configuration; neither
  occurrence is preferred.

## 3. Field item vocabulary and reserved keys

A field item has exactly the keys `key`, `heading`, `frontmatter_key`,
`option`, `label`, `kind`, `required`, `verbs`, `help`, and `values`. It
declares no `order` and no `enabled`; either key is rejected.

- `key` is required and stable. It identifies the field in field maps,
  selector paths, diagnostics, and browser form submissions.
- `heading` is the level-2 Markdown heading that stores the value, written
  with the `## ` prefix.
- `frontmatter_key` is the front-matter key that stores the value. It may
  differ from `key`.
- `option` is the long CLI option name without the leading `--`. When absent
  it is derived from `key` in kebab case.
- `label` and `help` are presentation-only.
- `kind` is exactly one of `text`, `multiline`, or `list`.
- `required` is a boolean.
- `verbs` is a subset of `create` and `edit` and defaults to both.
- `values` is optional. It is either a non-empty sequence of allowed strings,
  which restricts a scalar field to those exact values, or `record-ids` for a
  `list` field whose items must be valid record IDs. The field-domain
  validator enforces it; it adds no option and no verb of its own.
- A field must declare `heading`, `frontmatter_key`, or both. A field with
  neither is invalid. A field with both writes both locations from one value.
- Field keys are unique within a role: `fields`, `group.fields`, and
  `member.fields` separately, while the same key may appear in the group and
  member roles.
- The reserved field keys are `id`, `title`, `status`, `labels`, `references`,
  `created_date`, `updated_date`, `group`, `version`, `kind`, `form`, `path`,
  and `metadata`. A field may not use a reserved key as its `key` or as its
  `frontmatter_key`. `version`, `kind`, `form`, `path`, and `metadata`
  collide with the envelope, token, and form-post keys.
- Title is fixed and non-configurable: it is not a field, is always present,
  is single-line and non-empty, is required on create, and can never be cleared
  on edit. It is the first positional `TITLE` on `create` and `create-group`
  and `--title` on create, create-group, and edit. There is no `--term` option.
- Sequence position is the order of the field everywhere: body sections, CLI
  option order, help text, browser form order, and field-map traversal.

## 4. Duplicate YAML keys

- yaml_serde::from_str already rejects duplicate mapping keys. No custom
  parser. Duplicate keys are fatal for every load, including repair commands.
- load_effective_value maps the parse error to AppError::Validation.

## 5. Loaders and resolution against the seed

```rust
pub fn load_effective_value(config_path: &Path) -> Result<yaml_serde::Value, AppError>;
pub fn load_effective_config(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
pub fn load_effective_config_tolerant(config_path: &Path, project_dir: &Path) -> Result<Config, AppError>;
```

- load_effective_value: parse + wholesale merge defaults; no validation.
  `load_effective_value` parses the overlay and resolves `document_types`
  against the seed table, so a file that declares only one entry still yields
  all eleven seed slugs; the overlay is not a raw replacement.
- load_effective_config (strict): load_effective_value + browser YAML
  validation + deserialize + Config::validate(Some(project_dir)).
- load_effective_config_tolerant: same parse/deserialize/browser validation,
  but skips Config::validate (used by repair commands and folder relocation).
- merge_yaml: recursive for all keys except document_types, whose overlay map
  is resolved against the seed table, not deep-merged or raw-replaced.
- Resolution is per entry against the seed entry of the same slug:
  - Scalars fill from the seed when absent: `enabled`, `label`, `plural`,
    `order`, `summary`, `type_alias`, `default_status`, `reference_visibility`,
    and the model and roles. A present scalar overrides the seed.
  - Collections replace wholesale when present: `fields`, `group.fields`,
    `member.fields`, `statuses`, `slots`, and `api_aliases`. An absent
    collection inherits the seed collection; a present collection replaces it,
    including `fields: []`.
  - The `group` and `member` blocks resolve independently: a present block
    replaces only that role's collection, and an absent block inherits it.
  - `prefix` and `directory` are never filled from the seed.
  - The eleven seed slugs are always part of the effective configuration and
    resolve from the seed table whether or not the file declares them. A
    present `document_types` map is an overlay, not a replacement: declaring
    one entry does not remove the other seed types.
  - An absent non-seed (user-defined) slug stays absent. The written file
    never gains entries the command did not target.
  - A declared entry still fills scalars from the seed and replaces present
    collections wholesale, per the bullets above.
  - Seed slot sets are likewise always effective.
- A user-defined slug has no seed entry. Its defaults are `SingleMarkdown`,
  `enabled: true`, a label derived from the slug, an order after the seed
  types, empty summary and aliases, visible reference visibility,
  `statuses: [active, archived]`, no `default_status`, and one optional
  `description` field with heading `## Description` and kind `multiline`.

## 6. Repair and tolerant commands

- config get|set|unset|list use load_effective_value/tolerant load and do
  not validate the pre-existing document types on entry.
- config set materializes before editing:
  1. Load raw file_val.
  2. If key starts with document_types. and file_val has no
     document_types, materialize only the target seed entry with its seed
     prefix and directory.
  3. yaml_set(file_val, key, parsed).
  4. Build candidate: default Config YAML + merge_yaml (recursive, with
     `document_types` resolved against the seed table) + browser YAML
     validation + deserialize + Config::validate(Some(project_dir)).
  5. If invalid, return error before relocation/write; file byte-identical.
  6. Folder relocation (if applicable) then write file_val.
- A selector is `document_types.<slug>.<path>`. A field item is selected by
  key (`fields[story].heading`) or zero-based index (`fields[0].heading`).
  An unknown selector, a keyed selector that matches no unique field, or an
  out-of-range index is rejected with the same diagnostic form.
- `config set document_types.<slug>.<path>` materializes only the target entry
  and validates the resulting configuration.
- `fields` and the role field sequences are never written from a
  comma-separated value; adding, removing, reordering, or changing a field is
  a configuration file edit.
- `statuses` is the only list-valued configuration key here: its comma-separated
  value is split and stored as a YAML flow sequence. `slots` and `api_aliases`
  are declarative file data and are not `config set` or `config unset` targets.
- `config unset` removes a scalar override or reverts `statuses` to the seed
  list, validating the result.
- `config list/get` do not call Config::validate on the loaded config.
- `papyrus check` loads tolerantly, reports configuration findings, and runs
  the checks that do not depend on the effective document-type set.

## 7. init --force

- Replaces the file with the generated default set even when invalid.
- Writes each document type as a minimal entry containing only `prefix` and
  `directory`, plus `enabled` when the interactive walkthrough disables it.
- Validates the generated config with Config::validate(Some(project_dir)).
- Never builds a registry or runs record operations. lib.rs keeps
  registry = None for init.

## 8. Error aggregation format

- Separator: \n (no trailing newline).
- Each line: <key>: <message> (document-type lines use
  document_types.<slug>.<path> or document_types.<slug> when no narrower path
  exists).
- A field diagnostic names the field key when present and uses the keyed
  `fields[story].heading` form; an indexed selector uses `fields[0].heading`.
- Diagnostics are deterministic: top-level fields, then document_types in
  BTreeMap order; within an entry, entry-level keys then role blocks and field
  sequence order.

## 9. Registry

- ArtifactRegistry::build consumes the validated effective configuration and
  re-asserts its invariants. It derives document descriptors from the seed and
  the configuration only; no second field table or static document descriptor
  survives.
- A registry build error is a validation outcome; an invalid configuration
  never produces a partially valid type set.

## Open questions

- Top-level field diagnostics retain their historical messages; document-type
  lines use the templates above.
- Empty document_types maps and `fields: []` are valid.
- Prefix uniqueness is case-insensitive; directory duplication is checked only
  among enabled entries.
- Status char validation applies to every declared value, including active
  and archived.
