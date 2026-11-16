---
id: TECH-070
title: "Descriptor-Driven Typed Document Command Dispatch"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-16T17:47:31Z'
labels: []
references:
  - ADR-0016
  - ADR-0017
  - ADR-0018
  - ADR-0022
  - BR-001
  - BR-009
  - BR-010
  - BR-011
  - BR-012
  - BR-013
  - BR-014
  - BR-017
  - BR-018
  - BR-019
  - BR-020
  - REQ-01.004
  - REQ-01.005
  - REQ-05.001
  - REQ-05.002
  - REQ-05.007
  - REQ-05.008
  - REQ-05.010
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - TASK-108
  - TASK-110
  - TASK-117
  - TASK-119
  - TASK-120
  - TASK-121
status: verified
---
# TECH-070 Descriptor-Driven Typed Document Command Dispatch

## Description
**Ticket:** TASK-119
**Milestone:** m-18 (Descriptor Single-Source Cleanup)
**Status:** Refined — execution-ready
**UI mockups:** none. This ticket changes command construction and dispatch only; no user-visible interface is designed.
**Requirements:** REQ-07.001, REQ-07.002, REQ-07.003, REQ-07.004, REQ-05.001, REQ-05.002, REQ-05.007, REQ-05.008, REQ-05.010, REQ-01.004, REQ-01.005
**Business rules:** BR-001, BR-009, BR-010, BR-011, BR-012, BR-013, BR-014, BR-017, BR-018, BR-019, BR-020
**ADRs:** ADR-0016 (Accepted), ADR-0017 (Accepted), ADR-0018 (Accepted), ADR-0022 (Accepted)

## Purpose

ADR-0016 makes `document::run` the single dispatch path for every document type
and makes `cli::document_command(descriptor)` the single command builder.
ADR-0017 makes the artifact registry the only place that maps a slug to
behaviour. TASK-108 and TASK-110 delivered the generic (config-defined) path
and the shared storage path, but built-in typed types still retain per-type
`clap` enums, two `TypedHandler` command-construction matches, a second
dispatcher, and module-local directory joins.

This ticket closes that gap: every document command, built-in typed or
config-defined generic, is built from its `TypeDescriptor`, and every document
invocation enters `document::run`. Typed modules keep only content semantics
(field validation, section mapping, HTML rendering). Behaviour — help text,
flags, exit codes, IDs, file layout, `--json` — is unchanged.

The second record-level descriptor lookup
(`record::descriptor::descriptor_for_doc_type`) is deliberately retained and is
the subject of TASK-120.

## Scope

### In scope

- `src/registry.rs`: extend the typed descriptor with per-verb option metadata
  (verb, about text, arguments) and add a generic table for the seven-verb
  generic document family.
- `src/cli/tree.rs`: one descriptor-driven `document_command` for every content
  model; delete the `TypedHandler` match and every `augment_subcommands` call.
- `src/cli/dispatch.rs`: every `ArtifactCategory::Document` slug enters
  `document::run`; delete `dispatch_typed_module`, the typed `status` special
  case, and every `from_arg_matches`/`run` call into a typed module.
- `src/document.rs`: accept a generic typed argument map; delegate typed
  content verbs to the owning module through `TypedHandler`; resolve active and
  archived directories plus prefix from the `TypeDescriptor` and id padding from
  the effective `config.id_padding`; keep shared archive/delete/status handling
  for `Generic` and `Typed` content, with `HtmlMockup` delegated to
  `ui_mockup` (see §4).
- The ten typed document modules (`req`/`nfr` via `src/requirement.rs`,
  `glossary`, `rule`, `ui_spec`, `audit`, `tech_spec`, `use_case`,
  `test_case`, `test_report`, `ui_mockup`): expose a neutral typed handler,
  delete their `clap` enums and `run` functions, and stop joining
  `cfg.doc_directory` with `document_types.<slug>.directory`.
- Help-output golden snapshots captured **before** the refactor and asserted
  byte-for-byte **after** it, for every enabled slug and verb.

### Out of scope

- Removing `record::descriptor::descriptor_for_doc_type` and the parallel
  record descriptor table (TASK-120).
- Removing the process-global descriptor cache or slug leaks (TASK-121).
- Changing any typed module's field set, section order, body parser,
  serialization, HTML template, ID shape, status vocabulary, or on-disk path.
