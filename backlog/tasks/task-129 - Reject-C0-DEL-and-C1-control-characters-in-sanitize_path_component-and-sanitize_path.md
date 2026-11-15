---
id: TASK-129
title: >-
  Reject C0, DEL, and C1 control characters in sanitize_path_component and
  sanitize_path
status: Done
assignee:
  - rust-dev
created_date: '2026-11-15T11:29:00Z'
updated_date: '2026-11-15T12:36:00Z'
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

**What.** Apply the same control-character predicate already used by `sanitize_name` and `sanitize_path_string` to `sanitize_path_component`, and therefore to every `Normal` component of `sanitize_path`, without changing behavior for legitimate paths.

**Decisions.**
- **Predicate.** Reuse the existing control-character predicate that `sanitize_name` and `sanitize_path_string` already share; it matches the Unicode `Cc` category, which is exactly C0 (U+0000-U+001F), DEL (U+007F), and C1 (U+0080-U+009F). Do not add a second range check or a parallel helper.
- **Evaluation point.** Check the component as provided, before the function's existing surrounding-whitespace trim. NFR-02.002 requires failing input to be rejected rather than silently cleaned, and a check applied only after trimming would silently accept `"a\n"`, `"\ta"`, or `"a\u{85}"` (U+0085 is Unicode whitespace) as `"a"`. This deliberately differs from `sanitize_name`, which accepts `"Title\t"` as `"Title"`; that asymmetry is accepted and out of scope.
- **Redundancy.** Once the predicate is applied, the existing explicit NUL check in `sanitize_path_component` is redundant. It may be removed or kept; observable behavior must be identical either way.
- **Traversal guards unchanged.** `sanitize_path_component` continues to reject `.`, `..`, `/`, and `\`. AC #3's CurDir wording refers only to `sanitize_path`, which continues to skip `Component::CurDir`, reject `Component::ParentDir`, and reject components that are not valid UTF-8.
- **No partial output.** `sanitize_path` returns `None` on the first failing component, so a partially sanitized path is never returned to a caller and nothing is written to disk.

**In scope.** `src/sanitize.rs`: control-character rejection in `sanitize_path_component`; `sanitize_path` inherits it; unit tests, in the existing `#[cfg(test)]` module style, for both functions covering C0, DEL, and C1 boundaries, nested and absolute paths, and regression cases for legitimate Unicode, space, dot, hyphen, and underscore segments.

**Out of scope.** `sanitize_name` and `sanitize_path_string` behavior, including their trim-before-check ordering; error messages; exit codes; normalization or escaping; and caller changes in `src/context.rs`, `src/commands.rs`, or `src/lib.rs`.

**Assumptions.**
- "Control character" means the Unicode `Cc` category (C0, DEL, C1) only; format characters such as U+200B and U+202E stay accepted, consistent with the existing predicate.
- `sanitize_path_component`'s existing trimming of surrounding whitespace (`" title "` -> `"title"`) is retained; only control characters must be rejected rather than trimmed.
- Windows `Component::Prefix` handling is unchanged and untested here; the target platform is Debian amd64 (NFR-03.002).

**Dependencies.** None beyond TASK-123 (already complete), which introduced the shared predicate.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 `sanitize_path_component` returns `None` for any component containing a C0 (U+0000-U+001F), DEL (U+007F), or C1 (U+0080-U+009F) character, using the same predicate as `sanitize_name`, and evaluates that predicate on the component before trimming so a leading or trailing control character is rejected rather than trimmed away
- [x] #2 `sanitize_path` returns `None` when any component contains such a character, including nested, absolute, and single-component paths, and never returns a path built from a cleaned or trimmed component
- [x] #3 legitimate paths are unchanged: Unicode letters, spaces, dots, hyphens, underscores, `a.b`, and absolute paths such as `/abs/path` sanitize to the same result as before; `sanitize_path` still skips CurDir and still rejects ParentDir; `sanitize_path_component` still returns `None` for `.` and `..`
- [x] #4 tests cover the boundary characters of all three ranges (U+0000, U+001F, U+007F, U+0080, U+009F) for both functions, a nested path with exactly one contaminated component, an absolute contaminated path, and both a leading and a trailing control character
- [x] #5 `sanitize_name` and `sanitize_path_string` behavior is unchanged and all existing sanitizer tests pass unmodified
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 `cargo test` passes, including the new sanitizer unit tests and every existing test
- [x] #2 `cargo clippy --all-targets -- -D warnings` passes with no new warnings
- [x] #3 `cargo fmt --check` passes
- [x] #4 only `src/sanitize.rs` changes; callers in `src/context.rs`, `src/commands.rs`, and `src/lib.rs` stay untouched
- [x] #5 each acceptance criterion above has at least one direct assertion in the new tests
<!-- DOD:END -->

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Path sanitizers now reject C0/DEL/C1 before trimming: sanitize_path_component applies the shared has_control_character predicate to the raw input, so 'a\n', '\ta' and 'a\u{85}' are rejected rather than shortened; sanitize_path inherits per Normal component and returns None on the first failure, never a partial path. Legitimate Unicode/space/dot/hyphen/underscore/CurDir/absolute paths are unchanged; '.' and '..' components and ParentDir/traversal remain rejected. sanitize_name and sanitize_path_string are untouched (trim-then-check asymmetry pinned by test). Evidence: 14 sanitizer tests (boundaries U+0000/U+001F/U+007F/U+0080/U+009F, full sweep, nested/absolute contaminated, leading/trailing, regressions); full suite 850 passed / 0 failed / 22 ignored; clippy and fmt clean.
<!-- SECTION:FINAL_SUMMARY:END -->
