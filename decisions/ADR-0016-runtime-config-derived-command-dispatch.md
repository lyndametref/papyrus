---
id: ADR-0016
title: "Runtime Config-Derived Command Dispatch for Document Slugs"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-22T12:45:34Z'
labels: []
references:
  - ADR-042
---
# Runtime Config-Derived Command Dispatch for Document Slugs

## Status
accepted
## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-07.002 makes the `document_types` map key the document slug, and REQ-07.003 makes that slug the CLI command name: every enabled entry must expose `papyrus <document-slug>` with the baseline verb family `create`, `list`, `view`, `edit`, `archive`, `delete`. Front-matter-backed document types add `status <id> <value>` (REQ-07.001), a built-in typed type may add verbs of its own (`create-group`, `--group`, `supersede`), and UI mockups have no `status` verb. `papyrus --help` must list every enabled slug, `papyrus <slug> --help` must document that type's verbs, an unknown or disabled slug must be a usage error (exit code 2) with no fallback, and a configured slug can never shadow a built-in command.

Current code shape:

- `src/lib.rs` declares `Cli` and `Commands` with `clap` **derive** macros and dispatches with one `match` arm per variant. Twenty subcommands are compiled in, including eleven document-type variants (`Req`, `Nfreq`, `Glossary`, `UiSpec`, `TechSpec`, `Rule`, `UiMockup`, `Audit`, `UseCase`, `TestCase`, `TestReport`) that REQ-07.003 removes as static commands.
- `Cli::parse()` calls clap's exiting parser, so parse errors bypass `AppError`, the project's exit-code mapping (0/1/2/3/4, REQ-01.004) and the `--json` error object.
- Each document module owns its own `clap::Subcommand` enum (`ReqCmd`, `TechSpecCmd`, …) with type-specific verbs (`CreateGroup`, `Deprecate`, `Restore`, `Supersede`).
- `--json`, `--dir`, `--config`, `--no-color` are `global = true` derive arguments. They are needed to *find* the configuration, yet the configuration is needed to *build* the command tree — a circular dependency that derive cannot express.
- clap derive can only declare subcommands the compiler can see. A runtime entry set is not expressible; the only derive escape hatch is `allow_external_subcommands`, which disables per-subcommand `--help`, typed parsing, and help/usage listing.

Forces: no new dependency (ADR-0005); exit codes and JSON errors must stay exactly as documented in REQ-01.004; help must stay accurate for a set that changes per project; a failed configuration must fail closed (REQ-07.002); the built-in command surface must not regress; and the same built-in table must be reusable for validation (ADR-0019) and documentation (ADR-0021) so the three cannot drift.

## Decision
1. **One command tree is built at startup from the artifact registry** (`cli::tree(&ArtifactRegistry) -> clap::Command`, builder API). It is the single source of parsing, usage, and help. The builder and the derive macros are **split by surface**: `cli::tree` builds the root command and the global arguments (`--json`, `--dir`, `--config`, `--no-color`) with the builder API; `cli::tree::document_command` and `cli::tree::verb_command` build the entire document surface with the builder API; the built-in trees remain `clap` **derive** in `src/cli/builtin.rs` (`BuiltinCmd`, `ConfigCmd`, `InitArgs`) plus one `#[derive(Subcommand)]` enum per family in `src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`, `src/import.rs`, grafted onto the root with `BuiltinCmd::augment_subcommands(cmd)` and parsed with `BuiltinCmd::from_arg_matches`. `clap` remains the only parser and the only dependency.

   *Revision (2026-11-15, owner decision): this decision previously stated that the derive macros were replaced by explicit builder construction for `Cli` and for *every* built-in subcommand. Converting the built-in trees to the builder API was considered and rejected by the owner, so the split above — builder for the root, the global arguments, and the whole document surface, derive for the built-in families — is the contract, not an interim state.*

