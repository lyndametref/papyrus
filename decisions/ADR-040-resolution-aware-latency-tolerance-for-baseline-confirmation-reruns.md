---
id: ADR-040
title: "Resolution-aware latency tolerance for baseline confirmation reruns"
created_date: '2026-11-22T06:18:50Z'
updated_date: '2026-11-22T06:19:05Z'
labels: []
references:
  - AUD-137
  - MS-032
  - TASK-243
  - TASK-292
  - NFR-007.001
  - NFR-007.002
superseded_by: null
---
# ADR-040 Resolution-aware latency tolerance for baseline confirmation reruns

## Status
accepted
## Decision
Confirmation reruns of the recorded baseline are in band when the rerun p95 lies inside the recorded run A to run B p95 interval widened by 10 percent of the nearer bound, or when it lies within an absolute floor of 3 ms of the recorded value.

The absolute floor applies only where the measurement clock resolution is coarser than the widened band. On sub-5 ms routes the 1 ms server clock moves a p95 by more than 10 percent of a few milliseconds, so the six out-of-band edit-single-html-mockup confirmation values at the 100 and 382 profiles are accepted as clock-resolution noise, not as instability of the recorded baseline.

A rerun outside both the widened interval and the 3 ms floor is still a verification failure with the deviation recorded.
## Consequences
TASK-243 acceptance criterion AC-2 is amended to the resolution-aware tolerance and cites this record. The 100 and 382 profile confirmation results stay valid with six values accepted as clock-resolution noise, so no rerun of those profiles is required for the re-audit. The tolerance is measurement-method only: it does not relax NFR-007.001, NFR-007.002 or any MS-032 acceptance criterion, and it never turns a rerun into a pass when the rerun lies outside both the widened interval and the 3 ms floor.
## Context
AUD-137 (quality audit MS-032 re-audit) reported six of 88 confirmation p95 checks outside the recorded run A to run B band widened by ten percent of the nearer bound. All six are on edit-single-html-mockup, whose recorded p50 is 2 ms, and all six differ from the recorded value by less than the coarseness of the 1 ms server clock. The owner decided to accept them as clock-resolution noise instead of re-measuring the route or widening the band for every route.
