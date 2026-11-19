---
id: TASK-184
title: "Fix uniform single-document browser layout audit findings"
created_date: '2026-11-18T19:34:28Z'
updated_date: '2026-11-19T10:53:58Z'
labels: []
references:
  - REQ-06.012
  - RE-004
  - AUD-051
  - AUD-092
  - ADR-033
  - TECH-079
  - TC-024
  - TC-026
  - TC-027
  - TC-029
  - TC-032
status: Done
assignee:
  - rust-dev
milestone: MS-026
dependencies:
  - TASK-168
ordinal: 51
documentation:
  - src/browser/render.rs
  - src/browser/mod.rs
type: implementation
---

## Description
Correction task for AUD-051 and AUD-092 (MS-026): remove duplicate status display; move Decision type and Status record-meta under content per ADR-033; remove UiMockup document-type repetition; use actual mockup references count; reduce duplicated page_head paths, mockup legacy summary card, and complexity.

## Comments
- **rust-dev**: Fixed AUD-051/AUD-092 findings: removed duplicate status display (skip status in extra rows; removed redundant Decision Status from page_head sub line); removed Decision type/Status record-meta prepend above content; removed UiMockup 'UI mockup' document-type prepend; fixed mockup refs handling; updated t114_11 and browser_shell_cli detail_and_edit_field_sets for new layout. cargo fmt, clippy --all-targets (pre-existing warning only), cargo test --lib (426 passed), cargo test --test browser_cli (176 passed), cargo test --test browser_config_types_cli (19 passed), cargo test --test browser_shell_cli (12 passed). Pre-existing full-suite failures (list_characterization_replay, no_drift) remain.
