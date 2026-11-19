---
id: TASK-191
title: "Refine multiline stdin input implementation"
created_date: '2026-11-18T21:13:37Z'
updated_date: '2026-11-19T05:46:42Z'
labels:
  - bug
references:
  - RE-007
  - MS-029
  - TECH-078
  - REQ-07.003
  - REQ-07.004
  - REQ-04.001
  - REQ-04.002
  - REQ-02.001
  - REQ-02.002
  - REQ-03.001
  - REQ-01.004
  - NFR-02.002
  - BR-001
  - BR-005
  - BR-006
  - BR-008
  - TC-040
  - TC-041
  - TC-042
  - TC-043
  - TC-044
  - TC-045
  - TC-046
  - TC-047
  - TC-048
  - TC-049
  - TC-050
  - TC-051
  - TC-052
  - TC-053
  - TC-054
  - TASK-204
status: Done
assignee:
  - task-refiner
milestone: MS-029
dependencies:
  - TASK-190
ordinal: 2
documentation:
  - src/document.rs
  - src/cli/dispatch.rs
  - src/sanitize.rs
  - src/task.rs
  - src/milestone.rs
  - src/decision.rs
  - src/registry.rs
  - src/cli/tree.rs
  - README.md
  - tests/cli_help_golden.rs
  - tests/golden/cli-help/
type: refinement
---

## Description
Outcome: fix RE-007 so document-type, task, milestone, and decision `create`/`edit` commands accept multiline body/content text supplied on standard input, without weakening single-line validation. This refinement fixes the stdin contract for TASK-192; the approved test cases are TC-040 through TC-054 (TASK-190, Done).

## Scope
- Document types: `--description`/`-d` on `create` and `edit` for every document type that declares it - the generic config-defined model and the typed types (req, nfr, rule, glossary, audit, tech-spec, use-case, test-case, test-report, ui-spec, ui-mockup).
- Backlog: `--description`/`-d` on `task create`/`task edit` and `milestone create`/`milestone edit`.
- Decisions: `--decision`/`-d`, `--context`, and `--consequences` on `decision create`/`decision edit`.
- No other field accepts stdin multiline input. Titles, IDs, labels, references, documentation, assignee, milestone, depends-on, acceptance criteria, comments, type, ordinal, status, release, superseded-by, group, story, term, view, and applicability remain single-line and keep rejecting embedded newlines with exit code 4.

## Stdin contract (normative)
1. Sentinel: an accepting option selects standard input only when its value is exactly one ASCII hyphen `-`, for example `-d -`, `--description -`, `--description=-`, or `--context -`. No other value triggers reading, and no new flags are added, so the baseline command family stays as documented in REQ-07.003 and BR-001.
2. Opt-in only: when an accepting option is absent or holds a literal value, standard input is never read (TC-052). `--editor` remains the only other body-editing path and keeps its current behavior (TC-053).
3. Read once per invocation: read standard input to end-of-file as bytes and wait for EOF (Ctrl-D on a terminal). Exactly one accepting option may carry the sentinel per invocation; two or more is a validation error (exit code 4) naming the conflicting options, raised before reading standard input or writing anything.
4. Decoding: the input must be valid UTF-8; invalid bytes are a validation error (exit code 4).
5. Line endings: CRLF and lone CR normalize to LF.
6. Boundary: strip leading and trailing line terminators only; internal blank lines, internal newlines, and per-line spaces are preserved verbatim (TC-041). Input that is empty or whitespace-only after boundary stripping is a validation error (exit code 4).
7. Characters: LF is the only permitted control character; every other C0/DEL/C1 control, including TAB, is rejected with exit code 4, matching the existing single-line criterion (NFR-02.002).
8. Precedence and ordering: the resolved stdin text is the value of that option and follows the same code path as a literal value. On edit, field updates including the stdin value are applied before `--editor` opens, preserving today's order. On create, standard input is read and validated before an ID is allocated or a file is written.
9. Failures (empty or whitespace-only input, invalid UTF-8, multiple sentinels) exit 4 and leave the project unchanged: no ID allocated and no record created or modified.
10. Success output and `--json` shapes are unchanged from the literal-value path.

## Help and documentation
- Every accepting option's `--help` text states that multiline text can be supplied on standard input and identifies the option. Use the exact sentence: `Use "-" to read multiline text from standard input.` (TC-054).
- README.md manpage entries for the affected create/edit commands gain the same note, keeping BR-001, BR-005, BR-006, and BR-008 behavior documentation consistent.
- Regenerate the frozen help snapshots with `PAPYRUS_UPDATE_CLI_HELP=1 cargo test --test cli_help_golden`.

