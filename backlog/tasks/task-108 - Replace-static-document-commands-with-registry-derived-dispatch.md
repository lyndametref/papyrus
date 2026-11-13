---
id: TASK-108
title: Replace static document commands with registry-derived dispatch
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-13T21:25:00Z'
updated_date: '2026-11-13T21:25:00Z'
labels:
  - rust
milestone: m-17
dependencies:
  - TASK-105
  - TASK-110
documentation:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - docs/requirements/business/REQ-07.003.md
priority: high
ordinal: 41030
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
`src/lib.rs` compiles twenty subcommands, including eleven document-type variants and the `nfreq` alias, and each type owns a clap enum and match arm. REQ-07.003 requires the command tree to come from the effective configuration at startup: a new `document_types` entry must expose `papyrus <slug>` and a help line with no rebuild, an unknown or disabled slug must exit 2, and built-in names must never be shadowed. ADR-0016 also requires the parser to use the project exit codes and JSON errors instead of clap's exiting parser. This task performs the single cutover of the command surface, so the static commands and `nfreq` cannot coexist with the dynamic ones.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 src/cli/ builds the command tree with clap's builder API from cli::BUILTIN_COMMANDS and the artifact registry; no #[derive(Subcommand)] command enum remains and no module outside src/cli/ constructs or inspects a clap::Command.
- [ ] #2 Startup has the four ADR-0016 phases: a lexical --dir/--config/PAPYRUS_DIR/PAPYRUS_CONFIG pre-scan that stops at --, configuration load and validation, one parse, and dispatch to a built-in handler or Command::Document { slug, verb }.
- [ ] #3 Every enabled document slug gets create, list, view, edit, archive, and delete, plus status for front-matter-backed types and the declared extensions for req/nfr (create-group, --group) and ui-mockup (supersede, no status); all verbs parse into DocumentInvocation and enter document::run.
- [ ] #4 papyrus --help lists every enabled slug in slug order; papyrus <slug> --help lists that type's baseline verbs plus its extensions; papyrus --version and papyrus --help work with no configuration file and describe the default set.
- [ ] #5 An unknown slug, a disabled entry's slug, or a malformed option exits 2 with no fallback or abbreviation; --json applies to document commands and application errors keep exit codes 1, 3, and 4.
- [ ] #6 The eleven static document variants, their per-type clap enums and dispatch arms, and the nfreq alias are deleted; req and nfr keep working only because they are default document_types entries.
- [ ] #7 A configuration whose slug equals a built-in name is rejected before the tree is built; cli::tree never inserts a colliding command.
- [ ] #8 Table-driven characterization tests cover every built-in command's documented options, the generated help for a fixed config, the exit-2 paths, and the document invocation parsing.
<!-- AC:END -->
