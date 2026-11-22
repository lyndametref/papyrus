---
id: TASK-279
title: "Verify MS-036 performance: NFR targets and cache headers"
created_date: '2026-11-20T17:43:25Z'
updated_date: '2026-11-22T21:49:15Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
  - MS-036
  - MS-037
  - ADR-043
  - TECH-072
  - TECH-040
  - TC-178
  - TC-179
  - TC-181
  - TC-182
  - TC-187
  - TC-189
  - TASK-273
  - TASK-274
  - TASK-275
  - TASK-276
  - TASK-277
  - TASK-278
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-036
dependencies:
  - TASK-278
  - TASK-313
ordinal: 90
documentation:
  - tests/benchmark_baseline.rs
  - docs/benchmarks/MS-032-baseline.md
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/browser/assets.rs
  - src/browser/http.rs
  - src/browser/routes.rs
  - tests/browser_cli.rs
  - tests/characterization_replay.rs
  - tests/document_type_no_drift.rs
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
  - decisions/ADR-038-cacheable-immutable-browser-assets.md
  - decisions/ADR-043-component-build-script-as-the-packaging-entry-point.md
  - scripts/build-package.sh
  - scripts/build-deb.sh
type: milestone-verification
---

## Description
## Context
Milestone boundary verification for MS-036. TASK-273 implements the request-scoped record snapshot, TASK-274 the mtime-invalidated serving snapshot with fail-closed reload, TASK-275 the single-scan inventory, TASK-276 the versioned immutable vendor assets, ETag revalidation, and the build-path asset-version guard, TASK-277 the lazy diagram-only Mermaid loading, and TASK-278 the full-protocol benchmark and second-load cache measurement. This task verifies the milestone; it does not implement, repair, or change scope.

Prerequisites: TASK-273 through TASK-278 are Done; ADR-037 and ADR-038 are accepted; TECH-072 is verified; TC-178, TC-179, TC-181, TC-182, TC-187, and TC-189 are approved; the architecture impact task TASK-271 is retired by owner decision and no UI design task is planned for this performance work. MS-036 is a fidelity milestone: apart from the asset cache headers fixed by ADR-038, record, render, JSON, status-code, and header behaviour must be unchanged.

Packaging (recorded waiver, 2026-11-22T20:22:22Z):
- MS-036 is an internal checkpoint. Milestone verification compiles and discards a verification build; no RC version is determined and no package is retained. The packaging entry point `scripts/build-package.sh` (ADR-043) may be exercised during verification, but any artifact it produces is discarded.
- Scope of the waiver: MS-036 only. MS-035 carried the same deviation in its own recorded internal-checkpoint rule; no other milestone is covered.
- Reason: MS-037 AC-6 and AC-7 require exactly one retained release artifact, so an RC package from an intermediate checkpoint would create a second retained artifact at the single-release boundary.
- ADR-043 exception (architect action): ADR-043 decision 21 says every verified milestone from MS-035 onward publishes an RC package. That is wrong for MS-035 and MS-036, whose verification builds are discarded; the packaging gate effectively starts at MS-037. The architect must amend ADR-043 decision 21. This task records the exception and does not edit ADR-043.

Hardware deviation (recorded decision, 2026-11-22T20:22:22Z): NFR-007.001 and NFR-007.002 name 4 vCPU and 8 GiB RAM; the available host has 6 physical / 12 logical cores and 15.3 GiB. Verification re-runs the full NFR-007 protocol on the available host and records that deviation beside every result. The figures are reported as measured values with the deviation, they do not mark NFR-007.001 or NFR-007.002 satisfied, and the release report (MS-037) presents them as a caveat for the owner.

