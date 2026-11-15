# Feature Specification — Remove the Process-Global Descriptor Cache and Slug Leaks

**Ticket:** TASK-121
**Milestone:** m-18 (Descriptor Single-Source Cleanup)
**Status:** Refined — execution-ready
**UI mockups:** none. Backend refactor with no user-visible interface.
**Requirements:** none new. Behavior is preserved; the relevant constraints are
REQ-07.001 (three artifact categories), REQ-07.002 (config-defined types need
no code), REQ-07.003 (baseline document verbs), REQ-01.004 (exit codes).
**ADRs:** ADR-0017 (decision 10: "No new dependency, no new process model … no
cache"), ADR-0016 (one command tree built at startup from the registry),
ADR-0018 (generic document content model), ADR-0013 (record layer ownership).
**Predecessors:** TASK-119 (descriptor-driven dispatch), TASK-120 (registry as
the single record-descriptor source) — both completed.

## Purpose

Two `Box::leak` sites survive the TASK-119/TASK-120 refactor even though the
registry already owns every value they leak:

| Site | Leak |
|---|---|
| `src/record/descriptor.rs` (`GENERIC_DESC_CACHE`, `generic_document_descriptor`) | A `OnceLock<Mutex<BTreeMap<String, &'static RecordDescriptor>>>` plus one leaked slug, one leaked `RecordDescriptor`, and leaked section slices per distinct config-defined slug. |
| `src/cli/tree.rs` (`document_command`) | One leaked slug string per document command, only because `clap::Command::new` demands `'static` without clap's `string` feature. |

Effects of the current shape:

- memory grows per distinct slug and is never released;
- the descriptor's identity depends on process history (first caller wins the
  cache entry), which contradicts ADR-0017 decision 10's "no cache, no index";
- ownership is misrepresented: the `ArtifactRegistry` already holds an owned
  `slug`, `label`, `prefix`, `directory`, verbs, and record facts for exactly
  these types, and TASK-120 already builds a `RecordDescriptor` for every
  document type except the config-defined (generic) case.

TASK-121 removes the cache and every leak, and gives the two facts that are
dynamic per configuration — the slug string and the record descriptor derived
from it — a lifetime-correct owner.

## Current state (verified at TASK-121 start, branch `task/TASK-121`)

- `src/record/descriptor.rs:305-333`: `GENERIC_DESC_CACHE`, the
  `OnceLock<Mutex<BTreeMap<..>>>`, and `generic_document_descriptor(slug) ->
  &'static RecordDescriptor`, which leaks `slug`, the descriptor, and
  `&["## Description"]`/`&["status"]` slices.
- `src/cli/tree.rs:74-80`: `document_command` clones the slug and leaks it via
  `Box::leak` for `Command::new`.
- `rg Box::leak src/` returns exactly these three lines; `OnceLock`, `OnceCell`,
  `LazyLock`, `lazy_static`, and `mem::forget` appear nowhere else in `src/`.
- `RecordDescriptor` (`src/record/descriptor.rs:10-25`) is `Copy` and every
  field is `&'static str` / `&'static [&'static str]`. `DESCRIPTORS` holds the
  four backlog families; document descriptors come from
  `ArtifactRegistry::record_descriptor` (`src/registry.rs:2471`).
- `src/registry.rs:2662-2669`: `document_descriptor()` sets
  `record = Some(*generic_document_descriptor(slug))` for a type with no static
  template — the single remaining consumer.
- `RecordDescriptor` is consumed by reference everywhere except five value
  uses: `registry.rs:2666` (above), `migrate.rs:95`, `migrate.rs:115`,
  `migrate.rs:1040` (`.copied()`), and the `req`/`rule` record helpers in
  `requirement.rs:738` / `rule.rs:586` (both deref a `&RecordDescriptor`).
- `doc_type_key` lookups that need `.as_ref()` after the change: three in
  `src/record/descriptor.rs` (`configured_prefixes:44`, `resolve_directory:90`,
  `declared_statuses:171`) plus `src/record/migrate.rs:306-310`. The
  `is_some()` checks (`descriptor.rs:135,170`, `migrate.rs:305,1232`) need no
  change.
