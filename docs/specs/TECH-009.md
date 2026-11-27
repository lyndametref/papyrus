---
id: TECH-009
title: "Command Dispatch"
created_date: '2026-11-27T20:35:37Z'
updated_date: '2026-11-27T20:35:37Z'
labels: []
references:
  - REQ-002.001
  - REQ-002.002
  - REQ-002.005
  - REQ-002.006
  - REQ-004.001
  - REQ-004.002
  - REQ-005.015
  - REQ-007.003
status: verified
---
# TECH-009 Command Dispatch

## Description
Covers the command tree and the command families of the three record categories: the built-in backlog families, the configured document-type families, and the configured decision-type families. Covers their naming, verb sets, options, help, and the identifier shorthand. Record behavior, list and view content, the field schema, and the browser surface are separate specifications.

## Command tree

- The built-in backlog families `task`, `milestone`, and `release` always exist and are not configured (REQ-002).
- Every entry in `document_types` and every entry in `decision_types` exposes a command family named by its slug: `papyrus <slug>` (REQ-007.002, REQ-004.001).
- The configured part of the tree is built from the configuration at startup. Declaring a valid entry is enough to obtain the command and its help line; no rebuild and no per-type code is involved.
- Each entry contributes one command node carrying the verbs of its family, one help line, and one entry in the root help listing.
- A type that the configuration does not declare is absent from commands, views, and search.
- An unknown slug, or a slug that is not configured, is a usage error. Such a token never falls back to another command.
- A slug equal to a built-in command name is rejected when the configuration loads, so at invocation a slug always resolves to a configured type and never shadows a built-in command.

## Command naming

- A backlog family's name is fixed: `task`, `milestone`, or `release`.
- A configured record type's slug is the configuration key and the command name at once.
- Command outcomes use the common exit codes.
- Global options `--json`, `--dir`, `--config`, and `--no-color` apply to every command in every record category: backlog, document, and decision.

## Backlog command families

The three backlog families are built in, use fixed verbs, and take no model or field sequence from configuration (REQ-002.001, REQ-002.005, REQ-002.006).

- Task verbs: `create`, `edit`, `archive`, `restore`, `delete`, `list`, `view`. The task status changes through `edit`; there is no `status` verb.
- Milestone verbs: `create`, `close`, `archive`, `edit`, `delete`, `list`, `view`. `close` moves `open` to `closed`.
- Release verbs: `create`, `close`, `archive`, `edit`, `delete`, `list`, `view`. `close` moves `planned` to `released`.
- `restore` exists for tasks only; milestones and releases have no restore verb.
- Archiving a milestone or a release closes it first when it is still `open` or `planned`.
- Each backlog verb has a fixed option set and no field-derived options.

## Document command families

The verb family of a document type is a function of its record model. It is never declared in configuration, in a per-type table, or in a per-type module (REQ-007.003).

| Verb | Availability |
|---|---|
| `create`, `list`, `view`, `edit`, `archive`, `delete` | every model |
| `status <ID> <VALUE>` | `simple-record`, `grouped-record` |
| `create-group`, `--group` on `create` | `grouped-record` |
| `--status approved` on `edit` | `formatted-record` |

- Two document types of the same model expose the same verb set; their options differ only through their own field sequences.
- A `formatted-record` type has no `status` verb; `edit --status approved` moves `draft` to `approved` and is its only state transition.
- No `deprecate`, no `restore`, and no `supersede` verb exists for documents.
- `delete` permanently removes a record. An active record is archived first, and confirmation is required unless `--force` is given.

## Decision command families

- Every decision type exposes the same fixed verbs: `create`, `list`, `view`, `edit`, `accept`, `reject`, `deprecate`, `supersede`, `archive`, and `delete` (REQ-004.001).
- The command selects the decision type; there is no `--decision-type` option and no shared `decision` command family.
- A decision type has no `status` verb: each status verb performs exactly one transition, and no `edit` option sets the status (REQ-004.002).
- `archive` moves the file and changes no status; only `rejected`, `superseded`, and `deprecated` decisions are archivable (REQ-004.002).
- Decision commands have fixed options and no field sequence.

## Document field options

- For a document type, `create` and `edit` expose the options generated from the fields of the record's role, taken from the type's field sequence (REQ-005.015).
- A field contributes an option only on the verbs it declares. A field whose verbs exclude `edit` is never writable after creation; one whose verbs exclude `create` has no create option.
- Option order and per-option help prose come from the field sequence: the field name, the help text, and the position. A field change therefore changes the generated options and their help.
- Required and clearing rules, `list` and `multiline` handling, and `--editor` follow the field contract (REQ-005.015), applied to the type's field sequence for the record's role.
- `title` is not a field: it is the first positional `TITLE` on `create` and `create-group`, `--title` is accepted on `create`, `create-group`, and `edit`, and an edit can never clear it.
- The `group` link of a member record has no option.
- `--label` and `--reference` are the general repeatable metadata options and are not fields.
- No field option sets `status`. A Markdown-backed record's status is written only by `status` and `archive`; a `formatted-record` state only by `edit --status approved`.

## Help determinism

- Help output for one configuration is identical across runs, machines, working directories, terminal widths, and locales.
- It contains no absolute path, host name, timestamp, process identifier, or memory address.
- For a given configuration these are stable: the root about line, each family's help line and about line, the family order, and each verb's about line.
- A document type's about line is `Manage <slug> documents.`
- Configured record types are listed in the order in which the configuration declares them.
- Per-option help prose and option order are derived from the document field sequence, so a field change legitimately changes them.
- `papyrus --help` lists every built-in command and every configured slug.
- `papyrus <command> --help` shows the family's verbs with its options.

## Identifier shorthand

- For a document type, when no verb is given and the next argument is a record identifier of the type, `papyrus <slug> <ID>` behaves as `view <ID>`.
- For the task family, `papyrus task <ID>` behaves as `view <ID>` (REQ-002.002).
- No identifier shorthand exists for milestone, release, or decision commands.
- A token that is neither a verb of the family nor a record identifier of the type is a usage error.