2. **Built-ins come from one static name list.** `cli::BUILTIN_COMMANDS` is a list of **names only** (`BuiltinEntry { name: &'static str }`), not of `clap::Command`s and handlers. The names in that table (`init`, `config`, `task`, `milestone`, `release`, `decision`, `search`, `browser`, `import`, `help`, `version`) are the reserved names. Its consumers are the reserved-name validation of ADR-0019 (`src/config.rs`) and the pre-build slug-collision guard in `src/cli/tree.rs`; it is the single reserved-name source and remains **reserved for** the packaged man page (ADR-0021, still `Proposed` and deferred — not a delivered consumer). `help` and `version` stay clap-provided subcommands/options and are listed as reserved.

   *Revision (2026-11-15, owner decision): this decision previously stated that the table listed, for every built-in, its name, its `clap::Command`, and its handler, and it named the packaged man page as a consumer. The delivered table holds names only; the man page does not exist (`docs/man/` is absent and ADR-0021 selects no option), so the two real consumers are configuration validation and the tree's collision guard.*

3. **Two-phase startup, in fixed order.**

   - **Phase 1 — context pre-scan.** A lexical scan of `argv` (not a parser) extracts only `--dir <path>`, `--dir=<path>`, `--config <path>`, `--config=<path>` and the corresponding `PAPYRUS_DIR`/`PAPYRUS_CONFIG` environment variables, and resolves the project directory and configuration path exactly per REQ-01.003. The scan stops at `--`. It never interprets subcommands, verbs, or any other option.
   - **Phase 2 — configuration.** For every command except `init`, the selected configuration is loaded **and fully validated** (ADR-0019), then the registry is built (ADR-0017). A missing or invalid configuration aborts here with the documented outcome (exit 1 for not found, exit 4 for validation); no command tree is built from a partial type set.
   - **Phase 3 — single parse.** `cli::tree` is built from the registry and `try_get_matches_from(argv)` parses the real invocation exactly once. Phase 1's result is discarded except for selecting the file.
   - **Phase 4 — dispatch.** `match` on the resolved command: a built-in handler, or `Command::Document { slug, verb }`.

   `papyrus init` skips phases 2–3's configuration load, because REQ-01.001 requires it to run without a configuration file.

   *Revision (2026-11-20, amended by ADR-037): phases 1–4 and their order are unchanged. The validate-then-build sequence of phase 2 is also the reload path for the browser's serving snapshot: a configuration change is loaded and fully validated (ADR-0019), then the registry is built (ADR-0017), and only then is the route table derived, so a reload passes the same gates as startup. A reload failure after startup does not abort the process and does not produce a tree or snapshot from a partial type set; it keeps the last known good snapshot serving (ADR-037). The startup outcomes stated in phase 2 are unchanged.*

4. **Document commands are built by one function, not one type.** `cli::document_command(descriptor)` produces a `clap::Command` whose about line is `Manage <slug> documents.` and whose subcommands are the baseline verbs of REQ-07.003 (`create`, `list`, `view`, `edit`, `archive`, `delete`), plus `status <id> <value>` when the descriptor is front-matter-backed, plus any verb the descriptor declares as an extension. Every verb parses into one `DocumentInvocation { slug, verb: DocumentVerb, args: DocumentArgs }` value; `document::run(ctx, registry, invocation)` is the only handler. Type-specific verbs (REQ-05.001 `create-group`, REQ-05.002 `--group`, REQ-05.008 `supersede`) and the `status` presence rule are descriptor data, not a second handler (ADR-0017).

   *Revision (2026-11-15, owner decision): this decision previously left the runtime-name mechanism unstated. The `clap` `string` feature (enabled together with `derive` in `Cargo.toml`) is what lets a builder-built document command own a runtime slug — `Command::new(descriptor.slug.clone())` takes an owned `String`, so no `'static` leak is needed — while `derive` stays enabled for the built-in family trees of decision 1.*

