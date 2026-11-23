---
id: TASK-133
title: Implement `check --fix` duplicate-ID repair with refusals and confirmation
status: Done
assignee:
  - @rust-dev
created_date: '2026-11-15T16:12:00Z'
updated_date: '2026-11-15T18:04:00Z'
labels:
  - rust
  - cli
  - diagnostics
  - tests
milestone: MS-022
dependencies:
  - TASK-132
references:
  - docs/requirements/business/REQ-001.008.md
  - >-
    docs/architecture/adr/ADR-0029-read-only-corpus-diagnostics-with-duplicate-id-repair.md
  - src/record/store.rs
  - src/record/identity.rs
  - src/record/patch.rs
  - docs/specifications/project-diagnostics-check.md
priority: high
ordinal: 41510
---


## Description
**Why.** REQ-001.008 and ADR-0029 decision 7 settle `--fix` as a duplicate-ID-only repair that goes through the single write path (ADR-0013 decision 7). No ID-rewrite primitive exists today, and the repair must be safe for identity uniqueness while never rewriting a reference.

**What.** The `--fix` and `--force` flags on `check`, the deterministic keeper rule, ID reassignment through the allocator, the refusal rules, the confirmation prompt, the per-repair concurrency re-check, and the tests that pin the accepted semantics.

**Contract.** `docs/specifications/project-diagnostics-check.md` section 10 is the implementation contract (10.1 flags, 10.2 plan input, 10.3 keeper, 10.4 allocation, 10.5 write primitive, 10.6 refusals, 10.7 references, 10.8 prompt and terminal gate, 10.9 concurrency, 10.10 report, 10.11 exit codes, 10.12 verification). Sections 1-9 remain the read-only contract landed by TASK-137. REQ-001.008 and ADR-0029 decisions 7-9 are the authority behind them.

**In scope.**
- `BuiltinCmd::Check { fix, force }`; `--force` declares clap `requires = "fix"`, so `--force` without `--fix` is a usage error with exit 2 before any scan or write.
- Keeper rule: the member with the earliest valid `created_date` keeps the ID; a member with a missing or unparsable `created_date` sorts after every member with one, and ties break on the lowest project-relative path. Keeper order also fixes reassignment order.
- Reassignment: `record::identity::allocate_next` over the scanned numbers of the record's own family, configured prefix, and ID shape, plus numbers allocated earlier in the same run; prefix and padding come from the configuration; the new stem is exactly the new ID (directory and extension kept, cosmetic slug dropped).
- New `record::store::reassign_id(ctx, path, new_id) -> Result<PathBuf>`: no front matter means a `store::move_record` rename with unchanged bytes; with front matter it patches `id`, `updated_date` once, and the heading ID only when the heading agrees with the front-matter `id`, writes the new path first, then removes the old path. New `PatchOp::SetHeadingId`.
- Refusals, reported and left untouched, with no file of the group written: a member whose front-matter `id` and stem disagree; a member in a read-only legacy or read/import directory; a grouped-family group ID. No partial group repair; other groups are still repaired.
- No inbound reference rewriting. The prompt and the report list every affected entry from the `milestone`, `release`, `dependencies`, `references`, and `documentation` fields of every scanned record (family, record ID, path, field, raw value), excluding the reassigned record's own file.
- One confirmation prompt per run over all planned repairs on stderr, so stdout carries only the report; `y`/`yes` confirms, anything else, an empty line, or EOF cancels with no write and exit 4; `--force` skips the plan block and the prompt.
- Non-terminal refusal: `--fix`, no `--force`, a non-empty plan, and stdin not a terminal writes nothing, prints `error: --fix requires --force when standard input is not a terminal` to stderr, prints nothing to stdout, and exits 2.
- Concurrency: the file is re-read before each repair and its `store::version_token` compared with the scan-time token; a changed file or an appearing target path is skipped and reported as a `repair-skipped` finding.
- Exit 4 whenever the scan found anything, including after successful repairs, a cancellation, or a skipped repair; 0 only for a clean scan; 2 for the two usage paths; 1 for I/O.
- Tests: keeper and tie-break, allocation of two members in one run, three refusal classes, reference listing before the write with the referring file byte-identical after it, `--force` bypass, non-TTY exit 2, `--force` without `--fix` exit 2, idempotence, the crash state (new and old file both present) repaired by the next run, `--fix --force` goldens for text and JSON.

