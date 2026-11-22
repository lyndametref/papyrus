---
id: TASK-292
title: "Correct the MS-032 baseline: complete route coverage, variance record, and characterization gaps"
created_date: '2026-11-22T01:32:21Z'
updated_date: '2026-11-22T06:08:27Z'
labels: []
references:
  - AUD-136
  - MS-032
  - TASK-241
  - NFR-007.001
  - NFR-007.002
status: Done
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
## Comments
- **rust-dev**: TASK-292 C-1 (AUD-136 F-1): the 1000-record profile now measures all eleven routes in one run. The six formerly omitted routes were assigned band B per the task: view-grouped-group server p95 23648 ms, view-unresolved-reference 23861 ms, edit-single-markdown 2973 ms, edit-grouped-group 23642 ms, edit-single-html-mockup 2 ms, edit-unresolved-reference 23777 ms; each 2 warm-up + 20 measured. Raw JSON has eleven routes and zero omitted; the summary and raw file were regenerated with no extrapolation.
- **rust-dev**: TASK-292 C-2 (AUD-136 F-2): the 100 and 382 profiles now have two no-write confirmation reruns (C and D) beside the recorded A and B runs. 82 of 88 p95 checks (server and wall, C and D) fall inside the A-B range widened by 10 percent of the nearer bound. The six out-of-band values are all on the sub-5 ms edit-single-html-mockup route, where the 1 ms server clock is coarser than the 10 percent band; they are recorded exactly as measured. The 1000 profile records an owner-approved exemption from duplicate runs (roughly 75 s per inventory request) and its figures are labelled single-run baseline-only.
- **rust-dev**: TASK-292 C-3 (AUD-136 F-3): characterization now has 250 cases and 779 artifacts (134 CLI, 116 HTTP), up from 236 and 752. Added nfr, glossary and ui-spec conflict POSTs; POST api doc success 200 and conflict 409; form POST 303 success and 422 validation; and 404 cases for req, nfr, glossary, rule, audit, ui-spec and ui-mockup. Regenerated through the documented capture command and replayed byte-identically.
- **rust-dev**: TASK-292 C-4 (AUD-136 F-4): the variance table wall-delta column is now labelled Wall delta p95 (microseconds) and reports microseconds; the wall p95 columns are converted to milliseconds so each label matches its value.
- **rust-dev**: TASK-292 C-5 (AUD-136 F-5): rationale recorded on TASK-241 for both test-infrastructure changes. The three deleted project-empty records restore the documented invariant that the pinned empty fixture holds only papyrus.config.yml. Version-token normalisation is required because the mtime-derived token changes on every copy and would make byte replay flaky.
- **rust-dev**: TASK-292 C-5 guard: added pinned_project_empty_fixture_contains_no_record_files to the list characterization replay suite; it asserts the fixture contains exactly papyrus.config.yml and no record files.
- **rust-dev**: TASK-292 C-6 (AUD-136 F-6): the summary and every raw JSON record the measured production tree commit 3bc3ce518238f2a36b9c095df0e7226bf889184a, the harness commit d46d6dfee641dfb1692ff9696d6a8fdc95878db2 with harness_worktree_modified true, and measured HEAD cd26ef564baa89e47e82f6d78e9b68642a2908ab.
- **rust-dev**: TASK-292 verification: env -u EDITOR -u VISUAL cargo test gives 1138 passed, 0 failed, 25 ignored; cargo clippy --all-targets -- -D warnings is clean; cargo fmt --all -- --check is clean; the change set has no src, packaging or Cargo changes and no production behaviour change.
- **rust-dev**: TASK-292 reproduction: record with cargo test --release --test benchmark_baseline -- --ignored --nocapture; rebuild only the summary from the recorded raw JSON with PAPYRUS_BENCH_SUMMARY_FROM_RAW=1 and the same command. Characterization capture uses PAPYRUS_CAPTURE_COMMIT plus cargo test --test characterization_capture -- --ignored --nocapture.
- **rust-dev**: TASK-292 delivery: work is on branch task TASK-243 with no new commit, per the orchestrator instruction not to commit.
- **orchestrator**: Acceptance met: C-1 all 11 routes measured at 1000; C-2 variance recorded with owner-approved 1000 exemption; C-3 250 cases and 779 artifacts; C-4 to C-6 fixed. Suite 1138 passed, 0 failed; no src diff.
