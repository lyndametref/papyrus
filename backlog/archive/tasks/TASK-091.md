---
id: TASK-091
title: Make the browser console quiet by default and add debug request logging
status: Done
assignee:
  - @rust-dev
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-13T09:23:00Z'
labels:
  - browser
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-081
references:
  - docs/requirements/business/REQ-06.007.md
  - docs/requirements/non-functional/NFR-02.001.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-spec-revisions-task-86.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - README.md
priority: high
type: feature
ordinal: 39130
---


## Description
## Problem

`papyrus browser` currently writes a request line for every parsed request (`<method> <path> <status>`) to standard error regardless of options, so normal browsing is noisy and request-derived text appears in the console even though record content must not leak there (REQ-06.007, NFR-02.001). The existing pre-parse rejection lines (`- - <status>`, `- - 503`) are error diagnostics and are not the noise this ticket removes.

## Required behavior

REQ-06.007 and `docs/specifications/browser-requirements.md` section 2.12 require an error-only console by default and a browser-only `--debug` flag that logs exactly one line per received request with method, path, response status, and duration. Request headers and bodies are never logged. Errors are printed whether or not `--debug` is set. The exact line format, the duration unit, and the over-capacity exception are recorded in the specification.

## Scope

In scope:
- Add `--debug` as a boolean option of the `browser` command only; update the command usage string and `--help`, and reject the option everywhere else.
- Suppress parsed-request log lines unless `--debug` is set.
- With `--debug`, emit one stderr line per parsed request: `<method> <path> <status> <duration>ms`, path without query, decoded and truncated to 200 characters.
- Keep always-on error output: warnings, bind/startup/runtime failures, pre-parse rejection lines `- - <status>`, and the over-capacity line `- - 503`.
- Never log request headers, bodies, or record content.
- Keep stdout limited to the single startup line or startup JSON object, including with `--debug` and `--json`.
- Update the `browser` man page section in `README.md` (papyrus(1)) and the test suite.

Out of scope:
- Config keys, environment variables, or persisted settings for debug.
- Log files, log rotation, levels, timestamps, request IDs, or tracing.
- Any UI, HTTP response, route, record, or configuration behavior change.
- New dependencies or crate/application changes.

## Assumptions

- Assumption: the duration is a whole number of milliseconds with the `ms` suffix (for example `4ms`); `0ms` is valid. Chosen because REQ-06.007 requires a duration but leaves the unit and formatting open.
- Assumption: the duration is measured from the start of connection handling to the end of the response write.
- Assumption: parsed requests that return a 4xx or 5xx response are responses, not console errors, so they are logged only with `--debug`. "Errors are printed whether or not `--debug` is set" covers warnings, bind/startup/runtime failures, and pre-parse rejection lines.
- Assumption: the over-capacity line stays exactly `- - 503` with no duration, because the request is never read and the specification fixes that literal line.
- Assumption: `README.md` is the Papyrus man page for BR-004, so its browser section is updated by this ticket; TASK-86 revised only the specification corpus.
- Assumption: no UI design is required; this ticket changes console diagnostics only.

## Dependencies

- TASK-86 (completed) supplies the revised browser specification text.
- No other task is required before implementation.

## Acceptance Criteria
- [x] `papyrus browser --help` shows usage `papyrus browser [--host <host>] [--port <port>] [--no-open] [--debug]` and documents `--debug` as a browser-only flag that takes no value.
- [x] Passing `--debug` to any command other than `browser` (for example `papyrus req list --debug`), or as a global option before the subcommand (`papyrus --debug browser`), exits 2 with a usage message and starts nothing.
- [x] Without `--debug`, a parsed request returning 2xx, 3xx, 4xx, or 5xx writes no per-request line to stderr; stdout contains only the single startup line, or the single startup JSON object with `--json`.
- [x] With `--debug`, each parsed request writes exactly one stderr line formatted `<method> <path> <status> <duration>ms`, where `<duration>` is a non-negative integer (for example `GET /doc/REQ-001.001 200 4ms`).
- [x] The logged path excludes the query string, is percent-decoded, and is truncated to 200 characters; a request for `/?type=req` logs `/`, not `/?type=req`.
- [x] Request headers and bodies never appear in console output: a request carrying a unique header value and a unique body phrase produces no stderr containing either value, with or without `--debug`.
- [x] Without `--debug`, warnings and fatal messages still print (non-loopback bind warning, opener warning, bind failure), and pre-parse failures still print `- - <status>` (for example `- - 408`); the over-capacity rejection prints exactly `- - 503` with and without `--debug`.
- [x] With `--debug` and `--json`, all request and error lines go to stderr and stdout contains exactly one JSON startup object.
- [x] The `README.md` papyrus(1) browser section shows the usage line with `[--debug]` and documents that the flag logs one line per request without headers or bodies, consistent with `--help`.
- [x] `tests/browser_cli.rs` covers the quiet default, the debug line shape and duration, query stripping and 200-character truncation, absence of header/body content, always-on warnings and `- - 503`, rejection of `--debug` on another command and as a global option, and the help text.
- [x] `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check` pass, and `Cargo.toml` gains no dependency.

