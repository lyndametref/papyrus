# Release Lifecycle Semantics

## Purpose

Define the lifecycle of a Papyrus release record so that REQ-03.002, the README manpage (`papyrus(1)`), BR-007, and downstream implementation (TASK-8) agree before release behavior is implemented.

## Sources

- `README.md`, `papyrus(1)` → COMMANDS → release (manpage release command entry).
- `README.md`, `papyrus(1)` → STATUSES → Releases.
- `README.md`, `papyrus(1)` → STATUSES → Milestones (for contrast).
- `docs/requirements/business/REQ-03.002.md` — Manage Releases.
- `docs/business-rules/BR-007-release-command-behavior.md` — release command behavior.
- `docs/domain-model/business-glossary/release.md` — Release term.
- `docs/domain-model/object-model/business-object-model.md` — Release object description.
- TASK-22 (research task) and TASK-8 (downstream implementation).
- User clarification captured in TASK-22: releases are not archived; a release may instead be "released".

## Documented release behavior (README manpage)

The manpage documents these release commands only:

- `papyrus release create <title>`
- `papyrus release list`
- `papyrus release view <id>`
- `papyrus release edit <id> [--title <title>] [-d <description>]`

No `archive`, `restore`, or `delete` command is documented for releases. Release edit supports only title and description.

The manpage status line now reads:

- Releases: `active -> released` ("Releases group milestones.")

By contrast, milestones document `papyrus milestone archive <id>` and read `active -> archived`.

## Clarified lifecycle (product decision recorded by TASK-22)

1. **Release completion is distinct from archiving.** "Released" represents delivery completion of a release; archiving is a record-management action. These are not the same transition.
2. **Releases are not archived.** A release record is not moved off the board via an archive action; there is no documented release archive/restore command, and none is inferred.
3. **Release lifecycle is `active -> released`.** The terminal delivery state is `released`, not `archived`. The README STATUSES → Releases line and BR-007 excerpt must be updated to `active -> released` to match this decision.
4. **No new command is invented by this specification.** How a release transitions from `active` to `released` is not documented in the manpage and remains an open question (see Open Questions). Until resolved, implementation must not add an undocumented transition.

## Open Questions

1. **Mechanism to mark a release as released.** The manpage provides no status option on `papyrus release edit` and no dedicated command. Whether the state is set explicitly (e.g., a future `--status`/`release` command) or derived from milestone/task completion is unresolved. Record this in REQ-03.002 until a source or user decision resolves it.
2. **Stored vs. derived state.** Whether `released` is a stored release attribute or derived from the completion of its milestones is unresolved.
3. **README update.** The README STATUSES → Releases line has been updated to `active -> released`, and BR-001/BR-007 verbatim excerpts aligned. No further approval needed for this documentation update.

## Affected Artifacts

- `README.md` — STATUSES → Releases line; (no release archive command to add).
- `docs/requirements/business/REQ-03.002.md` — lifecycle wording and open question.
- `docs/business-rules/BR-007-release-command-behavior.md` — verbatim manpage excerpt if README changes.
- `docs/domain-model/business-glossary/release.md` — "active or archived" → "active or released" lifecycle wording.
- `docs/domain-model/object-model/business-object-model.md` — Release object description lifecycle wording.
- TASK-8 — acceptance criterion #3 ("release status is represented as documented without inferring an undocumented archive command").

## Traceability

| Claim | Source |
|---|---|
| Release commands are create/list/view/edit only | `README.md`, `papyrus(1)` → release (manpage) |
| No release archive/restore/delete command is documented | `README.md`, `papyrus(1)` → release (manpage) |
| Current manpage release status is `active -> released` | `README.md`, `papyrus(1)` → STATUSES → Releases |
| Milestone status is `active -> archived` with an archive command | `README.md`, `papyrus(1)` → milestone/STATUSES → Milestones |
| Releases are not archived; may be "released" | User clarification recorded in TASK-22 |
| Completion is distinct from archiving | User clarification recorded in TASK-22 |

## Assumptions

- Assumption: README.md is the sole behavior source for `papyrus release` (BR-007); no transition or command is inferred beyond it.
- Assumption: "released" is a delivery-completion state, not a record-management/archive state.
- Assumption: Requirements edits are performed by the requirements-engineer, not by this refinement task's implementer directly.
- Assumption: The README STATUSES → Releases update is in scope for this task because the user confirmed the manpage may be updated.
