---
id: TASK-204
title: "Extend multiline stdin to audit --content and glossary --definition"
created_date: '2026-11-19T05:36:19Z'
updated_date: '2026-11-19T10:54:54Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
  - TECH-078
  - TC-040
status: Done
assignee:
  - rust-dev
milestone: MS-029
dependencies:
  - TASK-192
ordinal: "4"
documentation:
  - src/audit.rs
  - src/glossary.rs
  - src/registry.rs
  - README.md
  - tests/multiline_stdin_cli.rs
  - tests/cli_help_golden.rs
  - tests/golden/cli-help/
type: implementation
---

## Description
Outcome: close the blocking AC-1 finding in AUD-115 and AUD-116 by extending the TASK-191 multiline stdin contract to audit `--content` and glossary `--definition` on create and edit. MS-029 AC-1 is not narrowed.

## Scope
- Accept the exact `-` sentinel (`--content -`, `--content=-`, `--definition -`, `--definition=-`) for `audit create/edit --content` and `glossary create/edit --definition`, reusing the shared stdin helpers implemented by TASK-192 (do not duplicate the contract).
- Read standard input to end-of-file once per invocation and apply the TASK-191 stdin contract unchanged: valid UTF-8 required; CRLF and lone CR normalized to LF; strip leading and trailing line terminators only; internal blank lines and per-line spaces preserved; LF the only permitted control character; empty or whitespace-only input, invalid UTF-8, and multiple sentinels exit 4.
- Failures exit 4 before anything is written: no ID allocated and no record created or modified.
- No other field changes: audit title, ID, labels, references, and audit-type, and glossary term, labels, and references stay single-line and keep rejecting embedded newlines with exit 4.
- Literal single-line values, `--json` shapes, exit codes, and the `--editor` path behave exactly as today.

## Help and documentation
- Every accepting option `--help` text carries the exact sentence `Use "-" to read multiline text from standard input.` composed with the existing field description instead of replacing it.
- README.md manpage entries for audit create/edit and glossary create/edit gain the same note.
- Regenerate frozen snapshots with `PAPYRUS_UPDATE_CLI_HELP=1 cargo test --test cli_help_golden`.

## Tests
- Add CLI regression tests to tests/multiline_stdin_cli.rs for audit and glossary multiline content on create and edit, in the TC-040 style: piped two-line and multi-paragraph stdin stored with line order and internal blank lines preserved, both sentinel forms, empty and whitespace-only stdin exit 4, invalid UTF-8 exit 4, multiple sentinels exit 4 before any read or write, CRLF normalization, and a literal value that does not read stdin.

## Contract record
- The TASK-191 contract keeps its substance. Record in the task evidence whether TASK-191 or MS-029 wording must name audit `--content` and glossary `--definition` explicitly; do not edit MS-029 acceptance criteria, whose AC-1 already covers document content text.

## Definition of done
- AUD-115 finding 1 and AUD-116 finding 1 are resolved by extension, not by narrowing AC-1.
- cargo fmt --check, cargo clippy --all-targets -- -D warnings, the multiline stdin suite, and the help golden suite pass; the only tolerated full-suite failures remain the pre-existing AUD-039 exceptions.
- No open material questions remain.
## Comments
- **rust-dev**: Implementation evidence: implemented exact - sentinel for audit --content and glossary --definition on create and edit using shared TASK-191 stdin helpers. Files: audit.rs, glossary.rs, registry.rs, README.md, multiline_stdin_cli.rs, cli-help goldens. Tests cover two-line and multi-paragraph content, both sentinel forms, CRLF normalization, empty or whitespace-only and invalid UTF-8 exit 4 with no ID allocation or write, literal values not reading stdin, and extended help assertions. Checks: cargo fmt --check pass; cargo clippy --all-targets -D warnings pass; cargo test --lib 431 passed; multiline_stdin_cli 25 passed; cli_help_golden 2 passed; full suite only the two pre-existing AUD-039 failures. No schema or new flag change. Multiple sentinels are structurally impossible per audit or glossary invocation because each has one accepting option; shared guard remains covered by decision regression test. Contract record: TASK-191 normative accepting set should explicitly add audit --content and glossary --definition; MS-029 AC-1 needs no change; neither edited. AUD-115 and AUD-116 finding 1 resolved by extension. Code commit 452a1a8 on the task branch.
