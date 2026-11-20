---
id: TASK-282
title: "Remove the dead descriptor, sanitizer, and per-type test artifacts"
created_date: '2026-11-20T17:43:51Z'
updated_date: '2026-11-20T17:43:51Z'
labels: []
references:
  - ADR-036
  - TECH-072
  - TECH-066
status: ToDo
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-281
ordinal: 30
documentation: []
type: implementation
---

## Description
Context: the refactor leaves dead helpers behind; keeping them hides regressions and inflates the binary.

Scope: remove all_descriptors, list_descriptors, is_managed_record, Record::is_html_mockup, apply_patches, read_store, to_text, build_raw, format_scalar_pub, canonical_key, matches_family_id, IdentitySpec::validate_input, sanitize_reference, sanitize_documentation, sanitize_path_value, edit_page_test_case, edit_page_test_report, conflict_page_test_case, and conflict_page_test_report. For each, confirm no live caller exists before removal and record the replacement path. Do not remove anything still used by Backlog or Decision handling.
