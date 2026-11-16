---
id: TASK-094
title: Write the Release 1.0 upgrade notes for the one-time Backlog.md import
status: ToDo
assignee: []
created_date: '2026-11-11T20:44:00Z'
updated_date: '2026-11-13T15:35:00Z'
labels:
  - documentation
  - import
  - release-1.0
milestone: MS-015
dependencies:
  - TASK-078
  - TASK-087
references:
  - docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md
  - docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md
  - docs/requirements/non-functional/NFR-04.004.md
  - docs/specifications/import-backlog-md-records.md
  - docs/specifications/release-1.0-native-layout-verification-and-packaging.md
  - docs/specifications/browser-requirements.md
  - docs/packaging.md
  - docs/requirements/business/REQ-01.005.md
  - docs/requirements/business/REQ-01.004.md
priority: low
type: docs
ordinal: 2000
---


## Description
ADR-0010 decision 11 and ADR-0011 decision 9 make the upgrade notes a Release 1.0 gate: the release that stops recognising Backlog.md storage stems (`backlog/tasks/task-N - <slug>.md`, `backlog/milestones/m-N - <slug>.md`) must document the one-time import, because a project still holding such records loses sight of them the moment the new version runs. TASK-98 verified the upgrade path against an isolated fixture and recorded that the notes did not exist yet (`docs/packaging.md` → "TASK-99 upgrade notes unavailable at verification time"), so this ticket must also reconcile the published wording with that evidence.

This ticket writes the user-facing Release 1.0 upgrade notes and a change summary, and links them from `README.md`. It is documentation only: no product behaviour, requirement, business rule, ADR, or packaging script changes.

## Deliverables and paths

1. New file `docs/release-1.0-upgrade-notes.md` — the canonical upgrade notes and Release 1.0 change summary.
2. `README.md` — a short "Upgrading to Release 1.0" section that links the notes file (placed before the Man Page section, after "Inspiration").
3. `docs/specifications/import-backlog-md-records.md` — one bullet in §11 recording the published notes path and its README link, and TASK-99 added to the traceability Ticket cell. No behaviour text changes.
4. This ticket — plan, notes, and evidence.

## Required content of the notes

The notes must be self-contained and executable by a user who has never read an ADR, and must cover:

- **Who must act:** any project whose backlog records are still in the Backlog.md layout, including a project that only upgrades the binary. No automatic import runs, no transitional reader or dual-recognition mode exists, and the new version does not list, view, search, or render unimported records.
- **What is in scope:** `backlog/tasks`, `backlog/milestones`, `backlog/archive/tasks`, `backlog/archive/milestones`. `backlog/docs`, `backlog/drafts`, `backlog/archive/drafts`, `backlog/decisions`, `backlog/config.yml`, and `backlog/.locks` are not imported and are reported as excluded.
- **Procedure:** initialize/select the target project (it must have `papyrus.config.yml`), run `papyrus import backlog --from <source-project-dir> --dry-run`, review the printed plan and report, save the report with `--report <path-inside-project>`, then run the same command without `--dry-run`. A dry run imports no records; with `--report` it still writes that report file to disk, and without `--report` it writes nothing.
- **Options:** `--from`, `--source-backlog-dir`, `--dry-run`, `--allow-existing`, `--report`, plus a statement that global options (`--dir`, `--config`, `--json`, `--no-color`) apply as for other commands. Exit codes per REQ-01.004: `0` success (every in-scope record imported, or planned under `--dry-run`); `1` runtime failure, including refusal without `--allow-existing` and skipped records (the summary lists them); `2` usage error; `4` validation/conflict, e.g. `--report` resolving outside the project directory.
- **Renumbering:** tasks receive fresh `TASK-nnn` numbers continuing from the highest existing number in the target (active, archived, deprecated); milestones continue from the highest milestone number already present in the target, using `MS-001` only when the target holds no milestone records (ADR-0010 decision 5); no source number is preserved and no source ID is accepted by any later command. The printed plan and the optional report are the only record of the mapping — no mapping file, sidecar, marker key, or extra front-matter field exists.
- **Source handling and ownership:** the source is opened read-only and left byte-identical; keep it until the imported records are verified; after import Papyrus owns the records. There is no reverse export, no re-import reconciliation, and no sync.
- **One-way and non-idempotent:** Backlog.md file compatibility is renounced; `papyrus import backlog` is the only bridge; a second run against the same source allocates new numbers and duplicates records, with the plan and same-title warnings as the review mechanisms; `--allow-existing` is required when the target already holds records of an imported family, otherwise the command refuses before writing.
- **Known import caveats:** statuses that do not match the target vocabulary are preserved verbatim with a warning; dependency and milestone references that cannot be resolved are dropped with a warning; invalid timestamps are replaced by the import time and reported as approximate; a mid-import write failure aborts with a partial import and no rollback; `--report` must point inside the project directory.
- **Post-import verification checklist:** imported tasks and milestones are listed, viewable, and searchable by their new Papyrus ID; the source tree is unchanged; a second run without `--allow-existing` is refused.
- **Other user-visible changes in Release 1.0** (change summary): the shared single record view layout used by every record type; the editable browser configuration view, including `config set backlog_directory` relocating the store as one atomic move; non-functional requirements grouped under their owning group; the browser console quiet by default with request logging only under the browser-only `--debug` flag; and the removal of the `storage_id`, `display_id`, and `storage_path` fields from JSON output, routes, and links in favour of Papyrus IDs.

