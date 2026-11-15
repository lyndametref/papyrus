---
id: TASK-127
title: >-
  Correct browser and init specifications to the settled archive lifecycles and
  directory creation
status: Done
assignee:
  - business-analyst
created_date: '2026-11-15T11:28:00Z'
updated_date: '2026-11-15T13:32:00Z'
labels:
  - docs
  - specifications
milestone: m-19
dependencies:
  - TASK-130
  - TASK-131
  - TASK-132
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - REQ-05.007
  - REQ-07.001
  - REQ-01.001
documentation:
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
modified_files:
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - docs/specifications/project-initialization-and-configuration.md
priority: medium
ordinal: 41320
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Three specification pages still describe the pre-ADR-0022 lifecycle. `docs/specifications/browser-managed-artifacts-and-lifecycles.md` lists milestones as active -> archived and ten document types as active -> deprecated with restore, and its read-only operation sentence still names deprecating and restoring. `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` section 6.3 says init creates a type directory and its `deprecated/` subfolder. `docs/specifications/project-initialization-and-configuration.md` OQ-1 leaves the init directory set, including whether deprecated subfolders are pre-created, implementation-defined. The settled contract is ADR-0022, ADR-0023, and the amended ADR-0016/0017/0018 (TASK-126).

**What.** Correct the lifecycle wording on the browser page, the directory-creation rule on the init walkthrough page, and resolve OQ-1 with the directories init actually creates.

**In scope.**
- Browser page: every document type `active -> archived` with no restore; UI mockups `pending -> approved -> archived` (supersession via `ui-mockup supersede`); milestones `open -> closed`; releases `planned -> released` in place with no archive; remove deprecate/restore from the read-only operation sentence. Browser lifecycle operations stay CLI-only.
- Init walkthrough section 6: a type directory is created only when the type is activated; `archived/` appears on the first archive and is never pre-created; no `deprecated/` subfolder.
- Project initialization spec: replace OQ-1 with the settled behavior and exact directory set (backlog directory with tasks/milestones/releases, doc_directory, decision_directory only when a decision type is enabled, one active directory per enabled document type, no archived/deprecated subfolders); keep the no-overwrite rule.

**Out of scope.** CLI behavior changes, ADR edits, and page layout or navigation changes.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 browser-managed-artifacts-and-lifecycles.md states documents active -> archived with no restore, mockups pending -> approved -> archived, milestones open -> closed, and releases planned -> released with no archive
- [x] #2 the read-only operation sentence no longer names deprecating or restoring, and no row retains deprecated lifecycle wording
- [x] #3 interactive-init-walkthrough-for-artifact-activation.md section 6 states that only the active type directory is created and archived/ appears on first archive; no deprecated/ subfolder is created
- [x] #4 project-initialization-and-configuration.md OQ-1 is resolved with the exact init directory set, the no-pre-created archived/deprecated rule, and the no-overwrite rule
- [x] #5 A grep for deprecated in the three pages finds only decision-status references or an explicitly historical note, never a document lifecycle or pre-created folder
- [x] #6 The pages agree with ADR-0016, ADR-0017, ADR-0018 decision 3, ADR-0022, and ADR-0023 with no contradicting lifecycle statement
<!-- AC:END -->
