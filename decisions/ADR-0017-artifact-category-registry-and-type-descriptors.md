---
id: ADR-0017
title: "Artifact Category Registry and Per-Type Behavior Descriptors"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-21T15:45:11Z'
labels: []
references: []
---
# Artifact Category Registry and Per-Type Behavior Descriptors

## Status
accepted
## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-07.001 fixes exactly three artifact categories — **backlog** (tasks, milestones, releases), **decision** (decision records), **document** (every type declared under `document_types`). Status models differ by category: task statuses are configurable through the top-level `statuses`/`default_status` (REQ-01.002); milestones are fixed `open -> closed`; releases fixed `planned -> released`; decisions use the five Nygard statuses; and every document type has a **main status** `active` or `archived` — the only statuses that affect storage location — plus optional **custom statuses** declared per type in `document_types.<slug>.statuses` with an optional `default_status`. Custom statuses are front-matter values only: no folder per status and no move on a status change. UI mockups keep `pending`/`approved`/`archived` in HTML metadata and declare no custom statuses.

REQ-07.003 fixes the document **baseline** command family to `create`, `list`, `view`, `edit`, `archive`, `delete` for every document type, and adds `status <id> <value>` for front-matter-backed types (it never archives). The baseline is a lower bound: built-in typed types add verbs and arguments — `req`/`nfr` have `create-group` and a required `--group` on `create` (REQ-05.001, REQ-05.002), and `ui-mockup` has `supersede` (REQ-05.008). REQ-05.007 gives every document type a single archive/delete policy.

Current code shape exposes four different partial models of the same idea:

- `src/record/descriptor.rs` declares fourteen `RecordDescriptor` values (tasks, milestones, releases, decisions, and ten document types). It is a **compile-time** list; `descriptor_for_doc_type(key)` returns `None` for any slug that is not one of the ten compiled keys, so a config-defined type is invisible to the record layer.
- `src/lib.rs` + one module per type (`src/req.rs`, `src/tech_spec.rs`, …) encode each type's verb set as its own `clap::Subcommand` enum and its own `match` dispatch.
- `src/browser/records.rs` declares a `Category` enum with fifteen variants and per-variant `key()`, `config_key()`, and `descriptor_name()` mappings, plus `form_for_stem` branching on `Category`.
- `src/browser/render.rs` branches on `Category` in the sidebar, page titles, list pages, detail pages, and edit forms.
- `src/config.rs` holds `DocTypeEntry { prefix, directory, enabled }` with no status model, content model, id shape, or verb set. `deploy` is no longer a default entry (REQ-07.002) and survives only as an ordinary user-added type, but the code still seeds it by default.

ADR-0013 fixes the record envelope, the single write path (`record::store`), the single mutation API (`record::patch`), and makes the descriptor the only extension point for a record type. ADR-0010 fixes configuration-driven directories and Papyrus-only identity. ADR-0012 fixes the front-matter key registry. None of them covers a type that exists only because configuration declares it.

Forces: one structural model for CLI, record layer, search, and browser (otherwise five places drift); type-specific behavior must remain possible for built-in types without special-casing outside that model; config-defined types must require no code; the single write path and front-matter contract must not be weakened; the artifact categories and fixed lifecycles of REQ-07.001 must be expressible as data, not as ad-hoc `match` arms.

## Decision
1. **One registry, built at startup, describes every artifact type.** `src/registry.rs` exposes `ArtifactRegistry` with three category collections and one descriptor type. It is assembled from two sources, in this order:
   - the static built-in descriptors (the built-in templates in `src/registry.rs` plus `record::descriptor` entries) for tasks, milestones, releases, and decisions;
   - one runtime `TypeDescriptor` per `document_types` entry, derived mechanically from `DocTypeEntry` plus the static document descriptor of the same slug when one exists.

   A slug that has no static descriptor resolves to the **generic** document descriptor (ADR-0018). The registry is the only object that maps a slug to behavior; no module may hold a second list of type names.

