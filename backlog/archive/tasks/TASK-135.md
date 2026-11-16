---
id: TASK-135
title: >-
  Document `papyrus check` in the README and business rules BR-001, BR-005, and
  BR-016
status: Done
assignee:
  - @tech-writer
created_date: '2026-11-15T16:12:00Z'
updated_date: '2026-11-15T18:59:00Z'
labels:
  - docs
  - cli
  - business-rules
milestone: MS-022
dependencies:
  - TASK-134
references:
  - README.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/specifications/project-diagnostics-check.md
  - tests/golden/cli-help/check.json
  - docs/business-rules/index.html
  - tests/document_type_no_drift.rs
modified_files:
  - README.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/index.html
priority: high
ordinal: 41530
---


## Description
**Why.** `papyrus check` is CLI-only and README is the behaviour source for the command (BR-001, BR-014), yet nothing documents it. The tolerant-surface change also alters the documented configuration and command contracts in BR-001, BR-005, and BR-016.

**What.** Document the command and re-align the business rules that quote the command surface.

**In scope.**
- README `COMMANDS` section gains `check`: synopsis, `--fix`, `--force`, output shape, finding kinds, exit statuses (0/4/1/2), the tolerant configuration behaviour, the read-only guarantee, and the intended `--fix` then clean-second-run loop.
- State that `--force` is meaningful only with `--fix` and that a non-TTY run without `--force` writes nothing and exits 2.
- BR-001: record `check` as a built-in command outside the document-family command set, add exit 4 as a findings outcome that is not an error, and note the read-only default.
- BR-005: re-sync the quoted README excerpt range if the new section shifts it; change no task behaviour text.
- BR-016: list `check` among the tolerant repair surfaces that run while the configuration is invalid, and add it to the reserved built-in names that a document-type slug may not take.
- Keep requirement files untouched; report any wording conflict with REQ-01.008 or REQ-07.002 instead of editing them.
- Regenerate the business-rules index if the change touches indexed headings.

**Out of scope.** Specifications (TASK-148), ADRs, requirement text, release notes, and man-page content.

**Assumptions.** ADR-0021 (man page) is still Proposed, so README remains the documented contract. BR-005 needs only range re-synchronisation, not new behaviour.

## Acceptance Criteria
- [x] README `papyrus(1)` -> COMMANDS contains a `check` entry that states the synopsis `papyrus check [--fix] [--force]`, both flags and their meaning, the seven finding kinds, the text and `--json` report shapes, the read-only default, the tolerant configuration load with named skipped checks, and exit statuses 0/4/1/2, with every statement consistent with `docs/specifications/project-diagnostics-check.md` section 1.1 and `tests/golden/cli-help/check.json`.
- [x] README states the `--fix` then clean-second-run loop, exactly one confirmation prompt per run, `--force` skipping that prompt and requiring `--fix`, and that a non-terminal run with repairs to apply and no `--force` writes nothing and exits 2.
- [x] BR-001 records `check` as a built-in command outside the config-defined document-command family, records exit 4 as a findings outcome that is not an error, and states that `papyrus check` is read-only unless `--fix` is given.
- [x] BR-005's quoted `task` excerpt heading names the actual README line range of the `task` section after the edit, and the quoted block is otherwise unchanged, with no task behaviour text edited.
- [x] BR-016 lists `check` among the tolerant repair surfaces that run while the configuration is invalid and among the reserved built-in names a document-type slug may not take.
- [x] No file under `docs/requirements/`, `docs/architecture/`, or `docs/specifications/`, no ADR, and no release note is edited; any wording conflict with REQ-01.008 or REQ-07.002 is reported in the ticket instead of edited.