5. **No dynamic escape hatch.** `allow_external_subcommands`, `ignore_errors`, `infer_subcommands`, `allow_missing_positional`, and trailing-argument passthrough are **not** used. A name that is not in the tree is not a command: clap emits `unrecognized subcommand` with the enabled command list and exits 2. A disabled entry is absent from the tree, so its slug behaves exactly like an unknown slug. There is no alias, abbreviation, case folding, or fuzzy match, and no path in the code may map an unknown slug onto a different command.

6. **Built-in names cannot be shadowed, by construction and by validation.** `cli::tree` never inserts a document command whose slug equals a built-in name; ADR-0019 rejects such a configuration first, so the runtime check is a defensive assertion, not the control. This keeps REQ-07.002's guarantee independent of insertion order.

7. **Help and error text are generated from the tree.**
   - `papyrus --help` lists built-ins and one line per enabled document type, derived from the registry in slug order.
   - `papyrus <slug> --help` lists the baseline verbs, `status` where the type has it, and any declared extension for a built-in type.
   - Unknown-slug, missing-verb, and malformed-option errors keep clap's plain-text stderr output and exit code 2 (REQ-01.004), unchanged from today.
   - Application errors (`AppError`) keep the existing mapping (1/3/4) and the existing `--json` error object.
   - `--json` from a document command behaves as for any other command because the flag is a global argument of the same tree.

8. **`--version` and `--help` do not require a configuration file.** If no configuration is found, `papyrus --version` prints the version and exits 0; `papyrus --help` lists the built-in commands plus the default document-type set, annotated that no configuration was found, and exits 0. Rationale: these are global options, not command executions, and REQ-01.003's fail-closed rule governs commands. Without this exception a missing or broken configuration would leave the user unable to see which commands exist. Command execution remains fail-closed.

9. **Startup work is bounded and observable.** The tree is built once per process in `O(built-ins + enabled types)`; no completion scripts, no per-request rebuilding, and no caching across processes. Building the tree is not allowed to perform I/O beyond the already-loaded configuration.

   *Revision (2026-11-20, amended by ADR-037): this decision bounds the CLI command tree, which is still built once per process with no I/O beyond the already-loaded configuration; it no longer describes the browser's serving metadata. The browser keeps one configuration, registry, and route snapshot and rebuilds it only when an input fingerprint changes, so a request that changes nothing performs no rebuild. No caching crosses process boundaries (ADR-037).*

10. **Removals are part of this decision.** The static document-type variants of `Commands`, their `clap::Subcommand` enums, and the `nfreq` command name are deleted, not retained as hidden aliases (REQ-07.002 and REQ-07.003 both forbid the alias). `req` and `nfr` keep working only because they are default `document_types` entries.

### Amendment (2026-11-20) — descriptor model cross-check

- **Decision 4 — verbs are derived from the model.** `cli::document_command` keeps one builder and one `DocumentInvocation`, but the verb family comes from `DocumentModel`, not from a descriptor declaration: the baseline verbs always, `status` for `SingleMarkdown` and `GroupedMarkdown`, `create-group` and `--group` only for `GroupedMarkdown`, and `supersede` only for `SingleHtmlMockup` (ADR-036 decision 8). The clause "plus any verb the descriptor declares as an extension" is superseded; verbs, options, and help are never per-type declarations.
- **Decision 4 — engine-generated title arguments.** The title arguments are engine-generated and uniform for every document type and model: the positional `TITLE` on `create` and `create-group`, and `--title` on `create`, `create-group`, and `edit` (ADR-036 decision 24). No type supplies its own title option, there is no `--term`, and the HTML mockup metadata uses the same title contract. Field options are generated from the field sequence, in sequence order, and offered only on the verbs listed by the field's `verbs` (ADR-036 decisions 15, 19, and 20).
- **Decision 7 — frozen about lines and type order.** The about line `Manage <slug> documents.`, the per-type lines of `papyrus --help`, and the verb and option groups of `papyrus <slug> --help` are frozen: regenerating the tree must reproduce today's about lines and the same ordering of enabled types (the seed `order` values reproduce the current registry order). Per-option help prose and option order are generated from the seed field data (`help`, `label`, and sequence position), so a seed change legitimately changes that prose and order; such a regeneration is not a help regression. Error text and exit codes are unchanged.
- **Not changed:** decisions 1-3, 5, 6, and 8-10, the builder/derive split, the two-phase startup, `BUILTIN_COMMANDS`, and this record's accepted status.

