---
id: TASK-281
title: "Refine the MS-037 cleanup, documentation, audit, and packaging tasks"
created_date: '2026-11-20T17:43:51Z'
updated_date: '2026-11-23T06:34:11Z'
labels: []
references:
  - TECH-069
status: Done
assignee:
  - task-refiner
milestone: MS-037
dependencies:
  - TASK-279
  - TASK-270
ordinal: 20
documentation: []
type: refinement
---

## Description
Confirm the cleanup tasks have everything needed: the exact dead-code list, the guard tests to extend, the documentation set, the gate list, the package script, and the evidence format each audit must produce. Split tasks if needed; return to the planner on any dependency or scope change.
## Comments
- **task-refiner**: Carried item recorded for execution: MS-036 AC-8 wording correction (keep-alive to Connection close) in TASK-289, owner planner.
- **task-refiner**: Carried item recorded: ADR-043 decision 21 exception for MS-035 and MS-036 in TASK-289, owner architect.
- **task-refiner**: Carried item recorded: duplicate AUD-145 and AUD-148 resolution in TASK-287, verification in TASK-289.
- **task-refiner**: Carried item recorded: NFR-007.001 and NFR-007.002 hardware deviation for the release report in TASK-287 and TASK-289.
- **task-refiner**: Carried item recorded: production LOC target of 28,000 is informational only in TASK-287 and TASK-289.
- **task-refiner**: Escalation: package-out refuses the builds output path that ADR-043 decision 7 and TECH-008 sections 1.1 and 2 require; TASK-288 and the TASK-289 RC gate are blocked pending planner and architect resolution.
- **task-refiner**: Refinement complete: TASK-282 to TASK-289 descriptions, references, documentation, and acceptance criteria are updated and grounded on the current tree.
- **orchestrator**: Acceptance met: TASK-282 to TASK-289 refined with grounded criteria; packaging guard rule, AC-8 wording, ADR-0020 and ADR-043 amendments recorded; TASK-288 and TASK-289 unblocked.
