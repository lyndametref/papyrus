---
id: TASK-245
title: "Refine the MS-033 record model foundation tasks"
created_date: '2026-11-20T17:41:19Z'
updated_date: '2026-11-22T08:53:43Z'
labels: []
references:
  - ADR-036
  - ADR-019
  - TECH-077
  - ADR-0019
status: Done
assignee:
  - task-refiner
milestone: MS-033
dependencies:
  - TASK-243
ordinal: 20
documentation: []
type: refinement
---

## Description
Confirm the model tasks have everything needed: the exact entry and field vocabulary, the merge rules, the diagnostic format, the seed contents for req, nfr, audit, glossary, rule, request, ui-mockup, ui-spec, tech-spec, use-case, test-case, and test-report, and the existing old-config load path. Split tasks if needed; return to the planner on any dependency or scope change. Do not start until TASK-243 is Done. Not to be implemented before MS-031 governance is accepted.
## Comments
- **task-refiner**: MS-033 refinement pass: TASK-246 to TASK-252 refined against the code at start commit c3103fc, ADR-036, ADR-037, ADR-039, amended ADR-0019 and ADR-0024, MS-033 AC-1 to AC-7, and the approved MS-031 test cases. Ready: TASK-246, TASK-247, TASK-248, TASK-250, TASK-251, TASK-252. Not ready: TASK-249, blocked on one architect question about document seed slots (ADR-039 decisions 6 and 25 against the five slot columns rendered by PRESENTATION today). TASK-249 seed text also named request as a seed entry; corrected to user-defined per ADR-036 decision 30. No dependency, ordinal, milestone, or assignee changes; no split proposed. Needs an architect decision on the seed slots before TASK-249 starts.
- **orchestrator**: Acceptance met: TASK-246 to TASK-252 refined with code-grounded acceptance criteria and the old-config load-identity invariant; blocking slots ambiguity escalated to TASK-293.
