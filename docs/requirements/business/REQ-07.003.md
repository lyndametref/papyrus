---
id: REQ-07.003
title: "Dispatch Config-Defined Document Commands"
created_date: '2026-11-13T21:00:12Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-07.003 Dispatch Config-Defined Document Commands

## User Story
As a project contributor, I want to run `papyrus <document-slug>` for any configured document type so that every document type is managed with a consistent command family.

## Group
REQ-07 Artifact Categories and Config-Defined Document Types

## Criticality
Must have

## Description
For every enabled `document_types` entry, Papyrus exposes a command named by its slug. Every document type — built-in default entries and user-added entries alike — has the **baseline command family**:

- `papyrus <document-slug> create <title> [--description <text>] [--label <label>...] [--reference <value>...]`
- `papyrus <document-slug> list`
- `papyrus <document-slug> view <id>`
- `papyrus <document-slug> edit <id> [--title <title>] [--description <text>] [--label <label>...] [--reference <value>...] [--editor]`
- `papyrus <document-slug> archive <id>`
- `papyrus <document-slug> delete <id> [--force]`

The baseline is a lower bound, not the complete verb set of every type. Front-matter-backed document types additionally accept `papyrus <document-slug> status <id> <value>` (REQ-07.001); UI mockups keep their state in HTML metadata and have no `status` command (REQ-05.008). A built-in typed document type may declare additional verbs beyond the baseline, and those verbs are documented by that type's requirement. In this release, the built-in typed types add `create-group`, plus the `--group` option on requirement creation, for business and non-functional requirements (REQ-05.001, REQ-05.002), and `supersede` for UI mockups (REQ-05.008). A config-defined generic document type offers the baseline family only (REQ-07.004).

There is no `deprecate` and no `restore` command for documents: `archive` replaces `deprecate`, and archiving has no inverse command (REQ-05.007). `delete` permanently removes a record, archiving an active record first, and asks for confirmation unless `--force` is given (REQ-05.007).

The command tree is built from the effective configuration when Papyrus starts. Adding a valid entry to `papyrus.config.yml` is therefore enough to obtain a working command: the same Papyrus binary exposes `papyrus <new-slug>`, and `papyrus --help` lists the new slug, without a rebuild and without per-type code. `papyrus --help` lists every enabled document slug alongside the built-in commands, and `papyrus <document-slug> --help` shows the baseline verbs plus any additional verbs of that type. Global options (`--json`, `--dir`, `--config`, `--no-color`) apply to these commands exactly as to other commands (REQ-01.004).

Built-in commands are never shadowed. A configuration whose slug equals a built-in command name is rejected at configuration load (REQ-07.002), so at invocation a slug always resolves to a document type. An unknown slug, or the slug of a disabled entry, is a usage error with no fallback to another command. Command outcomes use the common exit codes of REQ-01.004.

## Acceptance Criteria
- Every enabled `document_types` entry has a `papyrus <document-slug>` command.
- The command family of every document type includes at least the baseline verbs `create`, `list`, `view`, `edit`, `archive`, and `delete`.
- Front-matter-backed document types also accept `status <id> <value>`; UI mockups do not.
- A built-in typed document type may provide verbs and options beyond the baseline: `create-group` and `--group` for business and non-functional requirements (REQ-05.001, REQ-05.002), and `supersede` for UI mockups (REQ-05.008).
- `create` requires a title and accepts optional description, labels, and references.
- `edit` accepts title, description, labels, references, and `--editor`.
- No `deprecate` and no `restore` command exists for documents.
- `delete` permanently removes a record, archives an active record first, and requires confirmation unless `--force` is given.
- After a valid entry is added to `papyrus.config.yml`, the same Papyrus binary exposes `papyrus <new-slug>`, lists the slug in `papyrus --help`, and runs its baseline verbs, with no rebuild.
- `papyrus --help` lists every enabled document slug, and `papyrus <document-slug> --help` lists the baseline verbs plus any additional verbs of that type.
- Global options apply to document commands and `--json` output includes errors.
- An unknown slug or a disabled entry's slug produces a usage error (exit code 2) and does not execute another command.
- A slug that collides with a built-in command never shadows it, because the configuration is rejected at load (REQ-07.002).
- Outcomes use exit codes 0, 1, 2, 3, and 4 as defined in REQ-01.004.

## Open Questions
- Whether `papyrus <document-slug> <id>` should be accepted as a shorthand for `view <id>`, as `papyrus task <id>` is. Current requirement: no shorthand is required.
- Whether list filtering or sorting options are required beyond the existing behavior; current requirement defines no document-list filters.

## Traceability
- Source: customer-feedback handoff — in-session user feedback, Item 10 (needs clarification), raw quote:
  - "the document type artifacts can be created updated or deleted from the command line with papyrus <document-slug>. a New config attribute document-slug is needed in the config file."
- Project-owner decisions 2026-11-13 (refinement): verb set is `create`, `list`, `view`, `edit`, `delete`, `archive`; no `restore`; dispatch is dynamic only (static document commands are removed, including the `nfreq` alias); slug collisions with built-in commands are rejected at load; unknown or disabled slugs are usage errors.
- Project-owner decision 2026-11-13 (D1): the six base verbs are a baseline per document type, not an exhaustive set; built-in typed types may declare additional verbs (`create-group` and `--group` for REQ-05.001/05.002, `supersede` for REQ-05.008); the REQ-05 typed groups are unchanged.
- Project-owner decision 2026-11-13 (D5): `papyrus <document-slug> status <id> <value>` is added for front-matter-backed document types and never archives (REQ-07.001).
- Current state: `src/lib.rs` defines a static clap `Commands` enum, so no dynamic dispatch exists; this requirement is not implemented.
- REQ-01.004 — Use Common CLI Controls and Outcomes
- REQ-05.001 — Manage Business Requirements, REQ-05.002 — Manage Non-Functional Requirements (additional group verbs and `--group`)
- REQ-05.007 — Manage Document Archive State and Deletion (archive and delete semantics)
- REQ-05.008 — Manage UI Mockups (`supersede`, no `status` command)
- REQ-07.001 — Organize Artifacts into Three Categories (document status command)
- REQ-07.002 — Configure Document Types and Slugs (slug validation, enabled state)
- REQ-07.004 — Provide a Generic Content Model for Config-Defined Document Types
- BR-001 — Papyrus Command Behavior (documented command behavior)
- NFR-03.003 — Provide Command Man Pages in the Debian Package (covers built-in commands; config-defined slugs have no man page in this release, recorded as a release risk without amending the NFR)