## Definition of Done
- [x] #1 The `check` entry sits between the `search` entry and the `STATUSES` heading, and no other COMMANDS entry, the EXIT STATUS section, or the man-page scope note changed.
- [x] #2 Every statement in the `check` entry matches `docs/specifications/project-diagnostics-check.md` section 1.1 and `tests/golden/cli-help/check.json`: finding-kind names, flag semantics, one prompt per run, non-terminal refusal, tolerant load with named skips, and exit statuses 0/4/1/2.
- [x] #3 The README line that names `deprecated/` uses inert wording, so `cargo test --test document_type_no_drift` (D-06) passes, together with `cargo test --test cli_help_golden` and `cargo test --test config_list_keys_cli`.
- [x] #4 BR-005's heading names the measured `task` section range and its quoted block is byte-identical to the pre-edit block.
- [x] #5 `docs/business-rules/index.html` is regenerated with the bundled script and its diff contains only the BR-001 and BR-016 applicability text; the file was not hand-edited.
- [x] #6 `git diff --stat` touches only README.md, BR-001, BR-005, BR-016, and docs/business-rules/index.html, and `git diff` under tests/, docs/specifications/, docs/requirements/, and docs/architecture/ is empty.
- [x] #7 Every README line-range reference under docs/business-rules/ was re-measured; drift outside the edited files (BR-003, BR-004, BR-015) is recorded in the notes rather than edited, and the README still ends with the AI notice `*This project was created with AI assistance.*`.


## Implementation Notes
**Mode.** Documentation-only. No production code, test, specification, ADR, requirement, release-note, or man-page source change.

**Ownership split** (both parts must land in one branch so the index is regenerated once):
- `@tech-writer`: steps 1-3 (README `check` entry, README AI notice, BR-005 heading range).
- `@business-analyst` (orchestrator delegates): steps 4-6 (BR-001, BR-016, `docs/business-rules/index.html` regeneration).

**Verified baseline (2026-11-15).**
- README `papyrus(1)` -> COMMANDS headings: init 69, config 85, browser 110, import 128, task 153, milestone 273, release 326, decision 358, search 491; `STATUSES` heading 506.
- BR-005 quotes `README.md` lines 109-228. The real `task` section is lines 153-271 (heading line through the last entry line, i.e. two lines before the `milestone` heading). The quoted block body already matches the README text, so only the range is stale.
- BR-003 (config, quoted 71-93), BR-004 (browser, quoted 94-108), and BR-015 (search, quoted 692-706) are stale too. They are outside this task's file scope: report, do not edit.
- README line 918 is the AI notice `*This project was created with AI assistance.*`; it is present and must stay the last line.
- `docs/business-rules/index.html` is in sync with the 20 BR files today (the bundled generator produces no diff).

**Pinned insertion point.** The new `check` entry goes immediately after the `search` entry: after the `--limit <n>` term of `search` ("Maximum number of results.") and before the `STATUSES` heading.
- Primary reason: README orders entries by role - project setup (init, config), server/import, record families (task, milestone, release, decision), then cross-cutting commands (search). `check` is a cross-cutting read-only diagnostic over the whole corpus, so it joins `search`.
- Secondary reason: no quoted excerpt range in BR-003, BR-004, or BR-015 moves, so no out-of-scope rule gains new drift.
- Rejected alternative: after `config`, mirroring the frozen `papyrus --help` order (init, config, check, ...). README does not follow the help order today, and that position shifts the browser and task sections, adding drift to three out-of-scope rules.
- Consequence: no quoted range shifts because of this insertion. BR-005's range is still corrected (step 3) because it is already stale.

