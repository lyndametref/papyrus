---
id: TASK-126
title: >-
  Amend ADR-0016, ADR-0017, ADR-0018, and ADR-0023 for the derive/builder split,
  cross-category verbs, and release archive policy
status: To Do
assignee:
  - architect
created_date: '2026-11-15T11:28:00Z'
labels:
  - docs
  - architecture
milestone: m-19
dependencies: []
references:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
documentation:
  - docs/architecture/adr/index.html
modified_files:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
priority: high
ordinal: 41310
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The m-18 checkpoint found the accepted ADRs describing the delivered system inaccurately. ADR-0016 decision 1 says the clap derive macros were replaced for every built-in, while the built-in trees still use derive in `src/cli/builtin.rs` and the family modules and only the document surface is builder-built; the reserved-name table and downstream constraint 2 do not describe that split. ADR-0017 describes `verbs` as a document-only field even though every descriptor carries it, and ADR-0018 decision 3 has no record of the `ContentModel::HtmlMockup` exception that keeps archive/delete in the mockup module. ADR-0023 decision 2 still shows an archive location for releases although releases never move.

**What.** Amend the four accepted ADRs in place (no new ADR, status unchanged) so each records the delivered contract, then regenerate the ADR index.

**In scope.**
- ADR-0016: record the derive/builder split — built-in command trees stay clap-derive in `src/cli/builtin.rs` plus the family modules; `BUILTIN_COMMANDS` is the single reserved-name list consumed by validation and the man page; the document command surface is builder-built; owned command names use clap `string`. Align decision 1 and downstream constraint 2 to this split.
- ADR-0017 decision 3 and the `verbs` row: scope `verbs` to every category (Backlog, Decision, Document), not only documents, and restate that the document baseline is a lower bound.
- ADR-0018 decision 3: record the `ContentModel::HtmlMockup` archive/delete exception — those verbs stay in the mockup module, must call `record::store::archive_content`, and introduce no slug literal in `document::run`.
- ADR-0023 decision 2: releases have no archive location (`ArchivePolicy::None`) and change `planned -> released` in place.
- Regenerate `docs/architecture/adr/index.html` from the amended files.

**Out of scope.** Refactoring built-ins to the builder API (owner decided against it), changing any other ADR decision, and code changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ADR-0016 records the derive/builder split: built-in trees stay clap-derive in src/cli/builtin.rs and the family modules; BUILTIN_COMMANDS is the reserved-name list for validation and the man page; the document surface is builder-built; owned command names use clap string
- [ ] #2 ADR-0016 decision 1 and downstream constraint 2 no longer claim that all derive macros were removed and permit clap construction inside the cli built-in modules
- [ ] #3 ADR-0017 scopes the verbs field to all categories and keeps the document baseline as a lower bound
- [ ] #4 ADR-0018 decision 3 records the ContentModel::HtmlMockup archive/delete exception and requires record::store::archive_content with no slug literal in document::run
- [ ] #5 ADR-0023 decision 2 records ArchivePolicy::None for releases, with planned -> released in place and no archive folder
- [ ] #6 Each amendment is recorded in place with an amendment note and date; no ADR status changes and no contradictory statement remains in the four files
- [ ] #7 The ADR index is regenerated and lists the amended titles and statuses consistently with the files
<!-- AC:END -->