## Unchanged behavior
- Literal single-line arguments, their exact stored value, and the existing sanitization rules.
- The editor path.
- Exit codes (REQ-01.004): 0 success, 4 validation.
- All non-accepting fields.

## Assumptions
- MS-029 plans no architecture, technical-specification, or UI task: this is a CLI input-handling bug with no record-schema, storage, or UI change. TECH-078 is the roadmap entry that references RE-007; TC-040 through TC-054 are the approved verification design.
- No existing workflow depends on storing the literal value `-` as a description; the sentinel intentionally claims that exact value.
- TAB stays rejected because NFR-02.002 rejects control characters rather than cleaning them and this request only adds line breaks; no new escape mechanism is introduced.
- The clap lone-hyphen value works without `allow_hyphen_values`, verified against the current binary: `-d -` is accepted as an option value.
- At most one stdin value per invocation is sufficient; `decision` is the only command with multiple accepting options.

## Code locations and traceability
- Contract consumers: `src/document.rs` (shared document invocation plus generic create/edit), `src/cli/tree.rs` (document invocation parsing), `src/task.rs`, `src/milestone.rs`, `src/decision.rs`.
- Shared read/validate helper and builtin wiring: `src/sanitize.rs`, `src/cli/dispatch.rs`.
- Help declarations: `src/registry.rs`, `src/task.rs`, `src/milestone.rs`, `src/decision.rs`; frozen help snapshots in `tests/cli_help_golden.rs` and `tests/golden/cli-help/`.
- Documentation: `README.md`.
- Regression tests: TC-040 through TC-054 exercised through the CLI with piped stdin and/or unit helpers.

## Definition of done
- TASK-192 implements this contract for every listed command and field and satisfies all six acceptance criteria.
- Automated regression tests cover TC-040 through TC-054 and the contract edge cases: empty stdin, invalid UTF-8, multiple sentinels, CRLF normalization, and a literal option value not reading stdin.
- `cargo test` passes, including the regenerated help goldens.
- No open material questions remain.
## Acceptance Criteria
- [ ] AC-1: Document-type `create` and `edit` accept multiline `--description` or `-d` supplied on stdin when the option value is exactly `-`, for the generic model and every typed document type that declares `--description`, and `decision create` or `decision edit` accepts multiline `--decision` or `-d`, `--context`, and `--consequences` the same way; stored lines and internal blank lines are preserved in order (TC-040, TC-041, TC-042, TC-043, TC-044). Empty or whitespace-only stdin, invalid UTF-8, and more than one stdin sentinel in one invocation exit 4 with no write.
- [ ] AC-2: `task create` or `task edit` and `milestone create` or `milestone edit` accept multiline `--description` or `-d` supplied on stdin via the `-` sentinel; the stored description preserves line order and internal blank lines (TC-045, TC-046, TC-047, TC-048).
- [ ] AC-3: Titles, IDs, and every non-accepting field (labels, references, documentation, assignee, milestone, depends-on, acceptance criteria, comments, type, ordinal, status, release, superseded-by, group, story, term, view, applicability) still reject embedded newlines with exit 4 and leave records unchanged (TC-049, TC-050, TC-051).
- [ ] AC-4: A literal single-line option value is stored exactly as today and does not read stdin; the `--editor` path is unchanged (TC-052, TC-053); no new flags are added to the baseline command family.
- [ ] AC-5: Automated regression tests cover TC-040 through TC-054 plus empty stdin, invalid UTF-8, multiple sentinels, CRLF normalization, and the no-implicit-stdin case; `cargo test` passes and the help goldens are regenerated.
- [ ] AC-6: `--help` for every affected create and edit command documents the stdin path and the accepting fields using the sentence `Use "-" to read multiline text from standard input.`, and the README.md manpage entries are consistent (TC-054).
## Comments
- **task-refiner**: Refinement complete. Fixed stdin contract: exact `-` sentinel on accepting options, one stdin read per invocation to EOF, UTF-8 required, CRLF to LF, trim outer line terminators only, internal blank lines preserved, LF the only allowed control character, empty or whitespace-only input and multiple sentinels exit 4 with no write, no implicit stdin read, and --editor still runs after field patches. Accepting fields: --description on document, task, and milestone create and edit; --decision, --context, and --consequences on decision create and edit. No new flags. Help uses the sentence: Use "-" to read multiline text from standard input. Prerequisites: TASK-190 Done with TC-040 through TC-054 approved; MS-029 plans no architecture, technical-specification, or UI task because this is a CLI-only input fix. Implementation contract is in the description for TASK-192.
- **task-refiner**: Contract extension for MS-029 AC-1: audit options content and glossary options definition also accept the exact hyphen sentinel and read multiline text from standard input under the same contract. Implemented by TASK-204.
