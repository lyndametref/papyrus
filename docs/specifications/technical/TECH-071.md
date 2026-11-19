---
id: TECH-071
title: "Registry-Sourced Record Descriptors"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-16T17:47:32Z'
labels: []
references:
  - ADR-0013
  - ADR-0017
  - ADR-0018
  - ADR-0022
  - REQ-05.007
  - REQ-05.010
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - TASK-107
  - TASK-119
  - TASK-120
  - TASK-121
status: verified
---
# TECH-071 Registry-Sourced Record Descriptors

## Description
## Purpose

ADR-0017 decisions 1 and 4 make the artifact registry the only object that maps
a slug to behavior and require `descriptor_for_doc_type` to be replaced by a
registry lookup. TASK-119 routed every document command through
`document::run`, but the record layer still keeps a second, parallel table:
`record::descriptor::descriptor_for_doc_type` selects one of ten static
document entries, while `ArtifactRegistry` builds a `TypeDescriptor` for the
same slug from `document_types`. A config-defined type is described by a leaked
runtime record descriptor rather than by its registry entry, and the two tables
are kept aligned only by a test.

TASK-120 removes the second lookup for document types. Record-level data is
carried by the registry template that already owns the rest of the type, and
`ArtifactRegistry::record_descriptor` is the only accessor used by production
code. Backlog-family record descriptors (task, milestone, release, decision)
remain static because they are outside the document-type problem and are still
used by modules that have no registry at hand.

## Current state (verified)

- `descriptor_for_doc_type` has 26 production call sites in nine typed modules:
  `requirement.rs` (req/nfr), `glossary.rs`, `rule.rs`, `ui_spec.rs`,
  `audit.rs`, `tech_spec.rs`, `use_case.rs`, `test_case.rs`, `test_report.rs`.
  `ui_mockup.rs` has no call site.
- Further call sites: `src/document.rs` (`record_descriptor_for` helper used by
  generic create/list/view/edit and by archive/delete/status), `src/init.rs`
  (directory creation), and `src/record/migrate.rs` (skip check inside
  `managed_descriptor`).
- `src/record/descriptor.rs` defines 14 `RecordDescriptor` values; ten are
  document types. `RecordDescriptor` fields are all `&'static`; `sections` and
  `frontmatter_keys` are unused in production outside the descriptor methods.
- `src/registry.rs` already holds, per document template: `slug`, `label`,
  `content` (with `TypedSchema::sections`), `frontmatter_status`,
  `frontmatter_extra`, and `IdShape`. It builds `TypeDescriptor` from
  `document_types.<slug>` with prefix, directory, status model, verbs, and
  front-matter keys.
- Registry test `registry_agrees_with_record_descriptor_for_default_documents`
  compares the two tables; it is the only current guard.
- `deprecated/` folders are inert: `resolve_read_directories` returns active and
  `archived/` only (ADR-0022 decision 12, TASK-107). There is no production
  read of `deprecated/`, and `tests/document_type_no_drift.rs` D-01 forbids the
  literal in `src/`.

## Design

### 1. Record data lives on the registry template

Extend `DescriptorTemplate` (inside `src/registry.rs`) with the record-level
facts that are not already present:

| New template field | Purpose |
|---|---|
| `record_name` | Legacy migration key checked by `migrate.rs` (for example `"audit record"`, `"release"`); copied verbatim from the current `RecordDescriptor.name`. |
| `record_type` | Current `RecordDescriptor.record_type` value, copied verbatim to keep descriptor method branches unchanged. |
| `type_name` | Current `RecordDescriptor.type_name` value; retained as metadata (no production consumer today). |
| `display_id_pattern` | Current literal (for example `"REQ-###.###"`); `is_dotted_id()` keeps deriving from it. |
| `record_frontmatter_keys` | Type-specific keys only (`&["status"]`, or `&["status", "type"]` for audit, empty for ui-mockup), matching today's `RecordDescriptor.frontmatter_keys`. |

Sections need no new field: `ContentModel::Typed(schema).sections` already holds
the same arrays as `record::descriptor`'s section constants. For a generic
document type use `&["## Description"]`; for ui-mockup use the empty slice.

### 2. One accessor on the registry

- Add `record: Option<RecordDescriptor>` to `TypeDescriptor`. Document
  descriptors set `Some`; backlog and decision descriptors set `None`.
- Build document records in `document_descriptor()` from the matched template:
  `directory = "doc"`, `subpath = ""`, `doc_type_key = Some(template.slug)`,
  and the fields above. For a config-defined type with no template, set
  `record = Some(*generic_document_descriptor(slug))`; that constructor and its
  cache/leak stay unchanged until TASK-121.
