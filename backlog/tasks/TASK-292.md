---
id: TASK-292
title: "Correct the MS-032 baseline: complete route coverage, variance record, and characterization gaps"
created_date: '2026-11-22T01:32:21Z'
updated_date: '2026-11-22T01:32:21Z'
labels: []
references:
  - AUD-136
  - MS-032
  - TASK-241
  - NFR-007.001
  - NFR-007.002
status: ToDo
assignee:
  - rust-dev
milestone: MS-032
dependencies:
  - TASK-241
  - TASK-242
ordinal: 55
documentation:
  - docs/benchmarks
  - docs/audits/AUD-136.md
type: implementation
---

## Description
Context: AUD-136 (quality audit MS-032) returned 7 of 9 acceptance criteria passing, with AC-1 and AC-2 failing and six corrections required (C-1 to C-6). The owner has decided the scope: fix the audit findings rather than amend the milestone acceptance criteria. This task groups the corrections so MS-032 can be re-verified by TASK-243 with one evidence set.

Scope:

C-1 (high, F-1): measure the six routes omitted at the 1000-record profile with band B (2 warm-up + 20 measured, one run): view-grouped-group, view-unresolved-reference, edit-single-markdown, edit-grouped-group, edit-single-html-mockup, edit-unresolved-reference. Regenerate the summary and the raw JSON; no extrapolation; record the new sample counts.

C-2 (high, F-2, approved exemption): record two no-write confirmation reruns for the 100 and 382 profiles and a p95 variance comparison against the recorded A-B range widened by 10 percent of the nearer bound. For the 1000 profile record an explicit owner-approved exemption from the duplicate run, with the reason (pre-refactor latency, about 75 s per inventory request), and state that the 1000 figures are single-run baseline-only.

C-3 (medium, F-3): add the missing characterization cases: conflict POSTs for nfr, glossary and ui-spec; representative POST to the api doc route (success and 409); form-POST 303 and 422; 404 cases for the seven document types that lack one. Regenerate through the capture path, replay, and update the case and artifact counts.

C-4 (low, F-4): fix the wall-delta unit label in the baseline summary (microseconds versus milliseconds).

C-5 (low, F-5): record the rationale for the TASK-241 test-infrastructure changes (empty-fixture invariant repair and version-token normalisation) in the task evidence, and add a guard test asserting the pinned project-empty fixture contains no record files.

C-6 (low, F-6): record both the measured production tree commit and the harness commit in the baseline summary and the raw JSON.

Out of scope: changing production behaviour, changing test expectations to make the suite pass, amending the MS-032 acceptance criteria, re-issuing or archiving AUD-136, and any work belonging to MS-035 or MS-036.

Definition of done: every correction C-1 to C-6 has a recorded result in the task evidence, the regenerated benchmark and characterization numbers are reproducible by the documented commands, the suite is green with no production behaviour change, and the evidence answers AUD-136 point by point so TASK-243 can re-verify AC-1 and AC-2.

## Acceptance Criteria
- [ ] C-1 (high): the six routes omitted at the 1000-record profile (view-grouped-group, view-unresolved-reference, edit-single-markdown, edit-grouped-group, edit-single-html-mockup, edit-unresolved-reference) are measured with band B (2 warm-up + 20 measured, one run); the summary and raw JSON are regenerated with no extrapolation and the new sample counts recorded.
- [ ] C-2 (high): two no-write confirmation reruns are recorded for the 100 and 382 profiles with a p95 variance comparison against the recorded A-B range widened by 10 percent of the nearer bound; for the 1000 profile an explicit owner-approved exemption from the duplicate run is recorded with the stated reason (pre-refactor latency, about 75 s per inventory request) and the 1000 figures are stated to be single-run baseline-only.
- [ ] C-3 (medium): the missing characterization cases are added (conflict POSTs for nfr, glossary and ui-spec; representative POST to the api doc route, success and 409; form-POST 303 and 422; 404 cases for the seven document types that lack one), regenerated through the capture path, replayed, and the case and artifact counts updated.
- [ ] C-4 (low): the wall-delta unit label in the baseline summary is corrected (microseconds vs milliseconds).
- [ ] C-5 (low): the rationale for the TASK-241 test-infrastructure changes (empty-fixture invariant repair and version-token normalisation) is recorded in the task evidence, and a guard test asserts the pinned project-empty fixture contains no record files.
- [ ] C-6 (low): both the measured production tree commit and the harness commit are recorded in the baseline summary and raw JSON.
- [ ] The regenerated numbers are reproducible by the documented commands, the suite stays green, and no production behaviour changes.
- [ ] AUD-136 is addressed point by point in the task evidence.
