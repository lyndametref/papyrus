---
id: REQ-01.008
title: "Run Project Diagnostics (`papyrus check`)"
created_date: '2026-11-15T15:46:30Z'
updated_date: '2026-11-20T20:58:17Z'
labels: []
references:
  - RE-010
  - ADR-0019
  - ADR-036
---
# REQ-01.008 Run Project Diagnostics (`papyrus check`)

## User Story
As a project maintainer, I want to diagnose corpus and configuration integrity issues from the CLI so that problems can be found and, where safe, repaired before they affect other commands.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
`papyrus check` is a diagnostic command. It scans the project corpus and the configuration and prints a plain report of integrity findings. It is a tolerant repair surface: it loads the configuration the same tolerant way as `papyrus config get|set|unset|list` (REQ-01.002), so an invalid configuration does not block it, and it reports configuration findings instead of failing closed. The checked categories are:

- Duplicate record IDs across all managed record types (task, milestone, release, decision, and every configured document type).
- Self-referential task dependencies.
- Dependency cycles among task dependencies.
- Dangling references: a task whose `milestone` does not resolve to an existing milestone, and a milestone whose `release` does not resolve to an existing release.
- Front-matter/section conflicts, described below.
- Configuration validity (REQ-01.002, REQ-07.002, REQ-07.004), including invalid slugs, statuses, prefixes, directories, and duplicate slug keys; unknown top-level and field-item keys; unknown enum values (`kind`, `verbs`, `reference_visibility`); reserved field keys used as a field `key` or `frontmatter_key`; duplicate field keys within a role; a field with neither `heading` nor `frontmatter_key`; a field `order` or `enabled` key; and a `model`, `title`, `content`, or `id_shape` key.

A configuration finding names the offending location by the selector path `document_types.<slug>.<path>`, using the keyed `fields[story].heading` or indexed `fields[0].heading` form and naming the field key when one is present. A finding's selector path resolves to the offending leaf. A selector path that cannot resolve uniquely — an unknown key, a keyed selector that matches no unique field because the key is missing or duplicated, or an out-of-range field index — is itself reported as a configuration finding rather than silently skipped.

A front-matter/section conflict is a record that carries front matter and still retains a retired legacy metadata section for a field that front matter also carries, when the section's value disagrees with the front-matter value. The retired sections checked are `## Status`, `## Assignee`, `## Milestone`, `## Labels`, and `## Dependencies` on a task, and `## Release` on a milestone. Each section is read the way the legacy migration reads it — one non-empty line for `status`, `milestone`, and `release`, and a bullet list for `assignee`, `labels`, and `dependencies` — and list values are compared as sets of entries. A section that cannot be read to a single value for its field, for example a `## Status` section holding two non-empty lines, is reported as a conflict because agreement cannot be established. A retained section that agrees with the front matter is not reported. Where the record type keeps a `# <ID> <title>` body heading (REQ-05.010), a heading whose ID or title disagrees with the front matter is reported as the same kind of finding. Decision records are excluded: `## Status`, `## Date`, and `## References` are canonical body sections of a decision (REQ-05.010), not retired metadata.

Each finding is reported with its kind, the affected record or location, and a detail. A run with no findings exits successfully.

`papyrus check` never reads or inventories a `deprecated/` folder. Files under a `deprecated/` folder are not scanned, are not candidates for any finding, and produce no report entry (REQ-05.007).

When the effective configuration is invalid, `papyrus check` reports the configuration findings and still runs the checks that do not depend on the effective `document_types` set: duplicate IDs among task, milestone, release, and decision records, self-referential task dependencies, dependency cycles, dangling `milestone` and `release` references, and front-matter/section conflicts on task and milestone records. Checks that need the effective `document_types` set — duplicate IDs across document records and front-matter/section conflicts on document records — are skipped, and the report names them as skipped.

`--fix` applies repairs only where they are unambiguous and safe. It repairs duplicate record IDs and performs no other repair. For a duplicated ID, the record that keeps the ID follows a fixed keeper order: the keeper is the record with the earliest valid `created_date`; a record whose `created_date` is missing or cannot be parsed ranks after every record with a valid date; and a remaining tie — equal valid dates, or two records without a valid date — is broken by the lexicographically smallest project-relative path. Every other record carrying that ID is assigned a new ID from the regular allocator (REQ-01.005), and its file is renamed to the stem of the new ID. `--fix` does not rewrite references: a reference to a repaired ID is left unchanged, and the confirmation prompt lists, for each planned reassignment, every reference affected by it, naming the referring record and field, so the user can update them.

