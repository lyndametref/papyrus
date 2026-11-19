---
id: TASK-192
title: "Implement multiline stdin input for create and update commands"
created_date: '2026-11-18T21:13:40Z'
updated_date: '2026-11-19T10:54:54Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
status: Done
assignee:
  - rust-dev
milestone: MS-029
dependencies:
  - TASK-191
ordinal: 3
documentation:
  - src/document.rs
  - src/cli/dispatch.rs
  - src/sanitize.rs
type: implementation
---

## Description
Outcome: create and update commands accept multiline body and content field values from stdin for all affected document and backlog types, while identifiers, titles, and other single-line fields still reject embedded newlines. Existing argument and editor paths are unchanged, and CLI help documents the stdin input path.
## Comments
- **task-refiner**: Refinement contract fixed by TASK-191 (Done). Sentinel: exact `-` value on --description for document, task, and milestone create and edit; on --decision, --context, and --consequences for decision create and edit. One stdin read per invocation to EOF; UTF-8 required; CRLF to LF; trim outer line terminators only; internal blank lines preserved; LF the only allowed control character; empty or whitespace-only input and multiple sentinels exit 4 with no write; no implicit stdin read. Help sentence: Use "-" to read multiline text from standard input. See the TASK-191 description for the full contract, assumptions, and definition of done.
- **rust-dev**: Implementation evidence (rust-dev): branch task-TASK-192, commit 773164a; contract TASK-191.
- **rust-dev**: Shared stdin helpers live in the sanitize module: STDIN_SENTINEL, sanitize_multiline, Text, resolve_stdin_fields, resolve_text_field. Document create and edit resolve the -d - sentinel before ID allocation, writes, and typed dispatch; task, milestone, and decision resolve in run() before create and edit; typed document create and edit carry Text values.
- **rust-dev**: Help uses the exact sentence Use "-" to read multiline text from standard input on every accepting option; README manpage entries updated; help goldens regenerated and extended to task, milestone, and decision create and edit.
- **rust-dev**: The multiline stdin CLI test module implements TC-040 through TC-054 plus empty and whitespace-only stdin, invalid UTF-8, multiple sentinels before read and write, CRLF normalization, no-implicit-stdin, and a typed document type.
- **rust-dev**: Checks: cargo fmt --check pass; cargo clippy --all-targets pass; cargo test --lib 431 passed; multiline stdin CLI tests 20 passed; CLI help goldens 2 passed. Full cargo test --no-fail-fast passes except two pre-existing failures in untouched code: list characterization replay (decision empty-table assertion) and no drift (import module prefix-shaped-ID false positive).
- **rust-dev**: No new flags; literal single-line values and all non-accepting-field validation are unchanged.