2. **`TypeDescriptor` carries behavior, not presentation-only data.** At minimum: `slug` (the command name), `category` (`Backlog | Decision | Document`), `label` (curated for built-ins, derived for config-defined types), `prefix`, `directory` resolution (configuration key plus relative value), `enabled`, `status_model`, `verbs`, `content_model`, `frontmatter_keys`, `sections`, `id_shape`, and `archive_policy`.

   | Field | Values | Source of truth |
   |---|---|---|
   | `category` | `Backlog`, `Decision`, `Document` | REQ-07.001 |
   | `status_model` | `Fixed(&[State])`, `TaskConfigurable { statuses, default }`, or `DocumentMain { main: [active, archived], custom: &[State], default: State }` | REQ-07.001, REQ-01.002 |
   | `verbs` | every descriptor in every category: task `create/list/view/edit/archive/restore/delete`; milestone `create/list/view/edit/close/delete`; release `create/list/view/edit/close`; decision `create/list/view/edit`; document `create/list/view/edit/archive/delete`, plus `status` where the type stores a front-matter status, plus declared extensions | REQ-02.001–004, REQ-03.001–002, REQ-04.001, REQ-07.003, REQ-05.001/002/008 |
   | `content_model` | `Generic`, `Typed(&'static TypedSchema)`, `HtmlMockup` | REQ-07.004, REQ-05.008 |
   | `frontmatter_keys` | general keys plus per-type keys (`status` included only for a type that stores it) | REQ-05.010, ADR-0012 |
   | `id_shape` | `Plain`, `Dotted`, `Grouped` | REQ-01.005, REQ-05.001/002 |
   | `archive_policy` | `ArchivedFolder { state_field }` or `None` | REQ-05.007, REQ-04.002 |

   *Revision (2026-11-15, owner decision): the `verbs` row previously described the field as the document baseline plus `status` plus extensions, which read as a document-only field. `verbs` exists on every descriptor in every category, and the delivered sets are the ones listed above (backlog and decision sets from REQ-02.001–004, REQ-03.001–002, and REQ-04.001). The same correction applies to decision 1's assembly sentence, which no longer refers to `cli::BUILTIN_COMMANDS` handlers because that table carries names only (ADR-0016 decision 2).*

   *Revision (2026-11-15, owner decision): the milestone entry in the `verbs` row previously read `create/list/view/edit/archive/delete`. The milestone lifecycle is fixed to `open -> closed` (REQ-03.001, REQ-07.001) and is delivered by `papyrus milestone close <id>`, which rewrites `status` and `updated_date` in place through the single write path; milestone archive and milestone restore do not exist. No other category's verb set in that row changes, and ADR-0022 decision 5 and ADR-0023 decision 2 record the same contract.*

   *Amendment (2026-11-21): the milestone clause of the 2026-11-15 revision is superseded by ADR-032 decisions 2, 4 and 5 — the milestone lifecycle is `open -> closed -> archived`, a milestone has an archive location (the backlog archive milestones folder, also the legacy read/import location) and `papyrus milestone archive <id>` is the only path to `archived`; the milestone entry in the `verbs` row is superseded to `create/list/view/edit/close/archive/delete` and no milestone restore exists, and every other statement in this decision stands.*

   Only tasks are `TaskConfigurable`. Milestones, releases, and decisions are `Fixed`. A document descriptor is `DocumentMain`: `active` and `archived` are always present and cannot be removed, renamed, or replaced; every other declared value is a custom status; `custom` is empty when the entry declares no `statuses`; `default` is the entry's `default_status` or `active`. The registry never stores a second copy of the status list — it reads it from the validated configuration (ADR-0019). Configuration cannot change a descriptor's category, fixed lifecycle, verbs, content model, or id shape (REQ-07.001).

