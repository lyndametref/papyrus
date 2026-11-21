---
id: TECH-071
title: "Registry-Sourced Record Descriptors"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-21T00:29:39Z'
labels: []
references:
  - ADR-0013
  - ADR-0017
  - ADR-0022
  - ADR-0036
  - ADR-0037
  - ADR-0039
  - REQ-01.005
  - REQ-05.007
  - REQ-05.010
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
status: verified
---
# TECH-071 Registry-Sourced Record Descriptors

## Description
## Purpose

ADR-0036 makes the artifact registry the only object that maps a document slug
to behaviour and removes the second, parallel record-descriptor table. A
document descriptor is an owned value derived from the seed entry and the
effective configuration; it is not a `'static` value and it does not depend on a
process-global cache.

`ArtifactRegistry::descriptor` is the only accessor used by production code.
The registry owns every descriptor it publishes, including the descriptors of
user-defined types, and a `RecordDescriptor` is derived from the same
`TypeDescriptor` rather than from a separate static table. Backlog-family
record descriptors (task, milestone, release, decision) remain static because
those records stay bespoke.

## Current state and target

| Concern | Current | Target |
|---|---|---|
| Document descriptor source | Static per-type tables and a second record-descriptor table | `SEED_DOCUMENT_TYPES` plus the effective configuration, resolved into owned descriptors |
| Descriptor lifetime | `&'static` values and leaked per-slug values | Owned values with no `'static` requirement |
| Record descriptor | `descriptor_for_doc_type` beside the registry descriptor | `ArtifactRegistry::record_descriptor` derived from the same `TypeDescriptor` |
| User-defined type | A leaked runtime descriptor | An owned descriptor built from the configuration defaults |
| Field data | Per-type module constants and static arrays | The descriptor's resolved `FieldSpec` sequence |

## Design

### 1. Descriptors are owned values

A `TypeDescriptor` owns its `slug`, `label`, `plural`, `summary`, `prefix`,
`directory`, `enabled` flag, status vocabulary, `DocumentModel`, role field
sets, reference visibility, aliases, slots, and presentation facts. A field is
an owned `FieldSpec` carrying `key`, `heading`, `frontmatter_key`, `option`,
`label`, `kind`, `required`, `verbs`, and `help`; a role field set is a
sequence of those values.

The registry builds each descriptor from two inputs only:

- the static seed entry for a seed slug, which supplies the model, default
  roles and fields, default statuses, presentation defaults, and reference
  visibility;
- the effective configuration entry, which fills scalars and replaces
  collections under the resolution rules of the configuration contract.

The result is owned by the `ArtifactRegistry`. No descriptor type carries a
`'static` bound, a `&'static str` requirement, or a lifetime parameter, and no
descriptor is interned, leaked, or stored in a process-global map. A registry
built from one configuration describes exactly that configuration; a second
registry built from another configuration is independent.

`RecordDescriptor` follows the same rule. Its configuration-derived fields are
owned values (or `Cow` values that may borrow a static literal but are not
required to), and the registry's `TypeDescriptor.record` owns the record
descriptor for every document type, including a user-defined type. The value is
`Clone` but not `Copy`; hot paths pass `&RecordDescriptor`.

### 2. One descriptor and one accessor

- `ArtifactRegistry::descriptor(&self, slug) -> Option<&TypeDescriptor>` is the
  only slug-to-descriptor lookup.
- `ArtifactRegistry::record_descriptor(&self, slug) -> Option<&RecordDescriptor>`
  returns the record descriptor owned by that type descriptor. There is no free
  function that resolves a document record.
- A document descriptor's record view is derived from the same values as its
  command and browser views: slug, prefix, directory, ID shape, status model,
  field sequence, and presentation facts.
- Deleting the parallel document table leaves `DESCRIPTORS` with only the
  backlog and decision entries.

### 3. Field-source matrix

