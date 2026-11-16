---
id: TASK-121
title: >-
  Amend ADR-0016, ADR-0017, ADR-0018, and ADR-0023 for the derive/builder split,
  cross-category verbs, and release archive policy
status: Done
assignee:
  - architect
created_date: '2026-11-15T11:28:00Z'
updated_date: '2026-11-15T12:25:00Z'
labels:
  - docs
  - architecture
milestone: MS-020
dependencies: []
references:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - >-
    docs/architecture/adr/ADR-0021-man-page-strategy-for-a-dynamic-command-surface.md
documentation:
  - docs/architecture/adr/index.html
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
modified_files:
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
priority: high
ordinal: 41310
---


## Description
**Why.** The m-18 checkpoint found four accepted ADRs describing the delivered system inaccurately. Each amendment records what m-18 actually delivered; statuses stay `Accepted` and no new ADR is created.

**Delivered facts to record (evidence, verified on branch `task/TASK-126`).**

| Fact | Evidence |
|---|---|
| Built-in command trees are `clap` **derive** | `BuiltinCmd`, `ConfigCmd`, `InitArgs` in `src/cli/builtin.rs`; `#[derive(Subcommand)]` enums in `src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`, `src/import.rs`; grafted by `BuiltinCmd::augment_subcommands(cmd)` in `src/cli/tree.rs`, converted by `BuiltinCmd::from_arg_matches` in `src/cli/dispatch.rs` |
| Root command, global arguments, and the whole document surface are **builder-built** | `cli::tree` (root + `--json`/`--dir`/`--config`/`--no-color`), `cli::tree::document_command`, `cli::tree::verb_command` built from `descriptor.ordered_verb_specs()` |
| `BUILTIN_COMMANDS` holds names only | `BuiltinEntry { name: &'static str }`; consumers are config validation (`src/config.rs`) and the tree's slug-collision guard (`src/cli/tree.rs`); the man page does not exist (ADR-0021 is `Proposed` and deferred) |
| Owned runtime command names | `clap` features `derive` + `string` (`Cargo.toml`); `Command::new(descriptor.slug.clone())` owns its name, so no `'static` leak |
| `verbs` exists on every descriptor, in every category | `TASK_VERBS`, `MILESTONE_VERBS`, `RELEASE_VERBS`, `DECISION_VERBS`, document baseline sets in `src/registry.rs` |
| `ContentModel::HtmlMockup` keeps `archive`/`delete` in the mockup module | `document::run` routes `archive`/`delete` for `HtmlMockup` through `TypedHandler::UiMockup`; `src/ui_mockup.rs` archives via `record::store::archive_content`; keyed on content model, no slug literal in `src/document.rs` |
| Typed delegation is broader than extension verbs | for `ContentModel::Typed`, `create`/`create-group`/`list`/`view`/`edit`/`supersede` go to the owning module through `TypedHandler`; `archive`, `delete`, and `status` stay on the shared `document.rs` path |
| Releases never move | `ArchivePolicy::None` on the release template; `release close` rewrites `status` `planned -> released` in place |

**What.** Amend ADR-0016, ADR-0017, ADR-0018, and ADR-0023 in place, then regenerate the ADR index.

**Amendment convention (all four files).** Follow the ADR-0022 precedent: keep `## Status` starting with `Accepted` and append `; amended 2026-11-15 by owner decision (<scope of the amendment>) without status change.`; at each amended clause add an italic note `*Revision (2026-11-15, owner decision): <what changed and why>.*` immediately after the clause. Do not add a new ADR, do not change a status, do not renumber, do not create a markdown ADR index.

**A. ADR-0016 — `docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md`.**

