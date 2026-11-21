---
id: TASK-239
title: "Refine the MS-032 baseline and characterization tasks"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-21T16:46:09Z'
labels: []
references:
  - ADR-036
  - TECH-069
status: Done
assignee:
  - task-refiner
milestone: MS-032
dependencies:
  - TASK-237
ordinal: 20
documentation: []
type: refinement
---

## Description
Confirm that the fixture corpus, the benchmark harness, and the corrected inventory task have everything needed to start: fixture location and naming, corpus generation rules, reference hardware access, how the harness is invoked, and the exact observable surfaces to freeze (record bytes, JSON, status codes, CLI help, HTTP headers). Split implementation tasks if needed; return to the planner if a split changes dependencies, ordinals, or scope. Do not start until TASK-237 is Done.
## Acceptance Criteria
- [ ] TASK-240, TASK-241, TASK-242 and TASK-243 each carry a self-contained description grounded in the current repository, numbered testable acceptance criteria traceable to MS-032 AC-1 to AC-7, and the references and documentation paths the work needs.
- [ ] The fixture corpus location, naming, coverage and snapshot format, the benchmark invocation, corpus profiles and recorded protocol, the renderer and route enumeration, and the verification procedure are all fixed in the task descriptions, so no task has to invent them.
- [ ] The pre-refactor figures of inventory 14.2 s and view 1.8 s at 382 active records and the known root causes in ensure_front_matter, managed_descriptor, load_cfg_for, resolve_id and the per-request configuration load are stated for TASK-241 to capture.
- [ ] No dependency, ordinal, milestone or scope change was made, and every open question or proposed split is reported to the orchestrator.
## Comments
- **task-refiner**: TASK-240 to TASK-243 refined and left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done]; each now has a grounded description, numbered ACs traceable to MS-032 AC-1 to AC-7, references and documentation. No dependency or ordinal change; no split proposed. Reported open questions: the 382-record corpus provenance, the reference-hardware description versus the measured host, and the invented variance tolerance.
- **task-refiner**: Refinement complete for MS-032: TASK-240, TASK-241, TASK-242 and TASK-243 are execution-ready; no split, no plan change.
