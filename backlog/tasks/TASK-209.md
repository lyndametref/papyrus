---
id: TASK-209
title: "Implement lowest-ordinal milestone ordering, markers, and cleanup in browser"
created_date: '2026-11-19T11:02:54Z'
updated_date: '2026-11-19T18:43:11Z'
labels: []
references:
  - REQ-06.020
  - REQ-06.021
  - REQ-06.022
  - REQ-06.013
  - RE-008
  - TECH-011
  - TECH-062
  - UI-002
  - MOCK-022
  - BR-004
  - BR-005
  - BR-006
  - BR-007
  - ADR-0005
  - ADR-0006
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - ADR-0022
  - ADR-0032
  - TC-066
  - TC-067
  - TC-068
  - TC-069
  - TC-070
  - TC-071
  - TC-072
  - TC-073
  - TC-074
  - TC-075
  - TC-076
  - TC-077
  - TC-078
  - TC-079
  - TC-080
  - TC-081
  - TC-082
  - TC-083
  - TC-084
  - TC-085
  - TC-086
  - TC-087
  - TC-088
  - TC-089
  - TC-090
  - TC-091
  - TC-092
  - TC-093
  - TC-094
  - TC-095
  - TC-096
  - TC-097
status: ToDo
assignee:
  - rust-dev
milestone: MS-030
dependencies:
  - TASK-208
ordinal: 50
documentation:
  - src/task.rs
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/mod.rs
  - tests/browser_cli.rs
  - docs/specifications/technical/TECH-011.md
  - docs/specifications/technical/TECH-062.md
  - docs/specifications/ui/browser-backlog-records-ui.md
type: implementation
---

## Description
Code and automated tests for ordering milestones and releases by the lowest ordinal of their non-archived tasks ascending, lifecycle markers, and cleanup controls in the browser Backlog board. The ascending lowest-ordinal rule supersedes the earlier highest-ordinal descending rule.

Scope (browser combined Backlog board `/?type=task` only):
- Ordering (REQ-06.020): milestones inside a release and in the no-release group are ordered by the lowest ordinal among their non-archived tasks, ascending; a milestone with no non-archived task is ordered after every milestone that has one, by milestone ID ascending; ties by milestone ID ascending. Releases are ordered by the lowest ordinal among the non-archived tasks in their milestones, ascending; a release with no such task is ordered after every release that has one, by release ID ascending; ties by release ID ascending. The no-release group precedes all releases. Archived tasks do not contribute to any sort key. The order is recomputed from current record data on every render. Release grouping and the tasks-without-a-milestone section are unchanged.
- Lifecycle markers (REQ-06.021): the marker 🔘 is shown immediately before the title of an open milestone or a planned release; ✅ immediately before the title of a closed milestone or a released release. Archived milestones are not displayed and carry no marker. The marker reflects stored lifecycle state and changes after that state changes and the view is reloaded. The marker is shown in addition to the record ID and does not change grouping or ordering.
- Cleanup controls (REQ-06.022): each displayed closed milestone shows a cleanup control immediately after its title; each displayed released release that contains at least one closed milestone shows a cleanup control immediately after its title; a released release with no closed milestone shows none; open milestones and planned releases show none. Activating a milestone cleanup control shows a server-rendered confirmation listing the milestone ID that will be archived; confirming archives that milestone using milestone archive semantics (REQ-03.001) and removes it from the view; cancelling archives nothing. Activating a release cleanup control shows a server-rendered confirmation listing the IDs of the closed milestones in that release that will be archived; confirming archives each listed milestone, one by one; the release itself remains visible and released and is never archived. If archiving one milestone fails, the failure is reported for that milestone, the remaining milestones are still archived, and the failed milestone stays visible. The controls are browser-only and add no CLI command, JSON API endpoint, or write path.

Implementation surface:
- src/task.rs: add the task ordinal to TaskData (parsed from the `ordinal` front-matter key) so the browser can read it.
- src/browser/records.rs: read the task ordinal into the browser Record, and read milestone and release lifecycle status (`status` front-matter key) into Record.status (defaulting to open/planned when absent).
- src/browser/render.rs: sort milestones and releases by the lowest contained non-archived task ordinal with the tie and no-task rules above; render lifecycle markers in the milestone summary and release heading; render milestone and release cleanup controls and their server-rendered confirmation regions.
- src/browser/mod.rs: handle the milestone and release cleanup preview and confirmation requests (query parameters distinct from the task `Clean up all` `cleanup` parameter), validate inputs, archive each closed milestone through crate::milestone::archive, report partial failures, and redirect or re-render as appropriate.
- Automated tests in tests/browser_cli.rs covering TC-066 through TC-097.

Out of scope: CLI commands, JSON API endpoints, new write paths or ADRs, record files, front matter, other browser views, mockups, requirements, and the plan.

