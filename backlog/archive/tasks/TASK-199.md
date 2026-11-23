---
id: TASK-199
title: "Set UI-018 status metadata and reconcile with MOCK-020"
created_date: '2026-11-18T21:35:34Z'
updated_date: '2026-11-19T10:54:25Z'
labels: []
references:
  - AUD-107
  - AUD-108
  - ADR-034
  - UI-018
  - MOCK-020
status: Done
assignee:
  - ui-designer
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 56
documentation:
  - docs/specifications/ui/UI-018.md
type: ui-design
---

## Description
Correction for AUD-107 finding 7 and AUD-108 finding 7 (low): UI-018 has no status field while most UI specifications carry one, and MOCK-020 is approved although UI-018 states the UI specification stays draft until mockup approval. Set the UI-018 status per project convention, consistent with the approved mockup and the ADR-034 disposition, or record the convention; remove the contradictory draft-until-approval statement if it no longer applies. Done when papyrus ui-spec view UI-018 shows a configured status and the record is internally consistent.
## Comments
- **ui-designer**: Evidence (ui-designer): AUD-107 finding 8 and AUD-108 finding 7 corrected in documentation only; no code, test, requirement, rule, ADR, or technical specification changed. File: UI-018.md under docs specifications ui. Changes: status set to verified (was absent), reflecting approved MOCK-020 and the navigation implemented by TASK-173 and TASK-174; description rewritten to drop the false Project group claim and the contradictory UI-spec-stays-draft-until-mockup-approval sentence; the removal is now described as the three utility entries (Configuration, Managed artifacts and lifecycles, Agent skills and workflows) appended after the document-type groups, with the Backlog, Decisions, and Documents groups unchanged, per AUD-107 finding 8; a closing approval sentence records MOCK-020 approved and status verified. References added: MOCK-020, TASK-199, TASK-173. Consistency checked against ADR-034, TECH-080, REQ-006.018, and MOCK-020 rendered sidebar: no removed route or utility entry is presented as current and no technology choice is introduced. Validation: papyrus ui-spec view UI-018 shows status verified and the record is internally consistent; papyrus check reports only the 3 pre-existing frontmatter-conflict warnings on archived TASK-098, TASK-111, TASK-112, with no UI or reference inconsistency. Residual observations outside this task: MOCK-020 body annotations still use the phrase Project group, left untouched because it is an approved mockup and a wording change would require a reissued package; UI-017 still lacks a status field although MOCK-019 is approved.