- `src/registry.rs:3696-3724` (`no_parallel_document_record_table_remains`)
  already scans production lines of `record/descriptor.rs` and forbids
  `doc_type_key: Some("`. With an owned field the literal becomes
  `doc_type_key: Some(Cow::Borrowed("…"))`, which that pattern would miss, so
  the pattern must widen to `doc_type_key: Some(`; the stale "allowed until
  TASK-121" comment about the generic constructor is removed with the
  constructor.
- `src/record/migrate.rs:81-121` (`managed_descriptor`) builds a fresh
  `ArtifactRegistry` per call and returns an **owned** `RecordDescriptor` that
  outlives that registry. `migrate_corpus` (`1033-1041`) collects owned
  descriptors the same way.
- `clap = { version = "4", features = ["derive"] }` (`Cargo.toml:13`);
  `string` is not enabled. `clap 4.6.7` is already locked; the `string` feature
  pulls no additional package.
- `tests/cli_help_golden.rs` asserts byte-for-byte help for the top-level tree,
  all eleven typed slugs, every verb, the config-defined slug `note`, and the
  exit-2 usage errors. It is the behavior-preservation anchor.
- `tests/no_drift.rs` and `tests/document_type_no_drift.rs` already provide a
  production-line source scanner (`#[cfg(test)]` items skipped) that TASK-121
  can extend for the leak guard.

## Design

### 1. Ownership model: the registry owns document descriptors

`RecordDescriptor` stays one type and keeps its name, but the three fields that
can be configuration-derived become `Cow<'static, str>`:

| Field | Type after TASK-121 | Document source | Backlog source |
|---|---|---|---|
| `name` | `Cow<'static, str>` | `Cow::Borrowed(record_name)` for a template, `Cow::Owned(slug)` for a generic type | `Cow::Borrowed("task")` … |
| `label` | `Cow<'static, str>` | `Cow::Borrowed(template.label)` / `Cow::Owned(slug)` | `Cow::Borrowed("Task")` … |
| `doc_type_key` | `Option<Cow<'static, str>>` | `Some(Cow::Borrowed(template.slug))` / `Some(Cow::Owned(slug))` | `None` |
| `record_type`, `type_name`, `display_id_pattern`, `directory`, `subpath` | unchanged `&'static str` | static literals (`"document"`, `"GEN-###"`, `"doc"`, `""`) | unchanged |
| `frontmatter_keys`, `sections` | unchanged `&'static [&'static str]` | static slices | unchanged |

Consequences:

- `RecordDescriptor` is `Clone` but no longer `Copy`. It is still a small
  plain-data value; everything that needs a long-lived copy clones, and every
  hot path keeps passing `&RecordDescriptor`.
- `DESCRIPTORS` keeps its `const` form with `Cow::Borrowed(..)` values, so the
  four backlog families are still compile-time data with no allocation.
- The registry's `TypeDescriptor.record: Option<RecordDescriptor>` **owns** the
  document descriptor for each configured type, including config-defined ones.
  `ArtifactRegistry::record_descriptor(&self, slug) -> Option<&RecordDescriptor>`
  keeps its signature; borrowing from the registry is now the only way to get a
  document record.
- `generic_document_descriptor` and `GENERIC_DESC_CACHE` are deleted.
  `document_descriptor()` builds the generic record inline in the `None`
  template branch with the exact values the deleted constructor produced
  (`record_type == "document"`, `type_name == "document"`,
  `label == slug`, `display_id_pattern == "GEN-###"`, `directory == "doc"`,
  `subpath == ""`, `frontmatter_keys == &["status"]`,
  `sections == &["## Description"]`).
- There is no cache and no static storage for document records: two registries
  built from two configurations each describe only their own type set, and the
  same registry returns field-equal descriptors on every call.

Rejected alternative — make `RecordDescriptor<'a>` a borrowed view of the
registry. It reads well, but `managed_descriptor` must return a descriptor that
outlives its locally built registry (`migrate.rs:99-100`, `1035-1041`), so the
value would either need a second owned type or a lifetime escape hatch. Owning
the three dynamic fields inside the existing type is the smaller, safer change
and keeps every public signature and test comparison intact.

### 2. clap: owned command names

