---
id: TASK-104
title: Implement cursor-driven artifact activation in the papyrus init walkthrough
status: Implemented
assignee:
  - '@rust-dev'
created_date: '2026-11-13T16:17:00Z'
updated_date: '2026-11-13T16:31:00Z'
labels:
  - configuration
  - init
  - release-1.0
milestone: m-16
dependencies:
  - TASK-91
references:
  - docs/requirements/business/REQ-01.006.md
  - docs/specifications/interactive-init-walkthrough-for-artifact-activation.md
  - docs/requirements/business/REQ-01.001.md
  - docs/requirements/business/REQ-01.007.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
modified_files:
  - src/init.rs
  - src/error.rs
  - src/browser/mod.rs
priority: high
type: feature
ordinal: 39085
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Conformance gap against REQ-01.006. `papyrus init` must present artifact activation as a single cursor-driven list. The shipped `src/init.rs` (`build_config_interactive`, ~lines 152-187) still prints a static `[x]` block and reads one line of space-separated artifact names, so the user cannot move a cursor or toggle a row; only typed tokens work. This ticket replaces that presentation. No new requirement is introduced and no CLI surface changes.

Scope (all inside `src/init.rs` plus tests and the feature specification):
1. Render all artifacts as one vertical list: core rows `task`, `milestone`, `release` first (fixed active marker, "always active" indication, never highlighted, never toggled), then decision types `adr`, `bdr`, then document types in schema order (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`, `use-case`, `test-case`, `test-report`). All rows start active.
2. Exactly one selectable row carries the cursor on entry: the first selectable artifact (`adr`). The cursor never occupies a core row.
3. Up and Down arrow keys and `k` / `j` move the cursor to the previous/next selectable artifact only, clamping at the first and last selectable artifact (no wrap, no escape into core rows).
4. Space toggles only the cursor row between active (`[x]`) and inactive (`[ ]`); the change is visible immediately.
5. Enter confirms the list; the walkthrough then continues into the existing task/milestone/release prefix prompts, `backlog_directory`, decision and document prompts unchanged. The activation written to `papyrus.config.yml` equals the state displayed at Enter.
6. End of input on the list (closed stdin, Ctrl-D) confirms the current state and continues into the same prompts. Ctrl-C must terminate without leaving the terminal in a modified input mode and without writing `papyrus.config.yml`.
7. Redraw the whole list after every keypress so cursor position and every `[x]`/`[ ]` are always visible. Keys other than Space, `k`, `j`, and Enter must not change activation.
8. Remove the typed-token path entirely: no prompt may read artifact names from a text line.
9. Non-TTY path unchanged: no prompts, no stdin reads, all artifacts active, default prefixes/folders, `id_padding` 3.
10. Introduce an injectable key source so the list logic is testable without a terminal, and keep the real keyboard path as a thin adapter.

Constraints:
- No new CLI flag, environment variable, or configuration key. `--force`, `--no-git`, exit codes, and the generated configuration schema are unchanged.
- Terminal input handling must respect the dependency-free baseline (ADR-0001, ADR-0004, ADR-0005); see the ADR produced for TASK-104 before choosing any terminal-mode mechanism.
- Prefix/folder prompts keep their existing line-based validation, defaults, hints, and re-prompt behavior.
- Directory creation stays activation-driven (already implemented in `run_init_with_io`); do not regress it.

Out of scope: prefix/folder prompt semantics and wording, `--force`/`--no-git`, non-TTY defaults, configuration schema, other commands, mouse input, in-list search/filter, colours, README/BR-002 wording (the current manpage text stays accurate, so no business-rule excerpt sync is required).

Specification: `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` section 2 now states this cursor-driven contract, sections 1.2 and 4.4 no longer describe line-based or `y`/`n` activation, and the artifact set includes `use-case`, `test-case`, `test-report` (spec drift fixed during refinement).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 On a TTY, `papyrus init <name>` renders one vertical list: core rows `task`, `milestone`, `release` first (fixed active marker plus an "always active" indication), then decision types `adr`, `bdr`, then document types in schema order (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `deploy`, `tech-spec`, `use-case`, `test-case`, `test-report`); every selectable row starts `[x]`.
- [ ] #2 Exactly one selectable row is highlighted on entry and it is `adr`, the first selectable artifact; the highlight is rendered with a plain cursor marker (for example `>`) that is visible in captured output, and no core row is ever highlighted.
- [ ] #3 Down arrow and `j` move the highlight to the next selectable artifact; Up arrow and `k` move it to the previous; movement skips the core rows.
- [ ] #4 Highlight movement clamps: Up on `adr` leaves the highlight on `adr`, Down on `test-report` leaves it on `test-report`, and no wrap-around occurs in either direction.
- [ ] #5 Space toggles only the highlighted row between `[x]` and `[ ]`; exactly one row changes per Space press and a second press restores the previous state.
- [ ] #6 Core rows cannot be toggled: no key sequence moves the cursor onto `task`, `milestone`, or `release`, and their markers stay fixed while other rows toggle.
- [ ] #7 The whole list is written again after every keypress: each keypress produces a redraw containing the current cursor marker and the current `[x]`/`[ ]` state of every row; keys other than Up, Down, `k`, `j`, Space, and Enter (for example `x`, `q`, `1`) change no activation state.
- [ ] #8 Enter confirms the list and the walkthrough continues with the task, milestone, release prefix prompts and the folder prompts; the written `papyrus.config.yml` has `enabled: true` for every row shown `[x]` and `enabled: false` for every row shown `[ ]` at the moment Enter was pressed.
- [ ] #9 End of input on the list (closed stdin or Ctrl-D) confirms the current state and continues into the prefix/folder prompts; the run completes without hanging and writes `papyrus.config.yml`.
- [ ] #10 No typed-token activation path remains: no prompt accepts artifact names as text, the strings `Toggle artifacts` and `space-separated` no longer appear in `src/init.rs`, and text typed at any prompt does not change activation.
- [ ] #11 The non-TTY path is unchanged: no prompts, no `stdin` reads, all artifacts activated, default prefixes/folders, `id_padding` 3, and the existing init cases in `tests/cli.rs` still pass.
- [ ] #12 Automated tests drive the list through an injected key source and cover the initial cursor row, Up/Down and `k`/`j` movement, clamping at both ends, Space toggling, core rows unreachable, unrelated keys ignored, redraw content per keypress, Enter confirmation, and end-of-input confirmation.
- [ ] #13 `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` states the cursor-driven contract in section 2, contains no line-based or `y`/`n` activation wording in sections 1.2 and 4.4, and lists the artifact set including `use-case`, `test-case`, `test-report` (drift fixed during refinement; verify it stayed fixed).
- [ ] #14 Ctrl-C during the list exits without writing `papyrus.config.yml` and without leaving the terminal in a modified input mode (verified on a real TTY).
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 `cargo test` passes, including `tests/no_drift.rs` and the non-interactive init cases.
- [ ] #2 `cargo fmt --check` is clean.
- [ ] #3 The generated `papyrus.config.yml` validates (REQ-01.007) and only activated artifacts are enabled (BR-016).
- [ ] #4 No new CLI flag, environment variable, or configuration key is introduced.
- [ ] #5 No new crate dependency is added; terminal handling follows the TASK-104 architecture decision (ADR recorded in `docs/architecture/adr/`).
- [ ] #6 Behavior is verified on a real TTY, not only by unit tests: cursor movement, Space toggle, Enter confirm, end-of-input confirm, Ctrl-C, and terminal input mode restored afterwards; the verification is recorded in the implementation notes.
- [ ] #7 Implementation notes cite the architecture decision and the test names that cover each acceptance criterion.
- [ ] #8 The feature specification and the ticket still agree after implementation; any deviation is recorded as a task comment or a follow-up task rather than silently absorbed.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Implemented per ADR-0015 (stty cbreak, injectable KeySource/TerminalMode, append-only redraw).

Code:
- src/init.rs: Key/KeySource/ByteKeySource, TerminalMode/SttyTerminalMode/TerminalModeGuard, pure ArtifactList state machine, run_artifact_list, degraded line-confirmation path. Removed the typed-token prompt; non-TTY path untouched.
- src/error.rs: AppError::Interrupted -> message "interrupted", exit code 130.
- src/browser/mod.rs: exhaustive match arm for Interrupted (unreachable in browser config submit).

Automated verification:
- cargo test: pass (194 library tests including 20 new init tests; all integration suites pass).
- cargo clippy --all-targets -- -D warnings: clean.
- cargo fmt --all --check: clean.
- New tests: initial_cursor_is_adr_and_every_row_is_active, rows_follow_the_required_artifact_order, down_and_j_move_to_the_next_selectable_row, cursor_clamps_at_both_ends_without_wrapping, space_toggles_only_the_highlighted_row, core_rows_are_never_reachable_or_toggleable, unrelated_keys_do_not_change_activation, every_keypress_redraws_the_whole_list, enter_confirms_the_state_shown_on_screen, end_of_input_confirms_the_current_state, byte_key_source_parses_keys_and_escape_sequences, lone_escape_and_truncated_sequences_are_ignored, interrupted_reads_are_retried, ctrl_c_maps_to_interrupted_exit_code_130, typed_tokens_do_not_change_activation, fake_mode_success_relies_on_the_real_key_source_and_restores, stty_failure_degrades_to_one_render_and_line_confirmation, degraded_path_never_activates_from_typed_tokens, activation_from_the_list_reaches_the_generated_config, legacy_typed_token_prompt_text_is_absent.

Manual TTY verification (script-allocated PTY, scratch project under .tmp, cleaned up):
- Arrow/j/k + Space + Enter: adr and bdr written enabled: false; exit 0.
- Ctrl-C after the list rendered: exit 130, papyrus.config.yml absent, no artifact directories created.
- stty -g captured before the run and after exit was byte-identical: terminal mode restored.
<!-- SECTION:NOTES:END -->