1. `## Status`: append the `amended 2026-11-15` clause naming decision 1, decision 2, decision 4, decision 10's consequence, and downstream constraint 2.
2. Decision 1: replace the claim that the derive macros were replaced for *every* built-in with the split. Required content: `cli::tree` builds the root command and the global arguments with the builder API; `cli::tree::document_command`/`verb_command` build the entire document surface with the builder API; the built-in trees remain `clap` **derive** in `src/cli/builtin.rs` (`BuiltinCmd`, `ConfigCmd`, `InitArgs`) plus one `#[derive(Subcommand)]` enum per family in `src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`, `src/import.rs`, grafted with `BuiltinCmd::augment_subcommands(cmd)` and parsed with `BuiltinCmd::from_arg_matches`; `clap` remains the only parser and the only dependency. Add the revision note stating that converting the built-ins to the builder API was considered and rejected by the owner, so the split is the contract.
3. Decision 2: state that `cli::BUILTIN_COMMANDS` is a list of **names only** (`BuiltinEntry { name }`), not of `clap::Command`s and handlers; its consumers are the reserved-name validation (ADR-0019, `src/config.rs`) and the tree's pre-build slug-collision guard; it stays the single reserved-name source and remains reserved for the packaged man page (ADR-0021, still `Proposed` and deferred — not a delivered consumer). Add the revision note.
4. Decision 4: record that the `clap` `string` feature is what lets a builder-built document command own a runtime slug (`Command::new(String)`, no `'static` leak) while `derive` stays enabled for the built-in trees. Revision note.
5. Decision 10 consequence: correct "Deleting twelve static command enums" to eleven static document-type variants (ten per-type modules plus the `nfreq` alias) and scope the "lost compile-time argument checking" and "help text is no longer a compile-time constant" bullets to the document surface only.
6. Downstream constraint 2: restate so it matches the split — the five named built-in family modules and `src/cli/builtin.rs` may declare `clap` derive commands and are the only such declarations; no module may build or inspect a `clap::Command` for a document type outside `src/cli/`; `document::run` receives a `DocumentInvocation`, never `ArgMatches`. Revision note.

**B. ADR-0017 — `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md`.**

1. `## Status`: append the `amended 2026-11-15` clause naming the decision 2 `verbs` row, decision 3, and downstream constraint 3.
2. Decision 2 `verbs` row: state that the field exists on every descriptor in every category and list the delivered sets — task `create/list/view/edit/archive/restore/delete`; milestone `create/list/view/edit/archive/delete`; release `create/list/view/edit/close`; decision `create/list/view/edit`; document `create/list/view/edit/archive/delete` plus `status` where the type stores front-matter status plus declared extensions. Widen the source-of-truth column to REQ-02.001–004, REQ-03.001–002, REQ-04.001, REQ-07.003, REQ-05.001/002/008.
3. Decision 3: state that `verbs` is not a document-only field; the document baseline family stays a **lower bound** with `status` conditional and extensions declared by a built-in descriptor. Keep the owner D1 rationale sentence unchanged.
4. Downstream constraint 3: record that decisions **and releases** have no archive policy (`ArchivePolicy::None`); every other descriptor uses `ArchivedFolder`. Revision note.

**C. ADR-0018 — `docs/architecture/adr/ADR-0018-generic-document-content-model.md`.**

1. `## Status`: append the `amended 2026-11-15` clause naming decision 3.
2. Decision 3: replace the sentence that a typed descriptor "delegates only its extension verbs" with the delivered split: `document::run` is the single entry for every document type; for `Generic` it handles all verbs; for `Typed` it dispatches `create`/`create-group`/`list`/`view`/`edit`/`supersede` to the owning module through `TypedHandler` while `archive`, `delete`, and `status` stay on the shared path; for `HtmlMockup` it dispatches every verb including `archive` and `delete` to the mockup module.

   Required exception wording: *a mockup's `archive` and `delete` stay in the mockup module because the record is HTML with metadata in an embedded comment, not front matter; that module must perform the move through `record::store::archive_content` (destination written before the source is unlinked, ADR-0022 decision 2) and must not reimplement the shared front-matter patch path; the delegation is keyed on `ContentModel::HtmlMockup`, never on the `ui-mockup` slug, so `src/document.rs` introduces no slug literal; a mockup has no `status` verb.* Revision note.

**D. ADR-0023 — `docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md`.**

1. `## Status`: append the `amended 2026-11-15` clause naming decision 2.
2. Decision 2 table, Release row archive cell: `none — the record is never moved; its state is the front-matter status`. Add one sentence to decision 2: a release has `ArchivePolicy::None`, `release close` rewrites `status` from `planned` to `released` in place through the single write path, and no `archive/releases/` path is resolved by any code path or descriptor. Revision note.