### Amendment (2026-11-22) — command surface rulings

- **Decision 7 — verb about lines are not frozen per type.** The about line of each verb is engine-generated generic text derived from the document model and the field sequence; per-type verb about text is not preserved (ADR-042 ruling 2, ADR-036 decision 8). TASK-260 attributes every resulting help diff. The frozen surface is narrowed to the root about line, `Manage <slug> documents.`, each enabled type's help line, the type order, and the verb listing of `papyrus <slug> --help`; per-verb about text is generated prose like per-option prose.
- **Decision 7 — type order is effective configuration map order.** Enabled document types are listed in effective `document_types` map order (slug order), not seed `order`; the current order is the frozen order and seed `order` is presentation-only (ADR-042 ruling 4). The 2026-11-20 amendment's statement that "the seed `order` values reproduce the current registry order" is corrected.
- **Not changed:** decisions 1-6 and 8-10, the 2026-11-20 amendment's model-derived verbs and title arguments, and this record's accepted status.
## Consequences

**Positive:**

- REQ-07.003 becomes structural: adding an entry adds a command, a help line, and `--help` output with no code change.
- Parsing, usage text, help, and validation all read the same sources — the derive declarations for the built-ins, `BUILTIN_COMMANDS` for reserved names, and the registry for the document surface — so the reserved-name list cannot drift from the real command set.
- The circular dependency between global options and the command tree is resolved once, in one documented place, instead of being worked around per command.
- Deleting eleven static document-type variants (the ten per-type modules plus the `nfreq` alias) removes a large block of near-duplicate code that REQ-07.003 makes obsolete anyway; the built-in family trees of decision 1 are derive-built and stay unchanged.
- A `status` verb that exists only for front-matter-backed types makes the descriptor's status model (ADR-0017) visible in help without a second command builder.

**Negative:**

- Every invocation now loads and validates the configuration before argument parsing completes. A usage error in a project with an invalid configuration reports the configuration error (exit 4) rather than the usage error (exit 2).
- `papyrus --help` in a project directory with an invalid configuration fails closed (exit 4); the built-ins-only rendering applies only when no configuration file exists.
- Document help text is no longer a compile-time constant, so documentation tests and man-page generation for the document surface cannot simply embed it.
- On the document surface, the derive macros' compile-time argument checking is replaced by builder construction: a wrong argument name is a runtime help-text defect, not a compile error. The built-in family trees keep derive's compile-time checking. Characterization tests are the mitigation.

**Risks and mitigations:**

- *Pre-scan diverges from clap*: the scan recognizes only the four documented forms and stops at `--`; a matrix test compares phase-1 resolution with the final parsed global values for `--dir`, `--dir=`, `--config`, `--config=`, env-only, and mixed forms.
- *Behaviour change for existing scripts*: `papyrus nfreq …` now exits 2. This is required by REQ-07.002 and must appear in the release upgrade notes.
- *Argument-name drift in command construction*: one table-driven test per built-in asserts that every documented option parses, one run of each document verb covers the builder-built document surface, and one test asserts the full help output per command.
- *Config-before-parse hides usage errors*: documented in `papyrus(1)`; `papyrus --help` remains available when no configuration file exists.

