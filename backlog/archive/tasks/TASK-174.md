---
id: TASK-174
title: "Implement removal of browser sections"
created_date: '2026-11-16T21:03:23Z'
updated_date: '2026-11-19T10:54:24Z'
labels: []
references:
  - REQ-006.018
  - RE-005
  - REQ-006.003
  - REQ-006.005
  - REQ-006.008
  - NFR-001.002
  - ADR-034
  - TECH-080
  - UI-018
  - MOCK-020
  - BR-003
  - BR-004
  - TC-033
  - TC-034
  - TC-035
  - TC-036
  - TC-037
  - TC-038
  - TC-039
  - AUD-107
  - AUD-108
status: Done
assignee:
  - rust-dev
milestone: MS-027
dependencies:
  - TASK-173
ordinal: 50
documentation:
  - src/browser/render.rs
type: implementation
---

## Description
Code and automated tests for removing browser sections and routes.


## Comments
- **rust-dev**: Implemented removal of browser sections. Removed routes/handlers/pages for /config, /config/edit, /lifecycle, /skills and their query aliases; removed inventory hub cards and sidebar utility entries; removed lifecycle/skills/config render functions and sidebar active-state helpers. Other browser pages and navigation remain functional. Added/updated automated tests TC-033..TC-039 in tests/browser_cli.rs. Updated obsolete tests asserting removed surfaces. Checks: cargo fmt, cargo clippy --all-targets, cargo test --lib (426 passed), cargo test --test browser_cli (181 passed) all green.
