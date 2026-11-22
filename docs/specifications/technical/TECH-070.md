---
id: TECH-070
title: "Descriptor-Driven Document Command Dispatch"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-22T12:46:04Z'
labels: []
references:
  - ADR-0016
  - ADR-0017
  - ADR-0036
  - ADR-0039
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
  - REQ-05.011
  - REQ-07.001
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
  - ADR-042
status: verified
---
# TECH-070 Descriptor-Driven Document Command Dispatch

## Description
## Purpose

ADR-0016 makes `document::run` the single dispatch path for every document type
and makes `cli::document_command(descriptor)` the single command builder.
ADR-0036 replaces the per-type content model with one descriptor model: a
`TypeDescriptor` carries the slug, prefix, directory, enabled state, status
vocabulary, `DocumentModel`, role field sets, reference visibility, and
presentation facts, and no per-type module supplies verbs, options, or help.

Every document command — seed type or user-defined type, `SingleMarkdown`,
`GroupedMarkdown`, or `SingleHtmlMockup` — is built from its descriptor and
enters `document::run`. The verb set is a function of the document model; the
options are a function of the field sequence and the fixed title contract. The
engine emits only `PatchOp` values, and `record::store` remains the only writer.

## Scope

### In scope

- `src/registry.rs`: one descriptor per document type, with model-derived verbs
  and the field sequences that generate options.
- `src/cli/tree.rs`: one descriptor-driven `document_command` for every document
  model; no `TypedHandler` construction match and no per-type command enum.
- `src/cli/dispatch.rs`: every `ArtifactCategory::Document` slug enters
  `document::run`; no per-type module dispatch and no typed `status` special
  case.
- `src/document.rs`: one document engine parameterized by the descriptor; it
  resolves one scope, builds one field map, and emits only patch operations.
- Help text and option order generated from the descriptor and the field
  sequence, with the frozen about lines preserved.

### Out of scope

- Backlog and decision commands, which stay bespoke.
- Adding or removing a document model, a status model, or a configuration key.
- A new template engine, dependency, or second writer.

## Current state and target

| Concern | Current | Target |
|---|---|---|
| Document model | the superseded content model with generic, typed, and HTML variants | One `DocumentModel` enum with `SingleMarkdown`, `GroupedMarkdown`, `SingleHtmlMockup` |
| Typed subcommand tree | `TypedHandler` match calling per-module `augment_subcommands` | `document_command` builds from the descriptor's model-derived verbs and field sequence |
| Parsing | One `clap` enum per module | One `DocumentInvocation` with an argument map keyed by declared field options |
| Dispatch | Per-type module `run` functions | `document::run` over one descriptor-driven engine |
| Field storage | Per-type section mapping and writers | `PatchOp` values from the field sequence; `record::store` writes |
| Extension verbs | Module enum arms | Model-derived `create-group`, `--group`, and `supersede` routed through `document::run` |

## 1. Model-derived verbs and descriptor option metadata

The verb set is derived from the document model and is never declared in
configuration or supplied by a per-type module or table:

- Every model has `create`, `list`, `view`, `edit`, `archive`, and `delete`.
- `status <id> <value>` is added for `SingleMarkdown` and `GroupedMarkdown`.
- `create-group` and a required `--group` on `create` are added for
  `GroupedMarkdown`.
- `supersede` is added for `SingleHtmlMockup`; that model has no `status` verb.

Each declared field contributes one long option, generated from the field
sequence of the record's role:

- The option name is the field's `option`, or its `key` in kebab case when
  `option` is absent.
- The option takes the field's `label` and `help`; the field `kind` decides the
  value form: `text` is a single value, `multiline` also accepts `-` to read
  standard input, and `list` is repeatable.
- The option is offered only on the verbs listed in the field's `verbs`.
- A `required` field must be supplied on create; an edit may change it but not
  clear it, and an optional field may be cleared.
- Field options follow the field sequence order in parsing and help, including
  the role-appropriate sequence of a `GroupedMarkdown` type.

The title is fixed and is not generated from a field: it is the first
positional `TITLE` on `create` and `create-group`, and `--title` is accepted on
`create`, `create-group`, and `edit`. Supplying both forms on one invocation is
rejected. There is no `--term` option. `--label` and `--reference` remain
general repeatable metadata rather than fields, the engine-owned `group` link
has no option, and no create or edit option sets `status`.

The metadata carries no I/O and no slug-to-behaviour mapping. The registry is
data-only; the command builder, the parser, and the engine read it.

## 2. Command construction

`cli::tree::document_command(descriptor)`:

- keeps the slug as the command name and `Manage <slug> documents.` as the
  about line;
- includes a verb only when the descriptor's model exposes it;
- emits verbs in the canonical order `create`, `create-group`, `list`, `view`,
  `edit`, `archive`, `delete`, `supersede`, `status`;