- Add `ArtifactRegistry::record_descriptor(&self, slug) -> Option<&RecordDescriptor>`
  returning `descriptor(slug).and_then(|d| d.record.as_ref())`. It is the only
  slug-to-record lookup; there is no free function that resolves a document
  record.
- Delete `record::descriptor::descriptor_for_doc_type`. Remove the ten document
  entries from `DESCRIPTORS` and the section/key constants they alone used.
  `DESCRIPTORS` keeps task, milestone, release, and decision.

### 3. Field-source matrix

| `RecordDescriptor` field | Document source |
|---|---|
| `name` | `DescriptorTemplate.record_name` |
| `record_type` | `DescriptorTemplate.record_type` |
| `type_name` | `DescriptorTemplate.type_name` |
| `label` | `DescriptorTemplate.label` |
| `display_id_pattern` | `DescriptorTemplate.display_id_pattern` |
| `directory` | `"doc"` (the `doc_directory` configuration key) |
| `subpath` | `""` (the config entry supplies the relative directory) |
| `doc_type_key` | `Some(slug)`; generic descriptor gets the config slug |
| `frontmatter_keys` | type-specific keys from `frontmatter_status` + `frontmatter_extra`; generic `&["status"]` |
| `sections` | `TypedSchema.sections`; generic `&["## Description"]`; ui-mockup `&[]` |

`resolve_directory`, `configured_prefixes`, `matches_stem`,
`resolve_archived_directory`, `resolve_read_directories`, and
`declared_statuses` keep their current bodies and read the registry-supplied
`doc_type_key` plus the effective `Config`.

### 4. Call-site migration

| File | Change |
|---|---|
| `src/document.rs` | Delete `record_descriptor_for`; resolve `record` once in `document::run` through `registry.record_descriptor(&descriptor.slug)` and pass `&RecordDescriptor` to generic create/list/view/edit and to archive/delete/status helpers. Missing record is an internal `AppError::Runtime`. |
| `src/document.rs` `dispatch_typed` | Resolve the record once and pass it to every typed handler next to the descriptor. |
| `src/requirement.rs` | Replace `kind_dotted()` with `record.is_dotted_id()`; `handle` gains the record argument and forwards it to create/list/view/edit/archive/delete/create-group. |
| `src/glossary.rs` | Replace `glossary_dotted()` and the `handle` lookup with the passed record. |
| `src/rule.rs`, `src/ui_spec.rs`, `src/audit.rs`, `src/tech_spec.rs`, `src/use_case.rs`, `src/test_case.rs`, `src/test_report.rs` | Same pattern: delete each module-local dotted helper, pass the record into `handle` and its private create/list/view/edit functions, and use `record.is_dotted_id()`. |
| `src/ui_mockup.rs` | No change; HTML records have no shared record descriptor and keep the TASK-119 exception. |
| `src/init.rs` | Replace the `document_types` loop's `descriptor_for_doc_type` call with a registry build over the effective config, iterating enabled document descriptors and resolving directories from their record descriptor. Backlog and decision directory creation keeps `descriptor_for`. |
| `src/record/migrate.rs` | In `managed_descriptor`, remove the `descriptor_for_doc_type` skip; iterate the four backlog descriptors plus the registry's document record descriptors (built from the loaded config). Return an owned `RecordDescriptor` (it is `Copy`) so the registry can be dropped. In `migrate_corpus`, iterate the same two sets. Set `managed_descriptor`'s return type accordingly and adapt callers in `store.rs`, `resolve.rs`, and `migrate.rs` to borrow the owned value. |
| `src/registry.rs` | Replace the agreement test with the invariant tests in §6. |

`DocumentScope` stays unchanged: active, archived, prefix, and padding already
come from the descriptor and config. Modules must not join config paths again.

## 5. Legacy and no-front-matter behavior

- Reading a native record: unchanged. `resolve_read_directories` still returns
  the active directory plus `archived/` (and the task/milestone legacy alias),
  with no `deprecated/` path.
- Legacy `deprecated/` folders: unchanged and inert. They are not scanned,
  listed, migrated, or resolved, per ADR-0022 decision 12 and TASK-107. The
  ticket must not claim they are read.
- File without front matter: `store::write_atomic` calls `managed_descriptor`;
  the registry-backed document record keeps the same `matches_stem`,
  `resolve_read_directories`, `record_name`, and `record_frontmatter_keys`
  values as the deleted static entry, so `ensure_front_matter` and
  `adoption_ops` seed the same keys in the same order. The audit branch still
  matches `record_name == "audit record"`.
