---
id: TASK-241
title: "Implement the benchmark harness and record pre-refactor baselines at 100 and 1,000 records"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-20T17:40:57Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - TECH-069
status: ToDo
assignee:
  - rust-dev
milestone: MS-032
dependencies:
  - TASK-240
ordinal: 40
documentation: []
type: implementation
---

## Description
Context: current measurements are inventory 14.2 s and document view 1.8 s on 382 active records, far outside the NFR targets. A reproducible harness is needed before and after the change.

Scope:
- Generate corpora at 100 and 1,000 active records, including one GroupedMarkdown type with 200 members and one SingleHtmlMockup type with 50 records at the 1,000-record size.
- Measure inventory, document view, and document edit: 10 warm-up requests, 100 measured requests, p50 and p95, server-observed latency plus wall time.
- Record reference hardware, build profile, command line, and corpus hash with every result.
- Reproduce and record the 14.2 s and 1.8 s figures at 382 records so the improvement is comparable.
- The harness must run headless and must not depend on a browser.
Do not change production behavior. Do not start until TASK-239 is Done.
