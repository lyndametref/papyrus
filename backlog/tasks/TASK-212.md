---
id: TASK-212
title: "Correct MS-030 code audit findings (AUD-132)"
created_date: '2026-11-19T19:11:02Z'
updated_date: '2026-11-19T19:25:56Z'
labels: []
references:
  - AUD-132
  - MS-030
  - TASK-209
  - REQ-06.020
  - REQ-06.021
  - REQ-06.022
  - REQ-06.013
  - TC-066
  - TC-066..TC-097
status: Done
assignee:
  - rust-dev
milestone: MS-030
dependencies:
  - TASK-209
ordinal: 55
documentation:
  - src/browser/mod.rs
  - src/browser/render.rs
  - tests/browser_cli.rs
type: implementation
---

## Acceptance Criteria
- [ ] #1 TC-066..TC-097 implemented.
- [ ] #2 TC-066 implemented.
- [ ] #3 TC-066..TC-097 are implemented as integration tests in the browser_cli integration test file with strict assertions; no assertion uses OR to pass when markers or cleanup controls are missing.
- [ ] #4 Tests assert milestone and release ordering by lowest contained task ordinal, archived-task exclusion, no-task placement after tasked records, and milestone or release ID tie-breaking.
- [ ] #5 Tests assert open and closed milestone markers and planned and released release markers immediately before titles, and that archived milestones are not displayed.
- [ ] #6 Tests assert cleanup controls appear only for closed milestones and released releases containing closed milestones, immediately after titles.
- [ ] #7 Tests assert milestone cleanup confirmation lists the milestone ID, archives the milestone on confirm, and archives nothing on cancel.
- [ ] #8 Tests assert release cleanup confirmation lists closed milestone IDs, archives each listed milestone on confirm, leaves the release visible and released, and never archives the release.
- [ ] #9 Tests assert partial archive failure is reported, remaining milestones are still archived, and the failed milestone remains visible.
- [ ] #10 src browser mod.rs accepts only canonical milestone_cleanup and release_cleanup query or form parameters unless an alias is explicitly specified and documented.
- [ ] #11 src browser render.rs release-cleanup confirmation cancel link uses the same expand target as milestone-cleanup confirmation.
- [ ] #12 board_release_cleanup_submit captures a surviving expand target before archiving; successful cleanup does not fall back to an archived milestone.
- [ ] #13 The dead archived arm is removed from milestone_marker, or retained only with a documented harmless rationale.
- [ ] #14 cargo fmt check, cargo clippy all targets, cargo test test browser_cli, and cargo test pass, excluding the pre-existing unrelated list_characterization_replay failure.
## Description
Code audit AUD-132 requires corrections before MS-030 verification. Scope is limited to the browser backlog board: implement TC-066..TC-097 as strict integration tests and apply the low-severity cleanup fixes. No new CLI command, API endpoint, write path, or ADR.
## Comments
- **rust-dev**: done
- **rust-dev**: Branch task TASK-212 commit ed64453 strict tests
- **rust-dev**: TC-066 to TC-097 strict integration tests added
- **rust-dev**: Canonical cleanup params and surviving expand target
- **rust-dev**: Cancel link aligned dead archived arm removed
- **rust-dev**: fmt clippy pass browser_cli 214 passed full cargo only pre-existing failure
- **rust-dev**: Final commit 3bc3ce5 branch task TASK-212
- **rust-dev**: fmt and clippy pass browser_cli 213 passed full cargo only pre-existing failure
- **orchestrator**: Correction verified: cargo fmt --check, cargo clippy --all-targets, cargo test --test browser_cli (213 passed) pass; full cargo test still only the pre-existing unrelated list_characterization_replay failure. TC-066..TC-097 implemented as strict integration tests; alias params canonicalized; expand target and cancel link aligned; dead archived arm removed. TASK-212 Done.
