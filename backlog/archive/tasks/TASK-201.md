---
id: TASK-201
title: "Revise stale TECH specifications and packaging doc for ADR-034 removal"
created_date: '2026-11-18T22:20:44Z'
updated_date: '2026-11-19T10:54:26Z'
labels: []
references:
  - AUD-109
  - AUD-110
  - ADR-034
  - TECH-080
  - REQ-06.018
status: Done
assignee:
  - architect
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 57
documentation:
  - docs/specifications/technical/TECH-007.md
  - docs/specifications/technical/TECH-053.md
  - docs/specifications/technical/TECH-054.md
  - docs/specifications/technical/TECH-057.md
  - docs/specifications/technical/TECH-058.md
  - docs/specifications/technical/TECH-061.md
  - docs/specifications/technical/TECH-069.md
  - docs/packaging.md
type: implementation
---

## Description
Correction for AUD-109 finding 1 and AUD-110 finding 1 (high, blocks MS-027 AC-7). TECH-007, TECH-053, TECH-054, TECH-057, TECH-058, TECH-061, and TECH-069 remain status verified while presenting the removed configuration, lifecycle, and skills routes, sidebar entries, and inventory cards as current; the packaging document keeps the same removed routes in its route matrix, CSP coverage list, and a PASS row. Follow the TASK-170 and TASK-196 revision-banner precedent used by TECH-050, TECH-051, TECH-063, TECH-068, TECH-074, and TECH-076: add a narrow revision or retirement note per record citing ADR-034 and TASK-201, set status draft, and remove the stale route, sidebar-entry, and inventory-card text while preserving the historical content. Correct the packaging route matrix, CSP coverage list, and PASS row to the removed-surface contract. Do not change unrelated content. Done when papyrus check reports no new inconsistency and no verified specification or packaging section presents the removed surfaces as current.
## Comments
- **architect**: Evidence (architect), TASK-201. Revised per ADR-034 and REQ-06.018 following the TASK-170 and TASK-196 banner precedent. TECH-007, TECH-053, TECH-054, TECH-057, TECH-058, TECH-061 and TECH-069 each gained a narrow TASK-201 revision banner, the ADR-034, REQ-06.018 and RE-005 references, and status verified to draft. Stale route, sidebar-entry and inventory-card text was removed or marked retired, and historical clauses were kept. The packaging document PASS row, TASK-117 B-14 and B-15 rows, and the CSP coverage list were corrected to the removed-surface contract, and the TASK-83 relocation history was annotated. No code, tests, requirements, rules or decisions changed. papyrus check reports only the three pre-existing archived-task frontmatter-conflict warnings, with no new diagnostic.