**E. Index.** Regenerate `docs/architecture/adr/index.html` by invoking the `adr-index-visualization` skill (its bundled `generate_adr_index.sh`), which must print `wrote docs/architecture/adr/index.html: 24 decision records` and exit 0. Never hand-edit the file.

**Out of scope.** Refactoring the built-ins to the builder API (owner decided against it and the amendment records the split as the contract); any other ADR decision; ADR-0021 (still `Proposed`, body editable but not part of this ticket); requirements, business rules, specifications, and code; adding a new ADR; creating a markdown ADR index.

**Assumptions.**

- Assumption: the ADR-0022 amendment pattern (dated `## Status` clause plus inline italic revision note, status unchanged) is the accepted way to record an owner-authorized correction to an `Accepted` ADR body; the owner decision of 2026-11-15 supersedes the skill's body-frozen rule for exactly these clauses.
- Assumption: the man page must be described as *reserved for* ADR-0021 rather than *consumed*, because ADR-0021 is `Proposed` with no selected option and `docs/man/` does not exist.
- Assumption: the `verbs` values recorded are the delivered sets; a requirement change would require a new amendment, not a broader rewrite.
- Assumption: code is the evidence of record for these four topics (TASK-119/120/121/124/125 are `Done`); no requirement or business-rule file is edited.
- Assumption: no feature specification is authored for this ticket — the four ADRs are the canonical artifacts, and this ticket carries the amendment plan.
- Assumption: dependencies are `none` blocking; the evidence tasks are already complete, so no dependency edge is added.

## Acceptance Criteria
- [x] ADR-0016 decision 1 records the derive/builder split with the exact built-in modules (src/cli/builtin.rs plus src/task.rs, src/milestone.rs, src/release.rs, src/decision.rs, src/import.rs), the BuiltinCmd::augment_subcommands graft, and the builder-built root, global arguments, and document surface
- [x] ADR-0016 decision 2 states that BUILTIN_COMMANDS carries names only, names the real consumers (config validation, tree collision guard), and describes the man page as reserved for ADR-0021 (Proposed, deferred) rather than delivered; it no longer claims the table holds a clap::Command and a handler
- [x] ADR-0016 decision 4 records the clap string feature as the reason a builder-built document command can own a runtime slug without a static leak, and that derive stays enabled for the built-in trees
- [x] ADR-0016 downstream constraint 2 permits clap derive command declaration in src/cli/builtin.rs and the five named built-in family modules, forbids document-type clap::Command construction outside src/cli/, and states document::run receives a DocumentInvocation and never ArgMatches
- [x] ADR-0016 decision 10's consequence states eleven static document-type variants (ten per-type modules plus the nfreq alias) and scopes the lost compile-time check and non-constant help-text bullets to the document surface
- [x] ADR-0017 lists the verbs field per category with the delivered sets (task create/list/view/edit/archive/restore/delete; milestone create/list/view/edit/archive/delete; release create/list/view/edit/close; decision create/list/view/edit; document baseline plus status plus declared extensions) and cites the matching requirement IDs
- [x] ADR-0017 decision 3 states that verbs is a field of every descriptor in every category and that the document baseline family remains a lower bound with conditional status and declared extensions
- [x] ADR-0017 downstream constraint 3 records that decisions and releases both use ArchivePolicy::None and that every other descriptor uses ArchivedFolder
- [x] ADR-0018 decision 3 records the delivered delegation split (Generic everything in document::run; Typed verbs through TypedHandler with archive, delete, and status shared; HtmlMockup every verb including archive and delete in the mockup module) and the exception wording: record::store::archive_content, destination before unlink, keyed on ContentModel::HtmlMockup, no slug literal in src/document.rs, no status verb
- [x] ADR-0023 decision 2 shows no archive location for releases, states ArchivePolicy::None, and records planned -> released in place through release close with no archive/releases/ path resolved by any code path or descriptor
- [x] Each of the four files starts ## Status with Accepted and carries an amended 2026-11-15 by owner decision clause plus a dated italic revision note at every amended clause; no statement remains in those files contradicting the delivered contract on these four topics; no status change, no renumbering, no new ADR, no markdown ADR index
- [x] docs/architecture/adr/index.html is regenerated by the adr-index-visualization script, reports 24 decision records, shows the four ADRs as Accepted, and renders the amended sections; the file is never hand-edited

