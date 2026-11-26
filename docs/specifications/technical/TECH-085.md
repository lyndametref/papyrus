---
id: TECH-085
title: "Project Diagnostics (`papyrus check`)"
created_date: '2026-11-25T20:42:10Z'
updated_date: '2026-11-26T20:35:05Z'
labels: []
references:
  - REQ-001.008
status: verified
---
# TECH-085 Project Diagnostics (`papyrus check`)

## Description
Covers `papyrus check` only: scanned surfaces, findings and severity, configuration-finding locations, invalid configuration, repair, and exit status.

## Surface

- `check` scans the corpus and the configuration and prints a plain report.
- It loads the configuration tolerantly, as `papyrus config` does, so an invalid configuration does not block it; problems become findings.

## Checked Findings

- Duplicate record identifiers across task, milestone, release, decision, and every configured document type.
- Self-referential task dependencies and dependency cycles among tasks.
- Dangling references: a checked reference value resolving to no existing managed record.
- Unclaimed records: a Markdown file with managed record metadata that belongs to no configured document type, e.g. after a type was removed from the configuration.
- Configuration validity, including document-type entries and field specifications.

### Checked Values

A task's `milestone` and dependencies; a milestone's `release`; every record's `references` list; the values of every list-valued field restricted to record identifiers; a decision's `superseded_by`.

### Resolution

One identifier space spans all categories, so a record may reference a record of any other category. A reference to an archived record resolves and is not reported. A reference to a deleted record, a never-existing record, or a record whose type is no longer configured does not resolve and is reported.

## Findings and Severity

- A finding carries a severity, a kind, a subject naming the affected record or location, and a detail.
- Severity is `error` or `warning`. An invalid value of a known attribute is an error; an unknown attribute is a warning and is ignored by commands.
- The plain report and the JSON form carry the same findings and fields.

## Configuration Findings

- The location is the selector path `document_types.<slug>.<path>`, with field paths in keyed `fields[<key>].heading` or indexed `fields[<index>].heading` form. `<key>` is a field key declared by the type, not a reserved name; the field key is named when present.
- A selector path that cannot resolve uniquely — unknown key, missing or duplicate field key, out-of-range index — is itself reported, never skipped.

## Invalid Configuration

- `check` reports the configuration errors and still runs the checks that do not need the document-type set: duplicate identifiers among task, milestone, release, and decision records, self-references, dependency cycles, and dangling `milestone` and `release` references.
- Duplicate document identifiers and unclaimed records are skipped, and the report names each skipped check.

## Repair (`--fix`)

- `--fix` repairs duplicate identifiers and nothing else.
- Keeper order: earliest valid `created_date`; an undated record ranks after every dated record; a remaining tie takes the lexicographically smallest project-relative path.
- Every other record carrying the identifier receives a new identifier from the regular allocator, and its file is renamed to the new identifier's stem.
- `--fix` does not rewrite references; the prompt lists, per planned reassignment, every affected reference with the referring record and field.
- One confirmation covers the whole plan, and no repair is applied without it or `--force`. `--force` without `--fix` is a usage error.
- Without a terminal and without `--force`, `--fix` writes nothing and exits with a usage error naming `--force`.
- Immediately before a repair, `--fix` re-reads the file; a file changed after the scan skips the repair, reported as repair-skipped, and the duplicate finding stays reported.
- Dangling references, dependency cycles, self-references, configuration findings, and unclaimed records are never repaired.
- `--fix` does not migrate folders or rewrite any record beyond a duplicate-identifier reassignment.

### Refusals

`--fix` refuses a duplicated identifier, leaving it reported and changing no record of that identifier, when the file stem and front-matter `id` disagree, when the record is a group record of a grouped requirement family, or when the keeper or any record carrying it is in the import archive directory.

## Outcome

Exit status is 4 when at least one error-level finding exists, including a run where `--fix` repaired every repairable error, and 0 otherwise. Warnings alone do not change the status. `--json` renders every finding and severity as JSON. The command is command-line only; no browser affordance runs it.