- Disabled document type: still absent from `init` directory creation and from
  migration, as today. `record_descriptor` may still return it (registry keeps
  disabled entries), but command dispatch rejects disabled types before use.
- Config-defined type: `record_descriptor` returns the same leaked generic
  descriptor as today, so directory, prefix, status key, and matching behavior
  are unchanged.

## 6. Single-source invariant tests

Replace the two-table comparison with tests that assert derivation:

1. **Totality** — every `ArtifactCategory::Document` descriptor has
   `record_descriptor(slug) == Some`; the count equals the document descriptor
   count; a slug not in the registry (`deploy` when undeclared, and an unknown
   slug) returns `None`.
2. **Field agreement** — for every document descriptor: `doc_type_key ==
   Some(slug)`, `label` equals the descriptor label, `configured_prefixes(cfg)`
   equals `[descriptor.prefix]`, `is_dotted_id()` matches `id_shape != Plain`,
   `frontmatter_keys` contains `status` exactly when the status model is
   front-matter-backed, `sections` equals the typed schema sections (or the
   generic/HTML default), and `resolve_directory` equals
   `doc_directory.join(descriptor.directory.relative)`.
3. **No parallel document table** — `record::descriptor::DESCRIPTORS` contains
   no entry with `doc_type_key.is_some()`; a source scan forbids
   `descriptor_for_doc_type` and `doc_type_key: Some(` in
   `src/record/descriptor.rs` production lines.
4. **Built-in and config-defined parity** — with a config that declares `note`
   as a generic type, both `glossary` (typed built-in) and `note` resolve
   through `record_descriptor`, and each returns fields sourced from its own
   registry descriptor (`doc_type_key`, prefix, directory, status key,
   sections).

## 7. Test plan

Unit (`src/registry.rs`, `src/record/descriptor.rs`):
- Totality, field agreement, and parity tests from §6.
- `descriptor_count_excludes_ui_mockup` becomes the four-backlog-descriptor
  assertion; assert no document `doc_type_key` in `DESCRIPTORS`.
- Generic and ui-mockup record shapes (empty front-matter keys/sections).

Integration (existing suites, unchanged expectations):
- Every typed CLI suite: `req`, `glossary`, `business_rules`, `ui_spec`,
  `audit`, `technical_specifications`, `use_cases_test_cases_test_reports`,
  `ui_mockup`.
- `tests/document_dispatch.rs`, `tests/document_type_no_drift.rs`,
  `tests/migrate_records_cli.rs`, `tests/release_native_layout_smoke.rs`,
  `tests/search_*`, `tests/browser_*`, `tests/no_drift.rs`.
- A focused migration test: a document record without front matter still gets
  `status` (and `type` for audit) after the call-site change; a file in an
  inert `deprecated/` folder is still not managed.
- A config-defined type test: create/list/view/edit/archive/delete still work
  through the registry record descriptor.

Mechanical checks:
- `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- `rg descriptor_for_doc_type src/` returns nothing.

## 8. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Migration output drifts because `desc.name` branches change | Copy `record_name` verbatim, including `"audit record"`; run `tests/migrate_records_cli.rs` and the native-layout suite. |
| Registry rebuild on every `managed_descriptor` call adds cost on read/write paths | Build once per command or loop where possible; per-record builds are small and dominated by filesystem I/O. Caching is TASK-121. |
| Shrinking `DESCRIPTORS` breaks an untracked caller | The compiler catches every reference; `RECORD_DESCRIPTORS` re-export is updated or removed with its last user. |
| Typed module signature churn causes behavior drift | Change one module per commit, keep section/validation logic untouched, and rely on each module's CLI/smoke suite. |
| `ui-mockup` accidentally enters the shared record path | Keep the `ContentModel::HtmlMockup` exception and leave `ui_mockup.rs` untouched. |
| Generic leak/cache becomes entrenched | Leave `generic_document_descriptor` as the documented constructor; TASK-121 owns its removal. |

## Out of scope

- Removing the generic descriptor cache and slug leak (TASK-121).
- Rewriting the four backlog-family record descriptors or the modules that use
  them (`task.rs`, `milestone.rs`, `release.rs`, `import.rs`).
- Changing record file layout, front-matter keys, ID shapes, statuses,
  migration outcomes, CLI output, JSON, or help text.
- Editing requirements, business rules, or ADRs.
