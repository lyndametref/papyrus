---
id: TASK-128
title: >-
  Align the business object model and glossary lifecycles and remove the empty
  legacy deprecated folders
status: To Do
assignee:
  - domain-modeler
created_date: '2026-11-15T11:28:00Z'
labels:
  - docs
  - domain-model
milestone: m-19
dependencies:
  - TASK-126
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - REQ-05.007
  - REQ-07.001
documentation:
  - docs/domain-model/object-model/business-object-model.md
modified_files:
  - docs/domain-model/object-model/business-object-model.md
  - docs/domain-model/business-glossary/index.html
priority: medium
ordinal: 41330
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** The domain model still carries the pre-ADR-0022 lifecycle. `docs/domain-model/object-model/business-object-model.md` says requirements, glossary terms, UI specifications, technical specifications, audit records, and business rules are active or deprecated, technical specifications keep deprecate/restore, and audit records have an active/deprecated lifecycle. Glossary terms repeat the same wording (`business-requirement.md`, `business-rule.md`, `glossary-term.md`, `non-functional-requirement.md`, `technical-specification.md`, `ui-specification.md`, `GLO-002.md`, `GLO-003.md`, `GLO-005.md`, `GLO-007.md`, `GLO-012.md`), `release.md` and `GLO-008.md` say active or released, and `milestone.md` says active or archived. Two empty legacy `deprecated/` folders remain under `docs/requirements/non-functional/` and `docs/domain-model/business-glossary/`, which the model must not present as a lifecycle location.

**What.** Restate the document lifecycles as `active -> archived` with no restore, mockups as `pending -> approved -> archived`, milestones as `open -> closed`, and releases as `planned -> released`; regenerate the glossary index; remove the two empty legacy folders.

**In scope.** `business-object-model.md`; every glossary term file with the stale wording, including `release.md`, `GLO-008.md`, and `milestone.md`; the generated glossary index; removal of the empty `deprecated/` directories. Decisions keep the five Nygard statuses, so the word deprecated remains valid only as a decision status.

**Out of scope.** Editing record files, changing IDs or definitions beyond lifecycle wording, and removing any non-empty folder.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 business-object-model.md describes documents as active -> archived with no restore, mockups as pending -> approved -> archived, milestones as open -> closed, and releases as planned -> released
- [ ] #2 release.md and GLO-008.md say planned -> released; milestone.md says open -> closed
- [ ] #3 every glossary term that said active or deprecated, or mentioned restore/deprecate, now says active -> archived; no restore is described for documents
- [ ] #4 decisions still document the five Nygard statuses, including deprecated, and no decision wording was changed
- [ ] #5 the generated glossary index reflects the corrected terms
- [ ] #6 the empty docs/requirements/non-functional/deprecated and docs/domain-model/business-glossary/deprecated directories are removed; the removal is recorded in the task and no record file is deleted
- [ ] #7 A grep for deprecated across the domain model finds only decision-status uses or an explicitly historical note
<!-- AC:END -->
