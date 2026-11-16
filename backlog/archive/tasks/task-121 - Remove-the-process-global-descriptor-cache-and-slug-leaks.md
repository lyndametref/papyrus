---
id: TASK-121
title: Remove the process-global descriptor cache and slug leaks
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:30:00Z'
updated_date: '2026-11-15T10:10:00Z'
labels:
  - rust
  - refactor
milestone: m-18
dependencies:
  - TASK-120
references:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
documentation:
  - src/record/descriptor.rs
  - src/cli/tree.rs
  - >-
    docs/specifications/remove-the-process-global-descriptor-cache-and-slug-leaks.md
priority: medium
ordinal: 41230
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** `record::descriptor::generic_document_descriptor` keeps a process-global `OnceLock<Mutex<BTreeMap<..>>>` and fills it with `Box::leak`-ed slugs, descriptors, and section slices; `cli::tree::document_command` separately leaks one slug string per document command only to satisfy clap's `'static` command-name requirement. The leak is a lifetime workaround, not a design: memory grows per distinct slug, descriptor identity depends on process history, and it contradicts ADR-0017 decision 10 ("no cache"). The owner chose this refactor over documenting the leak in the ADR.

**What.** Delete the cache and every `Box::leak` in `src/`, and give the two dynamic facts — a document type's slug and the record descriptor derived from it — a lifetime-correct owner. The artifact registry already holds both: a config-defined type's record descriptor is built inside `ArtifactRegistry::build` from the type it already owns, and clap receives an owned command name.

**In scope.**

- `src/record/descriptor.rs`: remove `GENERIC_DESC_CACHE` and the leaked generic constructor; the three configuration-derived fields (record name, label, document-type key) are owned by the value instead of `&'static`, so a registry-supplied descriptor needs no leak.
- `src/registry.rs`: construct the record descriptor for every configured document type inside the registry build, including config-defined types with no static template.
- `src/cli/tree.rs`: build slug commands from owned names; enabling clap's `string` feature is acceptable because it adds no crate.
- The few value uses that relied on `Copy` (descriptor cloning) in `src/record/migrate.rs`, `src/requirement.rs`, and `src/rule.rs`.
- Tests: a source guard that fails when a leak or a static descriptor cache returns, and a double-build test proving two configurations share no state.

**Out of scope.**

- Changing descriptor content, the registry model, requirement-derived tables, or record behavior.
- Any new dependency (only the clap feature), any cache or interning table under another name, and any optimization of the existing per-call registry build in `record::migrate::managed_descriptor`.
- Editing requirements, business rules, or ADRs.

**Verification anchors.** `tests/cli_help_golden.rs` must pass byte-for-byte without regeneration; `tests/no_drift.rs` is the model for the leak-guard scanner; a config-defined slug (`note`) must keep dispatching to its own command and descriptor.

**Traceability.** ADR-0017 decisions 1, 4, 10; ADR-0016 decisions 1 and 4; ADR-0018 decision 1. Specification: `docs/specifications/remove-the-process-global-descriptor-cache-and-slug-leaks.md`. Depends on TASK-120 (registry as the single record-descriptor source) and TASK-119 (descriptor-driven command construction).

**Assumptions.**

- Assumption: no requirement or business-rule change is involved; behavior is preserved, so `docs/requirements/` and `docs/business-rules/` are not edited.
- Assumption: test identity of a descriptor means field equality, not pointer equality; nothing compares descriptor addresses today.
- Assumption: enabling clap's `string` feature does not violate ADR-0016 (clap stays the only parser and the only dependency).
- Assumption: the per-call registry build inside `record::migrate::managed_descriptor` is accepted as-is; it was already present before this ticket and must not be replaced by a new cache.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 Neither `GENERIC_DESC_CACHE` nor any `Box::leak` or `mem::forget` remains anywhere in `src/`, and a source-scanning test fails if a leak returns or if `record/descriptor.rs` or `cli/tree.rs` regains a static descriptor cache
- [x] #2 `generic_document_descriptor` no longer exists; a config-defined document type's record descriptor is owned by the `ArtifactRegistry` built from that configuration, and repeated `record_descriptor(slug)` calls on it return field-equal values
- [x] #3 Document slug commands are built from owned names — clap may gain the `string` feature, but no crate is added and no `Cargo.lock` package appears — and top-level help, slug help, and verb help are byte-identical to the committed golden snapshots without regeneration
- [x] #4 A test builds the command tree twice in one process, including once with a different `document_types` set in between, and asserts identical help output for the same registry, per-registry subcommand sets, and no cross-build state in the trees or the descriptors they resolve
- [x] #5 Existing CLI, browser, search, migration, and document-type tests pass unchanged; a config-defined slug still dispatches to its own command and to the descriptor carrying its prefix, directory, and status key
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean
- [x] #2 No new crate is added (only the clap feature changes) and no descriptor cache, interning table, or static slug map is introduced under another name
- [x] #3 The final summary states the ownership model (registry-owned record descriptor, owned clap command names), confirms descriptor identity no longer depends on process history, and confirms help text, dispatch, record I/O, and migration output are unchanged
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. **Ownership in `record::descriptor`.** Make the three configuration-derived `RecordDescriptor` fields owned (`Cow<'static, str>` for record name, label, and document-type key); the remaining fields stay `&'static` slices/literals, so `frontmatter_keys`, `sections`, and the descriptor methods keep their bodies. The type becomes `Clone` and loses `Copy`. Delete `GENERIC_DESC_CACHE`, the leaked generic constructor, and the `BTreeMap`/`Mutex`/`OnceLock` imports. Route the four `cfg.document_types.get(..)` lookups (`configured_prefixes`, `resolve_directory`, `declared_statuses`, and the default-status lookup in `record::migrate`) through `doc_type_key.as_ref()`; the `is_some()` checks are unaffected. `DESCRIPTORS` keeps its four static backlog entries with borrowed values.

