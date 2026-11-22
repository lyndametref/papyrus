---
id: TASK-289
title: "Verify MS-037 and confirm the single release is ready"
created_date: '2026-11-20T17:44:23Z'
updated_date: '2026-11-22T12:31:50Z'
labels: []
references:
  - REQ-07.004
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-037
dependencies:
  - TASK-288
ordinal: 100
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-037, and the only release boundary.

Scope: check every MS-037 acceptance criterion and every cross-milestone gate: full suite green, clippy clean, no_drift green, descriptor-leak guard green, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, record and JSON and status-code equivalence from MS-032 fixtures, semantic HTML equivalence, NFR-007.001 and NFR-007.002 met at 1,000 records, binary at or under 12 MiB, and no open blocking audit finding. Confirm exactly one release artifact exists and no intermediate release or RC is retained. Record the result and hand the release to the orchestrator for Phase 6.

Definition of done: this task is Done only when every milestone acceptance criterion passes, the quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded here, and the RC package for that version is published from the committed verified tree into the git-ignored builds directory by running each deployable component's scripts/build-package.sh --version <version> --out builds/<version>/.build/<component>, with the package evidence recorded here (version, commit SHA, script paths and commands, component archives, paths, sizes, sha256), or an explicit user waiver with scope, reason, and timestamp with timezone is recorded.

The RC package is a discardable verification build; the single final release artifact remains the scripts/build-package.sh package produced by TASK-288, which delegates Debian assembly to scripts/build-deb.sh.
## Acceptance Criteria
- [ ] After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
- [ ] The RC package does not replace the final release artifact: TASK-288 remains the single final release package produced with the release build script, and both the RC evidence and the final package evidence are recorded before the release leaves this task.
