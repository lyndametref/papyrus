---
id: TASK-264
title: "Refine the MS-035 browser engine tasks"
created_date: '2026-11-20T17:42:31Z'
updated_date: '2026-11-22T16:49:00Z'
labels: []
references:
  - ADR-039
  - TECH-068
status: Done
assignee:
  - task-refiner
milestone: MS-035
dependencies:
  - TASK-261
ordinal: 30
documentation: []
type: refinement
---

## Description
Confirm the browser tasks have everything needed: the record envelope shape, the generic route table, the 9 conflict renderers and 5 not-found pages from the MS-032 audit, semantic comparison rules, and the seed presentation lookup. Split tasks if needed; return to the planner on any dependency or scope change.
## Comments
- **task-refiner**: Planner escalation from TASK-264 refinement, AUD-144 finding 1: MS-034 AC-7 names only title and config deltas but the replay also records delta 7 (group link in front matter) and delta 8 (seeded status); TASK-260 AC-5 and TASK-261 AC-10 miscount the glossary cases (two usage-error, four migrated). Align the milestone and task criteria. No scope change made here.
- **task-refiner**: Planner escalation from TASK-264 refinement, AUD-144 finding 2: the independent rerun was not possible in the reviewer session; attach the TASK-261 rerun transcripts to the task evidence. No scope change made here.
- **task-refiner**: Planner escalation from TASK-264 refinement, MS-035 AC-9 packaging wording: AC-9 still phrases the gate as each deployable component's scripts build-package.sh with a per-component output path, while ADR-043 decision 6 resolves this single-app repository to the root scripts build-package.sh and scripts build-deb.sh. TASK-309 is still ToDo and TASK-270 depends on it. Align AC-9 and the TASK-270 matching wording, or record the user waiver with scope, reason, and timestamp with timezone. No scope change made here.
- **task-refiner**: Planner escalation from TASK-264 refinement, TASK-269 execution risk. TASK-269 must remove the last document TypedHandler use in document.rs edit_document, the ui-spec editor front matter re-adoption quirk guarded by the ui_spec_cli test, and must replace the per-type document JSON branches while preserving the MS-032 payload bytes. No seed-expressible expression for the ui-spec quirk was identified in ADR-036 or ADR-039, and the generic JSON work is not named in the current task text. The implementer must stop rather than add a slug branch. Planner to decide whether to split TASK-269 into generic JSON projection plus HTML branch deletion and semantic proof, or to approve a seed-expressible exception. No dependency or ordinal change made here.
- **planner**: planner: MS-035 escalations resolved. Packaging: MS-035 AC-9 and TASK-270 rewritten to the internal-checkpoint rule (no RC, no retained package, verification artifact discarded; TASK-309 creates the packaging entry point and may be exercised; TASK-288 uses it for the single retained release package). TASK-269 split into TASK-311 (269a, generic document JSON projections, ordinal 75) and TASK-269 (269b, HTML deletion, ops builders, TypedHandler variants, ordinal 80); TASK-270 depends on TASK-269 and TASK-309; TASK-309 depends on TASK-269. ui-spec editor quirk: option (a), descriptor-keyed engine rule with a test, recorded in EXPECTED-DELTAS. AUD-144: MS-034 AC-7 aligned to deltas 1-3, 7, and 8; TASK-260 AC-5 and AC-6 and TASK-261 AC-10 counts corrected; TASK-261 rerun transcripts recorded as attached evidence. Status unchanged.
- **orchestrator**: Acceptance met: TASK-265 to TASK-270 refined with semantic-equivalence method, no-slug-dispatch guards and testable criteria; packaging waiver aligned; TASK-269 split into TASK-311 (JSON projections) and TASK-269 (branch deletion); ui-spec quirk rule decided.
