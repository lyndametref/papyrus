---
id: TASK-272
title: "Refine the MS-036 performance tasks"
created_date: '2026-11-20T17:43:07Z'
updated_date: '2026-11-22T20:27:11Z'
labels: []
references:
  - ADR-037
  - ADR-038
  - NFR-007.001
  - NFR-007.002
status: Done
assignee:
  - task-refiner
milestone: MS-036
dependencies:
  - TASK-270
ordinal: 20
documentation: []
type: refinement
---

## Description
Confirm the performance tasks have everything needed: the snapshot boundary, the mtime inputs, the inventory scan entry point, the asset manifest and version source, the Mermaid lazy-load trigger, and the benchmark thresholds. Split tasks if needed; return to the planner on any dependency or scope change.
## Comments
- **task-refiner**: Refinement complete for TASK-273 to TASK-279: criteria added 9, 13, 9, 14, 9, 12, and 17 new plus 3 existing; every task is grounded in ADR-037, ADR-038, TECH-040, TECH-072, NFR-007.001 to 003, TC-178, TC-179, TC-181, TC-182, TC-187, TC-189, the MS-032 baseline document, and the named code symbols. Escalations: (1) TASK-279 packaging conflict: MS-036 AC-9 and ADR-043 decision 21 require an RC package for MS-036, while the instruction states the package is waived under the single-release rule; planner or owner must confirm an explicit waiver with scope, reason, and timestamp, or keep the RC gate. (2) ADR-038 decision 30 and ADR-043 decision 10 packaging asset-version guard is absent from the build-deb script; TASK-276 adds only the runtime version-binding test, so assign the packaging guard or amend the ADR. (3) NFR-007 reference hardware is 4 vCPU and 8 GiB, but the MS-032 baseline host is 6 physical and 12 logical cores with 15.3 GiB; confirm the measurement host and whether a non-reference result can count. (4) NFR-007.003 second-load measurement needs diagram and math pages plus cache simulation in the MS-032 harness; TASK-278 now requires it. No dependency, ordinal, or scope change; no split proposed.
- **task-refiner**: TASK-272 complete: all seven refinement targets have self-contained descriptions, assumptions, numbered testable criteria, references, documentation, and a definition of done; the packaging, packaging-guard, and reference-hardware escalations are reported to the orchestrator and planner. No dependency, ordinal, or milestone change was made.
- **orchestrator**: Acceptance met: TASK-273 to TASK-279 refined with 83 criteria; MS-036 packaging waiver recorded with UTC timestamp; asset-version guard assigned to TASK-276; hardware deviation decision recorded; second-load scope kept in TASK-278.
