---
id: TASK-289
title: "Verify MS-037 and confirm the single release is ready"
created_date: '2026-11-20T17:44:23Z'
updated_date: '2026-11-23T06:32:50Z'
labels: []
references:
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
  - MS-037
  - ADR-043
  - TECH-008
  - AUD-150
  - MS-036
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-037
dependencies:
  - TASK-288
ordinal: 100
documentation:
  - docs/benchmarks/MS-036-results.md
  - docs/audits/AUD-150.md
  - decisions/ADR-043-component-build-script-as-the-packaging-entry-point.md
  - scripts/build-package.sh
  - scripts/build-deb.sh
  - docs/packaging.md
  - docs/release-1.0-upgrade-notes.md
type: milestone-verification
---

## Description
## Context
Milestone boundary verification for MS-037 and the only release boundary (MS-037 AC-7). Dependency TASK-288 (final package) must be Done. This task verifies; it does not implement or repair.

## Scope
- Verify every MS-037 acceptance criterion AC-1 through AC-7 against the committed verified tree, with a command or artifact per criterion.
- Cross-milestone gates: full suite; clippy and fmt; `no_drift`; document-type no-drift; descriptor-leak guard; no new crates; zero document `TypedHandler` variants; no document slug dispatch outside the seed; record, JSON, and status-code equivalence against the MS-032 fixtures; semantic HTML equivalence; NFR-007.001 and NFR-007.002 measured on the recorded host with the hardware deviation; NFR-007.003 cache classes; binary at or under 12 MiB; no open blocking audit finding.
- Carried items to verify:
  - MS-036 AC-8 records `Connection: close` as the preserved behaviour (planner correction recorded 2026-11-23; verify the wording or record the escalation).
  - ADR-043 decision 21 exception for MS-035 and MS-036 is recorded in the 2026-11-23 amendment; verify it or record the architectural escalation.
  - The duplicate AUD-145 and AUD-148 records are resolved, and `papyrus check` reports no duplicate-id finding for them.
  - The hardware deviation (6 physical and 12 logical cores, 15.3 GiB versus the 4 vCPU and 8 GiB reference) is present in the release report.
  - Production LOC is recorded against the 28,000 internal target as informational only, not an acceptance criterion.
- RC and package gate per MS-037 AC-7: after every quality and code check passes, determine the RC version with `next-version` in RC mode and publish the RC package for that version under `builds/<version>/` with the same `scripts/build-package.sh` command, recording version, commit SHA, script paths and commands, archives, paths, sizes, and sha256; or record an explicit user waiver with scope, reason, and timestamp with timezone. The output path is permitted by the ADR-043 decision 7 rule implemented in TASK-283.
- Confirm exactly one release artifact exists and no intermediate release or RC is retained; the RC is discardable.
- Record the release report (the project's `release-reports` document under `docs/release-reports/`) with the NFR-007.001 and NFR-007.002 hardware deviation caveat for the owner.

## Out of scope
- Do not repair a failing criterion; report the command and artifact to the orchestrator.
- Do not edit MS-036, ADR-043, requirements, rules, specifications, mockups, or source.

## Definition of done
Every MS-037 acceptance criterion and cross-milestone gate has a recorded pass verdict with a reproducible command or artifact; the carried items are verified or escalated; the RC version and package evidence, or a recorded user waiver, are present; the release report contains the hardware deviation caveat; exactly one release artifact remains; the task is Done only then, and the release is handed to the orchestrator for Phase 6.

## Acceptance Criteria
- [ ] #1 After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] #2 The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] #3 This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
- [ ] #4 The RC package does not replace the final release artifact: TASK-288 remains the single final release package produced with the release build script, and both the RC evidence and the final package evidence are recorded before the release leaves this task.
- [ ] #5 Every MS-037 AC-1 through AC-7 has a recorded pass or fail verdict with a command or artifact.
- [ ] #6 The full suite, clippy, fmt, no-drift, document-type no-drift, descriptor-leak guard, and no-new-crate gates pass and are named.
- [ ] #7 Zero document TypedHandler variants and no document slug dispatch outside the seed are verified by the guard suite.
- [ ] #8 Record, JSON, and status-code equivalence and semantic HTML equivalence pass against the MS-032 corpus.
- [ ] #9 NFR-007.001 and NFR-007.002 results are presented with the hardware deviation and are not stated as NFR satisfaction; NFR-007.003 cache classes pass; the binary is at most 12,582,912 bytes.
- [ ] #10 MS-036 AC-8 records `Connection: close` as the preserved behaviour; the ADR-043 decision 21 exception is recorded in the 2026-11-23 amendment; the duplicate AUD-145 and AUD-148 records are resolved and papyrus check is clean for them.
- [ ] #11 The release report contains the NFR-007.001 and NFR-007.002 hardware deviation caveat and records production LOC against 28,000 as informational only.
- [ ] #12 Exactly one release artifact exists after the checks; no intermediate release or RC is retained; any RC produced is recorded as discardable.
- [ ] #13 The task is Done only when the carried items and the RC evidence or waiver are present, and the release is handed to the orchestrator for Phase 6.

## Comments
- **task-refiner**: Reproduced at refinement: the RC command output directory is refused by package-out, so the RC gate is blocked on the TASK-288 escalation.
- **planner**: planner resolution: the TASK-288 output-path escalation is cleared; ADR-043 decision 7 as amended 2026-11-23 permits the versioned builds path and TASK-283 implements and tests the guard. MS-036 AC-8 wording was corrected by the planner on 2026-11-23.
