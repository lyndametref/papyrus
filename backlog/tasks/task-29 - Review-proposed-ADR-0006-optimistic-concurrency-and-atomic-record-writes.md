---
id: TASK-29
title: Review proposed ADR-0006 optimistic concurrency and atomic record writes
status: Done
assignee:
  - '@architect'
created_date: '2026-11-03T21:11:00Z'
updated_date: '2026-11-04T18:51:00Z'
labels:
  - architecture
  - adr
  - review
milestone: m-2
dependencies: []
references:
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - docs/specifications/browser-requirements.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - TASK-6
  - TASK-7
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - src/browser/records.rs
  - src/browser/mod.rs
  - src/context.rs
  - tests/browser_cli.rs
type: task
ordinal: 7900
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0006 was written during the TASK-6 architecture checkpoint with status **Proposed**. It fixes how browser edits detect lost updates and how records are written without partial-file exposure, and every later writing surface (TASK-11, TASK-15, TASK-23 … TASK-27) inherits it. It must be decided before m-2 implementation starts.

Scope of this review: verify the ADR against the code actually delivered in m-1 (TASK-6, TASK-7) and against the specifications and requirements it cites. Decide accepted, rejected, or amend. If accepted or amended, update the ADR and regenerate the ADR index.

Review must check, at minimum:
- Decision vs. delivered code: version token composition, 400/409 handling, `create_new` temporary file with numeric suffix retry, `sync_all`, permission copy, rename, best-effort directory sync, failure-path cleanup, save-then-reread, exact ID matching, `Context::ensure_within` on every path, unknown-section preservation, section order, and trailing-newline rule.
- Context accuracy: the claim that the CLI and browser can write the same files, and the constraints attributed to NFR-04.001 and NFR-04.002.
- Consequences and residual risks: the documented check-then-rename race is stated honestly; any residual risk not recorded (for example per-record config re-reads on write, duplicate record parsing between read and write paths, temporary-file leftovers after crash).
- Checkpoint follow-ups the review must either fold into the ADR or explicitly defer: per-record config re-reads, duplicate record parsing, test temp-directory hygiene for write tests, and the manpage/CLI write-path claims that overlap record writes.

Follow-ups: this task may spawn separate follow-up tasks for any accepted gap. Do not create implementation tasks as part of this review unless the decision makes one necessary; capture them as follow-up items instead.