Assumptions:
- Prerequisites met: architecture settled and user-verified (TASK-205 Done; TECH-011 and TECH-062 draft verified, conformance conformant); UI design TASK-206 Done with UI-002 verified and MOCK-022 approved; test design TASK-207 Done with TC-066 through TC-097 approved; dependency TASK-208 Done.
- The browser reads milestone and release lifecycle status from the `status` front-matter key; managed milestones and releases always carry it.
- The task ordinal is read from the `ordinal` front-matter key; a missing or unparseable ordinal is treated as having no ordinal and does not contribute to a sort key.
- Active (non-archived) tasks and milestones come from the active backlog directories; archived records are already excluded by the index.
- Milestone and release cleanup reuse crate::milestone::archive and introduce no new write path (ADR-0013); the release itself is never archived (REQ-03.002).
- Rust 1.99.1 toolchain available (ADR-0001); Firefox 115+ and no-JavaScript support required (NFR-05.001, NFR-01.002); browser command behaviour governed by BR-004; views render only local data (NFR-01.003).
- Verification task TASK-210 tracks MS-030 verification.

Definition of done: TASK-209 implements the change in the files above, automated tests cover TC-066 through TC-097, every acceptance criterion is satisfied and verified by TASK-210, cargo fmt, cargo clippy, and cargo test pass, and no open material questions remain.
## Acceptance Criteria
- [ ] #1 Ordering: milestones within a release are ordered by the lowest ordinal of their non-archived tasks, ascending; a milestone with no non-archived task is ordered after every milestone that has one, by milestone ID ascending; milestones with equal lowest ordinals are ordered by milestone ID ascending (TC-066, TC-068, TC-069; MS-030 AC-1).
- [ ] #2 Ordering: milestones in the no-release group follow the same rule and the no-release group precedes all releases (TC-067, TC-068; MS-030 AC-2).
- [ ] #3 Ordering: releases are ordered by the lowest ordinal of the non-archived tasks in their milestones, ascending; a release with no such task is ordered after every release that has one, by release ID ascending; releases with equal lowest ordinals are ordered by release ID ascending (TC-070, TC-071, TC-072; MS-030 AC-3).
- [ ] #4 Ordering: archived tasks do not contribute to any milestone or release sort key (TC-073; MS-030 AC-1, AC-3).
- [ ] #5 Ordering: changing a task ordinal and reloading the view changes the position of its milestone and release accordingly (TC-074; MS-030 AC-1, AC-3).
- [ ] #6 Ordering: release grouping, the no-release group placement, and the tasks-without-a-milestone section are unchanged (TC-075; MS-030 AC-1, AC-2, AC-3).
- [ ] #7 Markers: every displayed open milestone shows 🔘 immediately before its title and every displayed closed milestone shows ✅ immediately before its title (TC-076, TC-077; MS-030 AC-4).
- [ ] #8 Markers: every displayed planned release shows 🔘 immediately before its title and every displayed released release shows ✅ immediately before its title (TC-078, TC-079; MS-030 AC-5).
- [ ] #9 Markers: archived milestones are not displayed and show no marker (TC-080; MS-030 AC-6).
- [ ] #10 Markers: a milestone marker changes from 🔘 to ✅ after the milestone is closed and the view is reloaded; a release marker changes from 🔘 to ✅ after the release is released and the view is reloaded (TC-081, TC-082; MS-030 AC-4, AC-5).
- [ ] #11 Markers: the lifecycle marker appears in addition to the record ID, not instead of it (TC-083; MS-030 AC-4, AC-5).
- [ ] #12 Markers: lifecycle markers do not change milestone or release grouping or ordering (TC-084; MS-030 AC-4, AC-5).
- [ ] #13 Cleanup: each displayed closed milestone shows a cleanup control immediately after its title; each displayed released release containing at least one closed milestone shows a cleanup control immediately after its title (TC-085, TC-086; MS-030 AC-7, AC-8).
- [ ] #14 Cleanup: a released release with no closed milestone shows no cleanup control; an open milestone and a planned release show no cleanup control (TC-087, TC-088, TC-089; MS-030 AC-7, AC-8, AC-9).
- [ ] #15 Cleanup: activating a milestone cleanup control shows a confirmation listing the milestone ID that will be archived (TC-090; MS-030 AC-10).
- [ ] #16 Cleanup: confirming archives that milestone and removes it from the view; cancelling archives nothing (TC-091, TC-092; MS-030 AC-10).
- [ ] #17 Cleanup: activating a release cleanup control shows a confirmation listing the IDs of the closed milestones in that release that will be archived (TC-093; MS-030 AC-11).
- [ ] #18 Cleanup: confirming archives each listed milestone, archived milestones disappear from the view, and the release remains visible and released (TC-094; MS-030 AC-11, AC-12).
- [ ] #19 Cleanup: a release cleanup never archives the release itself (TC-095; MS-030 AC-11, AC-12).
- [ ] #20 Cleanup: if archiving one milestone fails, the failure is reported for that milestone, the remaining milestones are still archived, and the failed milestone stays visible (TC-096; MS-030 AC-13).
- [ ] #21 Cleanup: the controls are available in the browser only and add no CLI command (TC-097; MS-030 AC-14).
- [ ] #22 Verification: cargo fmt --check, cargo clippy --all-targets, and cargo test pass; all TC-066 through TC-097 are satisfied and verified by TASK-210.
- [ ] #25 Implementation stays within the browser backlog board code and introduces no new CLI command, JSON API endpoint, write path, dependency, or ADR.
