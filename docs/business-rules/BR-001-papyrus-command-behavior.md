---
id: "BR-001:"
title: "Papyrus Command Behavior"
created_date: '2026-11-15T18:37:44Z'
updated_date: '2026-11-16T16:57:06Z'
labels: []
references: []
status: active
---
# BR-001: Papyrus Command Behavior

## Description

Papyrus exposes one command family per enabled `document_types` entry, named `papyrus <slug>`, whose baseline verbs are `create`, `list`, `view`, `edit`, `archive`, and `delete`, plus `status <id> <value>` for front-matter-backed types; the former static per-type document commands and the `nfreq` command name are removed. Alongside these config-defined families, Papyrus ships a fixed set of built-in commands documented in the man page; no document slug can shadow a built-in name (BR-016).

## Rationale

Based on REQ-07.003 — Dispatch Config-Defined Document Commands, REQ-07.002 — Configure Document Types and Slugs, and REQ-07.001 — Organize Artifacts into Three Categories, this rule exists because the document command set is no longer compiled per type: the effective configuration is the single source of the available document commands, so adding a valid entry must add a working command and removing an entry must remove it, without a rebuild or per-type code. REQ-01.004 — Use Common CLI Controls and Outcomes supplies the exit codes and the global options that apply to those commands. REQ-01.008 — Run Project Diagnostics adds a `check` command to the fixed built-in set, outside that config-defined family, and makes its finding outcome exit code 4 under REQ-01.004, so the built-in set and that outcome must be readable here without weakening the configuration-only document model.

## Applicability

Applies to every Papyrus invocation — including the built-in `check` command, which sits outside the config-defined document-command family — and to every document type declared under `document_types`.

### Command model

- Every enabled entry has `papyrus <slug> create|list|view|edit|archive|delete`.
- The baseline is a lower bound, not the complete verb set: front-matter-backed types also accept `status <id> <value>`, and a built-in typed type may declare additional verbs.
- `create-group` plus a required `--group` on `create` exist only for `req` and `nfr` (BR-009, BR-010); UI mockups add `supersede` and no `status`.
- `papyrus --help` lists every enabled document slug alongside the built-in commands, and `papyrus <slug> --help` lists that type's verbs.
- An unknown slug, or the slug of a disabled entry, is a usage error (exit code 2) with no fallback to another command.
- Global options (`--json`, `--dir`, `--config`, `--no-color`) apply as for every command; outcomes use exit codes 0, 1, 2, 3, and 4.
- `check` is a built-in command outside the config-defined document-command family: it takes no subcommands, is never created from a `document_types` entry, and its `--fix` and `--force` flags are command-local, not global.
- `papyrus check` is read-only by default: a run writes nothing unless `--fix` is given.
- Exit code 4 is a findings outcome, not an error: `papyrus check` exits 4 when the run reports at least one finding, including a run whose `--fix` repairs all succeeded, and 0 only when it found nothing; this is distinct from exit code 1 for an I/O failure and exit code 2 for a usage error.
- `papyrus check` loads the configuration tolerantly, like `papyrus config get|set|unset|list`: an invalid configuration is reported as `config-invalid` findings, the checks that need a valid `document_types` set are named as skipped, and the remaining checks still run (REQ-01.008).
- The former static document command inventory is not compiled in, and `nfreq` is not retained as a command or alias: the non-functional family is `nfr`.

### Categories and lifecycles

- Exactly three artifact categories exist: backlog (tasks, milestones, releases), decision, and document; each record type belongs to one category.
- Task statuses are configurable through `statuses` and `default_status`; the milestone (`open -> closed`) and release (`planned -> released`) lifecycles are fixed and configuration cannot change them.
- Decisions use only the Nygard statuses `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`; a decision is never archived.
- Documents have the main statuses `active` and `archived` plus optional configured custom statuses; only `archive` produces `archived`, archiving does not retain a previous custom status value, and documents have no inverse command.
- Archived records live in the `archived/` subfolder of the type's configured directory, which is the only document inactive location.
