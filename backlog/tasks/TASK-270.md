---
id: TASK-270
title: "Verify MS-035 browser engine: route and semantic equivalence"
created_date: '2026-11-20T17:42:52Z'
updated_date: '2026-11-22T20:12:09Z'
labels: []
references:
  - REQ-006.014
  - REQ-006.019
  - ADR-039
  - MS-035
  - ADR-043
  - TASK-309
  - AUD-144
  - TC-183
  - TC-184
  - TECH-040
  - ADR-042
  - TASK-288
  - MS-037
status: Done
assignee:
  - quality-gatekeeper
milestone: MS-035
dependencies:
  - TASK-269
  - TASK-312
ordinal: 90
documentation:
  - tests/fixtures/characterization/corpus
  - tests/characterization_replay.rs
  - tests/support/characterization.rs
  - tests/document_type_no_drift.rs
  - docs/specifications/technical/TECH-068.md
  - scripts/build-package.sh
  - scripts/build-deb.sh
  - tests/ui_spec_cli.rs
  - tests/fixtures/characterization/corpus/EXPECTED-DELTAS.md
type: milestone-verification
---

## Description
## Context
Milestone boundary verification for MS-035. The milestone goal is one descriptor-driven document list, view, edit, conflict, and not-found surface plus seed-declared presentation, with Backlog and Decision surfaces unchanged. TASK-265 to TASK-268 implement the surface; TASK-311 (269a) implements the generic document JSON projections and preserves the MS-032 payload bytes; TASK-269 (269b) removes the remaining per-type HTML branches, the eight document `edit::*_ops` builders, and the document `TypedHandler` variants and proves semantic equivalence. TASK-309 creates the packaging entry point `scripts/build-package.sh` required by ADR-043. This task verifies the milestone; it does not implement, repair, or change scope.

Prerequisites: TASK-269 and TASK-309 are Done; the architecture was settled by ADR-039 and the retired per-milestone architecture task; the UI design task is waived by the owner and archived as TASK-263; TECH-068 is verified; the MS-032 characterization corpus and TC-183 and TC-184 are approved. The MS-032 corpus expectations are strict for record files, JSON payloads, and status codes, and semantic (not byte-identical) for rendered document HTML; Backlog and Decision HTML stays byte-identical (MS-035 AC-2 and AC-8). MS-035 is an internal checkpoint: no RC version is determined and no package is retained; the verification build is compiled and discarded, and any `scripts/build-package.sh` exercise is discarded. The single retained release package is produced at the MS-037 release boundary by TASK-288 through the same script.

