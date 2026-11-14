---
id: TASK-121
title: Remove the process-global descriptor cache and slug leaks
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:30:00Z'
updated_date: '2026-11-14T20:30:00Z'
labels:
  - rust
  - refactor
milestone: m-18
dependencies:
  - TASK-120
references:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
documentation:
  - src/record/descriptor.rs
  - src/cli/tree.rs
priority: medium
ordinal: 41230
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** record::descriptor::generic_document_descriptor keeps a process-global OnceLock<Mutex<BTreeMap<String, &'static RecordDescriptor>>> and fills it with Box::leak-ed slugs, descriptors, and section slices; src/cli/tree.rs separately leaks each slug string only to satisfy clap's static command-name requirement. This is a lifetime workaround, not a design: it accumulates memory per distinct slug, makes descriptor identity depend on process history, and hides ownership the registry already has. The owner chose to refactor the code instead of documenting the leak in ADR-0017.

**What.** Delete GENERIC_DESC_CACHE and every Box::leak in src/, then give descriptor and command names a lifetime-correct owner. The registry (or a per-invocation owned value) supplies slugs and descriptors; nothing needs a static handle. Enabling clap's string feature is acceptable because it adds no crate, or an equivalent owned-name approach may be used. Behavior and performance for the normal one-invocation process must not regress.

**In scope.**

- src/record/descriptor.rs: remove the cache and the leaked constructor; return an owned descriptor or a registry-borrowed one.
- src/cli/tree.rs: build slug commands from owned names; no leaked slug.
- Any affected caller in src/document.rs and src/record/migrate.rs.
- Tests that build the command tree twice with different configurations and assert identical output and no cross-run state.

**Out of scope.**

- Changing descriptor content, the registry model, or record behavior; no new dependency, only an optional clap feature.
- Reintroducing a global cache under another name; caching is not required because the registry is built once per invocation.

**Note.** Depends on TASK-120 so the leak is removed after the duplicated record lookup, and on TASK-119 for the cli::tree.rs command construction.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Neither GENERIC_DESC_CACHE nor any Box::leak remains in src/; the source tree contains no Box::leak call
- [ ] #2 generic_document_descriptor no longer returns a value created by leaking; a config-defined type descriptor is owned by the registry or by the invocation that builds it
- [ ] #3 Clap slug commands are built from owned names — clap may gain the string feature, but no crate is added — and papyrus --help output is unchanged
- [ ] #4 Building the command tree twice in one process, including with different document_types sets, produces identical output and no shared state between builds
- [ ] #5 Existing CLI, browser, search, migration, and document-type tests pass unchanged; a config-defined slug still dispatches to its own command and descriptor
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo build, cargo test, and cargo clippy are clean.
- [ ] #2 No new crate is added; if the clap string feature is enabled, the lockfile change is limited to feature selection.
- [ ] #3 The final summary states the chosen ownership model and confirms descriptor identity no longer depends on process history.
<!-- DOD:END -->