- builds each verb's arguments from the field sequence and the fixed title
  contract, with no per-type match;
- leaves the four global options (`--json`, `--dir`, `--config`, `--no-color`)
  on the root command, so clap propagates them exactly as for other commands.

`tree()` keeps the reserved-name collision guard and inserts document commands
in descriptor order. Enabled types are ordered by seed `order`, ties broken by
slug. No module outside `src/cli/` constructs or inspects a `clap::Command`.

**Amendment (2026-11-22).** Enabled document types are listed in effective
`document_types` map order, which is slug order, not seed `order`. The current
order is the frozen order; seed `order` remains presentation data and never
reorders commands or lists (ADR-042 decision 4). The preceding seed-order
sentence is corrected accordingly.

## 3. Invocation contract

One parser, `parse_document_invocation`, produces:

- `slug`;
- `DocumentVerb`, including `CreateGroup` and `Supersede`;
- a generic argument map holding every explicitly present non-global field
  argument by field key and its values (empty value list for boolean switches);
  the map preserves "absent" versus "present with no value" for optional-value
  flags;
- the convenience fields used by shared metadata handling (`id`, `title`,
  `labels`, `references`, `editor`, `status_value`, `force`).

The parser does not know any type's option set. It rejects an unknown verb
instead of defaulting to `list`. The command tree makes that error unreachable
through the CLI; the check prevents a future tree defect from executing the
wrong verb. An unknown slug, or the slug of a disabled entry, is a usage error
with no fallback to another command.

## 4. `document::run` and the document engine

`run(ctx, registry, invocation)`:

1. Resolves the descriptor from the registry and validates category, enabled
   state, and verb membership.
2. Resolves one scope from the descriptor: configuration, active directory,
   archived directory, prefix, and the effective `config.id_padding`.
   Directory resolution is derived from the descriptor's configuration key and
   relative value and is contained within the project directory. `DocumentScope`
   is computed in `document.rs`, so the registry remains I/O-free.
3. Loads the target record for `edit`, `status`, `archive`, and `delete`, and
   builds the model-neutral field map for its model and role:
   `SingleMarkdown` uses the top-level `fields` with role `Single`;
   `GroupedMarkdown` uses `group.fields` with role `Group` or `member.fields`
   with role `Member`; `SingleHtmlMockup` has no field sequence and carries
   fixed metadata.
4. Builds the operation list for the verb. The engine emits only `PatchOp`
   values: `SetFrontMatter` or `SetFrontMatterValue` for a `frontmatter_key`,
   `SetSection` or `ClearSection` for a `heading`, `SetHeadingTitle` for the
   title, and the model-specific operations for the group link, HTML metadata,
   and supersession. List fields become front-matter sequences or section lists
   under the same operations.
5. Applies the operations through `record::patch` and writes once through
   `record::store::write_atomic`. The engine never writes a record body, never
   formats a `---` block, never re-renders a body, never calls `fs::write`, and
   holds no per-type writer. A create and an edit each produce one patch, and a
   patch that changes nothing performs no write.
6. Enforces required-field, reserved-key, group-link, and status rules when the
   patch is built, before any write. A failure produces one validation outcome
   and writes nothing.

`archive`, `delete`, and `status` use the shared record path for the two
Markdown models. `SingleHtmlMockup` keeps its HTML metadata path: `archive` and
`supersede` maintain the metadata comment, `delete` follows the HTML
archive-then-remove flow, and `status` is never routed there because the model
declares no such verb. Group archiving or deletion changes no member; membership
is engine-owned and immutable after creation.

## 5. Model-derived verb matrix

| Model | Records and IDs | Verbs | Field sets | Status behaviour | Engine-specific rules |
|---|---|---|---|---|---|
| `SingleMarkdown` | One record per ID; plain ID `<PREFIX>-<zero-padded sequence>`. | Baseline plus `status`. | One `fields` sequence. | Front-matter `status`: `active`, a declared custom status, or `archived`; `status` and `archive` are the only writers. | None. |
| `GroupedMarkdown` | A group record with a plain ID and member records with dotted IDs `<PREFIX>-<group-sequence>.<member-sequence>`. | Baseline plus `status` plus `create-group`. | `group.fields` and `member.fields`, each a sequence; no top-level `fields`. | Front-matter `status` on both roles, under the same rules as `SingleMarkdown`. | The engine owns the group link; membership is immutable after creation; no cascade on archive or delete; merged group-then-member ordering. |
| `SingleHtmlMockup` | One record per ID; plain ID. | Baseline plus `supersede`; no `status` verb. | No field sequence; `fields` must be empty. | Fixed `pending`, `approved`, and `archived` metadata in the HTML metadata comment; no custom statuses; `archive` is the only path to `archived`. | HTML body; the title contract applies to the HTML metadata; `supersede` replaces the superseded metadata. |