## Scope
- Verify MS-036 AC-1 to AC-8 explicitly and record an AC-by-AC verdict with the command or artifact that proves it. Verify AC-9 as the recorded internal-checkpoint rule and waiver, not as an RC gate.
- AC-1 and AC-2: on the available host (6 physical / 12 logical cores, 15.3 GiB), independently re-run the MS-032 harness at 1,000 records with the full NFR-007 protocol of 10 warm-up and 100 measured sequential requests per route; record the host, the hardware deviation from the NFR-007 reference hardware, command, corpus hash, sample counts, and per-route p50 and p95; record a measured pass or miss against the 250 ms inventory and 100 ms view and edit thresholds for every measured document model, including SingleMarkdown, GroupedMarkdown group and member, SingleHtmlMockup, and the unresolved-reference record, without presenting any result as NFR-007.001 or NFR-007.002 satisfaction.
- AC-3: confirm one scheduler-independent inventory build uses a single registry-wide scan; confirm no per-record configuration read, YAML parse, validation, or registry construction remains on the read path; confirm each enabled directory is listed once and each distinct record parsed once per request.
- AC-4: confirm the request-scoped snapshot serves all title and reference resolutions, is not stored in server state or shared across connections, and supplies no write content and no version token; confirm every write re-reads the live record through record::store, computes the token from live bytes, writes atomically, and a stale token still returns 409.
- AC-5: confirm the configuration, registry, and route snapshot is mtime-and-length invalidated, published by one atomic swap, and fail-closed on reload error; confirm the last known good snapshot keeps serving, one warning per distinct input state is written to standard error, no new status or page appears, and startup still exits 1 for a missing configuration and 4 for an invalid one; confirm host, port, and open edits do not rebind.
- AC-6: confirm the three cache classes literally: immutable vendor header set on GET and HEAD with no ETag and never 304; first-party no-cache plus strong SHA-256 ETag and the full 304 matrix; HTML, JSON, error, 404, and 405 no-store with no validator and never 304; no Content-Encoding or Vary; unversioned vendor paths 404; non-GET and non-HEAD asset methods 405 with Allow GET, HEAD; version segment equals the vendor directory and manifest version; the build-path asset-version guard of ADR-038 decision 30 passes on the committed tree and rejects each mismatch class.
- AC-7: confirm the 3.57 MB Mermaid and 275 KB KaTeX payloads are no longer sent no-store; confirm Mermaid is requested only on pages with a renderable diagram, exactly once and with defer, and never on a page without a diagram; confirm KaTeX is present only where math exists; confirm the offline fallback and render output are unchanged.
- AC-8: confirm Connection close is unchanged and the recorded measurement did not show a need for keep-alive; if it did, confirm the evidence was returned to the planner rather than changed silently.
- Record the second-load measurement: zero vendor body bytes on the second load, first-party assets revalidated with 304 and empty bodies, and HTML and JSON re-fetched with no-store, using the diagram page, the math page, and the cache simulation recorded by TASK-278.
- Verify the full suite, cargo clippy --all-targets -- -D warnings, cargo fmt --check, the no-drift and no-descriptor-leak guards, and the absence of new crates, assets, templates, routes, or client scripts. Run the characterization replay and confirm strict equality for record files, JSON payloads, stdout, stderr, exit codes, asset bodies, status codes, and headers, and semantic equality for rendered HTML with only the documented expected deltas.
- Record every verdict as an evidence comment on this task. If a criterion fails, do not repair it here; report the failing criterion, command, and artifact to the orchestrator.

## Grounding
MS-036 AC-1 to AC-9; MS-037 AC-6 and AC-7; ADR-037 (decisions 16-32); ADR-038 (decisions 5-30); ADR-043 decisions 21 and 23; TECH-072; TECH-040; NFR-007.001 to NFR-007.003; TC-178, TC-179, TC-181, TC-182, TC-187, TC-189; docs/benchmarks/MS-032-baseline.md; tests/benchmark_baseline.rs; tests/characterization_replay.rs; tests/document_type_no_drift.rs; src/browser/mod.rs; src/browser/records.rs; src/browser/assets.rs; src/browser/http.rs; src/browser/routes.rs; scripts/build-package.sh; scripts/build-deb.sh; TASK-273 to TASK-278.

