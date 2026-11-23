---
id: TASK-185
title: "Update browser tests for uniform single-document layout"
created_date: '2026-11-18T19:34:31Z'
updated_date: '2026-11-19T10:53:59Z'
labels: []
references:
  - REQ-006.012
  - RE-004
  - AUD-051
  - AUD-092
  - TC-024
  - TC-025
  - TC-026
  - TC-027
  - TC-028
  - TC-029
  - TC-030
  - TC-031
  - TC-032
status: Done
assignee:
  - test-manager
milestone: MS-026
dependencies:
  - TASK-168
ordinal: "52"
documentation:
  - tests/browser_cli.rs
  - tests/browser_config_types_cli.rs
  - tests/browser_shell_cli.rs
type: implementation
---

## Description
Correction task for AUD-051 and AUD-092 (MS-026): update stale assertions for removed document-type label and moved metadata rows; add characterization coverage for TC-024 through TC-032.

## Comments
- **test-manager**: Implemented automated coverage for TC-024..TC-032 in tests/browser_cli.rs (tc024 metadata order, tc025 all managed Markdown types, tc026 empty sections omitted, tc027 native collapsed details, tc028 no-JS structural, tc029 mockup exclusion, tc030 Firefox 115 structural, tc031 view/edit round trip, tc032 resolved/unresolved references). Verified stale tests t114_11 and browser_shell_cli detail_and_edit_field_sets already updated and passing. cargo test --test browser_cli: 185 passed; browser_config_types_cli and browser_shell_cli: all passed. Coverage: REQ-006.012, NFR-005.001, NFR-001.002, BR-004. No coverage gaps. Test cases remain approved.
