---
id: TASK-118
title: Implement `papyrus release close` for the planned to released transition
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:28:00Z'
updated_date: '2026-11-14T21:12:00Z'
labels:
  - rust
  - cli
  - browser
  - release
milestone: m-18
dependencies: []
references:
  - docs/business-rules/BR-007-release-command-behavior.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/business/REQ-07.001.md
documentation:
  - docs/specifications/milestone-and-release-management.md
  - src/release.rs
  - src/registry.rs
priority: high
ordinal: 41200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** BR-007 and REQ-03.002 document `papyrus release close <id>` as the only `planned -> released` transition, but `src/release.rs` has no `Close` variant and `ReleaseCmd` exposes only create/list/view/edit, so the command exits 2 and a release can never reach `released`. TASK-117 recorded this as G-5, release-blocking for the m-17 gate. The browser lifecycle text compounds the gap by advertising a non-existent `papyrus release status <id> released` command. The owner chose to implement the documented command rather than amend BR-007.

**What.** Add the `close` verb end to end: `Verb::Close` in `src/registry.rs` with `RELEASE_VERBS` updated, a `ReleaseCmd::Close { id }` variant in `src/release.rs`, and a `close()` implementation that resolves the release, reads its front-matter status, refuses anything but `planned`, sets `released`, and writes through `record::store::write_atomic` (the single write path, ADR-0013). Also write `status: planned` in `create()` so a native release starts in its documented state. Correct the browser help/lifecycle text in `src/browser/render.rs` to name `papyrus release close <id>` and the `planned -> released` lifecycle.

**Refinement decision (2026-11-14).** Verified in a scratch project: native `release create` currently writes no `status` key (only migration seeds `planned`). Per owner decision, `create()` gains `status: planned`, and `close()` treats a missing or empty front-matter status as `planned` so imported/legacy files remain closable. The stale specifications `docs/specifications/release-lifecycle.md` and `docs/specifications/milestone-and-release-management.md` are updated in this task to record the implemented transition; requirement, business-rule, and ADR files stay untouched.

**In scope.**

- `src/registry.rs`: `Verb::Close`, `Verb::as_str() == "close"`, `RELEASE_VERBS` includes it; registry tests assert the release verb set and that no archive/restore/delete/status verb is added.
- `src/release.rs`: `ReleaseCmd::Close { id }`, `close()`, status read with absent/empty treated as `planned`, rejection of any other status, `status: released` plus `updated_date` refresh, text (ID) and `--json` (`{"id","status"}`) output; `create()` writes `status: planned`.
- `src/browser/render.rs`: replace the `release status ... released` example with `papyrus release close <release id>` and change `active -> released` to `planned -> released`; keep wording consistent with BR-007.
- `docs/specifications/release-lifecycle.md` and `docs/specifications/milestone-and-release-management.md`: mark the transition as implemented and the old open questions resolved (documentation only).
- CLI integration tests and browser text assertions; no archive, restore, delete, or generic `status` verb is added for releases.

**Out of scope.**

- Reopening a release (`released -> planned`), bulk close, `--status` on `release edit`, and any release archive/delete behavior.
- Amending BR-007, REQ-03.002, REQ-07.001, or an ADR; these already describe the implemented behavior.
- The m-18 refactor and cache tasks; `release` stays a builtin command path in this task.

**Assumptions.**

- Assumption: `close` text output prints only the release ID, matching `release create`.
- Assumption: `close --json` emits one object with keys `id` and `status` set to `released`, consistent with the existing create JSON shape.
- Assumption: an absent or empty `status` is equivalent to `planned`; any other stored value other than `planned` is rejected with exit 4 and no write.
- Assumption: `updated_date` is refreshed only when the transition writes; a rejected close leaves the file byte-identical.
- Assumption: spec-file edits are documentation alignment only and do not change requirement or business-rule text.

