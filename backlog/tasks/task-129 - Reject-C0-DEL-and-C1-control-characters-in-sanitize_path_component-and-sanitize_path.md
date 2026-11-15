---
id: TASK-129
title: >-
  Reject C0, DEL, and C1 control characters in sanitize_path_component and
  sanitize_path
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-15T11:29:00Z'
labels:
  - rust
  - tests
milestone: m-19
dependencies: []
references:
  - NFR-02.001
  - NFR-02.002
documentation:
  - src/sanitize.rs
modified_files:
  - src/sanitize.rs
priority: low
ordinal: 41340
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** `sanitize_name` and `sanitize_path_string` reject every Unicode control character through `has_control_character` (C0, DEL, and C1), but `sanitize_path_component` only rejects `\0` and `sanitize_path` inherits that gap per component. A path component carrying a newline, escape, or a C1 byte is therefore accepted where a name with the same character is rejected. TASK-123 fixed the title path but left these two functions inconsistent, so the same input class behaves differently depending on which entry point receives it.

**What.** Apply the same control-character predicate to `sanitize_path_component`, and therefore to every component of `sanitize_path`, without changing behavior for legitimate paths.

**In scope.** `src/sanitize.rs`: control-character rejection in `sanitize_path_component`; `sanitize_path` inherits it; unit tests for both functions covering C0, DEL, and C1 boundaries, nested and absolute paths, and regression cases for legitimate Unicode, space, dot, hyphen, and underscore segments.

**Out of scope.** `sanitize_name` and `sanitize_path_string` behavior (already correct), error messages, exit codes, and normalization or escaping.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 sanitize_path_component returns None for any component containing C0 (U+0000-U+001F), DEL (U+007F), or C1 (U+0080-U+009F), using the same predicate as sanitize_name
- [ ] #2 sanitize_path returns None when any component contains such a character, including nested and absolute paths, and never returns a partially sanitized path
- [ ] #3 legitimate paths are unchanged: Unicode letters, spaces, dots, hyphens, underscores, CurDir components, and absolute paths still sanitize to the same result
- [ ] #4 tests cover the boundary characters of all three ranges for both functions plus a nested path with one contaminated component
- [ ] #5 sanitize_name and sanitize_path_string behavior and all existing sanitizer tests are unchanged
<!-- AC:END -->