## Definition of done
Every MS-036 acceptance criterion has a recorded pass verdict with a reproducible command or artifact; the inventory, view, and edit figures are recorded on the available host with the hardware deviation beside every result, compared with the thresholds but not claimed as NFR-007.001 or NFR-007.002 satisfaction; the cache class, 304 matrix, no-store, no-compression, version-binding, and build-path guard checks pass; Mermaid is diagram-only and the render and offline behaviour is unchanged; the full suite, clippy, fmt, no-drift, and no-descriptor-leak guards are green; the recorded internal-checkpoint waiver (scope MS-036 only, reason single retained release artifact at MS-037, timestamp 2026-11-22T20:22:22Z) stands in place of the RC gate and no package is retained; the task is not Done until those records are present.
## Acceptance Criteria
- [ ] #1 After all quality and code checks pass, the milestone RC version is determined with next-version in RC mode and recorded in this verification task.
- [ ] #2 The RC package is published with packaging from the committed verified tree into the git-ignored builds directory for that version, using each deployable component's build-package script with the version and output arguments fixed in the definition of done, and the package evidence is recorded in this task: version, commit SHA, script paths and commands, component archives, paths, sizes, and sha256; an explicit user waiver with scope, reason, and timestamp with timezone is the only alternative.
- [ ] #3 This task is not Done until the RC version is recorded and the package evidence or the recorded user waiver is present.
- [ ] #4 AC-by-AC verdict: MS-036 AC-1 to AC-8 each have a recorded pass or fail verdict with a reproducible command or artifact, and this task does not repair a failure, edit the milestone, requirements, specifications, or source.
- [ ] #5 Independent performance verdict: on the recorded reference hardware the MS-032 harness is re-run at 1,000 records with the full NFR-007 protocol of 10 warm-up and 100 measured sequential requests per route; the recorded output shows 10 warm-ups and 100 measured samples for every route, and a route with fewer than 100 samples is not accepted as an NFR comparison.
- [ ] #6 Inventory target: the independently measured inventory p95 at 1,000 records is below 250 ms, with p50 and p95, host, command, corpus hash, and sample counts recorded.
- [ ] #7 View and edit target: the independently measured view and edit p95 at 1,000 records is below 100 ms for every measured document model, covering SingleMarkdown, GroupedMarkdown group, GroupedMarkdown member, SingleHtmlMockup, and the unresolved-reference record; p50 and p95 are recorded for view and for edit.
- [ ] #8 Single-scan and no per-record construction: one inventory build uses one registry-wide scan, each enabled directory is listed once and each distinct record parsed once per request, and no per-record configuration read, YAML parse, validation, or registry construction remains on the read path.
- [ ] #9 Request snapshot: the request-scoped snapshot serves all title and reference resolutions, is not held in server state or shared across connections, supplies no write bytes or version token, and every write re-reads the live record through record::store, computes the token from live bytes, writes atomically, and still returns 409 on a stale token.
- [ ] #10 Snapshot reload: the configuration, registry, and route snapshot is mtime-and-length invalidated, published by one atomic swap, and fail-closed on reload error; the last known good snapshot keeps serving, one warning per distinct input state is written to standard error, no new status, header, page, JSON member, or exit code appears, startup still exits 1 for a missing configuration and 4 for an invalid one, and host, port, and open edits do not rebind.
- [ ] #11 Immutable vendor class: every vendor entry on GET and HEAD returns the exact immutable header set with no ETag and never 304, and the version segment equals the vendor directory and the manifest version for every entry.
- [ ] #12 First-party revalidation: app.css, app.js, and nav-state.js return no-cache and one strong SHA-256 ETag, and the 304 matrix passes for exact, star, weak, stale, malformed, comma-separated, and repeated If-None-Match lines on GET and HEAD.
- [ ] #13 No-store and method matrix: HTML, JSON, error, 404, and 405 responses return no-store with no ETag and never 304; no record-derived response is cacheable; unversioned vendor paths return 404; non-GET and non-HEAD asset methods return 405 with Allow GET, HEAD before conditional handling.
- [ ] #14 No compression: no response carries Content-Encoding or Vary, no Accept-Encoding negotiation changes a response, and Transfer-Encoding remains rejected.
- [ ] #15 Mermaid and KaTeX: the 3.57 MB Mermaid and 275 KB KaTeX payloads are not sent no-store; Mermaid is requested only on a page with a renderable diagram, exactly once with defer, and never on a page without a diagram; KaTeX is present only where math exists; the offline fallback and render output are unchanged.
- [ ] #16 Second-load transfer: the recorded measurement shows zero vendor body bytes on the second load, first-party assets revalidated with 304 and empty bodies, and HTML and JSON re-fetched with no-store.
- [ ] #17 Connection behaviour: Connection close is unchanged, and the measurement either shows no need for keep-alive or the evidence was returned to the planner without a silent change.
- [ ] #18 Behaviour preservation: the full suite, cargo clippy --all-targets -- -D warnings, cargo fmt --check, the no-drift and no-descriptor-leak guards pass; the characterization replay is strict for record files, JSON payloads, stdout, stderr, exit codes, asset bodies, status codes, and headers and semantic for rendered HTML with only the documented expected deltas; no new crate, asset, template, route, or client script is present.
- [ ] #19 Packaging verdict: MS-036 AC-9 is verified either by recording the RC version determined with next-version in RC mode and the RC package evidence from the committed verified tree under the git-ignored builds directory, with version, commit SHA, script paths and commands, archives, paths, sizes, and sha256, or by recording an explicit user waiver with scope, reason, and timestamp with timezone in this task; the task is not Done until one of these is present.
- [ ] #20 Evidence: every verdict is recorded as a comment on this task; a failing criterion is reported to the orchestrator with the command and artifact and is not repaired in this task.
- [ ] #21 Internal-checkpoint packaging (supersedes AC 1, AC 2, AC 3, and AC 19): after all quality and code checks pass, milestone verification compiles and discards a verification build; no RC version is determined, no package is published or retained, and no package evidence is recorded; the recorded waiver applies with scope MS-036 only, reason that MS-037 AC-6 and AC-7 require exactly one retained release artifact, and timestamp 2026-11-22T20:22:22Z; the build-package script of ADR-043 may be exercised during verification and any artifact it produces is discarded; ADR-043 decision 21 conflicts with this record, is flagged for the architect to amend, and is not edited by this task.
- [ ] #22 Hardware deviation verdict (supersedes the reference-hardware wording of AC 5, AC 6, and AC 7): the independent re-run uses the available host of 6 physical and 12 logical cores with 15.3 GiB RAM instead of the NFR-007 reference hardware of 4 vCPU and 8 GiB; the hardware deviation is recorded beside every route result and in the run header; the verdict per route is a measured pass or miss against 250 ms for inventory and 100 ms for view and edit on that host, and no verdict states or implies that NFR-007.001 or NFR-007.002 is satisfied; the results and the deviation are handed to MS-037 for the release report and the owner decision.
## Comments
- **planner**: planner: MS-036 is an internal checkpoint (MS-036 AC-9). Verification compiles and discards a build; no RC version, no retained package, no package evidence. Recorded waiver: scope MS-036 only, reason MS-037 AC-6 and AC-7 require exactly one retained release artifact, timestamp 2026-11-22T20:22:22Z (AC #21 supersedes AC 1, AC 2, AC 3, AC 19). ADR-043 decision 21 still requires an RC from MS-035 onward; flagged for the architect to amend with the MS-035 and MS-036 exception. Hardware deviation: verify on the available 6 physical and 12 logical core, 15.3 GiB host and report measured figures with the deviation, never as NFR-007.001 or NFR-007.002 satisfaction (AC #22).
- **planner**: planner: TASK-313 (read-path optimisation, ordinal 75) added to MS-036 after the TASK-278 threshold misses; it is now a prerequisite of this verification task alongside TASK-278, so its measurements and results table must be present before verification. Prerequisite list is TASK-273 to TASK-278 plus TASK-313.