`clap`'s `string` feature is enabled in `Cargo.toml`
(`features = ["derive", "string"]`), which makes `Command::new` accept a
`String`. `document_command` then passes `descriptor.slug.clone()` and the
`Box::leak` disappears. No crate is added: `clap_builder` is already a
dependency and the feature only flips `Str` from `&'static str` to an owned
representation. No `Cargo.lock` package change is expected (features are not
recorded there); if `cargo` does rewrite the lock, only `clap`'s dependency list
may change and no package may be added.

Unchanged because they are already `&'static str`: global arg ids, verb names
(`spec.verb.as_str()`), `ArgSpec.id`/`long`/`short`/`value_name`/`help`, and the
`about` line, which is already built from `format!` (owned `StyledStr`).

Command names are never stored beyond the `Command`: `dispatch` reads
`matches.subcommand()` (`src/cli/dispatch.rs:17`) and passes the borrowed
`&str` to `parse_document_invocation` and `DocumentInvocation::new`, so no
signature changes.

### 3. Call sites

| File | Change |
|---|---|
| `src/record/descriptor.rs` | `Cow` fields; drop `Copy`; delete the cache, the leaked constructor, and the `BTreeMap`/`Mutex`/`OnceLock` imports; three `cfg.document_types.get(..)` lookups go through `doc_type_key.as_ref()`. |
| `src/registry.rs` | `document_record()` builds `Cow::Borrowed` fields; the `None` template branch builds the generic record inline; delete the `generic_document_descriptor` call; widen the `no_parallel_document_record_table_remains` pattern to `doc_type_key: Some(`. |
| `src/record/migrate.rs` | `return Some(*desc)` → `Some(desc.clone())` at lines 95 and 115; `.copied()` → `.cloned()` at 1040; `doc_type_key.as_ref()` in the default-status lookup at 306-310. |
| `src/requirement.rs`, `src/rule.rs` | Test helper: `*registry.record_descriptor(..).unwrap()` → `.cloned().unwrap()`. |
| `src/cli/tree.rs` | `Command::new(descriptor.slug.clone())`; delete `slug_static`. |
| `src/document.rs` | No change; it already resolves the record once at `document.rs:222-231` through `registry.record_descriptor`. |

`managed_descriptor`'s per-call registry build predates TASK-121 (TASK-120
§8). It stays as-is: the ticket does not introduce a cache under another name,
and the removed leak never avoided that build, so read/write cost is unchanged
apart from a few small `Cow::Owned` allocations already dominated by the
registry build itself.

## Behavior preservation

- **Help and usage text** — byte-identical: same command names, order, about
  lines, verbs, args, and exit codes. `tests/cli_help_golden.rs` must pass with
  no regeneration (`PAPYRUS_UPDATE_CLI_HELP` must not be set).
- **Dispatch** — a config-defined slug still resolves through
  `dispatch → registry.descriptor(name) → document::run`, and its record
  descriptor carries the same `doc_type_key`, prefix, directory, status key,
  and stem matching as today.
- **Record I/O and migration** — `resolve_read_directories`,
  `matches_stem`, `configured_prefixes`, and `declared_statuses` keep their
  bodies; the record values are field-identical to the leaked ones, so
  front-matter seeding, adoption, archive paths, and migration output do not
  change. Legacy `deprecated/` folders stay inert (ADR-0022 decision 12).
- **Disabled types** — still absent from the command tree and still skipped by
  migration, exactly as TASK-120 left them.
- **Errors** — validation/exit-code mapping (REQ-01.004) untouched.

## Test plan

1. **Leak guard (new `tests/no_descriptor_leaks.rs`)**, using the
   `tests/no_drift.rs` production-line scanner over `src/`: fails on any
   `Box::leak`, `mem::forget`, `GENERIC_DESC_CACHE`,
   `generic_document_descriptor`, or `OnceLock`/`OnceCell`/`LazyLock` in
   `src/record/descriptor.rs` and `src/cli/tree.rs`. This makes AC #1
   regression-proof instead of a one-time grep. The existing
   `src/registry.rs` no-parallel-table test keeps its widened
   `doc_type_key: Some(` rule.
