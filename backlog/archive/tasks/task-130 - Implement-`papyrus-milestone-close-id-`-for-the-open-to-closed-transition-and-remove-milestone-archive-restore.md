---
id: TASK-130
title: >-
  Implement `papyrus milestone close <id>` for the open to closed transition and
  remove milestone archive/restore
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T12:42:00Z'
updated_date: '2026-11-15T12:57:00Z'
labels:
  - rust
  - cli
  - browser
  - tests
milestone: m-19
dependencies: []
references:
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-07.001.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
documentation:
  - src/milestone.rs
  - src/registry.rs
  - src/browser/render.rs
modified_files:
  - src/milestone.rs
  - src/registry.rs
  - src/browser/render.rs
  - README.md
priority: high
ordinal: 41302
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** REQ-03.001 fixes the milestone lifecycle to `open -> closed`, transitioned only by `papyrus milestone close <id>`, with no restore command (REQ-07.001, backlog category). The code does not match: `src/registry.rs` `MILESTONE_VERBS` exposes `Verb::Archive` and no `close`; `src/milestone.rs` has `MilestoneCmd::Archive` with an `archive()` implementation and no close variant; `README.md` line 281 and the browser help text still advertise `papyrus milestone archive <id>`. A milestone therefore can never reach `closed` through the documented command, and the removed verb is still offered. ADR-0022 decision 5 still claims milestones keep archive/restore; TASK-131 amends the ADR, this task delivers the behavior the amendment describes. TASK-127 cites the same lifecycle and depends on both.

**What.** Add the `close` verb end to end for milestones and delete archive/restore from the milestone command surface.

**In scope.**

- `src/registry.rs`: `MILESTONE_VERBS` becomes `create, list, view, edit, close, delete`; `Verb::Close` is reachable for milestones and archive/restore/status are not; update `backlog_descriptors_match_the_current_command_families` and any other registry test asserting the milestone verb set.
- `src/milestone.rs`: replace `MilestoneCmd::Archive { id }` with `MilestoneCmd::Close { id }`; implement `close()` that resolves the milestone, reads the front-matter `status`, refuses any value other than `open`, sets `status: closed`, refreshes `updated_date`, and writes through `record::store::write_atomic` (the single write path, ADR-0013); print the milestone ID on success and support `--json` with `id` and `status`.
- `src/milestone.rs`: `create()` writes `status: open` in front matter so a native milestone starts in its documented state instead of relying on a reader default.
- Error mapping: closed milestone -> validation error, exit 4, message naming the current status (`closed`) and the file byte-identical; unknown milestone ID -> exit 3 with the standard not-found message; `milestone archive` and `milestone restore` -> unknown subcommand, exit 2.
- `src/browser/render.rs`: replace the `papyrus milestone archive <id>` example/help with `papyrus milestone close <milestone id>` and state the `open -> closed` lifecycle; no milestone archive/restore text remains.
- `README.md`: replace the `papyrus milestone archive &lt;id&gt;` line with `papyrus milestone close &lt;id&gt;` and align the surrounding milestone lifecycle wording.
- Tests: CLI integration tests for close success, close-of-closed (exit 4, status named, file bytes unchanged), unknown ID (exit 3), and archive/restore rejected (exit 2); registry assertions for the new verb set; browser text assertion; `cargo test` green.

**Out of scope.** Reopening a milestone (`closed -> open`), bulk close, `milestone status <id> <value>`, `--status` on `milestone edit`, task verbs, release verbs, and any migration or relocation of milestone files that already exist under a milestone archive location. Requirements, business rules, ADRs, and specifications are not edited here; ADR-0022 is TASK-131 and the specification pages are TASK-127 and TASK-128.

**Assumptions.**

- A missing or empty front-matter `status` is treated as `open`, mirroring the imported-file handling TASK-118 established for `release close`; any other stored value other than `open` is rejected with exit 4 and no write.
- Text output prints only the milestone ID, matching `milestone create` and `release close`; `--json` emits one object with keys `id` and `status` set to `closed`.
- Existing archived milestone files stay readable through the unchanged list/view/resolve paths; removing the verbs does not delete or migrate files on disk.
- Removing `Verb::Archive` for milestones is a CLI surface change only; no exit-code or output change is made for tasks.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 papyrus milestone close <MS-...> on an open milestone writes status: closed, refreshes updated_date through record::store::write_atomic, and prints the milestone ID
- [x] #2 milestone create writes status: open in the front matter of the new milestone file
- [x] #3 Closing an already closed milestone exits 4, names closed as the current status, writes nothing, and leaves the file byte-identical
- [x] #4 Closing an unknown milestone ID exits 3 with the standard not-found message
- [x] #5 papyrus milestone archive <id> and papyrus milestone restore <id> both exit 2 as unknown subcommands, and archive/restore do not appear in milestone help or list of verbs
- [x] #6 Registry: MILESTONE_VERBS is exactly create, list, view, edit, close, delete; Verb::Close is registered for milestones; registry tests assert the new set and the absence of archive, restore, and status
- [x] #7 Browser help/lifecycle text and README name papyrus milestone close <id> and the open -> closed lifecycle, and no milestone archive/restore instruction remains in either
- [x] #8 Integration tests cover close success, close-of-closed, unknown ID, and rejected archive/restore; cargo test passes
- [x] #9 Behavior matches REQ-03.001 and REQ-07.001 and the TASK-131 amendment of ADR-0022 decision 5
- [x] #10 No task, release, or document behavior changes; only the milestone command surface and create/close lifecycle change
<!-- AC:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Milestone lifecycle is now close-only: MILESTONE_VERBS = create/list/view/edit/close/delete; create seeds status: open; close requires open, patches status: closed + updated_date through record::store::write_atomic, prints the ID (JSON {id,status}); closing a closed milestone exits 4 naming closed with a byte-identical file; unknown exits 3; milestone archive/restore exit 2 and are absent from help. Registry tests assert the exact verb set and absence of archive/restore/status. Browser skills text and README name papyrus milestone close and open -> closed with no archive/restore instruction. No task/release/document behavior changed. Evidence: milestone_cli 4/4, cli_dispatch/browser/search/release suites adapted and green, full suite 853 passed / 0 failed / 22 ignored; clippy and fmt clean. ADR-0022 amendment remains TASK-131.
<!-- SECTION:FINAL_SUMMARY:END -->
