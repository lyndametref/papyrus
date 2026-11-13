---
id: TASK-110
title: Implement the generic document handler for every document type
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
  - TASK-107
documentation:
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/requirements/business/REQ-07.004.md
  - docs/requirements/business/REQ-07.001.md
priority: high
ordinal: 41020
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
There is no handler that can serve an arbitrary `document_types` entry. Every default type owns a module with its own sections, required fields, ID logic, and verbs, so a type added through configuration has no create, list, view, edit, archive, delete, or status implementation. This task adds one descriptor-parameterized handler that implements the baseline family and the generic content model, delegates only the declared extension verbs of built-in types, and threads every read and write through the existing record envelope. The registry-derived command tree will call it after the cutover.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/document.rs exposes the invocation value and run(ctx, registry, invocation) for create, list, view, edit, archive, delete, and status, parameterized by TypeDescriptor with no slug literals or slug match arms.
- [ ] #2 Generic create writes exactly id, title, created_date, updated_date, labels, references, status plus the generated heading and an optional Description section; title is required and non-empty, description is optional, and labels/references come from repeatable options.
- [ ] #3 A new generic record starts at the type's default_status or active; status is not accepted as a create or edit option and is never written by those paths.
- [ ] #4 Generic edit patches title and heading, the description section, and appended labels/references, preserves unknown sections and keys, supports --editor for the description only, and performs no write when nothing changes.
- [ ] #5 Generic view, list, archive, and delete use record::document, record::patch, record::identity, and record::store; archive and status follow ADR-0022 and no direct filesystem write or front-matter formatting is added.
- [ ] #6 IDs use the descriptor prefix and configured id_padding and are allocated across the active and archived locations; files live under doc_directory plus the entry directory and archive into its archived/ subfolder.
- [ ] #7 Built-in Typed descriptors keep their existing field sets and verbs at the baseline commands and delegate only create-group/--group/supersede to type-specific code; the generic model adds no field to them.
- [ ] #8 Tests cover a user-added generic type end to end (create, list, view, edit, archive, delete, status), a custom status vocabulary and default, an undeclared status, a hand-added section, and the typed req/nfr/ui-mockup paths.
<!-- AC:END -->
