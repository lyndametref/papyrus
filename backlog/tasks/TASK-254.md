---
id: TASK-254
title: "Refine the MS-034 CLI engine tasks"
created_date: '2026-11-20T17:41:57Z'
updated_date: '2026-11-22T13:02:37Z'
labels: []
references:
  - ADR-036
  - REQ-007.003
  - TECH-070
  - ADR-042
status: Done
assignee:
  - task-refiner
milestone: MS-034
dependencies:
  - TASK-252
ordinal: 20
documentation: []
type: refinement
---

## Description
Confirm the CLI tasks have everything needed: the field vocabulary, the derived verb matrix per DocumentModel, the exact title contract, the config selector grammar, the module and ops-builder deletion list, and the help golden location. Split tasks if needed; return to the planner on any dependency or scope change. Not to be implemented before MS-031 governance is accepted.
## Comments
- **task-refiner**: Refinement complete for TASK-255 through TASK-261. Ready: TASK-257 uniform title, TASK-260 help goldens and fixture replay, TASK-261 milestone verification. Blocked: TASK-255 audit_type default security not expressible in the nine-key FieldSpec; TASK-256 mockup approved transition and CLI edit status option undecided; TASK-258 MS-034 AC-3 field-leaf set and unset contradicts ADR-0019 decision 10 amendment and ADR-0024 amendment; TASK-259 browser calls the eight document ops builders and TypedHandler document variants, and AC-4 says ten builders while the code has eight. Criteria added: 10 each to TASK-255, TASK-256, TASK-257, TASK-258; 6 to TASK-259; 8 to TASK-260; 10 to TASK-261. Escalations: audit default; verb about lines versus frozen about-line requirement; field-edit set and unset conflict; mockup approval path; browser ops dependency; type order versus seed order; MS-034 AC-3 list wording for api_aliases and slots; TASK-261 stale RC-package ACs 1 to 3. AUD-140: L-1 recorded in TASK-258, L-3 recorded in TASK-257 and TASK-261, L-4 recorded in TASK-258 and TASK-261; L-2 stays with MS-036. Proposed splits: TASK-256a grouped and TASK-256b mockup; TASK-259a CLI modules and schema, TASK-259b ops builders and browser after MS-035. No dependency, ordinal, or milestone change made.
- **planner**: ADR-042 rulings applied to the MS-034 plan: AC-3 narrowed to the read write split and statuses-only comma syntax; AC-4 corrected to eight document ops builders with ops-builder and TypedHandler deletion deferred to MS-035; TASK-255 audit field_defaults; TASK-256 approval path cleared and not split; TASK-258 narrowed contract; TASK-259 narrowed to the nine modules and transitional CLI tables; TASK-260 generated about lines and configuration map order; TASK-261 internal checkpoint and packaging waiver. No dependency, ordinal, or milestone change.
- **orchestrator**: Acceptance met: TASK-255 to TASK-261 refined with code-grounded criteria; ADR-042 rulings applied to AC-3 and AC-4 and to tasks 255, 256, 258, 259, 260 and 261; TASK-307 archived as invalid.