## Definition of Done
- [x] #1 All 11 acceptance criteria are verified with concrete evidence (test names or command output) recorded in the ticket.
- [x] #2 New and updated tests in `tests/browser_cli.rs` pass, including the existing browser suite.
- [x] #3 `cargo test --all-targets` passes.
- [x] #4 `cargo clippy --all-targets -- -D warnings` passes.
- [x] #5 `cargo fmt --check` passes.
- [x] #6 The `README.md` papyrus(1) browser section documents `--debug` consistently with `papyrus browser --help`.
- [x] #7 `docs/specifications/browser-requirements.md` section 2.12 matches the implemented format and exception; `docs/requirements/` and `docs/business-rules/` are unchanged.
- [x] #8 No dependency, route, HTTP response, record, or configuration behavior changed; the final summary lists every changed file.


## Implementation Notes
1. **CLI surface** (`src/lib.rs`): add a boolean `debug` field to the `Browser` subcommand (`#[arg(long)]`, no value), include `[--debug]` in `override_usage`, and pass the flag through the command dispatch into `browser::run`. Do not add a global option, environment variable, or configuration key.
2. **Runtime plumbing** (`src/browser/mod.rs`): extend `browser::run` with the debug flag, store it in `ServerState`, and pass it to the connection handler. Leave `reject_over_capacity` unconditional because its `- - 503` line is always printed.
3. **Request logging** (`src/browser/mod.rs`): start an `Instant` at the beginning of `handle_connection`, move the parsed-request log to after the response write, and emit `<method> <path> <status> <duration>ms` to stderr only when debug is set. Keep the existing path derivation (decoded path, no query, 200-character truncation).
4. **Always-on diagnostics**: keep warnings, bind/startup/runtime messages, `- - <status>` lines for pre-parse failures, and the exact `- - 503` over-capacity line regardless of the flag. Ensure no log line uses stdout.
5. **Man page** (`README.md`): update the `browser` usage line to `papyrus browser [--host <host>] [--port <port>] [--no-open] [--debug]` and add a `--debug` entry stating that it logs one line per received request (method, path, status, duration) without headers or bodies.
6. **Tests** (`tests/browser_cli.rs`): update `t01_command_shape` for the new usage text; change `unknown_path_is_404_for_every_method` and the over-capacity test so per-request log assertions run with `--debug` while `- - 503` stays unconditional; add tests for quiet default, debug format and duration, query stripping/truncation, header/body absence, `--debug` rejection on another command and as a global option, and `--json` stdout isolation.
7. **Verify**: run the focused browser test binary, then `cargo test --all-targets`, `cargo clippy --all-targets -- -D warnings`, and `cargo fmt --check`; record evidence against each acceptance criterion.


## Notes
Implementation complete on branch task/TASK-96.

Evidence:
- CLI: src/lib.rs adds browser-only --debug (boolean, no value), usage line updated; --debug rejected by clap (exit 2) for other commands and as a global option.
- Runtime: src/browser/mod.rs keeps debug in ServerState; per-request log emitted after the response write only when debug is set, as '<method> <path> <status> <duration>ms'; path remains decoded, query-free, truncated to 200 chars; '- - <status>' and '- - 503' remain unconditional.
- README.md papyrus(1) browser section updated with [--debug] and the flag description.
- Tests (tests/browser_cli.rs): t01_command_shape, debug_flag_is_rejected_outside_browser, browser_console_is_quiet_by_default, debug_logs_one_line_per_request_with_duration, debug_path_strips_query_decodes_and_truncates, console_never_contains_headers_or_body, over_capacity_line_is_unconditional_with_debug, debug_with_json_keeps_stdout_to_the_startup_object, readme_documents_browser_debug; existing timeout test asserts '- - 408'.
- cargo test --all-targets: exit 0, 37 binaries, 620 passed, 0 failed, 0 ignored.
- cargo clippy --all-targets -- -D warnings: exit 0.
- cargo fmt --check: exit 0.
- Cargo.toml unchanged (no dependency added).

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Implemented browser --debug request logging. Changed files: src/lib.rs (browser-only --debug flag), src/browser/mod.rs (gated one-line request logs with duration), tests/browser_cli.rs (9 new tests + updates), README.md (man page --debug), docs/specifications/browser-requirements.md (§2.12). Quiet by default; errors/over-capacity always printed; no headers or bodies logged; stdout startup-only. cargo test 620 passed, clippy/fmt clean.