- Adding or removing a verb, option, alias, status, configuration key, or
  dependency.
- Editing ADRs, requirements, or business rules.

## Current state and target

| Concern | Current | Target |
|---|---|---|
| Typed subcommand tree | `TypedHandler` match in `tree.rs` calling each module's `augment_subcommands` | `document_command` builds from `descriptor.verbs` + descriptor option metadata |
| Parsing | Each module's `clap` enum, parsed with `from_arg_matches` | One `DocumentInvocation` with a generic typed argument map |
| Dispatch | `dispatch_typed_module` matches `TypedHandler` and calls per-type `run` | `document::run` resolves the descriptor and delegates content verbs by `TypedHandler` |
| Typed `status` | Special case in `dispatch_document` | Shared `document::run` status handler |
| Directories / prefix | Module-local `type_entry` + `cfg.doc_directory.join(...)` | Resolved once from the `TypeDescriptor` and passed to the module |
| Extension verbs | Module enum arms (`create-group`, `supersede`) | Descriptor verbs routed through `document::run` to the module |

## 1. Descriptor option metadata

`TypedSchema` gains `verbs: &'static [VerbSpec]`. A `VerbSpec` carries:

- the `Verb` it describes;
- the verb's help about-line, byte-identical to today's derived text
  (including the empty about-lines used by `audit` and `ui-mockup`);
- an ordered list of argument specifications.

An argument specification carries every fact needed to reproduce the current
argument exactly: argument id, long name, short name, value name, help text,
positional index, required flag, `required_unless_present`, action kind
(single value, repeatable/appending value, boolean flag, optional value with
zero-or-one values), and conflicts.

Rules:

1. Argument declarations appear in the same order as the current `clap` derive
   fields. Clap assigns help display order from argument insertion order and
   interleaves propagated global arguments by that order, so this ordering is
   what makes help byte-identical. It is not cosmetic and must be tested.
2. Value names are explicit, because the derive macro uppercased the Rust field
   name rather than the long name. For example the requirement `--title` flag
   displays `<TITLE_FLAG>` while its long name is `title`; the generic document
   `--label` displays `<label>`. Both are preserved.
3. Argument ids are explicit and stable: `title` (positional) and `title_flag`
   (required-flag form) remain distinct; `no_requirement`, `ref`,
   `superseded_by`, `value`, and `force` keep their current ids because parsing
   and module code select by id.
4. The generic document family uses one static seven-verb table shared by every
   `ContentModel::Generic` slug. Typed tables live in `registry.rs` beside their
   `TypedSchema` and may share only identical argument definitions (id,
   label/reference, editor, force, value). Options whose help, requiredness,
   value name, or conflicts differ stay explicit per type; do not generalise
   them away.
5. A registry invariant test asserts that every document descriptor's verbs have
   a spec in its content model's table and that every spec verb is declared by
   the descriptor. A missing spec is a startup/validation defect, not a silent
   missing command.
6. The metadata carries no I/O and no slug-to-behaviour mapping. The registry
   remains data-only (ADR-0017 decision 5).

## 2. Command construction

`cli::tree::document_command(descriptor)`:

- keeps the slug as the command name and `Manage <slug> documents.` as the
  about line;
- includes a verb only when `descriptor.has_verb(verb)`;
- emits verbs in the canonical presentation order that matches today's enums:
  `create`, `create-group`, `list`, `view`, `edit`, `archive`, `delete`,
  `supersede`, `status`;
- builds each verb's arguments from its spec, with no per-type match and no
  `Box::leak` change (the existing leak of the slug name is retained);
- leaves the four global options (`--json`, `--dir`, `--config`, `--no-color`)
  on the root command, so clap propagates them exactly as today.

`tree()` keeps the reserved-name collision guard and inserts document commands
in registry order. No module outside `src/cli/` constructs or inspects a
`clap::Command`.

## 3. Invocation contract

One parser, `parse_document_invocation`, produces:

- `slug`;
- `DocumentVerb` (extended with `CreateGroup` and `Supersede`);
- a generic argument map holding every explicitly present non-global argument
  id and its values (empty value list for boolean switches); the map preserves
  "absent" versus "present with no value" for optional-value flags;