Two types of the same model expose the same verb set; each derives its option
set from its own field sequence and role.

## 6. Compatibility contract

### Help

- Byte-for-byte equality with the pre-refactor binary for the frozen about
  lines: the root about line, the per-type line `Manage <slug> documents.`,
  each enabled type's help line, each verb's about line, the type order, and
  the verb listing of `papyrus <slug> --help`.
- **Amendment (2026-11-22).** Per-verb about lines are not frozen per type. The
  engine generates each verb's about line as generic text from the document model
  and the field sequence; no per-type verb about text is preserved (ADR-042
  decision 2, ADR-036 decision 8).
  The frozen help surface is narrowed to the root about line, `Manage <slug>
  documents.`, each enabled type's help line, the type order, and the verb listing
  of `papyrus <slug> --help`; per-verb about text is generated prose like
  per-option prose.
- Per-option help prose and option order are generated from field data
  (`label`, `help`, and sequence position), so a seed change may regenerate
  them; such a regeneration is not a help regression.
- Unknown slug, disabled slug, unknown verb, missing required option, and
  malformed option produce a usage error with exit code 2 on standard error.
- The `audit` positional title is additive; existing `--title` names and
  meanings are preserved, and no existing positional argument is renamed,
  removed, or reordered.

### Behaviour

- Exit codes 0/1/2/3/4 and the `--json` error object are unchanged
  (REQ-01.004).
- Record files, JSON output, and status codes are strict; rendered HTML is
  semantically equivalent. A seed type keeps its field set, section layout, ID
  shape, file path, archive location, list/view text, and JSON keys.
- `create-group`, `--group`, and `supersede` remain available only where the
  model declares them; `status` remains absent for `SingleHtmlMockup`.
- Config-defined types use the same model-derived command family and the
  `SingleMarkdown` default.
- No `nfreq`, `deprecate`, or `restore` returns.

## 7. Generation invariants

- Every verb exposed by a model has a generated command with its arguments.
- Every declared field option appears only on the verbs in its `verbs` list,
  and no option exists for a reserved key.
- No document slug literal selects a verb, an option, or a help line.
- No per-type module, per-type command enum, or per-type verb table exists.
- The engine emits only `PatchOp` values; `record::store` is the only writer.
- A missing required field, a reserved field key, an invalid group link, or an
  invalid status is rejected before any write.

## 8. Test plan

1. **Help golden capture.** A fixture compares stdout, stderr, and exit code for
   the top-level tree, every enabled slug, every verb, a config-defined slug,
   and the exit-2 usage errors. The frozen about lines must match; generated
   per-option prose is compared against the seed.
2. **Model matrix.** One type per model runs create, list, view, edit, archive,
   delete, and the model-specific verbs (`status`, `create-group`, `supersede`)
   through `document::run`; grouped roles and HTML metadata are asserted.
3. **Patch and writer guard.** A contract test asserts the engine produces
   `PatchOp` values only, contains no `fs::write` or body rendering, and writes
   through `record::store`; an empty patch performs no write.
4. **Validation.** A create that omits a required field, an edit that clears a
   required field, a reserved field key, an unknown group, an archived group,
   and a cross-type group each produce one validation outcome and no write.
5. **No-drift scanner.** Production lines contain no document `clap` enum, no
   per-type `augment_subcommands`, no per-type `run` dispatch, no
   superseded content model, and no slug literal selecting document behaviour.
6. **Quality gates.** `cargo build`, full `cargo test`, and
   `cargo clippy --all-targets -- -D warnings` are clean.

## 9. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Help drifts because argument insertion order or value names differ | Golden capture; ordered field metadata; per-verb comparison; frozen about lines |
| Field options differ subtly between two types of the same model | Options are generated from each type's own field sequence; per-type CLI suites assert them |
| The generic path regresses while parsing is unified | Shared metadata handling and field-map tests; generic end-to-end tests |
| Requirement group validation loses sibling-type context | The engine reads the registry for the group-mismatch check; existing group tests |
| A model-specific operation becomes a second writer | Patch-only contract test; single writer remains `record::store` |
| A descriptor verb disappears silently | Generation invariant test asserts every model verb has a command and every field option is offered |

## 10. Assumptions

- **Assumption:** preserving record files, JSON, status codes, and the frozen
  about lines takes precedence over any later requirement wording; no document
  behaviour changes beyond the accepted descriptor model.
- **Assumption:** option metadata belongs on the descriptor's field sequence,
  not on a per-type schema; the model supplies the verbs.
- **Assumption:** extension verbs (`create-group`, `supersede`) enter
  `document::run` with every other document invocation.
- **Assumption:** directory resolution stays in `document.rs` so the registry
  remains free of I/O.
