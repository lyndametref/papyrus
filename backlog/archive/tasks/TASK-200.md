---
id: TASK-200
title: "Revise UI-013 for ADR-034 removal"
created_date: '2026-11-18T22:20:26Z'
updated_date: '2026-11-19T10:54:25Z'
labels: []
references:
  - AUD-109
  - ADR-034
  - UI-018
  - REQ-006.018
status: Done
assignee:
  - ui-designer
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 58
documentation:
  - docs/specifications/ui/UI-013.md
type: ui-design
---

## Description
Correction for AUD-109 finding 1 (high, blocks MS-027 AC-7). UI-013 remains status verified while line 43 lists Managed artifacts and Agent skills in the persistent sidebar, and its card text still reads as the current design. Apply the TASK-196 disposition pattern: add a revision banner citing ADR-034 and TASK-200, set status draft, remove the removed sidebar entries and card surfaces from the current design text, and preserve the historical content. Do not change unrelated content. Done when papyrus check reports no new inconsistency and no verified UI specification presents the removed surfaces as current.
## Comments
- **ui-designer**: Evidence (ui-designer): AUD-109 finding 1 corrected for UI-013 in documentation only; no code, test, requirement, rule, technical specification, or mockup file changed. File: docs specifications ui UI-013.md. Disposition: status verified to draft; TASK-200 banner added under the H1 citing ADR-034, TECH-080, REQ-006.018, RE-005, and UI-018 and stating the removed routes return 404; §2 sidebar list reduced to the current entries and a retirement sentence for the Managed artifacts and Agent skills entries added, preserving the historical design in the banner; §12 approval table and closing note record the draft revision while keeping MOCK-0013 approved as history. No inventory-card surface exists in UI-013, so the only stale current-design text was the §2 sidebar list cited at line 43; the audit card wording does not apply to this record. References added: ADR-034, TECH-080, REQ-006.018, TASK-200, UI-018. Validation: papyrus check reports only the 3 pre-existing frontmatter-conflict warnings on archived TASK-098, TASK-111, TASK-112, with no UI, reference, or route inconsistency; a scan of all status-verified UI specs finds no remaining current claim for the removed sidebar entries, routes, or cards. Assumption: Papyrus has no ui-spec deprecate verb, so UI-013 stays in place as draft with an explicit revision banner per the TASK-196 precedent.
