# BR-001: Papyrus Command Behavior

## Description

Papyrus exposes one command family per enabled `document_types` entry, named `papyrus <slug>`, whose baseline verbs are `create`, `list`, `view`, `edit`, `archive`, and `delete`, plus `status <id> <value>` for front-matter-backed types; the former static per-type document commands and the `nfreq` command name are removed.

## Rationale

Based on REQ-07.003 — Dispatch Config-Defined Document Commands, REQ-07.002 — Configure Document Types and Slugs, and REQ-07.001 — Organize Artifacts into Three Categories, this rule exists because the document command set is no longer compiled per type: the effective configuration is the single source of the available document commands, so adding a valid entry must add a working command and removing an entry must remove it, without a rebuild or per-type code. REQ-01.004 — Use Common CLI Controls and Outcomes supplies the exit codes and the global options that apply to those commands.

## Applicability

Applies to every Papyrus invocation and to every document type declared under `document_types`.

### Command model

- Every enabled entry has `papyrus <slug> create|list|view|edit|archive|delete`.
- The baseline is a lower bound, not the complete verb set: front-matter-backed types also accept `status <id> <value>`, and a built-in typed type may declare additional verbs.
- `create-group` plus a required `--group` on `create` exist only for `req` and `nfr` (BR-009, BR-010); UI mockups add `supersede` and no `status`.
- `papyrus --help` lists every enabled document slug alongside the built-in commands, and `papyrus <slug> --help` lists that type's verbs.
- An unknown slug, or the slug of a disabled entry, is a usage error (exit code 2) with no fallback to another command.
- Global options (`--json`, `--dir`, `--config`, `--no-color`) apply as for every command; outcomes use exit codes 0, 1, 2, 3, and 4.
- The former static document command inventory is not compiled in, and `nfreq` is not retained as a command or alias: the non-functional family is `nfr`.

### Categories and lifecycles

- Exactly three artifact categories exist: backlog (tasks, milestones, releases), decision, and document; each record type belongs to one category.
- Task statuses are configurable through `statuses` and `default_status`; the milestone (`open -> closed`) and release (`planned -> released`) lifecycles are fixed and configuration cannot change them.
- Decisions use only the Nygard statuses `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`; a decision is never archived.
- Documents have the main statuses `active` and `archived` plus optional configured custom statuses; only `archive` produces `archived`, archiving does not retain a previous custom status value, and documents have no inverse command.
- Archived records live in the `archived/` subfolder of the type's configured directory, which is the only document inactive location.