**Out of scope.** Reference rewriting, `deprecated/` migration, reference or cycle or self-dependency or front-matter-conflict repair, new exit codes, a browser repair surface, the `check` slug reservation and help text (TASK-139), README and business-rule prose (TASK-140).

**Assumptions.**
- ADR-0029 decisions 7 to 9 are the contract; the prompt is std-only (`std::io::IsTerminal`, `BufRead`) and needs no terminal crate, no raw mode, and no `stty`.
- `--yes` from earlier requirement wording is superseded by `--force`.
- A record with no front matter (HTML mockups and pre-front-matter Markdown) is repaired by renaming its file, because its identity lives in the stem; the repair never adopts front matter.
- `repair-skipped` is a new finding kind, permitted as an addition by ADR-0029 constraint 5, rather than a separate report section.
- The non-terminal refusal fires only when the plan is non-empty. A run with nothing to repair never prompts, so it does not fail.
- The prompt path is unit-tested through an injected reader/writer seam (the `init::run_init_with_io` precedent); the CLI test can only exercise the piped-stdin gate, because a pipe is not a terminal. The interactive end-to-end path is verified manually.
- Goldens assert the ISO-8601 UTC shape of `updated_date`, not its value, because it is the clock at write time.

**Open questions.**
1. Should the non-terminal refusal fire even when the plan is empty? Implementation follows the non-empty-plan reading recorded in the contract; owner: requirements engineer.
2. Is `repair-skipped` acceptable as a finding kind rather than a separate `skipped_repairs` JSON array? Implementation uses a finding because ADR-0029 decision 9 says the skip is reported "with a finding"; owner: architect.

## Acceptance Criteria
- [x] `check --fix` plans only `duplicate-id` findings whose `fixable` is true; the report still lists every finding, the repaired duplicate included, and no other finding kind is repaired.
- [x] A duplicate group's keeper is the member with the earliest valid `created_date`; a member with a missing or unparsable `created_date` sorts after every member with one, and any remaining tie breaks on the lowest project-relative path.
- [x] Every other member of the group gets the next free number of its own family, configured prefix, and ID shape from `record::identity::allocate_next`, and its file is renamed to a stem that is exactly the new ID, in the same directory and with the same extension.
- [x] `record::store::reassign_id` patches `id`, `updated_date` once, and the body heading ID when the heading agrees with the front-matter `id`, writes the new path atomically before removing the old one, and is the only mutator the repair uses.
- [x] A record whose front-matter `id` and file stem disagree, a duplicate whose keeper or member is in a read-only legacy or read/import directory, and a grouped-family group ID are refused: no file of the group is written, the finding stays reported, and other groups are still repaired.
- [x] Inbound references are never rewritten; the prompt and the report list every affected `milestone`, `release`, `dependencies`, `references`, and `documentation` entry with referring family, record ID, path, field, and raw value, excluding the reassigned record's own file.
- [x] One confirmation prompt covers the whole run; `y` or `yes` confirms, any other answer, an empty answer, or end-of-input cancels with no write, `--force` skips the prompt, and `--force` without `--fix` is a clap usage error with exit 2.
- [x] When `--fix` has planned repairs, stdin is not a terminal, and `--force` is absent, nothing is written, stderr names `--force`, stdout is empty, and the run exits 2.
- [x] A repair whose file changed after the scan is skipped and reported as a `repair-skipped` finding, and a second `--fix` on a repaired corpus writes nothing and reports no duplicate.
- [x] A run that found findings still exits 4 after successful repairs, and a corpus left between the two write steps (new file present, old file present) stays readable and is repaired by the next run.

## Definition of Done
- [x] #1 The `--fix` and `--force` behavior of `docs/specifications/project-diagnostics-check.md` section 10 is implemented and matches REQ-001.008 and ADR-0029 decisions 7-9.
- [x] #2 `cargo test` passes, including the new unit, CLI, and golden tests.
- [x] #3 `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` pass.
- [x] #4 No new crate or dependency is added.
- [x] #5 `src/check/` contains no `fs::write` and no `fs::remove_file`, and the read-only bytes-and-mtime test still passes for a run without `--fix`.
- [x] #6 The specification is updated in this change (section 10, the status line, and the `family_files` signature); no requirements file and no ADR is edited.
- [x] #7 A plain `check` run leaves the TASK-137 goldens byte-identical.