## Consistency requirements

- Every command line, option name, default, refusal rule, and exit code in the notes is checked against `papyrus import backlog --help` as shipped and against the recorded TASK-98 upgrade-path evidence in `docs/packaging.md`; a divergence is recorded in the ticket notes and escalated to TASK-73, never resolved by inventing behaviour.
- Where `docs/specifications/import-backlog-md-records.md` §1.6 and `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` decision 6 compress the outcomes to `0` success/planned and `1` runtime/validation failure, the notes follow the shipped four-code mapping (REQ-01.004: 0/1/2/4) and the stale specification/ADR wording is recorded in the ticket notes and escalated to TASK-73 rather than repeated.
- The notes make no claim of Backlog.md compatibility, no claim that IDs are preserved, and no promise of rollback or undo.

## Acceptance Criteria
- [x] `docs/release-1.0-upgrade-notes.md` exists, and `README.md` contains an "Upgrading to Release 1.0" section whose relative link to that file resolves.
- [x] The notes state up front who must act — every project still holding Backlog.md-layout records — and that the upgrade performs no automatic import, keeps those records on disk unrecognised by the new version, and offers no transitional reader or compatibility mode.
- [x] The notes name all four affected source directories (`backlog/tasks`, `backlog/milestones`, `backlog/archive/tasks`, `backlog/archive/milestones`) and state that `docs`, `drafts`, `decisions`, `config.yml`, and `.locks` are excluded.
- [x] The notes give the literal procedure: initialize or select a project with `papyrus.config.yml`, run `papyrus import backlog --from <source-project-dir> --dry-run`, review the printed plan and report, save it with `--report <path inside the project>`, then run the same command without `--dry-run`; they state that a dry run imports no records, that a dry run with `--report` still writes that report file, and that a dry run without `--report` writes nothing.
- [x] The notes contain a command reference whose options (`--from`, `--source-backlog-dir`, `--dry-run`, `--allow-existing`, `--report`), global-option statement, and exit codes match the shipped `papyrus import backlog --help` output and REQ-01.004: `0` success (imported, or planned under `--dry-run`); `1` runtime failure, including refusal without `--allow-existing`, and skipped records (listed in the summary); `2` usage error; `4` validation/conflict, for example `--report` resolving outside the project directory.
- [x] The notes state that task IDs are reallocated as fresh `TASK-nnn` numbers continuing from the target maximum, that milestones continue from the highest milestone number already present in the target using `MS-001` only when the target holds no milestone records (ADR-0010 decision 5), that no source number is preserved or accepted later, and that the printed plan and optional report are the only mapping record.
- [x] The notes state that the source store is opened read-only and left byte-identical, must be kept until the import is verified, and that Papyrus owns the records afterwards.
- [x] The notes state that Backlog.md file compatibility is renounced, that `papyrus import backlog` is the only bridge, that re-running against the same source duplicates records under new IDs, and that `--allow-existing` is required when the target already holds records of an imported family (refused before writing otherwise).
- [x] The notes document the import caveats: unmatched statuses preserved verbatim with a warning, unresolvable dependency or milestone references dropped with a warning, invalid timestamps replaced by import time and reported as approximate, a mid-import failure leaving a partial import with no rollback, and `--report` restricted to paths inside the project.
- [x] The notes include a post-import verification checklist: imported tasks and milestones are listable, viewable, and searchable by their new Papyrus IDs, the source tree is unchanged, and a second run without `--allow-existing` is refused.
- [x] The notes include an "Other user-visible changes in Release 1.0" section covering the shared single record view layout, the editable browser configuration view (including atomic `config set backlog_directory` relocation), non-functional requirement grouping, the quiet browser console with browser-only `--debug` request logging, and the removal of the `storage_id`, `display_id`, and `storage_path` fields in favour of Papyrus IDs.
- [x] The documented command, options, caveats, and exit codes were cross-checked against `papyrus import backlog --help` and the shipped exit-code mapping; the checked help output and any divergence are recorded in the ticket notes, with divergences escalated to TASK-73.
- [x] No requirement file, business rule, ADR, packaging script, or source file was modified; the only changed files are `README.md`, the new notes file, the import specification §11 location bullet and traceability cell, this ticket, and the escalation comment added to the TASK-73 ticket as required by AC12 — no other ticket is edited.