*Revision (2026-11-15, owner decision): the consequence bullets were aligned with the delivered split — the removed static surface is eleven document-type variants (ten per-type modules plus the `nfreq` alias), the lost compile-time argument checking and the non-constant help text apply to the document surface only, the built-in family trees keep derive, and help for the built-ins is generated from their derive declarations rather than from `BUILTIN_COMMANDS`.*

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Builder tree built at startup from the registry (this ADR) | Selected | Only route that satisfies help listing, per-slug `--help`, exit 2, and zero per-type code, without a new dependency. |
| Keep derive and add `allow_external_subcommands` | Rejected | No per-subcommand help, no listing in `--help`, option parsing becomes a second hand-written parser, and unknown-slug errors lose clap's exit-2 path. |
| Pre-parse `argv[1]` by hand and forward the remainder to clap | Rejected | Duplicates option parsing and error formatting, and every new verb needs a second parser. |
| Keep static commands and generate a wrapper per document type | Rejected | Requires code generation and a build step for a set that is meant to change at runtime; violates "no per-type code". |
| Dispatch by shell wrapper scripts installed per type | Rejected | Packaging would need a script per type; ADR-0002/ADR-0003 ship one binary. |
| Resolve the configuration inside each command handler instead of before parsing | Rejected | Then a document command cannot exist in the tree, so help and `--help` cannot be correct. |

## Open Questions

- Should `papyrus --help` with an **invalid** configuration also fall back to the built-ins-only rendering (decision 8 covers only "no configuration file")? Current decision: fail closed with exit 4, so the user is told what to fix.
- Should `papyrus <slug> <id>` be accepted as a shorthand for `view <id>` (REQ-07.003 leaves this open)? Current decision: no shorthand.
- Should a top-level `papyrus help <slug>` be required in addition to `--help`? Current decision: only the clap-provided `help` subcommand and the `--help` flag.

## Downstream constraints

1. `src/lib.rs` must contain no `#[derive(Subcommand)]` command enum after this ADR; command-tree construction and parse dispatch live in `src/cli/`.
2. `src/cli/builtin.rs` and the five built-in family modules (`src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`, `src/import.rs`) may declare `clap` derive commands and are the **only** such declarations. No module may build or inspect a `clap::Command` for a document type outside `src/cli/`, and `document::run` receives a `DocumentInvocation`, never an `ArgMatches`.

   *Revision (2026-11-15, owner decision): this constraint previously read "no module except `src/cli/` may construct or inspect a `clap::Command` or `ArgMatches`", which the delivered derive/builder split of decision 1 contradicts, because the built-in family trees are declared outside `src/cli/`. The permission is now enumerated — the named modules may declare derive commands, document-type command construction stays inside `src/cli/`, and the document handler boundary is the `DocumentInvocation` value. Constraint 1 is read the same way: the `src/lib.rs` prohibition stands, while `src/cli/` owns command-tree construction and parse dispatch.*
3. Document dispatch enters only through `document::run`; no `match slug` on string literals is permitted outside the registry (grep check).
4. Exit codes 0/1/2/3/4 and the `--json` error object are unchanged.
5. `status <id> <value>` never archives and never moves a file; `archive` is the only verb that produces `archived`, and the command tree must not offer `status` on a type without front-matter status (UI mockups).
6. `BUILTIN_COMMANDS` is the only list of built-in command names; ADR-0019 consumes it for reserved-name validation, and ADR-0021 is reserved to consume it once a man-page option is selected (that ADR is still `Proposed` and deferred, so the man-page consumer is not delivered).

   *Revision (2026-11-15, owner decision): this constraint previously read "ADR-0019 and ADR-0021 must consume it", which implied a delivered man-page consumer. The table carries names only; ADR-0019 consumes it today and ADR-0021 remains reserved, consistent with decision 2.*

## References

- `docs/requirements/business/REQ-07.001.md` — document main status, custom statuses, and the `status` command.
- `docs/requirements/business/REQ-07.002.md` — document slugs and reserved built-in names.
- `docs/requirements/business/REQ-07.003.md` — dynamic command family, help listing, unknown/disabled slug.
- `docs/requirements/business/REQ-01.003.md` — project and configuration selection, fail-closed.
- `docs/requirements/business/REQ-01.004.md` — global options and exit codes.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single crate.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — no new dependency.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — the registry the tree is built from.
- `docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md` — the validation that runs before parsing.
- Implementation: `src/lib.rs`, `src/cli/` (new), `src/document.rs` (new), `src/registry.rs` (new), `src/main.rs`.