2. **Double build (new `tests/tree_rebuild.rs`)** — in one process:
   - registry A (`document_types`: `note`), registry B (`memo`, and `glossary`
     disabled), both via `ArtifactRegistry::build(&Config)`;
   - `tree(A)` help equals a second `tree(A)` after `tree(B)` was built
     (`render_help()` string equality);
   - subcommand-name sets match each registry (A has `note`, not `memo`; B has
     `memo`, not `note`, and no `glossary`);
   - parsing `note create --title x` against the rebuilt A tree still yields
     slug `note` with verb `create`;
   - the two configs yield different `record_descriptor("glossary")`/`("note")`
     results per registry and identical results across repeated calls on one
     registry (field equality; no cross-registry state).
3. **Registry unit tests** (`src/registry.rs`) keep passing unchanged,
   including `record_descriptor_resolves_builtin_and_config_defined_types_alike`
   — `Cow` comparisons with `&str` keep those assertions valid.
4. **Descriptor unit tests** (`src/record/descriptor.rs`): `DESCRIPTORS.len() ==
   4` and no document `doc_type_key` remain; the generic record shape is
   asserted through a registry built with a config-defined type.
5. **Existing suites, unchanged**: `cli_help_golden`, `cli_dispatch`,
   `document_dispatch`, `document_type_no_drift`, `no_drift`, all typed CLI
   suites (`req`, `glossary`, `business_rules`, `ui_spec`, `audit`,
   `technical_specifications`, `use_cases_test_cases_test_reports`,
   `ui_mockup`), `migrate_records_cli`, `release_native_layout_smoke`,
   `search_*`, `browser_*`.
6. **Mechanical checks**: `cargo build`, `cargo test`,
   `cargo clippy --all-targets -- -D warnings`; `rg Box::leak src/` empty.

## Assumptions

- Assumption: no requirement or business rule changes; this is a refactor with
  preserved behavior, so `docs/requirements/` and `docs/business-rules/` are not
  edited.
- Assumption: enabling clap's `string` feature is acceptable and does not
  violate ADR-0016's "clap only dependency" rule; the ticket states it
  explicitly.
- Assumption: descriptor identity required by tests is field equality, not
  pointer equality; nothing in the codebase compares descriptor addresses.
- Assumption: the per-call registry build in `managed_descriptor` is out of
  scope for this ticket (TASK-120 §8 already deferred it) and must not be
  replaced by a new cache.

## Out of scope

- Changing descriptor content, the registry model, requirement-derived tables,
  or record behavior.
- Adding any crate; only the clap feature flag changes.
- Introducing any cache, static map, or interning table for descriptors or
  slugs.
- Optimizing `managed_descriptor`/`migrate_corpus` registry construction.
- Editing ADRs, requirements, or business rules.

## Risks and mitigations

| Risk | Mitigation |
|---|---|
| Dropping `Copy` breaks a value use at a distance | Compiler-enforced; the five known value uses are enumerated in §1 and §3, and a clone is semantically identical. |
| `Cow` comparisons silently change matching | `Cow<str> == &str` exists; the existing `doc_type_key`/`label` assertions and every stem-matching test run unchanged. |
| clap `string` changes help formatting | Golden suite is byte-for-byte and must not be regenerated. |
| Hidden cross-run state reappears later | `tests/no_descriptor_leaks.rs` source guard plus the double-build test. |
| Migration output drifts | `record_name`/`display_id_pattern`/key order are copied verbatim; `migrate_records_cli` and the native-layout smoke suite run unchanged. |
| Generic descriptor drifts from the deleted constructor | Unit test asserts the generic record shape (`record_type`, `label`, `doc_type_key`, `sections`, `frontmatter_keys`) through a config-defined type. |

## Traceability

- ADR-0017 decision 1 (registry is the only slug→behavior map), decision 4
  (record layer gains `Registry::descriptor(slug)`), decision 10 ("no cache").
- ADR-0016 decisions 1 and 4 (one command tree at startup; `document_command`
  builds slug commands from the descriptor).
- ADR-0018 decision 1 (generic content model: seven keys + `## Description`),
  which the generic record descriptor mirrors.
- TASK-120 spec §2 (the generic constructor is explicitly deferred to this
  ticket).