`--fix` refuses to repair a duplicated ID when any record carrying it meets one of three conditions: the record's file stem and front-matter `id` disagree, because its identity is ambiguous; the record is a group record of a grouped requirement family (a `REQ-XX` or `NFR-XX` group record), because a group ID is structural and is not allocator-assigned; or the record — the keeper or any other record carrying the ID — lives in a read-only legacy directory or the read/import archive directory, because repairing it would require a migration this requirement forbids. No record of that duplicated ID is changed, and the finding remains reported and unresolved.

`--fix` prompts once per run when it has planned repairs: one confirmation covers the whole repair plan, and the prompt lists each planned reassignment together with the references affected by it. `--force` skips that prompt. `--force` without `--fix` is a usage error (REQ-01.004). When standard input is not a terminal and `--force` is absent, `--fix` writes nothing and exits with a usage error that names `--force`, so an unattended run never blocks on a prompt. `--fix` does not migrate legacy `deprecated/` folders, does not repair dangling references, dependency cycles, self-references, front-matter/section conflicts, or invalid configuration, and does not modify records beyond the duplicate-ID repair. Those findings remain reported and unresolved.

Immediately before applying a repair, `--fix` re-reads the file and compares it with the state captured by the scan. When the file changed after the scan, the repair is skipped, the skip is reported as a `repair-skipped` finding, and the duplicate-ID finding remains reported and unresolved.

The exit code is 4 whenever the run found at least one finding, including a run in which `--fix` repaired every repairable finding. A run without findings exits 0.

The command is CLI-only in this release; no browser affordance runs it. Output honors the global `--json` option (REQ-01.004), and the JSON form carries the same findings as the plain report.