## Scope
- Verify MS-035 AC-1 to AC-8 explicitly and record an AC-by-AC verdict with the command or artifact that proves it. Verify AC-9 as the internal-checkpoint packaging rule.
- AC-1: enumerate the document renderers and prove exactly one conflict builder and one not-found builder remain; prove the nine conflict renderers and five not-found pages are gone, the conflict page is reachable only through a 409 submit, and the not-found page is the same 404 page for an unknown ID, an archived record, and a disabled type with no ID echo.
- AC-2: run the MS-032 characterization replay. Record files, JSON payloads, stdout, stderr, exit codes, asset bodies, HTTP status codes, and headers are strict. Document list, view, edit, conflict, and not-found HTML is compared semantically for selectors, text, navigation, form fields, links, and status codes. Only differences attributed to the uniform title block, seed presentation, or the MS-032 expected delta 6 (conflict and not-found collapse) are accepted; no other difference passes.
- AC-3: verify `GroupedMarkdown` group and member records render their role field sets and the read-only member list or group link; `SingleHtmlMockup` renders its fixed `status`, `task`, `specs`, `refs`, `references`, and `superseded_by` metadata; `SingleMarkdown` renders its single field set. Verify membership and the group link are never form controls.
- AC-4: verify every document label, list heading, count noun, empty state, inventory note, page title, navigation label, API alias, `?type=` legacy key, form label, help text, view section heading, and required marker equals the descriptor seed value, and no document entry remains in `PRESENTATION`.
- AC-5: verify reference visibility resolves from the seed per type and role; `req` and `nfr` are `Hidden` for both roles and render no reference list, every other seed type is `Visible`, and a user-defined type defaults to `Visible`; labels still render; the single-document view keeps its References disclosure; CLI, JSON, and stored front matter are unchanged.
- AC-6: verify one `form_post` serves every document type and both grouped roles with the fixed order and the status-code surface 415, 400, 404, 409, 422, and 303 plus 500; a stale token returns 409 with the submitted values and the live token and writes nothing; a validation failure returns 422 and writes nothing; a successful save answers 303 and is proven by a fresh read.
- AC-7: verify zero document `TypedHandler` variants remain, no document slug dispatch exists outside the seed and its tests, the eight document ops builders are deleted, and the no-drift and no-descriptor-leak guards pass.
- AC-8: verify the Backlog and Decision browser output is byte-identical to the MS-032 corpus, including the board, lists, views, edits, conflicts, and 404s, and that their templates and dispatch are unchanged.
- AC-9: after every quality and code check passes, verify that the milestone is treated as an internal checkpoint: no RC version is determined with `next-version`, no package is published or retained under `builds/` or elsewhere, and the verification build is compiled and discarded, in line with MS-032, MS-033, and MS-034 (AUD-138, AUD-140) and MS-037 AC-6. Confirm TASK-309 is Done and that `scripts/build-package.sh` exists as the ADR-043 entry point; it may be exercised (for example with `--version 0.0.0-verify --out ./.tmp/...`) but the produced artifact is discarded and is not a release artifact. Record the release boundary: MS-037 TASK-288 uses the same script to produce the single retained release package.
- Verify the full suite, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, the no-drift and no-descriptor-leak guards, and the absence of new crates, assets, templates, routes, or client scripts.
- Record every verdict as an evidence comment on this task. If a criterion fails, do not repair it here; report the failing criterion, the command, and the artifact to the orchestrator. Do not edit the milestone, requirements, specifications, or source.

## Grounding
MS-035 AC-1 to AC-9; MS-037 AC-6; ADR-039 (especially decisions 1, 10-13, 17, 22-28, and 35); ADR-042; ADR-043 (entry point, single-app resolution, and the internal-checkpoint rule); TECH-068; TECH-040 section 7; TC-183; TC-184; MS-032 `EXPECTED-DELTAS.md`; `tests/fixtures/characterization/corpus/`; `tests/characterization_replay.rs`; `tests/support/characterization.rs`; `tests/document_type_no_drift.rs`; `tests/ui_spec_cli.rs`; `src/browser/{mod,records,render,list,routes,shell,assets}.rs`; `src/record/edit.rs`; `src/registry.rs`; `scripts/build-package.sh`; `scripts/build-deb.sh`; TASK-309; TASK-288.

## Definition of done
Every MS-035 acceptance criterion has a recorded pass verdict with a reproducible command or artifact; the semantic replay shows only attributed differences; the Backlog and Decision corpus is byte-identical; the full suite, clippy, fmt, and no-drift guards are green; the internal-checkpoint rule is recorded (no RC version, no retained package, discarded verification build); TASK-309 is Done and `scripts/build-package.sh` may be exercised with its artifact discarded; the release boundary MS-037 TASK-288 is recorded as using the same script; and this task is not Done until those records are present.