## Implementation Notes
Contract: `docs/specifications/project-diagnostics-check.md` section 10. REQ-001.008 and ADR-0029 decisions 7-9 are the authority; neither is edited.

1. Flags and dispatch
   - `src/cli/builtin.rs`: `BuiltinCmd::Check { fix: bool, force: bool }`; `--fix` = "Repair duplicate record IDs.", `--force` = "Skip the repair confirmation prompt; requires --fix." with `#[arg(long, requires = "fix")]`. `--force` without `--fix` becomes a clap usage error (exit 2, no scan, no write); do not hand-roll the check and do not pin clap's wording.
   - `src/cli/dispatch.rs`: pass both flags to `check::run`.
   - `src/check/mod.rs`: `pub fn run(ctx: &Context, fix: bool, force: bool) -> Result<CommandOutput, AppError>`; with `fix == false` the output must stay byte-identical to the TASK-137 report.

2. Scan metadata for the repair (`src/check/scan.rs`)
   - `RecordMeta` gains `path: PathBuf`, `prefix: String`, `numbers: Vec<i64>`, `version: String`.
   - Fill them from the `(ParsedId, prefix)` already computed in `record_meta`, and capture `version` with `store::version_token(&fs::metadata(path)?)`. No second file read and no second corpus enumeration.

3. Keeper rule, allocation, plan (`src/check/fix.rs`, new)
   - Order a group: valid `created_date` ascending as UTC seconds, members with a missing or invalid date last, remaining ties on project-relative path ascending. First keeps the ID; the rest are reassigned in that order, which is also the plan and report order.
   - `src/record/frontmatter.rs`: add `pub fn timestamp_seconds(value: &str) -> Option<i64>` (inverse of `utc_from_unix`; accepts `Z` and `±HH:MM`). No crate.
   - Allocation: `identity::allocate_next` over the scanned numbers of the same family and case-insensitive prefix, plus numbers allocated earlier in the run. Plain record: `identity::format_id(prefix, n, cfg.id_padding)`. Member with two numbers: `identity::format_seq_id(identity::format_id(prefix, numbers[0], cfg.id_padding), n, cfg.id_padding)`.
   - New stem is exactly the new ID: drop a cosmetic `-<slug>` or ` - <slug>`, keep the directory and extension.
   - Plan input: only `duplicate-id` findings with `fixable == true`; a group is planned whole or refused whole.

4. Write primitive (`src/record/store.rs`)
   - `pub fn reassign_id(ctx: &Context, path: &Path, new_id: &str) -> Result<PathBuf, AppError>` plus the `Store` façade method.
   - No front matter (HTML mockups, pre-front-matter Markdown): `move_record` only; bytes unchanged.
   - With front matter: one `record::patch` apply of `SetFrontMatter("id", new_id)`, `SetFrontMatter("updated_date", frontmatter::utc_now())`, and `SetHeadingId(new_id)` only when the heading's first token equals the current front-matter `id`; then `write_atomic(ctx, new_path, &patched)`; then remove the old path.
   - `src/record/patch.rs`: add `PatchOp::SetHeadingId(String)` mirroring `SetHeadingTitle` (replaces the ID component of an existing `# <ID> <title>` heading, keeps the title, no-op when equal, never inserts a heading).
   - `src/check/fix.rs` holds no `fs::write`, no `fs::remove_file`, and no store mutator other than `reassign_id`; the only other store call is the read-only `version_token`.

5. Refusals and references
   - Never planned: `duplicate-id` with `fixable == false` (stem/front-matter mismatch, read-only legacy or read/import member, grouped-family group ID) and every other finding kind (dangling references, cycles, self-dependencies, front-matter conflicts including heading disagreements, invalid configuration, anything under `deprecated/`). Refusals do not abort the run; other groups are still repaired.
   - Reference candidates: fields `milestone`, `release`, `dependencies`, `references`, `documentation` of every scanned record; a scalar is one candidate, a list one per entry. Match when `identity::parse_id_parts` against the reassigned record's family prefix and shape yields the record's pre-repair canonical key. Skip the reassigned record's own path. Tuple = referring family, referring display ID, project-relative path, field, raw value as written; sort by `(family, subject, path, field, value)` and dedupe. Compute once before the writes.

