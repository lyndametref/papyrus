---
id: TASK-116
title: Write upgrade notes and align the README with dynamic document commands
status: To Do
assignee:
  - tech-writer
created_date: '2026-11-13T21:26:00Z'
labels:
  - docs
milestone: m-17
dependencies:
  - TASK-108
  - TASK-109
  - TASK-111
  - TASK-112
  - TASK-114
  - TASK-115
documentation:
  - docs/release-1.0-upgrade-notes.md
  - README.md
priority: high
ordinal: 41100
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Users upgrading to this release lose the static document commands and the `nfreq` alias, lose `deploy` as a default type, gain whole-map `document_types` semantics, and must move documents out of `deprecated/` themselves. The README still advertises a fixed command list and a seeded deploy type. This task publishes the migration steps and the new command/config contract. Man-page generation is explicitly deferred: ADR-0021 is still Proposed, so NFR-03.003 coverage for config-defined slugs is recorded as a release risk instead of documented as shipped.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Upgrade notes state that static document commands and nfreq are removed, req and nfr work only as default document_types entries, and deploy is no longer seeded but can be added as an ordinary type.
- [ ] #2 Upgrade notes explain whole-map document_types semantics, removal versus enabled:false, the new statuses/default_status keys, and both repair surfaces (config get|set|unset|list and init --force).
- [ ] #3 Upgrade notes explain the archived/ location, archive as the only way to archive, the absence of document restore, and that a legacy deprecated/ folder is inert and must be moved by hand.
- [ ] #4 README lists commands from the dynamic model rather than a fixed per-type table, documents document_types as the source of slugs and statuses, and links the upgrade notes.
- [ ] #5 README and the notes state that config-defined slugs have no man page until the ADR-0021 owner decision, so NFR-03.003 is not claimed for them; the gap is recorded as a release risk.
- [ ] #6 No man-page source, generator, packaging change, or papyrus(1) claim is added by this task.
<!-- AC:END -->
