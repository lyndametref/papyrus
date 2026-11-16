---
id: TASK-118
title: Reject C0 control characters in titles (F-117-1)
status: Done
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
updated_date: '2026-11-15T10:42:00Z'
labels:
  - rust
  - validation
milestone: MS-019
dependencies: []
references:
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/business/REQ-01.004.md
documentation:
  - src/sanitize.rs
  - tests/control_characters_cli.rs
priority: low
ordinal: 41250
---


## Description
**Why.** F-117-1 (recorded by TASK-117): the shared sanitizers reject NUL, CR, LF, and path separators but accept TAB (0x09), 0x01, and the rest of the C0 range, so a single-line value can carry a control character that corrupts list, view, browser, and front-matter rendering. Typed handlers (ui_spec, audit, rule, glossary, tech_spec, use_case, test_case, test_report, ui_mockup), config list parsing, and browser validation already reject control characters via `char::is_control()`; the shared sanitizer is the outlier, so older types accept input the rest of the product rejects (NFR-02.002).

**What.** Apply one shared control-character check in `sanitize_name` and `sanitize_path_string` so every create and edit path that normalizes a single-line value fails with exit 4 and writes nothing. Keep all other accepted input unchanged.

## Behaviour contract

- **Rejection set.** Every C0 control character U+0000–U+001F and DEL U+007F, at minimum. The check uses the crate's existing `char::is_control()` predicate, so U+0080–U+009F (C1) are rejected too. Rationale: this keeps the shared sanitizer equal to the typed handlers, config list parsing, and browser validation instead of looser than them. Tests must assert the whole C0+DEL range; C1 is asserted once for consistency.
- **Order.** Trim first (unchanged), then reject. Leading/trailing whitespace — space, TAB, CR, LF, 0x0B, 0x0C, Unicode whitespace — is still stripped, and input that trims to empty still returns `None`. A value is rejected only if the trimmed content still contains a control character: `"Ti\ttle"` -> `None`, `"Title\t"` -> `Some("Title")`, `"\t"` -> `None`.
- **Unchanged.** Internal spaces, Unicode text, ordinary punctuation, dots, and hyphens; `sanitize_name` keeps rejecting `/` and `\`; `sanitize_path_string` keeps rejecting `\`, `..`, absolute/root and prefix components; empty-after-trim still returns `None`.
- **Error surface.** The existing `AppError::Validation` path: exit 4, stderr `input cannot be normalized`. No new error code, flag, or message.

## Change surface

- `src/sanitize.rs` — one private helper, called by `sanitize_name` and `sanitize_path_string` on the trimmed value. No signature or public API change.
- No behaviour change elsewhere: `sanitize_path_component`, `sanitize_path`, `kebab_slug`, the aliases, config parsing, and browser validation stay as they are. A typed handler may drop a now-redundant local control check only if the resulting behaviour is identical.

## Test plan

- **Unit** (`src/sanitize.rs`): loop every scalar in U+0000–U+001F and U+007F placed inside `"a{ch}b"`; both functions return `None`. Accepted samples: `"Release notes"`, `"Résumé – v2"`, `"a.b"`, `"a-b"`, `"x (y) 100%"`; plus `"docs/design.md"` for `sanitize_path_string`. Trim interplay: `"  Ti\ttle  "` -> `None`, `"\t"` -> `None`, `"Title\t"` -> `Some("Title")`, `"   "` -> `None`. Re-assert the existing rejections (NUL/CR/LF for both; `/`/`\` for `sanitize_name`; `..`/`\`/absolute for `sanitize_path_string`) and one C1 sample for consistency.
- **CLI create** (new `tests/control_characters_cli.rs`, `init --no-git` fixture, `note` type injected into `document_types` as in `tests/cli_dispatch.rs`): a TAB and a 0x01 title for `req create --group REQ-001` (after `req create-group`), `note create`, `task create`, and `milestone create` -> exit 4, stderr contains `input cannot be normalized`, stdout empty, no new `.md` file in the record directory.
- **CLI edit**: on an existing `REQ-001.001` and `TASK-001`, `edit --title` with a TAB and with 0x01 -> exit 4 and byte-identical file (compare bytes; `updated_date` unchanged).

## Scope (in)

- The shared control-character check in `sanitize_name` and `sanitize_path_string` (src/sanitize.rs).
- Unit tests for the full rejection range, accepted samples, and trim interplay.
- CLI tests for four create families and two edit families, including exit code and absence of writes.

## Scope (out)

- Descriptions becoming multi-line or any change to which fields are single-line; descriptions stay single-line.
- Trimming, normalizing, or escaping control characters instead of rejecting them; no new error code or flag.
- Browser UI work: browser title/label/reference/path validation already rejects control characters.
- Reopening TASK-110 or TASK-117.
- `sanitize_path_component` / `sanitize_path` still accept C0/DEL in a component, so a TAB in `--dir` or a non-folder config string value still passes. Same defect class, different surface — recorded below as a follow-up candidate, not fixed here.
- Decomposing or splitting this ticket.

## Assumptions

- No legitimate single-line value contains C0, DEL, or C1 characters.
- Trim-then-reject is correct because trimming is established and documented behaviour (browser help text: "whitespace is trimmed"); an edge control character is removed by trimming, not rejected.
- Exit code 4 is the validation outcome (REQ-01.004).
- Test fixture: default config plus an injected `note` type (`enabled: true`, `prefix: NOTE`, `directory: notes/`); `req` needs an existing group.

## Traceability

- NFR-02.002 (sanitize input; reject, do not silently clean), REQ-01.004 (exit codes), F-117-1 in TASK-117 (closed by this ticket).
- Existing behaviour: src/sanitize.rs, src/config.rs (list values), src/browser/records.rs:815-860.

## Open question (non-blocking, default stated)

- Confirm the C1 inclusion (U+0080–U+009F). Default: include it via `char::is_control()`. If product wants C0+DEL only, the check must be an explicit range predicate and typed-handler checks stay as they are.

## Follow-up candidate (not this task)

- `sanitize_path_component` and `sanitize_path` accept C0/DEL in a component; a TAB in `--dir` or a non-folder config string value still passes. Route to a separate ticket if wanted.

## Acceptance Criteria
- [x] sanitize_name and sanitize_path_string return None when the trimmed value contains any character in U+0000..U+001F or U+007F, including TAB (0x09) and 0x01, verified for every character in that range
- [x] Trimming is unchanged and applied before the check: leading/trailing whitespace (space, TAB, CR, LF, 0x0B, 0x0C, Unicode whitespace) is still stripped, so "\t" and "   " return None and "Title\t" returns "Title", while "Ti\ttle" returns None
- [x] Legitimate input is unchanged: internal spaces, Unicode text, ordinary punctuation, dots, and hyphens are accepted; NUL, CR, LF, /, and \ rejections for sanitize_name remain, and \, .., absolute/root, and prefix-component rejections for sanitize_path_string remain
- [x] A create with a TAB or 0x01 in the title exits 4 with stderr containing "input cannot be normalized", writes no record file, and prints nothing on stdout, for req create (typed document), note create (config-defined slug), task create, and milestone create
- [x] An edit that introduces a TAB or 0x01 into --title exits 4 and leaves the file byte-identical (bytes and updated_date unchanged), for a typed document and a task
- [x] Unit tests in src/sanitize.rs cover the full U+0000..U+001F range plus U+007F for both functions, the accepted samples, the trim interplay, the existing rejections, and one C1 sample
- [x] CLI tests in tests/control_characters_cli.rs cover the four create families and both edit families and assert exit code 4, the validation message, and the absence of writes
- [x] No behaviour change outside the two sanitizers: sanitize_path_component, sanitize_path, kebab_slug, the aliases, config parsing, and browser validation behave as before, and a typed handler may drop a redundant local control check only if behaviour is identical

## Definition of Done
- [x] #1 cargo build, cargo test, and cargo clippy are clean.
- [x] #2 No new dependency, error code, or flag is introduced; ui_spec local helper may be simplified to the shared sanitizer.
- [x] #3 F-117-1 is closed with the tested character set recorded in the ticket notes.
- [x] #4 cargo fmt --check is clean, and tests/control_characters_cli.rs is recorded in the ticket's documentation metadata.
- [x] #5 Implementation notes record the final rejection set, the C1 decision, and that sanitize_path_component remains a separate follow-up candidate.


## Implementation Notes
1. Read src/sanitize.rs; add one private control-character helper (existing `char::is_control()` predicate) and call it from `sanitize_name` and `sanitize_path_string` after trimming, before the existing NUL/separator/traversal checks.
2. Add `#[cfg(test)]` unit tests: full U+0000..U+001F + U+007F loop for both functions, accepted samples (spaces, Unicode, punctuation, dots, hyphens, relative path), trim interplay, existing rejections, one C1 sample.
3. Add tests/control_characters_cli.rs: init fixture, inject the `note` type, assert exit 4 + message + no write for req/note/task/milestone create with TAB and 0x01 titles.
4. Add the edit cases: REQ-001.001 and TASK-001 `edit --title` with TAB and 0x01; assert exit 4 and byte-identical file.
5. Run cargo fmt --check, cargo build, cargo test, cargo clippy; record the tested character set and the C1 decision in the implementation notes and close F-117-1.