3. **The baseline verbs are a lower bound; `verbs` is a field of every descriptor, and document extensions are declared on a built-in descriptor.** `verbs` is **not** a document-only field: it is present on every descriptor in every category (decision 2), so a task, milestone, release, or decision descriptor states its own verb set. Within the `Document` category the baseline family `create`, `list`, `view`, `edit`, `archive`, `delete` remains a **lower bound** present on every descriptor; `status` is present only when the descriptor's status model stores a front-matter `status` (so every front-matter-backed document type, but not UI mockups). A built-in document descriptor may add a verb (`create-group`, `supersede`) and may add a required argument (`--group`), with its handler named by the descriptor. A config-defined descriptor has exactly the baseline plus `status`, and no extension point: REQ-07.004 forbids per-type schema and REQ-07.002 forbids per-type code. This is the owner's decision of 2026-11-13 (D1) on REQ-07.003's "same command family" wording: the shared family is the baseline, and the `req`/`nfr`/`ui-mockup` groups are unchanged.

   *Revision (2026-11-15, owner decision): this decision previously presented the baseline verbs without stating that `verbs` is carried by every descriptor in every category; the field's scope and the conditional `status` rule are now explicit and the D1 rationale sentence is unchanged.*

4. **One dispatch point per category, driven by the descriptor.**
   - CLI: `cli::tree` inserts **document** verbs from the descriptor (ADR-0016), and the document handler receives the descriptor, never a slug literal. Backlog and decision commands are not tree-built: they are `clap` **derive** commands declared in their family modules (`src/task.rs`, `src/milestone.rs`, `src/release.rs`, `src/decision.rs`), and only their names come from `cli::BUILTIN_COMMANDS` — `cli::tree` grants them no descriptor-driven verb set.
   - Record layer: `record::descriptor` keeps the static definitions and gains `Registry::descriptor(slug)`; `descriptor_for_doc_type` is replaced by a registry lookup so config-defined types are first-class readers/writers.
   - Search and browser: both iterate `registry.enabled(category)` instead of a static list of keys.

   *Revision (2026-11-15, owner decision): this decision previously said that `cli::tree` inserts "the descriptor's verbs", which is true only for the `Document` category. `src/cli/tree.rs` skips every non-document descriptor, so backlog and decision verbs come from their `clap` derive enums and `BUILTIN_COMMANDS` supplies only their names; the document surface is the only descriptor-driven one.*

5. **The registry does not write, parse, or format records.** It supplies metadata only. Every read and write still goes through `record::document`, `record::patch`, `record::identity`, and `record::store` (ADR-0013 decisions 1–4, ADR-0012 decision 6). Archive, delete, and supersede are expressed as `PatchOp` sequences plus `record::store` moves (ADR-0022). A descriptor may not contain a `render_*`, `parse_*`, or `fs::write` path.

6. **Adding a type is data.** A new `document_types` entry yields `category = Document`, `status_model = DocumentMain { main: [active, archived], custom: <declared statuses or empty>, default: <default_status or active> }`, `verbs = baseline + status`, `content_model = Generic`, `frontmatter_keys = general + status`, `id_shape = Plain`, `archive_policy = ArchivedFolder`, with `prefix` and `directory` from the entry. No match arm, no new module, and no new template is added for it.

7. **Built-in types keep their typed behavior inside the registry, not beside it.** `req`/`nfr` (grouped ids, `create-group`, `--group`), `ui-spec` (`--view`), `rule` (`--rationale`, `--applicability`, requirement references), `audit` (`--type`), `tech-spec`/`use-case`/`test-case`/`test-report` (title/description/applicability), and `ui-mockup` (HTML metadata, `supersede`, no `status` verb and no custom statuses) are `Typed` descriptors. Their sections, keys, and required fields move from the per-type modules into their descriptor; the per-type modules shrink to validation and patch-building or disappear.

8. **The registry is the only place that knows the category and status model of a type.** `ArtifactCategory` replaces `browser::records::Category` and its `key()/config_key()/descriptor_name()` mappings. Presentation strings for built-ins may stay in `browser::render`, but they are looked up by slug from a static table, not by a per-category match. The browser's per-type status filter and status display read `status_model` from the same place (ADR-0020).