## Definition of Done
- [x] #1 Only the four named ADR files and docs/architecture/adr/index.html are modified; no new ADR file, no markdown ADR index, no code, requirement, business-rule, or specification change
- [x] #2 git diff shows in-place edits only: no ADR removed, renamed, renumbered, or re-statused
- [x] #3 Every amended clause is traceable to a row of the delivered-facts table in the description; no unverified behavior is asserted
- [x] #4 The adr-index-visualization script exits 0 and reports 24 decision records, and index.html is newer than the four amended ADR files
- [x] #5 Implementation notes list the exact edits per ADR (section, clause, revision-note text) for the m-19 checkpoint review


## Notes
## Implementation Notes

### ADR-0016

Status line (verbatim): Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-15 by owner decision (decision 1 derive/builder split, decision 2 `BUILTIN_COMMANDS` names only, decision 4 `clap` `string` feature, decision 10's consequence count, downstream constraint 2, and downstream constraint 6) without status change.

Amended clauses and revision notes (verbatim):

- Clause context: 1. **One command tree is built at startup from the artifact registry** (`cli::tree(&ArtifactRegistry) -> clap::Command`, builder API). It is the single source o
  Note: *Revision (2026-11-15, owner decision): this decision previously stated that the derive macros were replaced by explicit builder construction for `Cli` and for *every* built-in subcommand. Converting the built-in trees to the builder API was considered and rejected by the owner, so the split above — builder for the root, the global arguments, and the whole document surface, derive for the built-in families — is the contract, not an interim state.*

- Clause context: 2. **Built-ins come from one static name list.** `cli::BUILTIN_COMMANDS` is a list of **names only** (`BuiltinEntry { name: &'static str }`), not of `clap::Comm
  Note: *Revision (2026-11-15, owner decision): this decision previously stated that the table listed, for every built-in, its name, its `clap::Command`, and its handler, and it named the packaged man page as a consumer. The delivered table holds names only; the man page does not exist (`docs/man/` is absent and ADR-0021 selects no option), so the two real consumers are configuration validation and the tree's collision guard.*

- Clause context: 4. **Document commands are built by one function, not one type.** `cli::document_command(descriptor)` produces a `clap::Command` whose about line is `Manage <sl
  Note: *Revision (2026-11-15, owner decision): this decision previously left the runtime-name mechanism unstated. The `clap` `string` feature (enabled together with `derive` in `Cargo.toml`) is what lets a builder-built document command own a runtime slug — `Command::new(descriptor.slug.clone())` takes an owned `String`, so no `'static` leak is needed — while `derive` stays enabled for the built-in family trees of decision 1.*

- Clause context: ## Consequences
  Note: *Revision (2026-11-15, owner decision): the consequence bullets were aligned with the delivered split — the removed static surface is eleven document-type variants (ten per-type modules plus the `nfreq` alias), the lost compile-time argument checking and the non-constant help text apply to the document surface only, the built-in family trees keep derive, and help for the built-ins is generated from their derive declarations rather than from `BUILTIN_COMMANDS`.*

- Clause context: 2. `src/cli/builtin.rs` and the five built-in family modules (`src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`, `src/import.rs`) may decla
  Note: *Revision (2026-11-15, owner decision): this constraint previously read "no module except `src/cli/` may construct or inspect a `clap::Command` or `ArgMatches`", which the delivered derive/builder split of decision 1 contradicts, because the built-in family trees are declared outside `src/cli/`. The permission is now enumerated — the named modules may declare derive commands, document-type command construction stays inside `src/cli/`, and the document handler boundary is the `DocumentInvocation` value. Constraint 1 is read the same way: the `src/lib.rs` prohibition stands, while `src/cli/` owns command-tree construction and parse dispatch.*

- Clause context: 6. `BUILTIN_COMMANDS` is the only list of built-in command names; ADR-0019 consumes it for reserved-name validation, and ADR-0021 is reserved to consume it once
  Note: *Revision (2026-11-15, owner decision): this constraint previously read "ADR-0019 and ADR-0021 must consume it", which implied a delivered man-page consumer. The table carries names only; ADR-0019 consumes it today and ADR-0021 remains reserved, consistent with decision 2.*

### ADR-0017

Status line (verbatim): Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-15 by owner decision (decision 2 `verbs` row, decision 3, decision 4 CLI bullet, and downstream constraint 3) without status change.

Amended clauses and revision notes (verbatim):

- Clause context: | `archive_policy` | `ArchivedFolder { state_field }` or `None` | REQ-05.007, REQ-04.002 |
  Note: *Revision (2026-11-15, owner decision): the `verbs` row previously described the field as the document baseline plus `status` plus extensions, which read as a document-only field. `verbs` exists on every descriptor in every category, and the delivered sets are the ones listed above (backlog and decision sets from REQ-02.001–004, REQ-03.001–002, and REQ-04.001). The same correction applies to decision 1's assembly sentence, which no longer refers to `cli::BUILTIN_COMMANDS` handlers because that table carries names only (ADR-0016 decision 2).*

- Clause context: 3. **The baseline verbs are a lower bound; `verbs` is a field of every descriptor, and document extensions are declared on a built-in descriptor.** `verbs` is *
  Note: *Revision (2026-11-15, owner decision): this decision previously presented the baseline verbs without stating that `verbs` is carried by every descriptor in every category; the field's scope and the conditional `status` rule are now explicit and the D1 rationale sentence is unchanged.*

- Clause context: 4. **One dispatch point per category, driven by the descriptor.**
  Note: *Revision (2026-11-15, owner decision): this decision previously said that `cli::tree` inserts "the descriptor's verbs", which is true only for the `Document` category. `src/cli/tree.rs` skips every non-document descriptor, so backlog and decision verbs come from their `clap` derive enums and `BUILTIN_COMMANDS` supplies only their names; the document surface is the only descriptor-driven one.*

- Clause context: 3. Every descriptor's `archive_policy` follows REQ-05.007 and ADR-0022; decisions **and releases** have no archive policy (`ArchivePolicy::None`), and every oth
  Note: *Revision (2026-11-15, owner decision): this constraint previously named only decisions as having no archive policy. Releases are the second family that never moves: their descriptor carries `ArchivePolicy::None` and `release close` rewrites `status` in place (ADR-0023 decision 2).*

### ADR-0018

Status line (verbatim): Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-15 by owner decision (decision 3 delivered delegation split) without status change.

Amended clauses and revision notes (verbatim):

- Clause context: A mockup's `archive` and `delete` stay in the mockup module because the record is HTML with metadata in an embedded comment, not front matter; that module must 
  Note: *Revision (2026-11-15, owner decision): this decision previously stated that a typed descriptor delegates only its extension verbs and that the baseline verbs are handled here. The delivered split is broader on both sides — typed modules also own the content verbs `create`/`list`/`view`/`edit` and their extensions, and the mockup module owns `archive` and `delete` as well — so the per-verb ownership above replaces the earlier sentence.*

### ADR-0023

Status line (verbatim): Accepted (2026-11-13) — accepted by explicit user decision of 2026-11-13; amended 2026-11-15 by owner decision (decision 2 release archive location) without status change.

Amended clauses and revision notes (verbatim):

- Clause context: A release has `ArchivePolicy::None`: `release close` rewrites `status` from `planned` to `released` in place through the single write path, and no `archive/rele
  Note: *Revision (2026-11-15, owner decision): the Release row previously listed `<backlog_directory>/archive/releases/` as its archive location. Releases never move — the descriptor carries `ArchivePolicy::None` and the record's state is its front-matter `status`, so the row and the sentence above now record the delivered contract.*

### Additional in-place edits without a standalone revision note

- ADR-0016 downstream constraint 1: wording narrowed to "command-tree construction and parse dispatch live in `src/cli/`" so it no longer forbids the derive declarations permitted by constraint 2; covered by the constraint 2 revision note.
- ADR-0017 decision 1: assembly sentence no longer references `BUILTIN_COMMANDS` handlers (the registry has its own templates), and a REQ-02/03/04 reference bullet was added; the correction is recorded in the decision 2 revision note.

### Index

`generate_adr_index.sh` -> "wrote docs/architecture/adr/index.html: 24 decision records", exit 0; regenerated output byte-identical to the committed page; four ADRs remain Accepted with 2026-11-15 amendment text. No code, requirement, or status changes.