| `RecordDescriptor` field | Document source |
|---|---|
| `name` | The descriptor's record name or the slug |
| `record_type` | The descriptor's record type |
| `type_name` | The descriptor's type name |
| `label` | The descriptor's label |
| `display_id_pattern` | The model's ID shape |
| `directory` | The descriptor's configured directory, relative to `doc_directory` |
| `subpath` | Empty; the configuration entry supplies the relative directory |
| `doc_type_key` | The descriptor slug |
| `frontmatter_keys` | The `frontmatter_key` of each declared field for the role, plus `status` where the model stores it there |
| `sections` | The `heading` of each declared field for the role |

`resolve_directory`, `configured_prefixes`, `matches_stem`,
`resolve_archived_directory`, `resolve_read_directories`, and
`declared_statuses` read the descriptor-supplied values plus the effective
`Config`; they do not read a static document table.

### 4. Call sites

- `document::run` resolves the descriptor once and passes the record descriptor
  to the engine with the field map.
- The record layer, migration, init, and search consume the registry's
  descriptors; they do not call a free document-descriptor function.
- `managed_descriptor` and the migration walker use the backlog descriptors plus
  the registry's document record descriptors built from the loaded
  configuration. They return an owned value when the registry cannot outlive
  the call.
- `init` creates directories by iterating the enabled document descriptors and
  resolving each directory from its descriptor.
- Backlog-family modules keep their static descriptors and are unchanged.

`DocumentScope` stays the runtime carrier of active directory, archived
directory, prefix, and padding. Modules do not join configuration paths
themselves.

### 5. Legacy and no-front-matter behaviour

- Reading a native record is unchanged: the active directory plus `archived/`
  are read; a legacy `deprecated/` folder is inert and is never scanned,
  listed, migrated, or resolved.
- A file without front matter is adopted through the same `record_name`,
  `matches_stem`, `resolve_read_directories`, and `record_frontmatter_keys`
  values as before, so `ensure_front_matter` and `adoption_ops` seed the same
  keys in the same order.
- A disabled document type is absent from command dispatch, init directory
  creation, and migration. The registry may still describe it, but no command
  runs against it.
- A user-defined type resolves through the same accessor as a seed type; its
  owned descriptor supplies its directory, prefix, status key, fields, and
  presentation defaults.

### 6. Single-source invariant tests

1. **Totality.** Every `ArtifactCategory::Document` descriptor has a record
   descriptor; the count equals the document descriptor count; an undeclared or
   unknown slug returns `None`.
2. **Field agreement.** For every document descriptor, the record descriptor's
   slug, label, prefix, directory, ID shape, status key, and sections agree with
   the descriptor's owned values and resolved field sequence.
3. **No parallel document table.** `record::descriptor::DESCRIPTORS` contains
   no document entry, and a source scan forbids a free document-descriptor
   constructor in production lines.
4. **Built-in and user-defined parity.** A built-in type and a user-defined
   type each resolve through the registry and return fields sourced from their
   own descriptor, not from shared static data.

## Test plan

- Unit tests in `src/registry.rs` and the record descriptor module: totality,
  field agreement, no parallel table, and built-in/user-defined parity.
- A configuration with two registries built from different `document_types`
  values returns field-equal descriptors on repeated calls and no cross-registry
  state.
- Every typed CLI suite and the migration, native-layout, search, and browser
  suites run unchanged.
- Mechanical checks: `cargo build`, `cargo test`, and
  `cargo clippy --all-targets -- -D warnings` are clean.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Migration output drifts because a record name branch changes | Keep the record name and key order derived from the same descriptor values; run the migration and native-layout suites |
| Registry rebuild cost on a read or write path | Build once per command or loop where possible; per-record builds are dominated by filesystem I/O |
| Shrinking the static table breaks a caller | The compiler catches every reference; the re-export is updated with its last user |
| Typed module signature churn causes behaviour drift | Change one consumer at a time and keep validation and section logic untouched |
| A leaked or static descriptor reappears | Contract test forbids a process-global descriptor map and a free document-descriptor constructor |

## Out of scope

- Rewriting the four backlog-family record descriptors or the modules that use
  them.
- Changing record layout, front-matter keys, ID shapes, statuses, migration
  outcomes, CLI output, JSON, or help text.
- Introducing a cache, static map, or interning table for descriptors or
  slugs.