**Step 1 - README.md: add `papyrus(1)` -> COMMANDS -> `check`.**
Markup: section heading at column 0 as `<b style="color: light-dark(#0969da, #58a6ff);">check</b>`; body indented 7 spaces; option terms indented 8; synopsis inside `<code style="font-weight: bold;">`; spacing follows the existing `search` entry. Required blocks, in this order:
1. Summary - `Run read-only project diagnostics.` plus one sentence: the run scans every managed record family (task, milestone, release, decision, and every configured document type) and the configuration and reports integrity findings; CLI-only, no browser route.
2. Synopsis - `papyrus check [--fix] [--force]`, then one line stating that `--json`, `--dir`, `--config`, and `--no-color` are the global options (GLOBAL OPTIONS).
3. Options - `--fix`: "Repair duplicate record IDs." `--force`: "Skip the repair confirmation prompt; requires `--fix`."
4. Report shape - one finding per line `<kind>: <family> <subject> - <detail> (<locations>)`, sorted by kind, family, subject, and location; a clean run prints exactly `no findings`; when a check was skipped a final `skipped: <identifiers>` line follows; `--json` emits one object with the keys `findings`, `skipped`, and `repaired`.
5. Finding kinds - indented term list, one line per kind, exact identifiers: `duplicate-id`, `self-dependency`, `dependency-cycle`, `dangling-milestone`, `dangling-release`, `frontmatter-conflict`, `config-invalid`. State that `config-invalid` carries no family and no subject and its location is the configuration file.
6. Read-only guarantee - nothing is written unless `--fix` is given; a plain run renames, relocates, and rewrites nothing; a `deprecated/` folder stays inert: never read, never listed, no finding (REQ-05.007). The word "inert" (or another allowlisted phrase: not read, legacy, recover, follow, applies to) is mandatory, because `tests/document_type_no_drift.rs` D-06 rejects a README line that names `deprecated/` without inert wording.
7. `--fix` - repairs duplicate record IDs only; the keeper is the record with the earliest valid `created_date`, records with a missing or unparsable date rank last, and a remaining tie goes to the smallest project-relative path; every other record gets the next ID from the allocator and its file is renamed to the new stem; the plan and the single confirmation prompt go to standard error so `--json` stays parseable; affected references are listed and never rewritten; `--force` skips the prompt and requires `--fix`; a planned repair that cannot be applied is reported as `repair-skipped`; a non-terminal run with a non-empty plan and no `--force` writes nothing and exits 2.
8. Tolerant configuration - the configuration is loaded like `papyrus config get|set|unset|list`, so an invalid configuration does not block the run: each diagnostic line becomes a `config-invalid` finding, the checks that need a valid `document_types` set are named as skipped, and the remaining checks still run.
9. Intended loop - run `papyrus check` read-only, inspect the report, then `papyrus check --fix` (interactive) or `papyrus check --fix --force` (unattended), then re-run `papyrus check`: the report is `no findings`, the exit status is 0, and a second `--fix` writes nothing.
10. Exit status - 0 nothing found; 4 at least one finding, including a run whose repairs succeeded, a cancelled run, and a skipped repair; 1 I/O failure; 2 usage error (a parse error, `--force` without `--fix`, the non-terminal refusal). State that 4 here is a findings outcome, not an error.
Do not edit the shared EXIT STATUS section, the other COMMANDS entries, or the scope note in the `## Man Page` intro.

**Step 2 - README.md: AI notice.** Re-check after the insertion that the final line is still `*This project was created with AI assistance.*`; add it only if it is missing.

**Step 3 - BR-005: range only.** Replace the heading line `### papyrus(1): task — `README.md` lines 109–228` with `### papyrus(1): task — `README.md` lines 153–271` (keep the existing em dash and code spans). Re-measure before writing: `grep -n '">task</b>$' README.md` gives the first line and the next `">milestone</b>$` heading gives the boundary; the range runs from the `task` heading line through two lines before the `milestone` heading. The quoted block stays byte-identical: no task behaviour sentence changes.

**Step 4 - BR-001 (business-analyst).**
- Description: append one sentence - "Alongside these config-defined families, Papyrus ships a fixed set of built-in commands documented in the man page; no document slug can shadow a built-in name (BR-016)."
- `### Command model`: insert three bullets after the global-options/exit-codes bullet and before the "former static document command inventory" bullet:
  - "`check` is a built-in command outside the config-defined document-command family: it takes no subcommands, is never created from a `document_types` entry, and its `--fix` and `--force` flags are command-local, not global."
  - "`papyrus check` is read-only by default: a run writes nothing unless `--fix` is given."
  - "Exit code 4 is a findings outcome, not an error: `papyrus check` exits 4 when the run reports at least one finding, including a run whose `--fix` repairs all succeeded, and 0 only when it found nothing."