## Notes
## Implementation Notes

- Rejection set: U+0000–U+001F (C0) and U+007F (DEL), enforced in the trimmed value by one private helper using char::is_control.
- C1 decision: char::is_control also rejects U+0080–U+009F; chosen so the shared sanitizer is not looser than the typed handlers, config list parsing and browser validation that already use is_control.
- Trim-then-reject: trailing/leading whitespace (space, TAB, CR, LF, VT, FF, Unicode whitespace) is trimmed by the existing str::trim, so 'Title\t' becomes 'Title' and '\t' alone is empty; interior controls are rejected.
- Unchanged surfaces: sanitize_path_component, sanitize_path, kebab_slug, config parsing, browser code; aliases sanitize_reference/sanitize_documentation/sanitize_path_value delegate and therefore inherit the rejection.
- Follow-up candidate: sanitize_path_component/sanitize_path still accept C0/DEL in --dir and path-like config values (separate ticket).


## Comments
author: @task-refiner
created: 2026-11-15T10:27:00Z

---

Refined for execution. Rewrote the description with a behaviour contract (rejection set, trim-then-reject order, unchanged input, error surface), change surface (src/sanitize.rs only), a concrete unit + CLI test matrix, scope in/out, assumptions, traceability, and a follow-up candidate. Replaced the 5 loose ACs with 8 testable ACs and added 2 DoD items plus a 5-step plan. Docs metadata now points at src/sanitize.rs and tests/control_characters_cli.rs.

