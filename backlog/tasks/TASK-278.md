---
id: TASK-278
title: "Re-run the benchmark suite and record NFR-007.001 and NFR-007.002 results"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-20T17:43:20Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-275
  - TASK-277
ordinal: 80
documentation: []
type: implementation
---

## Description
Context: the milestone exit is numeric, so the measurement must be produced the same way as the baseline.

Scope: run the MS-032 harness on the recorded reference hardware at 100 and 1,000 records, with warm-up and 100 measured requests, and record p50 and p95 for inventory, view, and edit. Compare against the baseline and against the NFR thresholds. Also record second-load bytes transferred to confirm vendor assets are not re-downloaded and app assets revalidate with 304. Keep keep-alive as is unless the measurement shows a need; if it does, record the evidence and return to the planner rather than changing it silently.
