---
id: TASK-231
title: "Finalize NFR-007.001 to NFR-007.003 performance and asset-caching requirements"
created_date: '2026-11-20T17:39:46Z'
updated_date: '2026-11-20T17:39:46Z'
labels: []
references:
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
  - ADR-037
  - ADR-038
status: ToDo
assignee:
  - requirements-engineer
milestone: MS-031
dependencies:
  - TASK-219
  - TASK-220
ordinal: 150
documentation:
  - docs/requirements/non-functional/NFR-007.001.md
  - docs/requirements/non-functional/NFR-007.002.md
  - docs/requirements/non-functional/NFR-007.003.md
type: requirements
---

## Description
Context: a draft NFR-007 group and its three NFRs were captured while planning: NFR-007.001 inventory p95 under 250 ms at 1,000 records, NFR-007.002 view and edit p95 under 100 ms at 1,000 records, NFR-007.003 asset caching contract. They are drafts and have not been user-verified.

Scope: review, correct, and take the three NFRs through user verification.
- Confirm the reference hardware statement: 4 vCPU, 8 GiB RAM, SSD, Linux, release build with LTO and strip, loopback HTTP, single server process.
- Confirm the measurement method: generated 1,000-record corpus including one GroupedMarkdown type with 200 members and one SingleHtmlMockup type with 50 records; 10 warm-up requests then 100 measured requests; p50 and p95 reported; cold start and config-reload requests excluded.
- Confirm that NFR-007.003 owns header-level assertions for immutable vendor assets, no-cache plus ETag and 304 for app.css and app.js, no-store for HTML and JSON, assets only, and no compression.
- If the project declines asset caching, replace NFR-007.003 with an explicit exclusion recorded in ADR-038 rather than leaving it unmet.
- Record the baseline figures that motivate the targets: inventory 14.2 s and a document view 1.8 s at 382 active records.