- the existing convenience fields (`id`, `title`, `description`, `labels`,
  `references`, `editor`, `status_value`, `force`) derived from the map for the
  generic path, so generic behaviour is unchanged.

The parser does not know any type's option set, and it does not default an
unknown verb to `list`: it returns a validation error instead. The command tree
guarantees this error is unreachable through the CLI; the check prevents a
future tree defect from executing the wrong verb.

## 4. `document::run` dispatch

`run(ctx, registry, invocation)`:

1. Resolves the descriptor from the registry and validates category, enabled
   state, and verb membership exactly as today.
2. Resolves one scope from the descriptor: configuration, active directory,
   archived directory, and prefix. Directory resolution is derived from the
   descriptor's configuration key and relative value and is contained within the
   project directory. Id padding is not a descriptor field: `DocumentScope`
   reads the effective `config.id_padding` together with the loaded
   configuration. The scope is computed in `document.rs`, not in the registry, so
   the registry remains I/O-free.
3. Handles `archive`, `delete`, and `status` through the existing shared record
   path for `Generic` and `Typed` content. The single exception is
   `HtmlMockup` (`ui-mockup`): its `archive` and `delete` stay in
   `src/ui_mockup.rs` because HTML records carry their metadata as comments
   inside the `.html` file and have no shared record descriptor, and its
   `ui-mockup delete` follows that module's HTML archive-then-remove flow.
   `ui-mockup` declares no `status` verb, so `status` is never routed there.
   The exception is keyed on `ContentModel::HtmlMockup`, never on a slug
   string, and no other typed module implements or delegates these verbs.
