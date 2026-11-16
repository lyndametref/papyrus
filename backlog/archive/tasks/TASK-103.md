---
id: TASK-103
title: Replace static document and decision commands with registry-derived dispatch
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
updated_date: '2026-11-14T10:13:00Z'
labels:
  - rust
milestone: MS-018
dependencies:
  - TASK-100
  - TASK-105
documentation:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - docs/requirements/business/REQ-07.003.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/requirements/business/REQ-04.002.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
priority: high
ordinal: 41030
---


## Description
`src/lib.rs` compiles twenty subcommands, including eleven document-type variants and the `nfreq` alias, and each type owns a clap enum and match arm. REQ-07.003 requires the command tree to come from the effective configuration at startup: a new `document_types` entry must expose `papyrus <slug>` and a help line with no rebuild, an unknown or disabled slug must exit 2, and built-in names must never be shadowed. ADR-0016 also requires the parser to use the project exit codes and JSON errors instead of clap's exiting parser. This task performs the single cutover of the command surface, so the static commands and `nfreq` cannot coexist with the dynamic ones.

The cutover also removes the one static verb that contradicts a validated requirement: `papyrus decision archive` (REQ-04.002) and its `DecisionCmd::Archive` variant, dispatch arm, and now-unreachable handler. Decisions have no archive command or archive state and no archive policy (ADR-0017, ADR-0023 decision 3), so leaving the command would keep the registry and the CLI in disagreement. The document and decision command surface, dispatch, handler removal, and CLI tests belong to this task; the BR-008 and README wording corrections belong to TASK-115 and TASK-116.

## Acceptance Criteria
- [x] src/cli/ builds the command tree with clap's builder API from cli::BUILTIN_COMMANDS and the artifact registry; no #[derive(Subcommand)] command enum remains and no module outside src/cli/ constructs or inspects a clap::Command.
- [x] Startup has the four ADR-0016 phases: a lexical --dir/--config/PAPYRUS_DIR/PAPYRUS_CONFIG pre-scan that stops at --, configuration load and validation, one parse, and dispatch to a built-in handler or Command::Document { slug, verb }.
- [x] Every enabled document slug gets the baseline verbs plus its declared extensions; generic slugs parse into DocumentInvocation and enter document::run, while typed slugs retain module dispatch for create/edit (field sets/sections) and route archive/delete/status through document::run (per ADR-0017/TASK-110).
- [x] papyrus --help lists every enabled slug in slug order; papyrus <slug> --help lists that type's baseline verbs plus its extensions; papyrus --version and papyrus --help work with no configuration file and describe the default set; generic slugs parse into DocumentInvocation and enter document::run, while typed slugs retain module dispatch for create/edit and share storage/archive/delete/status through document::run.
- [x] An unknown slug, a disabled entry's slug, or a malformed option exits 2 with no fallback or abbreviation; --json applies to document commands and application errors keep exit codes 1, 3, and 4.
- [x] The eleven static document variants, their per-type clap enums and dispatch arms, and the nfreq alias are deleted; req and nfr keep working only because they are default document_types entries.
- [x] A configuration whose slug equals a built-in name is rejected before the tree is built; cli::tree never inserts a colliding command.
- [x] Table-driven characterization tests cover every built-in command's documented options, the generated help for a fixed config, the exit-2 paths, and the document invocation parsing.
- [x] DecisionCmd::Archive, its dispatch arm, and the now-unreachable decision archive handler are deleted; papyrus decision archive <id> exits 2 as an unknown subcommand for adr and bdr, and the table-driven tests assert the command is absent while decision create, list, view, and edit still work.
