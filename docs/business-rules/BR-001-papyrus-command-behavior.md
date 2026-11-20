---
id: "BR-001:"
title: "Papyrus Command Behavior"
created_date: '2026-11-15T18:37:44Z'
updated_date: '2026-11-20T21:05:52Z'
labels: []
references: []
status: active
---
# BR-001: Papyrus Command Behavior

## Description

For every enabled `document_types` entry, Papyrus generates one command family named `papyrus <slug>` from that entry's document descriptor, exposing the baseline verbs `create`, `list`, `view`, `edit`, `archive`, and `delete` plus the verbs derived from the entry's document model — `status <id> <value>` for `SingleMarkdown` and `GroupedMarkdown`, `create-group` plus a required `--group` on `create` for `GroupedMarkdown`, and `supersede` for `SingleHtmlMockup`, which has no `status` verb — with the title always the first positional `TITLE` on `create` and `create-group` and `--title` on `create`, `create-group`, and `edit`, while the backlog and decision command families stay bespoke and unchanged.

## Rationale

Based on REQ-07.003 — Dispatch Config-Defined Document Commands, REQ-07.002 — Configure Document Types and Slugs, REQ-07.004 — Provide One Schema-Driven Document Content Model with Three Document Models, and REQ-07.001 — Organize Artifacts into Three Categories, this rule exists because the document command set is generated from the descriptor instead of being compiled per type: the verb set is a function of the document model, the options are a function of the field sequence, and the title contract is uniform, so adding a valid entry adds a working command and removing or disabling an entry removes it, without a rebuild and without a per-type module, verb table, or help text. RE-010 D4 keeps backlog and decision records bespoke, so only document commands are generated and those surfaces must not change. REQ-01.004 — Use Common CLI Controls and Outcomes supplies the exit codes and the global options, and REQ-01.008 — Run Project Diagnostics adds the built-in `check` command and its findings outcome, which must stay readable here without weakening the descriptor-derived document model.

## Applicability

Applies to every Papyrus invocation — including the built-in commands such as `check`, which sit outside the generated document-command family — and to every enabled document type declared under `document_types`.

### Command model

- Every enabled entry has the baseline verbs `create`, `list`, `view`, `edit`, `archive`, and `delete` under `papyrus <slug>`.
- The command family is generated from the entry's document descriptor when Papyrus starts, after the effective configuration is validated (BR-016); no per-type module, compiled command, or configuration key supplies a verb, an option, or a help line.
- `status <id> <value>` is generated only for `SingleMarkdown` and `GroupedMarkdown`; a `SingleHtmlMockup` type has no `status` verb, and its fixed `pending`, `approved`, and `archived` metadata states are reached only through `archive` and `supersede`.
- `create-group`, plus a required `--group` on `create`, is generated only for `GroupedMarkdown`; only `req` and `nfr` are of that model (BR-009, BR-010).
- `supersede` is generated only for `SingleHtmlMockup`; only `ui-mockup` is of that model.
- No document command has `deprecate` or `restore`: `archive` replaces `deprecate` and archiving has no inverse command.
- Two document types of the same model expose the same verb set; each derives its field options from its own field sequence and role.
- `papyrus --help` lists every enabled document slug alongside the built-in commands, and `papyrus <slug> --help` lists that type's model-derived verbs with their generated options and help.
- An unknown slug, or the slug of a disabled entry, is a usage error (exit code 2) with no fallback to another command.
- Global options (`--json`, `--dir`, `--config`, `--no-color`) apply as for every command; outcomes use exit codes 0, 1, 2, 3, and 4.
- `check` is a built-in command outside the generated document-command family: it takes no subcommands, is never created from a `document_types` entry, and its `--fix` and `--force` flags are command-local, not global.
- `papyrus check` is read-only by default: a run writes nothing unless `--fix` is given.
- Exit code 4 is a findings outcome, not an error: `papyrus check` exits 4 when the run reports at least one finding, including a run whose `--fix` repairs all succeeded, and 0 only when it found nothing; this is distinct from exit code 1 for an I/O failure and exit code 2 for a usage error.
- `papyrus check` loads the configuration tolerantly, like `papyrus config get|set|unset|list`: an invalid configuration is reported as `config-invalid` findings, the checks that need a valid `document_types` set are named as skipped, and the remaining checks still run (REQ-01.008).
- The former static document command inventory is not compiled in, and `nfreq` is not retained as a command or alias: the non-functional family is `nfr`.

### Title contract

- The title is always the first positional `TITLE` on `create` and `create-group`, and `--title` is accepted on `create`, `create-group`, and `edit`.
- A title is required on create: a create with no title produces one validation outcome and writes no file. Supplying both the positional `TITLE` and `--title` on one invocation is rejected as a conflicting title source, and nothing is written.
- The title is single-line and non-empty, and an edit can never clear it: an empty `--title` is rejected and nothing is written.
- `title` is fixed and non-configurable: it is not a field, and no field may declare it as its `key` or `frontmatter_key` (BR-016).
- `audit` and `create-group` gain the positional `TITLE` additively: their existing `--title` option keeps its name and meaning, and no existing positional argument of any document command is renamed, removed, or reordered.
- There is no `--term` option on any document command; a glossary term is the record title, so `papyrus glossary` uses this contract and `--definition` remains its declared field.
- The HTML mockup metadata uses the same title contract.

### Categories and lifecycles

- Exactly three artifact categories exist: backlog (tasks, milestones, releases), decision, and document; each record type belongs to one category.
- Backlog commands (`task`, `milestone`, `release`) and the `decision` command are unchanged by the generated document-command surface: they stay bespoke, keep their own verbs and options, and no document model, field sequence, or configuration key declares or alters them.
- Task statuses are configurable through `statuses` and `default_status`; the milestone (`open -> closed -> archived`) and release (`planned -> released`) lifecycles are fixed and configuration cannot change them.
- Decisions use only the Nygard statuses `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`; a decision is never archived.
- Documents have the main statuses `active` and `archived` plus optional configured custom statuses; only `archive` produces `archived`, archiving does not retain a previous custom status value, and documents have no inverse command.
- Archived records live in the `archived/` subfolder of the type's configured directory, which is the only document inactive location.
