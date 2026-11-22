---
id: TASK-261
title: "Verify MS-034 CLI engine: command equivalence and help attribution"
created_date: '2026-11-20T17:42:17Z'
updated_date: '2026-11-22T16:03:32Z'
labels: []
references:
  - REQ-07.003
  - REQ-05.001
  - ADR-0016
  - ADR-036
  - REQ-05.008
  - TECH-070
  - AUD-140
  - ADR-042
  - TECH-024
status: In-Progress
assignee:
  - quality-gatekeeper
milestone: MS-034
dependencies:
  - TASK-260
  - TASK-310
ordinal: 90
documentation:
  - tests/golden/cli-help
  - tests/fixtures/characterization/corpus
  - tests/cli_help_golden.rs
  - tests/characterization_replay.rs
  - tests/nested_config_cli.rs
type: milestone-verification
---

## Description
## Context
Milestone boundary verification for MS-034. The milestone is an internal checkpoint of the document command surface only: one field-driven handler replaces the nine typed modules, the title contract is uniform, config selectors work, help goldens are regenerated with every diff attributed, and the MS-032 fixtures replay with only documented deltas. The eight document ops builders and the document TypedHandler variants remain until MS-035 deletes them with the browser branches.

## Scope
- Check every MS-034 acceptance criterion AC-1 to AC-7 with reproducible commands and recorded expected and observed values.
- AC-1: baseline verb matrix per DocumentModel, including help and a smoke run; create-group and --group only for GroupedMarkdown; supersede only for SingleHtmlMockup; no status verb for SingleHtmlMockup.
- AC-2: title contract on create, create-group, edit, and the HTML metadata; no --term; additive audit and create-group positional titles; empty, whitespace-only, multi-line, and dual-source titles rejected with no write.
- AC-3: config get and list resolve keyed and numeric field selector leaves; config set and unset reject fields/group.fields/member.fields edits as file-edit-only; target-only materialization; only statuses keeps comma syntax; api_aliases and slots are declarative file data; selector-path diagnostics.
- AC-4: the nine typed modules are gone, ui_mockup remains, no transitional document CLI schema or verb table remains, the eight document ops builders and document TypedHandler variants are retained with deletion deferred to MS-035 TASK-266/TASK-269, and backlog and decision modules are unchanged.
- AC-5: help goldens regenerated; every diff from the MS-032 snapshot attributed; help deterministic.
- AC-6: field-derived options and the model-specific verbs appear exactly where the seed and DocumentModel allow.
- AC-7: the MS-032 CLI fixtures replay; record files and exit codes match except the documented glossary --term usage-error delta and any other recorded title/config delta.
- Run the full suite, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`.
- Record the AUD-140 follow-ups L-1, L-3, and L-4 with owner and status.

## Out of scope
Fixing a defect, changing a requirement, ADR, specification, test case, golden, fixture, or source, and verifying MS-035 behavior. A failed criterion is recorded with a finding and left open.

## Gate
Verification does not start while a material decision from TASK-255, TASK-256, TASK-258, or TASK-259 is unresolved. ADR-042 resolves the TASK-255, TASK-256, TASK-258, and TASK-259 decisions; TASK-259's ops-builder deletion is deferred to MS-035. A blocked dependency is a finding, not a pass.

## Grounding
MS-034 AC-1 to AC-7; ADR-036; ADR-042; TECH-070; TECH-024; REQ-07.003; REQ-05.001; REQ-05.008; AUD-140; TC-119, TC-120, TC-121, TC-125, TC-126, TC-185.

## Definition of done
Every criterion has a pass or a recorded finding with evidence; the suite and quality gates are green or their failures are recorded; the internal-checkpoint rule and packaging waiver are recorded; the AUD-140 follow-ups are recorded; no product code or managed artifact is changed by this task.

## Acceptance Criteria
- [ ] #1 This milestone is an internal checkpoint: no RC version is determined and no RC package is published; the verification build is compiled and discarded per TECH-024 section 1 and the MS-037 single-release rule.
- [ ] #2 Packaging is waived for this milestone verification as recorded for MS-032 (AUD-138) and MS-033 (AUD-140): internal checkpoint, no RC, no package, verification build discarded; the waiver is recorded in the audit record and the task comment and is not a finding.
- [ ] #3 This task is not Done until the internal-checkpoint rule and the packaging waiver are recorded in the audit record and task comment.
- [ ] #4 AC-1 verified: for every enabled document type, the help verb set and a smoke run of the baseline verbs match DocumentModel; create-group and --group appear only for GroupedMarkdown, supersede only for SingleHtmlMockup, and SingleHtmlMockup has no status verb. (MS-034 AC-1)
- [ ] #5 AC-2 verified: the uniform title contract holds on create, create-group, edit, and the HTML metadata; no document command accepts --term; the audit and create-group positional titles are additive; empty, whitespace-only, multi-line, and dual-source titles are rejected with no write. (MS-034 AC-2)
- [ ] #6 AC-3 verified against the narrowed contract: config get and list resolve keyed and numeric field selector leaves; config set and unset reject every selector inside fields, group.fields, or member.fields as file-edit-only with a selector-path diagnostic and no write; materialization writes only the target entry; only statuses keeps comma syntax; api_aliases and slots are declarative file data; field type errors name the selector path. (MS-034 AC-3; ADR-042 rulings 5 and 6)
- [ ] #7 AC-4 verified: the nine typed modules are gone; ui_mockup remains; no transitional document CLI schema or verb table remains; the eight document ops builders and the document TypedHandler variants are retained and their deletion is deferred to MS-035 TASK-266/TASK-269 as recorded; backlog and decision modules, commands, help, and behavior are unchanged. (MS-034 AC-4; ADR-042)
- [ ] #8 AC-5 verified: the CLI help goldens are regenerated, every difference from the MS-032 snapshot is attributed in the recorded report, and help is deterministic across two runs. (MS-034 AC-5)
- [ ] #9 AC-6 verified: field-derived options appear exactly where the seed allows, in sequence order and only on the field verbs; create-group, --group, and supersede appear exactly where the DocumentModel allows. (MS-034 AC-6)
- [ ] #10 AC-7 verified: the MS-032 CLI fixture replay passes with the four glossary --term usage-error cases and any other documented deltas; every other record file and exit code matches. (MS-034 AC-7)
- [ ] #11 The full test suite is green, cargo clippy --all-targets -- -D warnings and cargo fmt --check are clean, and the verification evidence records the suite counts and commands run. (DoD)
- [ ] #12 AUD-140 follow-ups are recorded with status and owner: L-1 field string type-error diagnostics (TASK-258), L-3 TECH-077 section 6 step 4 wording (architect with the MS-034 spec pass, recorded with TASK-257), and L-4 REQ-07.002 and TC-114 seed-materialization wording (requirements-engineer, Phase 2). (AUD-140)
- [ ] #13 Every finding is recorded with expected and observed values, the owning surface, and a disposition; this task fixes no defect and adds no product behavior. (DoD)
## Comments
- **task-refiner**: Evidence to record: the exact commands, the test counts, the help diff report location, the fixture replay result with the four glossary --term cases, the module and ops-builder absence checks, and the backlog and decision no-change check. Use project-local .tmp for scratch and remove it before finishing; do not retain a verification package.
- **task-refiner**: AUD-140 mapping: L-1 is the field string type-error diagnostic work in TASK-258; L-3 is the TECH-077 section 6 step 4 merge_yaml wording fix owned by architect with the MS-034 spec pass and recorded with TASK-257; L-4 is the REQ-07.002 and TC-114 seed-materialization wording owned by requirements-engineer in Phase 2. L-2 (NFR-007.001) is owned by MS-036 and is out of scope here.
- **planner**: ADR-042 rulings and planner alignments applied to MS-034: AC-3 is narrowed to the read write split and statuses-only comma syntax; AC-4 is corrected to eight document ops builders with ops-builder and TypedHandler deletion deferred to MS-035; TASK-255 carries the audit field_defaults; TASK-256 keeps the CLI mockup edit --status; TASK-258 implements the narrowed read write contract; TASK-259 is narrowed; TASK-260 attributes generated verb about lines and keeps configuration map order. Verification may start once the execution tasks are Done.
- **planner**: ADR-042 item 7 applied: AC #1-#3 are replaced by the internal-checkpoint rule and the packaging waiver wording used by MS-032 (AUD-138) and MS-033 (AUD-140); no RC version or RC package is required. MS-034 AC-8 and TASK-307 still state the old RC-gate premise; they were not changed by this planner pass and are reported as an open decision to the orchestrator.
- **planner**: Planner: TASK-307 archived as premise invalid (build-deb.sh is the single build script per the TASK-235 architect ruling; MS-037 AC-6 corrected) and removed from this task's dependencies; MS-034 AC-8 corrected to the internal-checkpoint rule. Verification scope remains MS-034 AC-1 to AC-7.
- **code-auditor**: AUD-142 re-audit: corrections required, AUD-141 not fully closed; F7 help-option panic and F15 edit sentinel remain open, TASK-310 needs two corrections.
