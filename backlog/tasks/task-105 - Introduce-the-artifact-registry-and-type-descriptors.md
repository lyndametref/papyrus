---
id: TASK-105
title: Introduce the artifact registry and type descriptors
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:24:00Z'
labels:
  - rust
  - architecture
milestone: m-17
dependencies: []
documentation:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/requirements/business/REQ-07.001.md
priority: high
ordinal: 41000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Papyrus models the same artifact set in four partial places: a compile-time record descriptor list, one clap subcommand enum per document type, `browser::records::Category`, and hard-coded search keys. A document type declared only in configuration is therefore invisible outside `Config`, and every new surface needs its own list. This task creates the single runtime model that the CLI, record layer, browser, and search will consume, so the rest of the feature has one source of type behavior. It is an internal refactor: no command name or output should change in this task.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/registry.rs exposes ArtifactRegistry built from Config with the Backlog, Decision, and Document category collections and a descriptor lookup by slug.
- [ ] #2 TypeDescriptor carries slug, category, label, prefix, directory, enabled, status model, verbs, content model, front-matter keys, id shape, and archive policy as specified by ADR-0017.
- [ ] #3 Built-in typed behavior is preserved as descriptor data: task/milestone/release/decision descriptors, all eleven default document types, req/nfr create-group with --group, ui-mockup supersede with no status verb and no custom statuses, and the Dotted/Grouped/Plain id shapes.
- [ ] #4 A configured slug with no static descriptor resolves to a generic descriptor: Document category, baseline verbs plus status, Generic content model, Plain id shape, ArchivedFolder policy, and prefix/directory from the entry.
- [ ] #5 DocTypeEntry gains an ordered statuses list and an optional default_status; existing configuration files load without error at this stage (validation is a later task).
- [ ] #6 ArtifactRegistry::build is fallible and asserts its invariants (unique slug, unique prefix, built-in collision, non-empty directory, a document status list containing active and archived with a valid default) without requiring a filesystem.
- [ ] #7 Unit tests cover built-in, typed, and generic descriptors and each registry invariant failure; no module outside the registry maps a slug to behavior.
<!-- AC:END -->
