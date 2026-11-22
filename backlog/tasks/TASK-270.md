---
id: TASK-270
title: "Verify MS-035 browser engine: route and semantic equivalence"
created_date: '2026-11-20T17:42:52Z'
updated_date: '2026-11-22T12:31:50Z'
labels: []
references:
  - REQ-06.014
  - REQ-06.019
  - ADR-039
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-035
dependencies:
  - TASK-269
ordinal: 90
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-035.

Scope: check every MS-035 acceptance criterion. Confirm one conflict page and one not-found page remain; confirm the semantic fixture comparison passes with attributed differences only; confirm group, member, HTML mockup, and SingleMarkdown rendering; confirm reference visibility from the seed; confirm zero document TypedHandler variants and no slug dispatch outside the seed; confirm Backlog and Decision surfaces unchanged; confirm the full suite is green.

Definition of done: this task is Done only when every milestone acceptance criterion passes, the quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded here, and the RC package for that version is published from the committed verified tree into the git-ignored builds directory by running each deployable component's scripts/build-package.sh --version <version> --out builds/<version>/.build/<component>, with the package evidence recorded here (version, commit SHA, script paths and commands, component archives, paths, sizes, sha256), or an explicit user waiver with scope, reason, and timestamp with timezone is recorded.
## Acceptance Criteria
- [ ] After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