- Do not change the document-family, category, or lifecycle text.

**Step 5 - BR-016 (business-analyst).**
- `### Validation`, Fail-closed bullet: add `check` to the exception list - "every command except the repair surfaces (`config get|set|unset|list`), `init --force`, and `check` refuses to run while the effective configuration is invalid".
- `### Validation`, Reserved-names bullet: insert `check` after `config`, matching the REQ-07.002 order: `init`, `config`, `check`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, `version`.
- `### Repair surfaces`: add one bullet - "`papyrus check` loads tolerantly, reports each configuration diagnostic as a `config-invalid` finding, runs only the checks that do not depend on the effective `document_types` set, and repairs no configuration value (REQ-01.008)."
- Do not change precedence, key inventory, or `document_types` text.

**Step 6 - regenerate the business-rules index.** Run `./.opencode/skills/business-rules-index-visualization/scripts/generate_business_rules_index.sh` from the project root. Expect `wrote docs/business-rules/index.html: 20 business rules`. Never hand-edit the file. The resulting diff must contain only the BR-001 and BR-016 applicability text.

**Step 7 - verification.**
- `git diff --stat` - only README.md, BR-001, BR-005, BR-016, and docs/business-rules/index.html.
- `cargo test --test document_type_no_drift --test cli_help_golden --test config_list_keys_cli` - the README wording guard, the frozen help goldens, and the README config-list assertions still pass.
- `git diff tests/ docs/specifications/ docs/requirements/ docs/architecture/` - empty.
- Re-measure every `README.md` line-range reference under `docs/business-rules/` and record drift outside the edited files in the notes.

**Conflicts to report, not edit.**
- REQ-01.008 says `--fix` "prompts for confirmation before applying each repair" (and an AC says "each repair prompts for confirmation"). Spec section 10.8 and the frozen help text define exactly one prompt for the whole run. README documents one prompt; the requirement wording needs its owner.
- REQ-01.008's refusal list names a read-only legacy directory; spec section 5.1 additionally refuses a member in the read/import directory, and section 10.9 adds the `repair-skipped` finding (permitted by ADR-0029 constraint 5). README follows the spec; the requirement is silent rather than contradictory.
- Pre-existing stale README ranges in BR-003, BR-004, and BR-015 (see baseline). Out of scope here; needs a follow-up ticket.


## Notes
README papyrus(1) gains a check entry (synopsis, --fix/--force, scanned finding kinds, refusals, repaired:/reference: report lines, exit 0/4/1/2, tolerant configuration, read-only default, --help availability). BR-001 records check as a built-in outside the document family and exit 4 as a findings outcome; BR-016 lists check among tolerant repair surfaces and reserved names; BR-005 range corrected to 153-271 with the quoted block unchanged; business-rules index regenerated. Quality review: pass, all 6 AC and 7 DoD verified against the shipped binary (920 tests pass). Follow-ups recorded, not ticketed: stale README ranges in BR-003 (85-108), BR-004 (110-126), BR-015 (491-505).


## Comments
author: orchestrator
created: 2026-11-15T18:59:00Z

---

Provenance note for commit 83f79fd: the REQ-01.008 edits between this task's commits came from an orchestrator-dispatched requirements-engineer correction of wording defects found during TASK-137/138 review (one prompt per run, read/import refusal class, repair-skipped, keeper tie-break, --force). They are a separate documentation work item, not part of TASK-140, and the requirement now matches README, BR-001, BR-016, and the shipped binary.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
papyrus check is documented where the product's behaviour contract lives: the man page entry covers the synopsis, both flags, the scanned finding kinds plus the repair-run repair-skipped kind, the --fix refusal classes, the repaired:/reference: report shape, exit statuses 0/4/1/2, tolerant configuration loading with named skips, the read-only default, the help surface being available under a broken configuration, and the --fix then clean-second-run loop. BR-001, BR-016, and BR-005 were re-aligned so the rules that quote the command surface stay truthful, and the business-rules index was regenerated from the sources.