Acceptance of the ADR content is an explicit user decision: the ticket may only record `accepted` after the user has confirmed it through the orchestrator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The ADR and the delivered m-1 implementation (TASK-6/7 code, tests, packaging) have been read and the review evidence is cited in the ticket.
- [x] #2 Decision, context, consequences, and residual risks are verified point by point against the code and the cited specifications/requirements, with discrepancies listed.
- [x] #3 A decision of accepted, rejected, or amend is recorded in the ticket with rationale and evidence.
- [x] #4 If accepted or amended: ADR-0006 Status is updated (and content amended where needed) and docs/architecture/adr/index.html is regenerated per the architecture-decision-record and adr-index-visualization skills.
- [x] #5 The user has explicitly confirmed the acceptance decision through the orchestrator, and that confirmation is recorded in the ticket.
- [x] #6 Follow-up items (including the checkpoint follow-ups named in the description) are captured in the ticket as follow-up notes or new tickets, without being implemented here.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Review evidence recorded in the ticket, each claim tied to a file path, test, or spec section.
- [x] #2 ADR-0006 status and content match the recorded decision; a rejection states the reason and what replaces the decision.
- [x] #3 docs/architecture/adr/index.html regenerated and consistent with ADR-0006 when the ADR changed.
- [x] #4 Follow-ups captured for rejected or amended decisions, with owners implied by the affected area.
- [x] #5 Explicit user acceptance recorded; no task closure without it.
- [x] #6 No changes outside the ADR files and the ADR index.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read ADR-0006 end to end; record its Status (Proposed), Date, Context claims, Decision bullets, Consequences, Risks/mitigations, Alternatives, and References.
2. Read the delivered m-1 implementation and tests: src/browser/records.rs (version_token, valid_version_token, apply_edit, write_atomic), src/browser/mod.rs (form_post, api_post, resolve_record), src/context.rs (ensure_within); tests/browser_cli.rs (edit round-trip, stale version, atomic write, permissions, symlink escape).
3. Read the cited specifications, requirements, business rules, and ADRs: docs/specifications/browser-requirements.md sections 5, 7 (esp. 7.10-7.12), 9; docs/requirements/non-functional/NFR-04.001.md, NFR-04.002.md, NFR-02.002.md; docs/architecture/adr/ADR-0001, ADR-0004, ADR-0005.
4. Verify Decision vs delivered code point by point: version token composition "<mtime-seconds>.<mtime-nanos>.<length>"; 400 on missing/malformed token; 409 on mismatch with no write; matching token proceeds; create_new temporary file with numeric suffix retry; sync_all; permission copy; rename; best-effort directory sync; failure-path cleanup; save-then-reread (303 redirect re-reads from disk; JSON re-reads and returns updated record); exact, case-sensitive ID resolution against active directory listing; Context::ensure_within on every read/write path (record directory, record file, temporary file); unknown-section preservation content-identically; original section order; first heading "# <ID> <title>"; single trailing LF.
5. Verify Context accuracy: the claim that the CLI and browser can write the same files (compare CLI write path in src/requirement.rs with browser write path); constraints attributed to NFR-04.001 (text-file persistence) and NFR-04.002 (project-directory boundary).
6. Verify Consequences and residual risks against reality; list any risk not recorded (per-record config re-reads on write; duplicate record parsing between read and write paths; temporary-file leftovers after crash; CLI write-path non-atomicity and possible data loss relative to browser edits).
7. Verify module references and boundaries: src/browser/records.rs, src/browser/mod.rs, src/context.rs; tests/browser_cli.rs.
8. Review each checkpoint follow-up and decide fold into ADR or explicitly defer: per-record config re-reads; duplicate record parsing; test temp-directory hygiene for write tests; manpage/CLI write-path claims overlapping record writes.
9. Record the decision (accepted / rejected / amend) in the ticket with rationale and evidence, using a claim-by-claim evidence table (ADR section | code/spec/test reference | match/discrepancy).
10. If accepted or amended: update ADR-0006 Status and content, then regenerate docs/architecture/adr/index.html per the architecture-decision-record and adr-index-visualization skills; record the changed files.
11. Capture follow-ups as ticket notes or new tickets (not implemented here), with owners implied by the affected area.
12. Obtain explicit user confirmation of the acceptance decision through the orchestrator and record it in the ticket; do not close the ticket without it.
13. Final verification: ADR status/content matches the recorded decision; index regenerated and consistent; no changes outside ADR files and the ADR index.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
# TASK-29 Review of ADR-0006 (optimistic concurrency and atomic record writes)

## Decision recommendation

AMEND (amendments applied). Core decision (optimistic concurrency via metadata version token + atomic same-directory rename) is sound and matches delivered m-1 code. Amendments applied to ADR-0006 while it remains Proposed: scope note clarifying CLI writes are a separate non-atomic path; residual risks for crash leftovers, CLI non-atomicity, duplicate parsing, per-request config re-reads; references to src/requirement.rs. Final acceptance (Proposed -> Accepted) remains pending explicit user confirmation through the orchestrator; no user confirmation recorded here.

## Evidence table (ADR section | code/spec/test reference | result)