2. **Registry owns document descriptors.** In `src/registry.rs`, `document_record()` fills the borrowed fields from the template, and the "no template" branch of `document_descriptor()` builds the generic record inline with the exact values the deleted constructor produced (`record_type`/`type_name` = `document`, label = slug, `GEN-###`, `doc` directory, `status` key, `## Description` section) using owned slug values. `record_descriptor()` keeps its `Option<&RecordDescriptor>` signature, so `document.rs:222`, `init.rs:436`, and the registry tests stay as they are. Widen the `no_parallel_document_record_table_remains` pattern from `doc_type_key: Some("` to `doc_type_key: Some(` so an owned per-slug literal cannot reintroduce the parallel table, and drop its "allowed until TASK-121" comment.

3. **Fix the five value uses** that relied on `Copy`: `migrate.rs:95` and `115` clone instead of deref-copy; `migrate.rs:1040` uses `.cloned()`; the `req`/`rule` test helpers use `.cloned().unwrap()`. No signature change elsewhere; every production path keeps passing `&RecordDescriptor`.

4. **clap owned names.** Add the `string` feature to the existing clap dependency (no crate added, lockfile package set unchanged), and pass `descriptor.slug.clone()` to `Command::new`, removing the leak in `cli::tree::document_command`. Verb names, arg ids, and help strings stay `&'static str`, so only the command name changes.

5. **Guards.** Add `tests/no_descriptor_leaks.rs` using the `tests/no_drift.rs` production-line scanner: fail on `Box::leak`, `mem::forget`, `GENERIC_DESC_CACHE`, `generic_document_descriptor`, or `OnceLock`/`LazyLock` in `record/descriptor.rs` and `cli/tree.rs`. Add `tests/tree_rebuild.rs`: build registry A (`document_types` = `note`) and registry B (`memo`, `glossary` disabled); assert `tree(A).render_help()` is identical before and after `tree(B)` is built, subcommand-name sets match their own registry, `note create --title x` still parses to slug `note` after B was built, and repeated `record_descriptor` calls on one registry are field-equal while the two registries never see each other's types.

6. **Verify no behavior change.** `tests/cli_help_golden.rs` passes byte-for-byte without `PAPYRUS_UPDATE_CLI_HELP`; typed CLI suites, `document_dispatch`, `document_type_no_drift`, `no_drift`, `migrate_records_cli`, the native-layout smoke suite, `search_*`, and `browser_*` pass untouched. `rg Box::leak src/` is empty; `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Ownership model: RecordDescriptor.name/label are Cow<'static,str> and doc_type_key is Option<Cow<'static,str>>; the registry owns the record descriptor for every configured document type, built inline in registry::document_descriptor, so descriptor identity depends only on the registry and never on process history. GENERIC_DESC_CACHE, generic_document_descriptor and Box::leak are deleted; cli::tree builds clap commands from owned slug names using clap's existing string feature (no crate or lockfile change). Behavior unchanged: 822 tests pass / 0 failed / 22 ignored, cli_help_golden byte-identical without regeneration, record I/O/migration/typed dispatch/search/browser suites unchanged. New guards: tree_rebuild (two registries, differing document_types, independent trees) and no_descriptor_leaks (source scan for leaks/caches).
<!-- SECTION:FINAL_SUMMARY:END -->