9. **Construction is fallible and total.** `ArtifactRegistry::build(&Config)` returns an error if any invariant fails (missing prefix, unknown category, duplicate slug, disabled type with a command, verb/extension mismatch, a status model whose custom list is absent for a type that has one, a status verb on a type without front-matter status), and on success contains exactly the configured type set. Validation (ADR-0019) runs first; this constructor is the second gate and is also called in tests without a filesystem.

10. **No new dependency, no new process model.** The registry is plain data built in-process; it introduces no index file, no background scan, and no serialization format.

   *Revision (2026-11-20, amended by ADR-037): the no-cache clause of this decision is superseded; the rest remains in force. The registry is still plain data built in-process, with no new dependency, no new process model, no index file, no background scan, and no serialization format. ADR-037 adds one in-server serving snapshot that holds the validated configuration, the registry, and the route table, is invalidated by input-file modification time, and is swapped atomically; that snapshot is the only cache this decision now permits.*

### Amendment (2026-11-21) — milestone archive

- **Downstream constraint 3.** The clauses assigning milestones `ArchivePolicy::None` and naming `<backlog_directory>/archive/milestones/` a read/import location only, including the 2026-11-15 revision, are superseded by ADR-032 decisions 2, 4 and 5; the milestone lifecycle is `open -> closed -> archived`, a milestone has an archive location (the backlog archive milestones folder, also the legacy read/import location) and `papyrus milestone archive <id>` is the only path to `archived`. The rest of this clause stands.
## Consequences

**Positive:**

- One model replaces four partial ones (`RecordDescriptor` list, per-type CLI enums, `browser::Category`, hard-coded `search::DOC_KEYS`), so a new type cannot appear in the CLI and be missing from the browser or search.
- The status-model split of REQ-07.001 becomes testable data: one test asserts that only tasks use the top-level configurable statuses, that milestone/release/decision states are fixed, and that a document descriptor always includes `active` and `archived` while its custom statuses come from configuration.
- Per-type behavior for built-ins stays available without weakening the generic path; a custom document status needs no code, only a validated list entry.
- The browser and search refactors become mechanical iterations over the registry rather than hand-written per-type additions.

**Negative:**

- A larger upfront refactor than adding a `Generic` branch to the existing `Category` enum: fourteen modules, five browser files, and search are touched.
- Descriptors gain fields that duplicate requirement knowledge (status models, verb sets). Drift is possible between a descriptor and its REQ; tests must assert the requirement-derived table.
- Runtime descriptors mean fewer compile-time guarantees; a malformed descriptor is caught at startup (exit 4) instead of at build time. A document descriptor whose custom status list is taken from configuration makes the registry depend on the validated configuration, so the registry cannot be built before ADR-0019 validation.
- Static `&'static str` descriptors must become a mix of static and owned values, which changes function signatures across the record layer. Custom status lists are owned data, which is the main source of that change.

**Risks and mitigations:**

- *Regression while collapsing per-type modules*: migrate one type at a time behind characterization tests for output text, JSON, and files on disk; keep the typed descriptor path for built-ins.
- *Type-specific validation lost in the generic path*: `TypedSchema` keeps required fields and field validators; a test per built-in asserts its create/edit rejections.
- *Two registries during transition*: `record::descriptor` and `registry` must not both answer "which types exist"; a single accessor is the rule, and a grep check forbids `match` on slug literals outside `registry.rs`.
- *Descriptor/requirement drift*: the status-model and verb tables are asserted against the requirement-derived expectations in one test fixture.
- *Status list read twice*: the registry must expose the validated list by reference, not re-parse the configuration, or a `status` command could accept a value the validator rejects.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| One runtime registry composed from static built-ins plus config entries (this ADR) | Selected | Single model for CLI, record layer, search, and browser; keeps typed behavior for built-ins and zero code for config types. |
| Extend `RecordDescriptor` with an enum of "generic document" and keep everything else static | Rejected | Leaves the static list authoritative, so config-defined types remain outside the record layer and every surface needs its own exception. |
| Keep `browser::Category` and add a `Generic(String)` variant | Rejected | Pushes slug strings through render code and keeps per-type match arms; the browser and CLI would still disagree about type sets. |
| Per-type code generation at build time from a fixed schema list | Rejected | Contradicts REQ-07.002 ("no code change"), needs a build step, and cannot see a project's configuration at build time. |
| Descriptor per type with a trait object implementing create/edit/archive | Rejected | Pulls record formatting and writing back into the type layer, breaking ADR-0013 decisions 4 and 7. |
| Model artifacts as a plugin registry loaded at runtime | Rejected | New dynamic-loading surface, packaging and security cost, no requirement. |

