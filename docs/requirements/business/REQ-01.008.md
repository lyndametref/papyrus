# REQ-01.008 Run Project Diagnostics (`papyrus check`)

## User Story
As a project maintainer, I want to diagnose corpus and configuration integrity issues from the CLI so that problems can be found and, where safe, repaired before they affect other commands.

## Group
REQ-01 Project Setup and CLI Access

## Criticality
Should have

## Description
`papyrus check` is a read-only diagnostic by default. It scans the project corpus and configuration and prints a plain report of integrity findings. The checked categories are:

- Duplicate record IDs across all managed record types (task, milestone, release, decision, and every configured document type).
- Self-referential task dependencies.
- Dependency cycles among task dependencies.
- Dangling references: a task whose `milestone` does not resolve to an existing milestone, and a milestone whose `release` does not resolve to an existing release.
- Front-matter/section conflicts, where a record's front matter and its body disagree on a value that front matter is authoritative for (REQ-05.010).
- Configuration validity (REQ-01.002, REQ-07.002), including invalid slugs, statuses, prefixes, directories, and duplicate keys.

Each finding is reported with its kind, the affected record or location, and a detail. A run with no findings exits successfully.

`--fix` applies repairs only where they are unambiguous and safe: it repairs duplicate record IDs (reassigning IDs so each record is unique) and performs no other repair. `--yes` skips confirmation prompts for the repairs performed under `--fix`. `--fix` does not migrate legacy `deprecated/` folders, does not repair dangling references, dependency cycles, self-references, front-matter/section conflicts, or invalid configuration, and does not modify records beyond the duplicate-ID repair. Those findings remain reported and unresolved.

Configuration validity is checked first. If the effective configuration is invalid, `papyrus check` reports the configuration findings and does not run the checks that require a valid configuration; it still exits with a validation outcome.

The command is CLI-only in this release; no browser affordance runs it. Output honors the global `--json` option (REQ-01.004).

## Acceptance Criteria
- `papyrus check` runs the documented diagnostic checks and prints a plain report.
- The report lists each finding with its kind, affected record or location, and detail.
- Duplicate record IDs are detected across every managed record type.
- Self-referential task dependencies are detected.
- Dependency cycles among task dependencies are detected.
- A task referencing a missing milestone is reported as a dangling reference.
- A milestone referencing a missing release is reported as a dangling reference.
- Front-matter/section conflicts are detected.
- Configuration validity is checked and any violation is reported.
- A run with no findings exits with code 0.
- A run with one or more findings exits with a validation outcome (exit code 4, REQ-01.004).
- `--fix` repairs duplicate record IDs only.
- Under `--fix`, repairs that require confirmation prompt unless `--yes` is given.
- `--fix` does not migrate legacy `deprecated/` folders and does not repair dangling references, dependency cycles, self-references, front-matter/section conflicts, or invalid configuration.
- When the effective configuration is invalid, configuration findings are reported and checks requiring a valid configuration are not run.
- The command is available only from the CLI in this release; no browser affordance runs it.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → check (to be documented)
- REQ-01.002 — Read and Edit Project Configuration (configuration validity, repair surfaces)
- REQ-01.004 — Use Common CLI Controls and Outcomes (exit codes; global `--json`)
- REQ-01.005 — Assign Stable Record IDs (duplicate-ID detection and repair)
- REQ-02.001 — Create and Describe Tasks (task dependencies)
- REQ-02.003 — List and Filter Tasks (dependency readiness semantics)
- REQ-03.001 — Manage Milestones (milestone `release` reference)
- REQ-03.002 — Manage Releases (release lifecycle; dangling release reference context)
- REQ-05.010 — Record Metadata in YAML Front Matter (front-matter/section conflicts)
- REQ-07.002 — Configure Document Types and Slugs (configuration validity)
- Maintainer decision 2026-11-15 (in-session feedback): `papyrus check` mirrors backlog doctor, scoped to papyrus corpus; `--fix` repairs duplicate IDs only; `--yes` skips confirmation; no legacy folder migration; CLI only; findings exit 4.