## Definition of Done
- [x] #1 `docs/release-1.0-upgrade-notes.md` is present, self-contained, and readable without any ADR or ticket; it covers every required content item in the description.
- [x] #2 The README "Upgrading to Release 1.0" section links the notes with a relative path that resolves in the repository.
- [x] #3 The notes' command reference and exit-code statements match the shipped `papyrus import backlog --help` output recorded in the ticket notes.
- [x] #4 The notes agree with the TASK-98 upgrade-path evidence in `docs/packaging.md`; any divergence is recorded and escalated to TASK-73 instead of being silently resolved.
- [x] #5 The change summary statements are traceable to the specifications and tickets named in the plan.
- [x] #6 The ticket notes record the help output used, the cross-check results, and any open items for TASK-73.
- [x] #7 No scratch files remain under `./.tmp/` after the work.
- [x] #8 Only README.md, docs/release-1.0-upgrade-notes.md, docs/specifications/import-backlog-md-records.md (§11 bullet + traceability cell), this ticket, and the TASK-73 escalation comment changed; no requirement, business-rule, ADR, packaging-script, or source-code change.


## Implementation Notes
1. Re-read the authoritative sources and extract only the user-facing facts: `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` (decision 11, decision 5 renumbering), `docs/architecture/adr/ADR-0011-one-way-backlog-md-import.md` (decisions 2–10), `docs/specifications/import-backlog-md-records.md` §11–§12, `docs/specifications/release-1.0-native-layout-verification-and-packaging.md` §2.2 and §6, and the TASK-98 evidence in `docs/packaging.md` (upgrade-path bullet and "TASK-99 upgrade notes unavailable at verification time").
2. Capture the shipped interface as evidence: run `papyrus import backlog --help` (and `papyrus --help` for global options) and record the exact usage line, option list, defaults, and refusal wording in the ticket notes; use this output as the only source for the option table.
3. Resolve the five other user-visible changes against their specifications so the change summary is factual: the shared single record view layout and NFR grouping in `docs/specifications/browser-requirements.md` (§4c, §4.10a) with TASK-93; the configuration view and atomic store relocation in TASK-94/`browser-configuration-view-single-record-layout-and-lifecycle-configuration.md`; the quiet console and browser-only `--debug` in `browser-requirements.md` §4.1/§4.12 with TASK-96; the removed storage-ID fields in `release-1.0-native-layout-verification-and-packaging.md` §2.2 and `remove-legacy-record-serializers-and-enforce-the-no-drift-rule.md` with TASK-97/98.
4. Write `docs/release-1.0-upgrade-notes.md` with these sections in order: "Who must act", "What the import covers" (included/excluded directories table), "Before you start" (target project with `papyrus.config.yml`, source kept, review mechanisms), "Procedure" (dry run → review → saved report → real run, with literal commands and expected output shape), "Command reference" (option table copied from the help output; exit codes from REQ-01.004 and the shipped `AppError` mapping, since the help text prints no exit codes), "What changes" (ID reallocation, milestone remapping, source read-only, ownership), "One-way behaviour and limits" (non-idempotence, `--allow-existing`, caveats, partial import), "Verify the import" (checklist), and "Other user-visible changes in Release 1.0".
5. Add the README section "Upgrading to Release 1.0" before the Man Page section: two or three sentences that state who must act, the one command, and a relative link to `docs/release-1.0-upgrade-notes.md`; do not duplicate the notes or change the existing man-page import entry.
6. Add the TASK-99 traceability row to `docs/specifications/import-backlog-md-records.md` (Artifact row: release upgrade notes; Ticket: TASK-99; at minimum extend the existing Ticket cell) without touching behaviour text.
7. Cross-check the finished notes mechanically: every backticked command and flag appears in `papyrus import backlog --help`; each of the four affected directories and the excluded paths appears; each claim in the post-import checklist was exercised by TASK-98's evidence; the README link resolves to an existing relative path.
8. Record the cross-check results and any divergence in the ticket notes, update the specification traceability, leave the ticket acceptance criteria checked with evidence, and run a final consistency pass on README, notes, and specification (no stale references, no duplicate or contradictory statements).


## Notes
Help output used (verbatim):\nImport tasks and milestones from a Backlog.md store\n\nUsage: papyrus import backlog [OPTIONS] --from <source-project-dir>\n\nOptions:\n  --from <source-project-dir>  Source project directory\n  --json                       Emit stable, machine-readable JSON\n  --dir <path>                 Project directory. Default: the current working directory\n  --source-backlog-dir <dir>   Source backlog directory (default: <from>/backlog)\n  --config <file>              Configuration file. Default: papyrus.config.yml in the project directory\n  --dry-run                    Plan and report without writing anything\n  --allow-existing             Allow import into a target that already holds records of an imported family\n  --no-color                   Disable ANSI color output\n  --report <path>              Write the report to a path inside the project directory\n  -h, --help                   Print help\n\nExit codes verified: 0 success/planned, 1 runtime/refusal/skipped, 2 usage, 4 validation. Divergence from import spec §1.6/ADR-0011 (1 runtime/validation) escalated to TASK-73.