Decisions recorded:
- Rejection set is the C0 range U+0000..U+001F plus DEL U+007F, implemented with the crate's existing `char::is_control()` predicate; consequence is C1 U+0080..U+009F are also rejected, which keeps the shared sanitizer equal to typed handlers, config list parsing, and browser validation rather than looser.
- The check runs on the trimmed value, because trimming is established, documented behaviour; edge whitespace is stripped, interior controls are rejected. Without this the AC would be untestable for values like "Title\t".
- `sanitize_path_component` / `sanitize_path` still accept C0/DEL in a component (TAB in `--dir`, non-folder config string). Same defect class, kept out of scope here and flagged as a follow-up candidate.

Assumptions: no legitimate single-line value contains C0/DEL/C1; exit 4 is the validation outcome (REQ-01.004); `note` fixture injected as in tests/cli_dispatch.rs; `req` needs an existing group. Dependencies: none; TASK-110 and TASK-117 stay closed.

Open question (non-blocking, default stated): confirm C1 inclusion. Execution-ready: yes.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
F-117-1 closed: one private has_control_character helper using char::is_control rejects U+0000..U+001F, U+007F and C1 (U+0080..U+009F) in sanitize_name and sanitize_path_string after trim. Trim behavior is preserved (trailing whitespace trimmed; interior controls rejected), and legitimate spaces/Unicode/punctuation/dots/hyphens/path strings remain accepted; NUL/CR/LF/separator/traversal rejections retained. Create with TAB/0x01 exits 4 with no file for req, note, task, milestone; edit on REQ/TASK exits 4 with byte-identical file. Evidence: sanitize unit tests 5/5 including the full C0+DEL loop and a C1 sample; control_characters_cli 2/2; full suite 835 passed / 0 failed / 22 ignored; clippy and fmt clean. Follow-up candidate recorded: sanitize_path_component/sanitize_path still accept controls in --dir/path values.
