---
id: TASK-246
title: "Replace 'static descriptors with owned values derived from the seed and configuration"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-22T10:00:27Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - TECH-071
  - TECH-072
  - TC-128
status: Done
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-245
ordinal: 30
documentation:
  - src/registry.rs
  - src/record/descriptor.rs
  - docs/specifications/technical/TECH-071.md
  - docs/specifications/technical/TECH-072.md
type: implementation
---

## Description
Context: MS-033 start commit c3103fc. The descriptor layer still assumes static data:
- `src/registry.rs`: `TypedSchema` holds `sections: &'static [&'static str]`, `extensions: &'static [Verb]`, `verbs: &'static [VerbSpec]`; `ArgSpec` holds `id`, `long`, `value_name`, `help`, `required_unless_present`, `conflicts_with` as `&'static str`; `VerbSpec` holds `about: &'static str` and `args: &'static [ArgSpec]`; `TypeDescriptor.frontmatter_keys` is `Vec<&'static str>`.
- `src/record/descriptor.rs`: `RecordDescriptor` keeps `record_type`, `type_name`, `display_id_pattern`, `directory`, `subpath` as `&'static str` and `frontmatter_keys`, `sections` as `&'static [&'static str]`; only `name`, `label`, and `doc_type_key` are `Cow<'static, str>`.
- `src/registry.rs::document_descriptor` builds owned `TypeDescriptor` values from `DocTypeEntry`, but its schema and verb metadata still point into the static `DOCUMENT_TEMPLATES`; `document_record` inlines `&["status"]` and `&["## Description"]` for a user-defined slug.

Scope:
1. `ArgSpec` and `VerbSpec` own their data. Text members become owned-or-borrowed (`Cow<'static, str>` or `String`) and `VerbSpec.args` becomes an owned sequence. Drop `Copy` where ownership forbids it; keep `Clone`, `Debug`, `PartialEq`, `Eq`.
2. `RecordDescriptor` owns every fact it publishes: `name`, `label`, `record_type`, `type_name`, `display_id_pattern`, `directory`, `subpath`, `doc_type_key`, `frontmatter_keys`, `sections`. It stays `Clone` and not `Copy`; hot paths pass `&RecordDescriptor`.
3. `TypeDescriptor.frontmatter_keys` becomes owned and keeps the written order: the general keys, then `status` where the model stores it, then the type-specific keys.
4. Every descriptor is reconstructible from the values just built. `ArtifactRegistry::build(&Config)` builds each document descriptor and its `record: Option<RecordDescriptor>` from the configuration entry and the current template facts. No descriptor type carries a lifetime parameter or a `'static` bound, and nothing is interned, leaked, or held in a process-global map (TECH-071 section 1, TECH-072 section 1, ADR-037 decision 31).
5. Verbs stop being a hand-written per-type list. The derivation is keyed on the document model, which TASK-247 introduces; TASK-249 ("Derive verbs, options, and help from the fields and model") replaces the remaining document verb arrays with that derivation. This task removes the `'static` storage and keeps the current content-model to verb-spec relationship as data, so the tree compiles and behaves identically. This is a sequencing clarification inside the existing dependency chain, not a plan change.
6. Backlog (`task`, `milestone`, `release`) and decision descriptors keep their slug, category, label, prefix source, directory key, subpath, ID shape, status model, verb list, argument metadata (id, long, short, value name, help, positional, required, required-unless-present, conflicts-with, action), front-matter keys, and order.

Invariants:
- Two registries built from two different `document_types` maps are independent: building one cannot change the other's descriptors or command tree, and repeated lookups return field-equal values (TECH-072; `tests/tree_rebuild.rs`).
- No process-global descriptor cache, interning table, or leaked slug is introduced. ADR-037's serving snapshot (TASK-274, MS-036) is the only cache the deliverable will have; this milestone adds none.
- Old configuration files must parse to the same effective document-type descriptor set, and no CLI or browser behaviour changes in this milestone: document verbs, long and short options, option order, help text, root type order, record bytes, JSON, HTTP status codes, and browser pages stay byte-identical (MS-033 AC-1, AC-7).

Out of scope: the configuration file shape, the seed table, selector paths, the read path, and the nine typed document modules. No document field, heading, option, or role set is added or changed here, and no dependency, ordinal, milestone, or assignee changes.

