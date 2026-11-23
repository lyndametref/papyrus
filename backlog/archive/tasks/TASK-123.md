---
id: TASK-123
title: >-
  Align the business object model and glossary lifecycles and remove the empty
  legacy deprecated folders
status: Done
assignee:
  - domain-modeler
created_date: '2026-11-15T11:28:00Z'
updated_date: '2026-11-15T13:47:00Z'
labels:
  - docs
  - domain-model
milestone: MS-020
dependencies:
  - TASK-121
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - REQ-005.007
  - REQ-007.001
documentation:
  - docs/domain-model/object-model/business-object-model.md
modified_files:
  - docs/domain-model/business-glossary/index.html
priority: medium
ordinal: 41330
---


## Description
**Why.** The domain model still carries the pre-ADR-0022 lifecycle. `docs/domain-model/object-model/business-object-model.md` says requirements, glossary terms, UI specifications, technical specifications, audit records, and business rules are active or deprecated, technical specifications keep deprecate/restore, and audit records have an active/deprecated lifecycle. Glossary terms repeat the same wording (`business-requirement.md`, `business-rule.md`, `glossary-term.md`, `non-functional-requirement.md`, `technical-specification.md`, `ui-specification.md`, `GLO-002.md`, `GLO-003.md`, `GLO-005.md`, `GLO-007.md`, `GLO-012.md`), `release.md` and `GLO-008.md` say active or released, and `milestone.md` says active or archived. Two empty legacy `deprecated/` folders remain under `docs/requirements/non-functional/` and `docs/domain-model/business-glossary/`, which the model must not present as a lifecycle location.

**What.** Restate the document lifecycles as `active -> archived` with no restore, mockups as `pending -> approved -> archived`, milestones as `open -> closed`, and releases as `planned -> released`; regenerate the glossary index; remove the two empty legacy folders.

**In scope.** `business-object-model.md`; every glossary term file with the stale wording, including `release.md`, `GLO-008.md`, and `milestone.md`; the generated glossary index; removal of the empty `deprecated/` directories. Decisions keep the five Nygard statuses, so the word deprecated remains valid only as a decision status.

**Out of scope.** Editing record files, changing IDs or definitions beyond lifecycle wording, and removing any non-empty folder.

## Acceptance Criteria
- [x] business-object-model.md describes documents as active -> archived with no restore, mockups as pending -> approved -> archived, milestones as open -> closed, and releases as planned -> released
- [x] release.md and GLO-008.md say planned -> released; milestone.md says open -> closed
- [x] every glossary term that said active or deprecated, or mentioned restore/deprecate, now says active -> archived; no restore is described for documents
- [x] decisions still document the five Nygard statuses, including deprecated, and no decision wording was changed
- [x] the generated glossary index reflects the corrected terms
- [x] the empty docs/requirements/non-functional/deprecated and docs/domain-model/business-glossary/deprecated directories are removed; the removal is recorded in the task and no record file is deleted
- [x] A grep for deprecated across the domain model finds only decision-status uses or an explicitly historical note


## Notes
Lifecycles aligned with ADR-0022/0023 and REQ-003.001/03.002/05.007.

Changed:
- docs/domain-model/object-model/business-object-model.md: documents active -> archived (no restore); new Lifecycles table covers UI mockups pending -> approved -> archived, milestones open -> closed via papyrus milestone close, releases planned -> released via papyrus release close (never archived), tasks (configurable + archive/restore), decisions (five Nygard statuses, unchanged).
- docs/domain-model/context-map.md: technical specification lifecycle corrected to active -> archived, no restore.
- Glossary terms corrected: business-requirement.md, business-rule.md, glossary-term.md, non-functional-requirement.md, technical-specification.md, ui-specification.md, release.md, milestone.md, GLO-002.md, GLO-003.md, GLO-005.md, GLO-006.md, GLO-007.md, GLO-008.md, GLO-012.md.
- docs/domain-model/business-glossary/index.html regenerated: 12 terms.
- Removed empty legacy folders docs/requirements/non-functional/deprecated and docs/domain-model/business-glossary/deprecated. Both verified empty (ls -A returned nothing) and removed with rmdir, so no record file was deleted. Other deprecated/ folders were not touched.
- Grep deprecated across docs/domain-model/ returns only decision-status uses (business-object-model.md lines 51 and 70) plus the explicitly historical note that a legacy deprecated/ folder is not a lifecycle location (line 72).

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Domain model and glossary aligned to the settled lifecycles: documents active -> archived with no restore; UI mockups pending -> approved -> archived; milestones open -> closed via papyrus milestone close; releases planned -> released via papyrus release close and never archived; tasks keep configurable statuses plus archive/restore; decisions keep the five Nygard statuses unchanged. Updated business-object-model (incl. a Lifecycles table and a historical note that legacy deprecated/ is not a lifecycle location), context-map technical-specification lifecycle, and 15 glossary term files; glossary index regenerated and byte-identical to the committed page (12 terms). Removed the empty docs/requirements/non-functional/deprecated and docs/domain-model/business-glossary/deprecated directories; no record file deleted. Grep finds deprecated only in decision-status and historical-note contexts. Full suite 853 passed / 0 failed / 22 ignored.