1. Version token "<mtime-seconds>.<mtime-nanos>.<length>" - src/browser/records.rs version_token (format!("{}.{}.{}", secs, subsec_nanos, len)); valid_version_token; tests version_token_shape - match
2. Missing/malformed token -> 400 - form_post (valid_version_token check -> HttpError 400); api_post (same); tests t21 - match
3. Mismatched token -> 409, no write - form_post version != record.version -> 409 HTML, no write_atomic; api_post -> json_error 409; tests t17, t21 - match
4. Matching token proceeds - form_post/api_post call write_atomic after checks - match
5. create_new temp file with numeric suffix retry - write_atomic loop suffix 0..1000, "<file>.tmp" then "<file>.tmp.<n>", create_new(true); tests t18 - match
6. sync_all - handle.write_all + handle.sync_all; tests t18 (indirect) - match
7. Permission copy - fs::set_permissions(&temporary, original_permissions); tests atomic_write_preserves_permissions - match
8. rename - fs::rename(&temporary, &safe_target); tests t18 - match
9. Best-effort directory sync - if let Ok(dh)=fs::File::open(&directory){let _=dh.sync_all();} - match
10. Failure-path cleanup (handled) - if result.is_err(){let _=fs::remove_file(&temporary);} - match (handled paths only; crash leaves temp - now a documented risk)
11. Save-then-reread form - form_post returns Response::redirect("/doc/{id}") (303); GET re-reads disk; tests t13 - match
12. Save-then-reread JSON - api_post resolve_record after write, returns updated record; tests t21 - match
13. Exact, case-sensitive ID resolution - resolve_id compares path.file_stem() == Some(id); tests t19, t11 - match
14. Context::ensure_within on record dir/file/temp - active_dir, load_record_from_path, resolve_id, write_atomic all call ensure_within - match
15. Unknown-section preservation content-identically - parse_doc keeps all ## sections; apply_edit clones and only edits known sections; tests t14 - match
16. Original section order - apply_edit preserves order; set_section inserts known sections by known_order; tests t14 - match
17. First heading "# <ID> <title>" - apply_edit lines.push(format!("# {} {}", id, title)); tests t14 - match
18. Single trailing LF - apply_edit pops trailing empties, joins "\n", push '\n'; tests parse_and_preserve_unknown_sections - match
19. Context accuracy: CLI and browser write same files - Both target same active dirs; CLI uses fs::write (non-atomic); location claim holds, preservation/atomicity differ (amended) - partial (amended)
20. NFR-04.001 text-file persistence - Writes .md UTF-8 text files only - match
21. NFR-04.002 project boundary - ensure_within on all paths; symlink escape -> 404; tests t19 - match
22. NFR-02.002 input sanitization - valid_id, validate_title, validate_multiline, percent-decode, ensure_within - match
23. Check-then-rename race stated honestly - ADR decision + consequences + spec 7.12.12 state it - match

## Discrepancies

1. CLI write path (src/requirement.rs) is non-atomic and does not preserve unknown sections or participate in optimistic concurrency, while ADR-0006's preservation/atomicity contract is browser-only. ADR amended with scope note; full alignment deferred (follow-up).
2. Crash between create_new and rename can leave <id>.md.tmp* files; ADR now documents this residual risk (handled-failure cleanup was already documented; crash case now explicit).
3. Duplicate record parsers (records::parse_doc vs requirement::parse_document) plus apply_edit; drift risk now documented.
4. Per-request config re-reads (browser-requirements.md 3.18) are expected; now recorded as non-defect.
5. ADR's "remove the temporary file on every failure path" tightened to "every handled failure path" for honesty.

## Follow-ups (captured, not implemented)

- FU-1: Align CLI record writes (src/requirement.rs create/create_group/edit/deprecate/restore) with ADR-0006: atomic write, sync_all, permission copy, version token, section preservation. Owner: backend-dev/rust-dev.
- FU-2: Single parser/serializer for records shared between browser and CLI (records::parse_doc vs requirement::parse_document vs apply_edit). Owner: rust-dev.
- FU-3: Test temp-directory hygiene for write tests: tests/browser_cli.rs and src/browser/records unit tests create .tmp dirs under CARGO_MANIFEST_DIR/.tmp; ensure cleanup on panic and no leftover .tmp files. Owner: quality-gatekeeper/test author.
- FU-4: Manpage/CLI write-path documentation: clarify that CLI writes are non-atomic and not version-checked, overlapping record writes with browser; update papyrus(1)/papyrus-config(5) claims if needed. Owner: tech-writer.
- FU-5: Optional startup sweep to remove stale <id>.md.tmp* files in record directories. Owner: rust-dev (deferred).
- FU-6: Final acceptance of ADR-0006 (Proposed -> Accepted) pending explicit user confirmation through orchestrator. Do NOT claim confirmation occurred.

## User confirmation status

NOT recorded. AC #5 (explicit user acceptance through orchestrator) is not satisfied by this review; orchestrator must obtain and record it. ADR-0006 remains Proposed.

User explicitly confirmed acceptance of ADR-0006 (amended) through the orchestrator on 2026-11-04 (TASK-29 approval). ADR-0006 Status flipped Proposed -> Accepted; index regenerated (6 records). AC #5 and DoD #5 satisfied.
<!-- SECTION:NOTES:END -->