## Acceptance Criteria
- [ ] #1 MS-035 AC-1 passes: exactly one document conflict builder and one document not-found builder remain; the nine conflict renderers and five not-found pages are gone; a stale-token submit returns 409 with the one conflict page; an unknown ID, an archived record, and a disabled type all return the same 404 page with no ID echo; backlog and decision 404 behavior is unchanged.
- [ ] #2 MS-035 AC-2 passes: the MS-032 corpus replay compares record files, JSON payloads, stdout, stderr, exit codes, asset bodies, HTTP status codes, and headers strictly, and compares document list, view, edit, conflict, and not-found HTML semantically for selectors, visible text, navigation, form fields, links, and status codes; only differences attributed to the uniform title block, seed presentation, or expected delta 6 are accepted, and no other difference passes.
- [ ] #3 MS-035 AC-3 passes: GroupedMarkdown group and member records render their role field sets and the read-only member list or group link; SingleHtmlMockup renders the fixed status, task, specs, refs, references, and superseded_by metadata; SingleMarkdown renders its single field set; membership and the group link are never form controls.
- [ ] #4 MS-035 AC-4 passes: every document label, list heading, count noun, empty state, inventory note, page title, navigation label, API alias, ?type= legacy key, form label, help text, view section heading, and required marker equals the descriptor seed value; no document entry remains in PRESENTATION and no slug fallback supplies a string.
- [ ] #5 MS-035 AC-5 passes: reference visibility resolves from the seed per type and role; req and nfr are Hidden for both roles with no reference list, every other seed type is Visible, and a user-defined type defaults to Visible; labels still render; the single-document view keeps its References disclosure; CLI, JSON, and stored front matter are unchanged.
- [ ] #6 MS-035 AC-6 passes: one form_post serves every document type and both grouped roles; the status-code surface is 415, 400, 404, 409, 422, 303, and 500; a stale token returns 409 with the submitted values and live token and writes nothing; a validation failure returns 422 and writes nothing; a successful save answers 303 and a fresh read proves the save.
- [ ] #7 MS-035 AC-7 passes: zero document TypedHandler variants remain; no document slug dispatch exists outside the seed and its tests; the eight document ops builders are deleted; the no-drift and no-descriptor-leak guards pass; document render and JSON builders read only the envelope and the field map.
- [ ] #8 MS-035 AC-8 passes: the Backlog and Decision browser output is byte-identical to the MS-032 corpus for the board, lists, views, edits, conflicts, and 404s, and their templates and dispatch are unchanged.
- [ ] #9 MS-035 AC-9 passes as an internal checkpoint after every quality and code check passes: no RC version is determined with next-version, no package is published or retained under builds/ or elsewhere, and the verification build is compiled and discarded, in line with MS-032, MS-033, and MS-034 and MS-037 AC-6.
- [ ] #10 TASK-309 is Done and scripts/build-package.sh exists as the ADR-043 entry point; it may be exercised with --version <version> --out <dir>, and any produced artifact is discarded and is not a release artifact; the evidence records the command and the discard, or records why the script was not exercised. The release boundary (MS-037 TASK-288) uses the same script to produce the single retained release package.
- [ ] #11 This task is not Done until the internal-checkpoint rule (no RC version, no retained package, discarded verification build), the TASK-309 completion, and the release-boundary note are recorded.
- [ ] #12 Every criterion has an AC-by-AC verdict recorded as an evidence comment with a reproducible command or artifact; the full suite, cargo clippy --all-targets -- -D warnings, cargo fmt --check, and the no-drift and no-descriptor-leak guards pass; no new crate, asset, template, route, or client script is present; any failing criterion is reported to the orchestrator and not repaired in this task.

## Comments
- **task-refiner**: Refinement note: MS-035 AC-9 still uses plural per-component packaging wording; this task applies ADR-043 decision 6 (single-app root scripts build-package.sh). The wording is escalated to the planner in TASK-264; do not edit the milestone. TASK-309 must be Done before the package step.
- **planner**: planner: MS-035 packaging policy aligned to the internal checkpoint used by MS-032, MS-033, and MS-034. AC-9 to AC-11 now require no RC version and no retained package; the verification build is compiled and discarded. TASK-309 still creates the packaging entry point and it may be exercised, with the artifact discarded. The release boundary TASK-288 in MS-037 uses the same script to produce the single retained release package. Dependencies remain TASK-269 (269b, the last of the split) and TASK-309.
- **orchestrator**: Verification PASS (AUD-147): AC-1 to AC-9 met, 0 blocker, 0 high, 0 medium, 3 low (decision-page corpus coverage, payload count 94, tracked root deb at the release boundary); code gate clear via AUD-146; 1355 tests pass; verification package discarded.