6. Prompt, confirmation, terminal gate
   - Plan and prompt go to stderr; stdout carries only the report, so `--json` stays parseable.
   - Plan block, printed only when the plan is non-empty and `--force` is absent:
     ```
     fix plan: <n> reassignment|reassignments
     reassign: <family> <old display id> <old project-relative path> -> <new id>
     reference: <family> <referring id> (<project-relative path>) <field> = "<raw value>"
     apply repairs? [y/N]
     ```
     Singular `1 reassignment`; the prompt line ends with a newline and the stream is flushed before reading.
   - Read one line from stdin, trimmed and ASCII-lowercased: `y` or `yes` confirms. Any other line, an empty line, or EOF cancels: nothing written, stderr `repair cancelled; nothing written`, stdout the unchanged report with `repaired: []`, exit 4.
   - `--force` skips the plan block and the prompt entirely.
   - Gate: `fix && !force && !plan.is_empty() && !std::io::stdin().is_terminal()` prints `error: --fix requires --force when standard input is not a terminal` to stderr, nothing to stdout (no JSON object; a usage error like a clap error), and exits 2.
   - Structure the confirmation as an injected reader/writer seam (the `init::run_init_with_io` precedent) so the prompt path is unit-testable without a terminal.

7. Concurrency per repair
   - Before each repair re-read `fs::metadata(path)`; a token different from the scan token skips the repair and pushes a finding: kind `repair-skipped`, `family`/`subject` of the record, `locations` `[path]`, `fixable: false`, `detail` `record changed since the scan; repair skipped`, or `target path <path> already exists; repair skipped` when the new path appeared.
   - After the token matches, re-read the content and patch it. Skipped repairs are not `repaired` entries; the original `duplicate-id` finding stays in the report and the run keeps exit 4.

8. Report and exit codes
   - Text: finding lines unchanged (7.2), then `repaired: <family> <old id> -> <new id> (<old path> -> <new path>)` per applied repair in plan order, then the `reference:` lines of step 5, then the `skipped:` line when present.
   - JSON `repaired` entry: `{"kind":"duplicate-id","family":…,"id":<old id>,"path":<old path>,"new_id":…,"new_path":…,"references":[{"family":…,"subject":…,"path":…,"field":…,"value":…}]}` in that key order; `[]` for a read-only run, nothing to repair, a cancellation, or a skipped repair.
   - Exit: 4 when the scan found anything (fully repaired, cancelled, or skipped included); 0 only for a clean scan; 2 for `--force` without `--fix` and the terminal gate; 1 for I/O.

9. Tests
   - Unit `src/check/fix.rs`: keeper ordering (offset equivalence, tie on path, missing or invalid date last), two members of one group allocated sequentially in one run, plain versus member allocation, confirmation parsing (`y`, `Y`, `yes`, `n`, empty, EOF).
   - Unit `src/record/store.rs`: `id`, `updated_date`, and heading ID patched; heading untouched when it disagrees; no-front-matter record renamed with identical bytes; old path gone and the new path parsing to the new ID; unrelated bytes identical.
   - CLI `tests/check_cli.rs`: keeper and tie-break, three refusal classes, reference listing before the write with the referring file byte-identical after it, `--force` bypass, non-terminal exit 2, `--force` without `--fix` exit 2, idempotence (second `--fix` writes nothing and the second run is clean), exit 4 after a successful repair, the crash state (new and old file both present) repaired by the next run.
   - Goldens `tests/golden/check/`: a `--fix --force` text report and JSON object, with `updated_date` normalised to its ISO-8601 UTC shape rather than a fixed value.


## Notes
Implemented src/check/fix.rs (keeper rule, allocation, plan, references), record::store::reassign_id with PatchOp::SetHeadingId, frontmatter::timestamp_seconds, --fix/--force flags with clap requires, and an injectable prompt/stdin seam. Per-repair version re-check reports repair-skipped. Quality review: pass; 905 tests pass, clippy/fmt clean, manual matrix over keeper/tie-break, three refusal classes, reference listing, non-TTY exit 2, idempotence, crash-state recovery.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
check --fix repairs duplicate record IDs only, through record::store::reassign_id as the single write path: deterministic keeper (earliest valid created_date, path tie-break), allocator-assigned new IDs with the new stem equal to the new ID, atomic new-then-remove ordering, and whole-group refusals for stem/front-matter mismatches, read-only legacy members, and grouped-family group IDs. Inbound references are never rewritten but are listed in the prompt and the report. One confirmation prompt per run (y/yes), --force bypass, non-TTY without --force exits 2, changed files are skipped as repair-skipped, exit stays 4 after repair, and a second run is clean.
