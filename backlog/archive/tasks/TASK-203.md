---
id: TASK-203
title: "Retire browser configuration-view clauses and T-98-08 view equivalence in TECH-002, TECH-060, TECH-065"
created_date: '2026-11-18T22:48:56Z'
updated_date: '2026-11-19T10:54:26Z'
labels: []
references:
  - AUD-113
  - ADR-034
  - REQ-06.018
  - RE-005
  - TECH-002
  - TECH-060
  - TECH-065
status: Done
assignee:
  - architect
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 60
documentation:
  - docs/specifications/technical/TECH-002.md
  - docs/specifications/technical/TECH-060.md
  - docs/specifications/technical/TECH-065.md
type: implementation
---

## Description
Correction for AUD-113 finding 1 (high, blocks MS-027 AC-7). TECH-002, TECH-060, and TECH-065 remain status verified, carry no revision or retirement banner, and still present the removed browser configuration view as current: TECH-002:153, TECH-002:162, TECH-002:207 (with historical mentions at 61 and 95); TECH-060:159; TECH-065:76, TECH-065:153, and the T-98-08 verification row at TECH-065:237 ("browser view equivalence"), which the rewritten t98_08 no longer tests. All three reference REQ-06.008, which is archived. Follow the TASK-201 and TASK-170 revision-banner precedent: add a narrow retirement note per record citing ADR-034, REQ-06.018, RE-005, and this task; mark the configuration-view clauses and the T-98-08 browser-view equivalence as retired and downgrade each record to status draft; keep the CLI-only config-set and relocation clauses current; preserve historical content and change nothing unrelated. Done when papyrus check reports no new inconsistency and no verified specification presents the removed browser configuration view or the T-98-08 browser-view equivalence as current.
## Comments
- **architect**: Evidence (architect), TASK-203. Retirement notes added to TECH-002, TECH-060 and TECH-065.
- **architect**: TECH-002: status verified to draft; references ADR-034, REQ-06.018 and RE-005 appended; TASK-203 banner added. Section 3 item 12, the browser-view tail of item 21, the out-of-scope browser list-editing item and the relocation assumption are marked retired. The config set list-key, config list and whole-folder relocation clauses (BR-003, REQ-01.002) remain current.
- **architect**: TECH-060: status verified to draft; same references appended; banner added. Section 8 item 7 (the browser configuration view calling the config set relocation path) is marked retired; section 8 items 1-6 remain current.
- **architect**: TECH-065: status verified to draft; same references appended; banner added. The relocation scope bullet, section 7 item 4 and the T-98-08 browser-path and browser view equivalence row are marked retired; the CLI-only config-set and installed-binary relocation clauses remain current.
- **architect**: Retired text is retained in place as history in all three records. No code, tests, requirements, rules or decisions changed. papyrus check reports only the three pre-existing archived-task frontmatter-conflict diagnostics, with no new diagnostic.
