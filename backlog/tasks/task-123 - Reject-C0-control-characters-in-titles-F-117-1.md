---
id: TASK-123
title: Reject C0 control characters in titles (F-117-1)
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
labels:
  - rust
  - validation
milestone: m-18
dependencies: []
references:
  - docs/requirements/non-functional/NFR-02.002.md
documentation:
  - src/sanitize.rs
  - tests/cli_dispatch.rs
priority: low
ordinal: 41250
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** TASK-117 recorded F-117-1: sanitize_name rejects NUL, CR, LF, and path separators but accepts TAB (0x09), 0x01, and the rest of the C0 range, so a title can contain a control character that corrupts list, view, browser, and front-matter rendering. One typed module (ui_spec) already rejects every control character through a local helper, which means the shared sanitizer is inconsistent with the newest code and older types accept input the spec calls invalid (NFR-02.002).

**What.** Reject every C0 control character (U+0000 to U+001F) and DEL (U+007F) in the shared sanitizers sanitize_name and sanitize_path_string, so every create and edit path fails with exit 4 and writes nothing. Keep all other accepted input unchanged.

**In scope.**

- src/sanitize.rs: one shared control-character check in sanitize_name and sanitize_path_string.
- Confirmation that single-line values, internal spaces, Unicode text, punctuation, and the existing NUL/CR/LF/path-separator rejections behave as before.
- Unit tests in src/sanitize.rs and CLI tests covering TAB and 0x01 in at least a document title (req and a config-defined slug) and a task or milestone title.

**Out of scope.**

- Making descriptions multi-line or changing what fields are single-line; descriptions stay single-line as today.
- Reopening TASK-110 or TASK-117; this is a follow-up to their delivered validation.
- Trimming, normalizing, or escaping control characters instead of rejecting them; no new error code or flag.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 sanitize_name and sanitize_path_string return None for every character in U+0000..U+001F and U+007F, including TAB (0x09) and 0x01
- [ ] #2 Legitimate input is unchanged: internal spaces, Unicode text, ordinary punctuation, dots, and hyphens are still accepted; the existing NUL, CR, LF, and path-separator rejections remain
- [ ] #3 A create with a TAB or 0x01 in the title exits 4 with the existing validation error and writes no file, for a document type and for a task or milestone
- [ ] #4 An edit that introduces a control character into a single-line field exits 4 and leaves the file byte-identical
- [ ] #5 Unit tests cover the full C0 range and DEL plus accepted samples; CLI tests cover at least two record families and assert exit code and absence of writes
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo build, cargo test, and cargo clippy are clean.
- [ ] #2 No new dependency, error code, or flag is introduced; ui_spec local helper may be simplified to the shared sanitizer.
- [ ] #3 F-117-1 is closed with the tested character set recorded in the ticket notes.
<!-- DOD:END -->
