---
id: TASK-279
title: "Verify MS-036 performance: NFR targets and cache headers"
created_date: '2026-11-20T17:43:25Z'
updated_date: '2026-11-20T17:43:25Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
status: ToDo
assignee:
  - quality-gatekeeper
milestone: MS-036
dependencies:
  - TASK-278
ordinal: 90
documentation: []
type: milestone-verification
---

## Description
Milestone boundary verification for MS-036.

Scope: check every MS-036 acceptance criterion on the recorded reference hardware. Re-run the harness independently of the implementer; confirm inventory p95 under 250 ms and view and edit p95 under 100 ms at 1,000 records; confirm no per-record config or registry construction; confirm atomic swap and fail-closed reload; confirm vendor immutable, app no-cache plus ETag and 304, and HTML and JSON no-store; confirm Mermaid is requested only on diagram pages; confirm the full suite is green and record/render behavior is unchanged.