Assumptions:
- `Cow<'static, str>` is acceptable where a value may borrow a seed literal, because TECH-071 section 1 permits owned-or-borrowed values that are not required to borrow.
- The existing typed schema constants remain the source of document section and verb data until TASK-249 derives it; deleting those modules is TASK-259 in MS-034.

Definition of done: `cargo build`, `cargo test`, and `cargo clippy --all-targets -- -D warnings` are clean; the CLI-help goldens and the characterization replay are unchanged; no `&'static` member remains on `ArgSpec`, `VerbSpec`, `RecordDescriptor`, or `TypeDescriptor`; the descriptor-leak source guard passes.
## Acceptance Criteria
- [ ] `ArgSpec`, `VerbSpec`, `RecordDescriptor`, and `TypeDescriptor` declare no `&'static` member and no lifetime parameter; `VerbSpec.args` is an owned sequence, and `cargo build` passes without a lifetime argument on any of them (TECH-071 section 1, TECH-072 section 1).
- [ ] A `RecordDescriptor` for a configuration-defined slug (`note`, prefix `NOTE`, directory `notes`) returns its own name, label, doc-type key, front-matter keys, and sections, and `configured_prefixes(&Config)` returns the entry prefix; two registries built from different `document_types` maps return different, independent record descriptors (TECH-071 section 2, ADR-037 decision 31).
- [ ] `TypeDescriptor.frontmatter_keys` for every seed type equals the pre-change sequence, verified on `Config::default()`: `id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, then the type-specific keys in the current written order (REQ-05.010, ADR-0023 decision 5).
- [ ] A source guard fails on `Box::leak`, `OnceLock`, a `static Mutex`, a `thread_local` descriptor map, or a free document-descriptor constructor in production lines under the source tree, and passes on the current tree; `no_descriptor_leaks.rs` covers this (ADR-037 decision 31, TECH-072 downstream constraint 6).
- [ ] `tree_rebuild.rs` passes unchanged: two command trees and two registries in one process share no state, and the second build leaves the first tree's help and record descriptors unchanged.
- [ ] Backlog and decision descriptors are unchanged: every task, milestone, release, and decision entry of the cli-help verb goldens matches byte-identically, and the task, milestone, release, and decision suites pass.
- [ ] Document output is unchanged for the cli-help fixture configuration: the root type order, each `papyrus <slug> --help` verb list, and each `create` and `edit` long-option list and order match the cli-help goldens (MS-033 AC-7, TC-128).
- [ ] `cargo test` reports zero failures and `cargo clippy --all-targets -- -D warnings` is clean.
## Comments
- **task-refiner**: Refined for MS-033: description grounded in registry.rs ArgSpec, VerbSpec, TypeDescriptor and record descriptor.rs RecordDescriptor; 8 testable acceptance criteria added; TECH-071 and TECH-072 documented. Verb derivation sequencing clarified: storage conversion here, derivation in TASK-249 where DocumentModel exists. No dependency, ordinal, milestone, or assignee change.
- **rust-dev**: -
- **rust-dev**: Changed: registry.rs ArgSpec and VerbSpec own Cow text with owned Vec args; TypedSchema and ContentModel own their data; TypeDescriptor.frontmatter_keys is Vec<String>; the record descriptor owns text and sequences and the backlog and decision descriptors are built on demand; consumers updated.
- **rust-dev**: Tests: no_descriptor_leaks guards process-global state and the four descriptor structs; registry tests cover the pre-change front-matter key order, config-defined record ownership and independence, and owned VerbSpec values.
- **rust-dev**: Verification: cargo build clean; env -u EDITOR -u VISUAL cargo test 1145 passed, 0 failed, 26 ignored; tree_rebuild 7 of 7, cli_help_golden 2 of 2, characterization_replay 3 of 3; cargo clippy --all-targets -D warnings clean; cargo fmt --check clean.
- **rust-dev**: Note: AC3 key order conflicts with ADR-0023 decision 5 and the pre-change code, so the pre-change order was kept. No commit made (per instruction).
- **orchestrator**: Acceptance met: descriptor types are owned with no static member or lifetime, config-defined records carry their own registry-local descriptor, seed front-matter order unchanged, no process-global state or leaking constructors (new guards); suite 1145 passed, clippy clean.
