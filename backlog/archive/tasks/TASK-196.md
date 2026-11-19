---
id: TASK-196
title: "Revise UI-010, UI-011, UI-012, UI-014, UI-015 for ADR-034 removal"
created_date: '2026-11-18T21:35:23Z'
updated_date: '2026-11-19T10:54:25Z'
labels: []
references:
  - AUD-107
  - AUD-108
  - ADR-034
  - TECH-080
  - UI-018
  - REQ-06.018
status: Done
assignee:
  - ui-designer
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 53
documentation:
  - docs/specifications/ui/UI-010.md
  - docs/specifications/ui/UI-011.md
  - docs/specifications/ui/UI-012.md
  - docs/specifications/ui/UI-014.md
  - docs/specifications/ui/UI-015.md
type: ui-design
---

## Description
Correction for AUD-107 finding 5 and AUD-108 finding 3 (medium): UI-010, UI-011, UI-012, UI-014, and UI-015 remain status verified while presenting removed routes and sidebar entries as the current design. Stale references: UI-011 line 44 keeps the skills route; UI-014 lines 42, 49, and 57 keep the configuration and lifecycle routes and a sidebar Configuration entry; UI-010 line 50, UI-012 lines 63-64, and UI-015 line 67 keep the removed routes in route tables. Add revision or retirement notes, or a status update, consistent with ADR-034 and following the TECH revision-banner precedent, so no verified UI specification claims the removed surfaces are current. Do not change unrelated content. Done when the five specifications carry their disposition and papyrus check reports no inconsistency.
## Comments
- **ui-designer**: Evidence (ui-designer): AUD-107 finding 5 and AUD-108 finding 3 corrected in documentation only; no code, test, requirement, rule, or technical specification changed. Files: UI-010.md, UI-011.md, UI-012.md, UI-014.md, UI-015.md under docs specifications ui. Dispositions, all five status verified to draft, each with a TASK-196 banner citing ADR-034 and TECH-080: UI-010 lifecycle page section 6 and its sidebar entry marked retired, lifecycle route row removed, front-matter metadata scope retained unchanged; UI-011 page retired in full, skills route row and sidebar entry removed, active elements none, kept as history only; UI-012 lifecycle and skills sidebar entries, route rows, and 320 px views removed, inventory-hub note added that the cards no longer exist, navigation and metadata scope retained; UI-014 configuration view section 4 and lifecycle page section 9 marked retired, config and lifecycle route rows and their responsive rows removed, the three utility sidebar entries removed, config editing recorded as CLI-only; UI-015 Project group row removed, lifecycle route row removed, lifecycle page delta marked retired, sidebar treatment and active elements reduced to Backlog, Decisions, Documents. No removed route is presented as served; remaining mentions sit in the retirement banners or in explicitly retired sections and resolved OQs. References added to all five: ADR-034, TECH-080, REQ-06.018, TASK-196, UI-018. Validation: papyrus check reports only the 3 pre-existing frontmatter-conflict warnings on archived TASK-098, TASK-111, TASK-112, with no UI, reference, or route inconsistency; cargo test --test browser_cli 181 passed including tc033 to tc039 which read UI-010, UI-011, UI-012; cargo test --test ui_spec_cli 11 passed. Assumption: this Papyrus version has no ui-spec deprecate verb, so UI-011 stays in place with an explicit retirement banner and status draft instead of being moved or deleted.
