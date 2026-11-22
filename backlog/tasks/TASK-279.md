---
id: TASK-279
title: "Verify MS-036 performance: NFR targets and cache headers"
created_date: '2026-11-20T17:43:25Z'
updated_date: '2026-11-22T12:31:50Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-036
dependencies:
  - TASK-278
ordinal: 90
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-036.

Scope: check every MS-036 acceptance criterion on the recorded reference hardware. Re-run the harness independently of the implementer; confirm inventory p95 under 250 ms and view and edit p95 under 100 ms at 1,000 records; confirm no per-record config or registry construction; confirm atomic swap and fail-closed reload; confirm vendor immutable, app no-cache plus ETag and 304, and HTML and JSON no-store; confirm Mermaid is requested only on diagram pages; confirm the full suite is green and record/render behavior is unchanged.

Definition of done: this task is Done only when every milestone acceptance criterion passes, the quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded here, and the RC package for that version is published from the committed verified tree into the git-ignored builds directory by running each deployable component's scripts/build-package.sh --version <version> --out builds/<version>/.build/<component>, with the package evidence recorded here (version, commit SHA, script paths and commands, component archives, paths, sizes, sha256), or an explicit user waiver with scope, reason, and timestamp with timezone is recorded.
## Acceptance Criteria
- [ ] After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