## Acceptance Criteria
- `papyrus check` runs the documented diagnostic checks and prints a plain report.
- The report lists each finding with its kind, affected record or location, and detail.
- Duplicate record IDs are detected across every managed record type.
- Self-referential task dependencies are detected.
- Dependency cycles among task dependencies are detected.
- A task referencing a missing milestone is reported as a dangling reference.
- A milestone referencing a missing release is reported as a dangling reference.
- A task or milestone whose retained legacy section disagrees with its front matter is reported as a front-matter/section conflict naming the field and both values.
- A `# <ID> <title>` body heading whose ID or title disagrees with the front matter is reported as a front-matter/section conflict.
- A retained legacy section that cannot be read to a single value is reported as a front-matter/section conflict, while a retained section that agrees with the front matter is not reported.
- A decision's `## Status`, `## Date`, and `## References` sections are not reported as front-matter/section conflicts.
- Configuration validity is checked and any violation is reported.
- Configuration validity covers invalid slugs, statuses, prefixes, directories, and duplicate slug keys.
- Configuration validity covers an unknown top-level key and an unknown field-item key, each reported with a `document_types.<slug>.<path>` location.
- Configuration validity covers an unknown enum value for `kind`, `verbs`, or `reference_visibility`.
- Configuration validity covers a reserved field key used as a field `key` or `frontmatter_key`, and a duplicate field key within a role.
- Configuration validity covers a field with neither `heading` nor `frontmatter_key`, a field `order` or `enabled` key, and a `model`, `title`, `content`, or `id_shape` key.
- A configuration finding names its location by the selector path `document_types.<slug>.<path>`, using the keyed `fields[story].heading` or indexed `fields[0].heading` form, and names the field key when one is present.
- A selector path that cannot resolve uniquely — an unknown key, a missing or duplicate field key, or an out-of-range field index — is reported as a configuration finding rather than skipped.
- A run with no findings exits with code 0.
- A run with one or more findings exits with code 4, including a run in which `--fix` repaired every repairable finding.
- An invalid configuration does not stop `check`: the run reports the configuration findings instead of failing before the checks.
- With an invalid configuration, the checks that do not depend on the `document_types` set still run, and the report names the skipped checks.
- `check` loads the configuration tolerantly like `config get|set|unset|list` and does not refuse to run on an invalid configuration.
- `--fix` repairs duplicate record IDs only.
- For a duplicated ID, the keeper is the record with the earliest valid `created_date`; a record with a missing or unparsable `created_date` ranks after every record with a valid date, and a remaining tie — equal valid dates, or two records without a valid date — is broken by the lexicographically smallest project-relative path.
- Every other record carrying a duplicated ID receives a new ID from the regular allocator, and its file is renamed to the stem of the new ID.
- `--fix` does not rewrite references to a repaired ID, and its confirmation prompt lists, for each planned reassignment, every affected reference with the referring record and field.
- `--fix` refuses to repair a duplicated ID that involves a record whose file stem and front-matter `id` disagree, a `REQ-XX` or `NFR-XX` group record, or a record — keeper or any other record carrying the ID — in a read-only legacy directory or the read/import archive directory, and changes none of that ID's records.
- One confirmation prompt covers the whole repair plan: under `--fix`, a single prompt lists every planned reassignment with its affected references, and no repair is applied unless that prompt is confirmed or `--force` is given.
- A repair whose file changed after the scan is skipped, reported as a `repair-skipped` finding, and leaves the duplicate-ID finding reported and unresolved.
- `--force` without `--fix` is a usage error (exit code 2).
- When standard input is not a terminal and `--force` is absent, `--fix` writes nothing and exits with a usage error naming `--force`.
- `--fix` does not migrate `deprecated/` folders and does not repair dangling references, dependency cycles, self-references, front-matter/section conflicts, or invalid configuration.
- `check` never reads or inventories a `deprecated/` folder and reports no finding for files in it.
- `--json` renders the report, including every finding, as machine-readable JSON (REQ-01.004).
- The command is available only from the CLI in this release; no browser affordance runs it.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → check (to be documented)
- RE-010 — Descriptor-driven document engine refactor (accepted, not reopenable): D10 (presentation facts in the seed), D14 (nested group and member blocks, field sequence, selector paths, rejected `model`, `title`, `content`, and `id_shape` keys).
- ADR-0019 — Fail-Closed Document-Type Configuration Validation (decisions 1-7 validation; decision 10 tolerant repair surfaces; amendment: rejected keys and field invariants, indexed field diagnostics, collection replace, selector paths).
- ADR-036 — Config-declared document field schema and one descriptor model: decision 23 (reserved field keys), decision 34 (unknown keys and enum values, missing `heading`/`frontmatter_key`, reserved and duplicate field keys, `order`/`enabled`, and `model`/`title`/`content`/`id_shape` diagnostics).
- ADR-0029 — Read-Only Corpus Diagnostics (`papyrus check`) with Duplicate-ID Repair (keeper order, reassignment, refusal classes including the read/import archive directory, `--force`, one prompt per run, per-repair concurrency re-check).
- REQ-01.002 — Read and Edit Project Configuration (tolerant repair surfaces, selector paths, validated writes).
- REQ-01.004 — Use Common CLI Controls and Outcomes (exit codes; global `--json`).
- REQ-01.005 — Assign Stable Record IDs (duplicate-ID detection, allocator used by `--fix`).
- REQ-02.001 — Create and Describe Tasks (task dependencies).
- REQ-02.003 — List and Filter Tasks (dependency readiness semantics).
- REQ-03.001 — Manage Milestones (milestone `release` reference; retired `## Release` section).
- REQ-03.002 — Manage Releases (release lifecycle; dangling release reference context).
- REQ-05.007 — Manage Document Archive State and Deletion (inert `deprecated/` folder).
- REQ-05.010 — Record Metadata in YAML Front Matter (front-matter authority; decision body sections).
- REQ-07.002 — Configure Document Types and Slugs (configuration validity, field items, selector paths).
- REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models (rejected model and `id_shape` keys, field vocabulary).
- Maintainer decision 2026-11-15 (in-session feedback): `papyrus check` mirrors backlog doctor, scoped to papyrus corpus; `--fix` repairs duplicate IDs only; no legacy folder migration; CLI only; findings exit 4.
- Project-owner decision 2026-11-15: `check` is a tolerant repair surface — it loads configuration tolerantly like `config get|set|unset|list`, reports configuration findings, still runs the checks that do not need a valid configuration, and exits 4 on findings.
- Project-owner decision 2026-11-15: `--fix` skips prompts with `--force` (not `--yes`), and `--force` is meaningful only together with `--fix`; a duplicate-ID repair keeps the record with the earliest valid `created_date`, ranks records with a missing or unparsable `created_date` after all dated records, and breaks remaining ties by path; no inbound reference rewriting, and the confirmation lists the affected references; stem/front-matter mismatches, grouped-family group IDs, and duplicates involving a read-only legacy directory are refused; the exit code is 4 when findings were found even if repaired.
- Contract clarification 2026-11-15 (ADR-0029 decisions 7, 8, 9; `docs/specifications/project-diagnostics-check.md` §5.1, §10.6, §10.8, §10.9): the read/import archive directory is refused alongside a read-only legacy directory; one confirmation prompt covers the whole plan; a repair whose file changed after the scan is reported as `repair-skipped` and its duplicate-ID finding stays unresolved.
- Project-owner decision 2026-11-15: `check` never reads or inventories a `deprecated/` folder.