4. For `create`, `create-group`, `list`, `view`, `edit`, and `supersede`:
   - `Generic` runs the existing generic implementations unchanged;
   - `Typed` and `HtmlMockup` delegate to the owning module selected by
     `TypedHandler` (never by a slug string), passing the descriptor, the scope,
     the verb, the argument map, and the registry (requirements need the sibling
     kind's descriptor for the group-mismatch check).
5. A verb absent from the descriptor is rejected before delegation; no module
   sees an unsupported verb.

Typed module handler shape: one entry function per module that matches the verb
and maps the argument map into the module's existing internal create/edit/list/
view/create-group/supersede logic. Validation, section mapping, body parsing,
and HTML handling remain inside the module.

## 5. Per-type migration matrix

Recommended order is simplest-first, ending with the two riskiest types. It
differs from a requirement-first order on purpose: the near-identical types
prove the metadata/builder/scope plumbing before group logic and HTML are
touched.

| Order | Module | Verbs / options to encode | Type-specific behaviour retained |
|---|---|---|---|
| 1 | tech-spec | create title + `--description`; edit `--title`, `--description`, `--editor`; labels/references; status | Description section; body mapping |
| 2 | use-case | tech-spec set plus create/edit `--applicability` | Description + Applicability sections; clearing applicability |
| 3 | test-case | same as use-case | same as use-case |
| 4 | test-report | same as use-case | same as use-case |
| 5 | glossary | `--term`, `--definition`; no title; labels/references; status | Definition section; term/definition parsing |
| 6 | ui-spec | title, `--view`, `--description`; edit `--view`, `--description`, `--editor` | View + Description sections |
| 7 | audit | `--title`, `--content`, `--audit-type`; no positional title; edit `--editor`; no verb about text | Audit Type + Content sections; front-matter `type`; audit-type validation |
| 8 | rule | title, `--description`, `--rationale`, `--applicability`, `--requirement`; edit `--no-requirement` (optional value), `--editor` | Rationale/Applicability/Requirements sections; requirement list add/remove |
| 9 | req / nfr | title positional or `--title`, `--story`, `-d/--description`, required `--group`, labels/references; `create-group` title/story/description; edit `--story` | Group-ID validation and mismatch across req/nfr; dotted/grouped IDs; `--story` accepted but ignored for nfr, as today; nfr keeps the current business-requirement about text |
| 10 | ui-mockup | `--task`, required repeatable `--spec`, repeatable `--ref`; edit `--status` and `--superseded-by` (conflicting), `--editor`; `supersede <old> <new>`; no `status` verb; no verb about text | HTML rendering/parsing, metadata comments, `.html` paths, supersede semantics |

Per module, one change unit: add the spec table, switch its command construction,
then move its handler to the neutral entry point, delete its enum/`run`, and
replace module-local configuration/directory helpers with the scope. The tree
change can land first with the enum still used only for dispatch, because the
metadata reproduces the same argument ids; this keeps help locked before
dispatch changes begin.

## 6. Compatibility contract

### Help

Byte-for-byte equality with the pre-refactor binary for:

- top-level `--help` (default configuration and a fixture configuration with a
  generic slug);
- every enabled built-in typed slug `--help`;
- every verb `--help` of every typed slug, including `create-group` and
  `supersede`;
- the generic fixture slug and all its verbs;
- unknown slug, disabled slug, `nfreq`, unknown verb, missing required option,
  and malformed option (stderr and exit code 2);
- an application error with `--json` (stdout, stderr, exit code).

Known preserved oddities, which are explicitly required not to change:

- `audit` and `ui-mockup` verbs have an empty about-line.
- `nfr` verbs carry the "business requirement" about text.
- `req create --title` displays `<TITLE_FLAG>`.
- `rule edit` uses the next-line help layout because of its long optional-value
  flag.
- `ui-mockup edit` offers `--status` and `--superseded-by` even though
  `ui-mockup` has no `status` verb.

### Behaviour

- Exit codes 0/1/2/3/4 and the `--json` error object are unchanged
  (REQ-01.004).
- Each type keeps its field set, section layout, ID shape, file path, archive
  location, list/view text, and JSON keys (for example glossary list JSON keeps
  its `term` key).
- `ui-mockup archive` and `ui-mockup delete` keep their module path and HTML
  metadata semantics, and `ui-mockup` keeps no `status` verb.
- Config-defined types keep the seven-field generic model and the generic
  command about-lines.
- `create-group`, `--group`, and `supersede` remain available only where
  declared; `status` remains absent for `ui-mockup`.
- No `nfreq`, `deprecate`, or `restore` returns.

## 7. TASK-119 / TASK-120 boundary

Removed by TASK-119:

- per-type `clap` enums and their `run` functions;
- `TypedHandler` command-construction match in `src/cli/tree.rs`;
- `dispatch_typed_module`, its `from_arg_matches` calls, and the typed `status`
  special case in `src/cli/dispatch.rs`;
- the slug-based `document::archive`, `document::delete`, and `document::status`
  helpers and their typed-module call sites;
- module-local `load_config`, `type_entry`, `active_dir`, and `archived_dir`
  helpers and every `cfg.doc_directory` join in the ten typed modules.

Retained by TASK-119, removed by TASK-120:

- `record::descriptor::descriptor_for_doc_type` call sites in the ten typed
  modules. After TASK-119 they are used only for dotted-ID resolution; their
  directory and prefix inputs come from the descriptor.
- `document::record_descriptor_for`, `src/init.rs`, `src/record/migrate.rs`, and
  the registry agreement test.
- `generic_document_descriptor` as the generic record-descriptor constructor.

The final summary must list every removed enum, dispatch arm, helper, and
directory join, and name the remaining lookups as TASK-120 scope.

## 8. Test plan

1. **Pre-refactor golden capture (first commit).**
   `tests/cli_help_golden.rs` plus `tests/fixtures/cli-help/papyrus.config.yml`
   and `tests/golden/cli-help/`. A regeneration switch writes the files; normal
   runs compare stdout/stderr/exit code byte-for-byte. Commit the captured
   files before changing production code.
2. **Builder unit and invariant tests.**
   Every typed descriptor verb has a spec and vice versa; canonical verb order
   is asserted; generic and typed specs produce the expected argument ids,
   value names, requiredness, and conflicts.
3. **Dispatch tests.**
   Direct `document::run` tests exist for a config-defined generic slug
   (create, list, view, edit, archive, delete, status) and for a built-in typed
   slug (`req`/`nfr`, `ui-mockup`), covering typed create, list, view, and
   archive; both reach the same entry point and preserve their outputs.
   `create-group`, `supersede`, typed `edit`, and typed `delete` are asserted by
   the per-type CLI suites, not by direct `document::run` tests. Adding direct
   `document::run` coverage for those verbs (and for the `ui-mockup`
   archive/delete exception) is optional hardening and not a TASK-119 exit
   condition.
4. **Existing suites unchanged.**
   `tests/cli_dispatch.rs`, `tests/req_cli.rs`, `tests/glossary_cli.rs`,
   `tests/business_rules_cli.rs`, `tests/ui_spec_cli.rs`,
   `tests/audit_records_cli.rs`, `tests/technical_specifications_cli.rs`,
   `tests/use_cases_cli.rs`, `tests/test_cases_cli.rs`,
   `tests/test_reports_cli.rs`, `tests/ui_mockup_cli.rs`, the matching smoke
   suites, `tests/release_document_types_smoke.rs`, and
   `tests/document_type_no_drift.rs` must pass; only the
   `document_invocation_parsing_table` cases may need extension for the new
   argument map and extension verbs.
5. **Mechanical guards.**
   No `ReqCmd`/`GlossaryCmd`/`RuleCmd`/`UiSpecCmd`/`AuditCmd`/`TechSpecCmd`/
   `UseCaseCmd`/`TestCaseCmd`/`TestReportCmd`/`UiMockupCmd` in `src/`; no
   `augment_subcommands` or `TypedHandler` in `src/cli/`; no
   `dispatch_typed_module`; no `from_arg_matches` for document types; no
   `cfg.doc_directory` join in the ten typed modules. Extend
   `tests/document_type_no_drift.rs` rather than adding a third scanner.
6. **Quality gates.** `cargo build`, full `cargo test`, and
   `cargo clippy --all-targets -- -D warnings` clean.

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Help drifts because argument insertion order or value names differ | Golden capture first; explicit ordered metadata; assert ids/value names; per-verb golden comparison |
| Options differ only subtly between near-identical types (required vs optional, label vs reference, applicability) | Keep literal per-type tables; share only identical definitions; per-type CLI suites |
| Generic path regresses while argument parsing is unified | Keep the convenience fields and generic functions unchanged; generic end-to-end and editor tests |
| Requirement group validation loses sibling-type lookup | Pass the registry to the requirement handler for the req/nfr mismatch check; existing group tests |
| UI mockup HTML or supersede regresses | Preserve module HTML/parse code; targeted ui-mockup tests plus `document.rs` typed tests |
| Typed list/view output changes by switching to generic list/view | Delegate list/view to the owning module; never route typed list/view through the generic implementation |
| Directory scope bypasses containment or creates directories in the wrong place | Resolve scope once in `document.rs` with the existing containment helper; path assertions in per-type tests |
| Enum removal breaks tests that construct enums directly | Update the four `src/document.rs` constructions and the parsing table in the same change |
| A descriptor verb without a spec silently disappears from help | Registry invariant test plus builder assertion |
| Hidden behaviour change in exit codes or JSON | Golden/behavioural matrix and existing suites; no handler error text changes |

## 10. Assumptions

- **Assumption:** preserving current behaviour takes precedence over BR-009 to
  BR-020 wording where they already disagree (for example `nfr --story` and
  `nfr` about-lines). No requirement or business-rule edit is made here.
- **Assumption:** option metadata belongs on `TypedSchema` in `registry.rs`; no
  new ADR is needed because ADR-0016 decision 4 and ADR-0017 decision 7 already
  place commands and typed options in descriptor data.
- **Assumption:** extension verbs (`create-group`, `supersede`) also enter
  `document::run` in this ticket, because the ticket routes every document
  invocation through one handler.
- **Assumption:** directory resolution stays in `document.rs` (pure descriptor
  inputs plus existing containment) to keep the registry free of I/O.

## 11. Traceability

- ADR-0016 decisions 4, 5, 7 and downstream constraints 3–5.
- ADR-0017 decisions 2, 3, 4, 7 and downstream constraints 1, 5.
- REQ-07.003 (dynamic command family, help, unknown/disabled slug), REQ-07.004
  (generic model), REQ-07.001 (status), REQ-05.001/05.002 (group verbs),
  REQ-05.008 (mockup/supersede), REQ-01.004 (exit codes/global options),
  REQ-01.005 (ID padding).
- BR-001, BR-009–BR-014, BR-017–BR-020 (per-type command content).
- TASK-108 (CLI cutover), TASK-110 (generic handler), TASK-117 (checkpoint
  finding), TASK-120 (second lookup), TASK-121 (descriptor cache).
