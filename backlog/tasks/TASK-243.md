---
id: TASK-243
title: "Verify MS-032 baseline: reproducible benchmarks, fixtures, and frozen behavior"
created_date: '2026-11-20T17:41:02Z'
updated_date: '2026-11-22T01:32:24Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - TECH-069
  - TC-178
  - TC-179
  - TC-182
  - TC-183
  - TC-184
  - ADR-039
status: In-Progress
assignee:
  - quality-gatekeeper
milestone: MS-032
dependencies:
  - TASK-241
  - TASK-242
  - TASK-292
ordinal: 60
documentation:
  - docs/audits
type: milestone-verification
---

## Description
Context: this is the MS-032 milestone-boundary verification. MS-032 is a baseline-only milestone: it must freeze observable behaviour and performance before the refactor, and it must not change product behaviour. The verification therefore proves three things at once: the baseline artefacts are reproducible, the record of current behaviour is complete, and production behaviour is untouched.

Inputs: MS-032 acceptance criteria AC-1 to AC-7; the benchmark harness and results from TASK-241 under `docs/benchmarks/`; the characterization corpus, expected-delta list and CLI help goldens from TASK-240 under `tests/fixtures/characterization/corpus/` and `tests/golden/cli-help/`; the renderer and route audit from TASK-242 under `docs/audits/`; NFR-007.001, NFR-007.002 and NFR-007.003; TC-178, TC-179, TC-182, TC-183 and TC-184.

Scope:

1. Check every MS-032 acceptance criterion and record pass or fail with an evidence pointer. Record the outcome in a quality audit record created with `papyrus audit create --audit-type quality --reference MS-032 --reference TASK-243`, stored under `docs/audits/`, and in a task comment. This mirrors the MS-031 exit record AUD-134.

2. Reproducibility of the benchmark (AC-1, AC-4): rerun the harness twice per recorded profile on the recorded host and compare each rerun p95 with the recorded run-to-run variance. The variance band for a route is the recorded run A and run B p95 range widened by ten percent of the nearer bound. A rerun outside the band is a failure with the deviation recorded. Confirm the 382-record measurement and the transcribed 14.2 s and 1.8 s figures are both present with provenance.

3. Reproducibility of the fixtures (AC-2, AC-6): rerun the capture command and confirm `git status --porcelain` is empty for the corpus and CLI-help fixture trees, and that the recomputed corpus hash equals the recorded hash. Confirm the replay gate passes from a clean checkout and that its negative self-check still fails on a one-byte change.

4. Completeness of the record (AC-2, AC-3, AC-5): confirm the audit record lists nine conflict-page renderers and five not-found pages with file and line and that a fresh re-enumeration matches; confirm the CLI help golden set has a file for every document command and every config subcommand and that the golden set completeness check passes.

5. No behaviour change (AC-7): take the MS-032 start commit recorded by the verifier, run `git diff --name-only <start>..HEAD` and confirm no file under `src/`, `packaging/` or `scripts/` and no dependency or release-profile change in `Cargo.toml`. Any such change is a failure unless the user has accepted it in writing.

6. Suite green (AC-7): run the full `cargo test` suite, record the pass and fail counts, and record the test count as information only. Record the run date with timezone, the rustc and cargo versions, and the host description.

Out of scope: fixing findings, changing fixtures, benchmarks, audits or source, and redefining the acceptance criteria. A failed criterion is recorded with a finding and left open.

Definition of done: the audit record covers AC-1 to AC-7 with evidence pointers, both rerun sets are recorded, fixture regeneration is proven byte-identical, the nine-and-five audit and the help golden set are confirmed, the no-change diff proof is recorded, the suite result is recorded, and the task comment summarises the outcome.

## Acceptance Criteria

- [ ] Each MS-032 acceptance criterion AC-1 to AC-7 is checked and recorded as pass or fail with an evidence pointer in a quality audit record that references MS-032 and TASK-243, and the task carries a summary comment.
- [ ] The benchmark harness is rerun twice per recorded profile on the recorded host, and every rerun p95 falls inside the recorded variance band widened by ten percent of the nearer bound; a value outside the band fails verification with the deviation recorded (MS-032 AC-1).
- [ ] The 382-record measurement and the historical 14.2 s inventory and 1.8 s view figures both appear in the benchmark summary document with provenance (MS-032 AC-4).
- [ ] Fixtures regenerate byte-identically: a capture rerun leaves the corpus and CLI-help fixture trees unchanged in git status, and the recomputed corpus hash matches the recorded hash (MS-032 AC-2, AC-6).
- [ ] The replay gate passes from a clean checkout and its negative self-check still fails on a one-byte change.
- [ ] The audit record lists nine conflict-page renderers and five not-found pages with file and line, and a fresh re-enumeration matches, so MS-035 can use the record as its deletion checklist (MS-032 AC-3).
- [ ] The CLI help snapshot has a file for every document command and every config subcommand, and the golden set completeness check passes (MS-032 AC-5).
- [ ] The MS-032 diff from the recorded start commit touches no file under the source, packaging or scripts directories and changes no dependency or release-profile entry in the manifest (MS-032 AC-7).
- [ ] The full test suite passes with zero failures and the run date, rustc version and host are recorded; the test count is recorded as information only (MS-032 AC-7).
- [ ] Verification closes only when AC-1 to AC-7 all have a pass; any fail is recorded with a finding and the affected criterion is left open.

## Comments
- **task-refiner**: Refined by task-refiner for MS-032: description grounded in the current routes, renderers, harness conventions and root-cause call sites; acceptance criteria added; references and documentation set. No dependency, ordinal or milestone change. Status left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done].