## Open Questions

- Should extensions (`create-group`, `--group`, `supersede`) be expressible for a **config-defined** type later? Current decision: no.
- Should the registry expose status states to the browser lifecycle page (REQ-06.003) directly, or through a presentation table? Current decision: status model from the registry, curated description text from a static built-in table.

## Downstream constraints

1. No module other than `src/registry.rs` may enumerate document-type keys or map a slug to behavior.
2. `record::store` remains the only writer and `record::patch` the only mutation API; descriptors carry no I/O.
3. Every descriptor's `archive_policy` follows REQ-05.007 and ADR-0022: tasks use `ArchivedFolder`; milestones, decisions, and releases use `ArchivePolicy::None`; document descriptors use `ArchivedFolder` (ADR-0023 decision 2).

   *Revision (2026-11-15, owner decision): this constraint previously named only decisions as having no archive policy. Releases are the second family that never moves: their descriptor carries `ArchivePolicy::None` and `release close` rewrites `status` in place (ADR-0023 decision 2).*

   *Revision (2026-11-15, owner decision): the milestone descriptor also carries `ArchivePolicy::None`; this constraint previously assigned `ArchivedFolder` to it. The milestone lifecycle is `open -> closed` by `papyrus milestone close <id>`, which writes `status` and `updated_date` in place, and no milestone command archives or restores a milestone. The legacy `<backlog_directory>/archive/milestones/` folder is a read/import location only, not a lifecycle archive destination (ADR-0023 decisions 2 and 4).*
4. Descriptor fields must be derivable from REQ-07.001/002/003/004 and REQ-05.010; a field that cannot be traced to a requirement is not added.
5. The per-type modules may keep validation and patch construction but must lose their `clap` enums, their dispatch `match`, and their directory/ID logic.
6. A descriptor's custom status list is the validated configuration's list, read-only; no command may add, remove, or rename a status at runtime, and `archived` is never accepted as a custom value.

## References

- `docs/requirements/business/REQ-07.md`, `REQ-07.001.md`, `REQ-07.002.md`, `REQ-07.003.md`, `REQ-07.004.md`.
- `docs/requirements/business/REQ-02.001.md`–`REQ-02.004.md`, `REQ-03.001.md`–`REQ-03.002.md`, `REQ-04.001.md` — task, milestone, release, and decision verb sets carried by `verbs`.
- `docs/requirements/business/REQ-05.001.md`, `REQ-05.002.md`, `REQ-05.007.md`, `REQ-05.008.md`, `REQ-05.010.md`.
- `docs/requirements/business/REQ-01.002.md`, `REQ-01.005.md`.
- `docs/architecture/adr/ADR-0010-papyrus-native-on-disk-record-layout.md` — configured directories and identity.
- `docs/architecture/adr/ADR-0012-papyrus-owned-front-matter-metadata.md` — front-matter key registry.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — envelope, single writer, descriptor as extension point.
- `docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md` — how the registry becomes a command tree.
- `docs/architecture/adr/ADR-0018-generic-document-content-model.md` — the generic content model for config-defined types.
- Implementation: `src/registry.rs` (new), `src/record/descriptor.rs`, `src/lib.rs`, `src/search.rs`, `src/browser/records.rs`, `src/browser/render.rs`.