**Dependencies.** None.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `papyrus release close <id>` is listed in `papyrus release --help` and on a `planned` release moves `status: planned` to `status: released`, preserving title, description, labels, references, and body, and refreshing only `updated_date` among timestamps
- [x] #2 The transition is implemented with `Verb::Close` in `RELEASE_VERBS`, a `ReleaseCmd::Close` variant, and a `close()` that writes through `record::store::write_atomic`; no direct file write is added in the release path
- [x] #3 Closing an already `released` release exits 4 with a message naming the current status and leaves the file byte-identical; an unknown ID exits 3; no archive, restore, delete, or `--status` verb appears for releases
- [x] #4 `--json` on close emits exactly one JSON object carrying the release ID and new status; text mode prints the ID or a one-line confirmation consistent with `release create`
- [x] #5 No occurrence of `papyrus release status <id> released` remains; `src/browser/render.rs` names `papyrus release close <id>` and the browser lifecycle text matches BR-007
- [x] #6 CLI integration tests cover close success, already-released rejection, unknown ID, JSON output, and front-matter/body preservation; a browser test asserts the corrected lifecycle text
- [x] #7 BR-007, REQ-03.002, and REQ-07.001 are satisfied without editing requirement, business-rule, or ADR files
- [x] #8 `papyrus release create` writes `status: planned` in the front matter, and `close` treats a missing or empty status as `planned`; any other value is refused
- [x] #9 `docs/specifications/release-lifecycle.md` and `docs/specifications/milestone-and-release-management.md` describe `close` as implemented and mark the previous open questions resolved, with no edits to requirement, business-rule, or ADR files
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 `cargo build`, `cargo test`, and `cargo clippy` are clean.
- [x] #2 G-5 is reproducible as fixed: the TASK-117 command `papyrus release close REL-001` exits 0 and `release view` shows `status: released`.
- [x] #3 No new dependency or flag is added; release remains a fixed `planned`/`released` lifecycle with no configuration coupling.
- [x] #4 Registry verb list, CLI help, and the browser lifecycle text agree, and the no-drift test file still passes.
- [x] #5 Specification files no longer state that the release transition is unimplemented or unresolved.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. `src/registry.rs`: add `Verb::Close` and its `as_str()` arm; include `Verb::Close` in `RELEASE_VERBS`; extend the registry unit test to assert `release.has_verb(Verb::Close)` and that Archive/Restore/Delete/Status stay absent.
2. `src/release.rs`:
   a. Add `ReleaseCmd::Close { id: String }` with a help comment, and dispatch it in `run()` to `close(ctx, &id)`.
   b. In `create()`, append `PatchOp::SetFrontMatterValue("status", "planned")` after `references` so the front-matter order is id, title, created_date, updated_date, labels, references, status.
   c. Implement `close(ctx, id_raw)`: sanitize the ID; load config; `resolve_release`, mapping `None` to `AppError::NotFound("not found: <id>")` (exit 3); call `record.ensure_writable()`; `store::read_document`; parse front matter and read `status`; treat missing/empty as `planned`; if not `planned`, return `AppError::Validation` naming the current status (exit 4) before any write.
   d. Build `PatchOp::SetFrontMatter("status", "released")` and `SetFrontMatter("updated_date", frontmatter::utc_now())`, apply with `record::patch::apply_patch`, and write via `Context::ensure_within` + `record::store::write_atomic` only.
   e. Return `CommandOutput { text: id, json: ctx.json.then(|| json!({"id": id, "status": "released"})) }`.
3. `src/browser/render.rs`: change the release/audit group text to `planned -> released` and the snippet to `papyrus release close {release_example}`; leave routes.rs lifecycle text unchanged (already correct).
4. Specs: update the two files under `docs/specifications/` to state that `close` is implemented, `status` is stored and seeded `planned` on create, absent status is read as `planned`, and OQ-1/OQ-2 are resolved; do not touch requirements, business rules, or ADRs.
5. Tests:
   - `tests/cli_dispatch.rs`: add `close` to the `release --help` needle list.
   - New `tests/release_close_cli.rs`: create -> close exits 0 and prints the ID; file has `status: released`, title/description/labels/references/body preserved, `created_date` unchanged, `updated_date` refreshed; second close exits 4, stderr names `released`, file bytes identical; unknown ID exits 3; `--json` prints exactly one object with `id` and `status`; a fixture with no status closes successfully; create fixture contains `status: planned`.
   - `tests/browser_cli.rs` skills test: assert the page contains `papyrus release close REL-001` and `planned`/`released` lifecycle wording, and does not contain `papyrus release status`.
6. Run `cargo fmt --check`, `cargo build`, `cargo test`, `cargo clippy --all-targets`.
<!-- SECTION:PLAN:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
papyrus release close <id> implemented: Verb::Close + RELEASE_VERBS, ReleaseCmd::Close, close() patches status released + updated_date through record::store::write_atomic; create seeds status: planned; absent/empty status treated as planned; repeat close exit 4 naming status with byte-identical file; unknown exit 3; text prints ID, JSON {"id","status":"released"}; no archive/restore/delete/--status. Browser lifecycle text and README release section corrected to planned -> released with close. Evidence: release_close_cli 8/8, cli_dispatch 9/9, full cargo test 797 passed / 0 failed, clippy and fmt clean. G-5 resolved.
<!-- SECTION:FINAL_SUMMARY:END -->
