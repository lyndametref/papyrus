---
id: TASK-100
title: Introduce the artifact registry and type descriptors
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:24:00Z'
updated_date: '2026-11-14T07:43:00Z'
labels:
  - rust
  - architecture
milestone: MS-018
dependencies: []
documentation:
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - docs/requirements/business/REQ-007.001.md
  - docs/architecture/adr/ADR-0016-runtime-config-derived-command-dispatch.md
  - docs/architecture/adr/ADR-0018-generic-document-content-model.md
  - >-
    docs/architecture/adr/ADR-0019-fail-closed-document-type-configuration-validation.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/architecture/adr/ADR-0023-consolidated-record-layout-and-field-map.md
  - docs/requirements/business/REQ-007.002.md
  - docs/requirements/business/REQ-007.003.md
  - docs/requirements/business/REQ-007.004.md
  - docs/requirements/business/REQ-006.014.md
priority: high
ordinal: 41000
---


## Description
Papyrus models the same artifact set in four partial places: a compile-time record descriptor list, one clap subcommand enum per document type, `browser::records::Category`, and hard-coded search keys. A document type declared only in configuration is therefore invisible outside `Config`, and every new surface needs its own list. This task creates the single runtime model that the CLI, record layer, browser, and search will consume, so the rest of the feature has one source of type behavior.

Scope of this task: add `src/registry.rs` with `ArtifactRegistry`, `TypeDescriptor`, and the supporting enums; declare the built-in descriptors (task, milestone, release, one per configured decision type, and the eleven default document types); derive a generic descriptor for any other configured slug; and extend `DocTypeEntry` with `statuses` and `default_status` without changing the serialization of existing files. The registry is built from `Config` alone, has no filesystem access, and rejects its own invariants.

Out of scope: no module consumes the registry yet (`src/lib.rs`, `record::descriptor`, `search`, and `browser` stay unchanged); configuration validation of document types stays with TASK-109; the default `document_types` map stays as it is, so removing `deploy` and seeding statuses remains TASK-112; and no command name, output, exit code, or file layout changes in this task.

The implementation plan carries the exact type definitions, the per-type descriptor table, the invariant-to-error table, and the test plan. Design authority is ADR-0017 with ADR-0016, ADR-0018, ADR-0019, ADR-0022, and ADR-0023.

## Acceptance Criteria
- [x] src/registry.rs exposes ArtifactRegistry::build(&Config) -> Result<ArtifactRegistry, RegistryError> with the Backlog, Decision, and Document category collections (in_category and enabled) and an exact, case-sensitive descriptor(slug) lookup; enabled(category) yields only enabled descriptors and in_category(category) yields every descriptor of the category.
- [x] TypeDescriptor carries slug, category, label, prefix, directory (a DirectorySpec naming the configuration key plus the relative value), enabled, status_model, verbs, create_options (the required --group declaration), content_model (Generic, Typed(schema), or HtmlMockup(schema), the schema naming the typed handler and the body sections), frontmatter_keys, id_shape, and archive_policy as specified by ADR-0017; no field is added without a requirement or ADR source.
- [x] Built-in typed behavior is descriptor data: task, milestone, release, and one descriptor per configured decision type keep their current verb sets; the eleven default document types (req, nfr, glossary, rule, ui-spec, ui-mockup, audit, tech-spec, use-case, test-case, test-report) carry their current labels, sections, front-matter keys, and archive policies; req and nfr declare the Grouped id shape, create-group, and a required --group on create; ui-mockup declares HtmlMockup with the pending/approved/archived HTML states, supersede, and no status verb and no custom statuses; deploy is not a built-in descriptor and nfreq is neither a slug nor an alias; the Plain, Dotted, and Grouped id shapes are all defined and asserted.
- [x] A configured document slug with no static template resolves to the generic descriptor: Document category, baseline verbs plus status, Generic content model, Plain id shape, ArchivedFolder archive policy with the front-matter status field, general front-matter keys plus status, the slug as label, and prefix, directory, and enabled from the entry.
- [x] DocTypeEntry gains statuses: Option<Vec<String>> and default_status: Option<String>, both with serde defaults and omitted from serialization when absent, so existing configuration files load unchanged and re-serialize without new keys; declaration order is preserved, duplicates are not sorted away, and an explicit empty list stays distinguishable from an absent list; no document-type validation is added in this task.
- [x] ArtifactRegistry::build is fallible and filesystem-free (the only input is &Config; no std::fs, std::path, or environment access) and returns a dedicated RegistryError for each of: duplicate slug; collision with a built-in registry slug, compared case-insensitively; empty prefix; duplicate prefix, compared case-insensitively and including disabled entries; empty directory; absolute or traversing directory; empty status list; duplicate status; active or archived missing or repeated in the status list; statuses declared on a type without front-matter status; an undeclared default_status; archived as default_status; a status-verb/status-key mismatch; an extension-verb/content-model mismatch; and restore on a document. Every RegistryError converts to AppError::Validation with a message naming document_types.<slug>.<field> (exit code 4).
- [x] The registry reproduces the current built-ins instead of replacing them: src/lib.rs, record::descriptor, search, and browser stay unchanged and keep compiling, and a characterization test asserts that the registry agrees with record::descriptor for every default document slug (prefix, directory value, sections, dotted-ness) so the two lists cannot drift before TASK-107 deletes the static one; no other module gains a new slug-to-behavior mapping in this task.
- [x] Unit tests cover built-in, typed, and generic descriptors and every registry invariant failure listed in the implementation plan, plus backward-compatible DocTypeEntry deserialization and serialization; the whole test suite, including tests/no_drift.rs, passes.

## Definition of Done
- [ ] #1 src/registry.rs compiles and cargo build succeeds with no other module behavior changed.
- [ ] #2 ArtifactRegistry::build(&Config) produces the documented descriptor set, and each of the sixteen invariant checks has its own RegistryError variant and a message in document_types.<slug>.<field> form.
- [ ] #3 All unit tests named in the implementation plan exist and pass; cargo test and cargo clippy are clean, including the unchanged tests/no_drift.rs.
- [ ] #4 DocTypeEntry loads a configuration that declares only prefix, directory, and enabled, and re-serializes it without statuses or default_status.
- [ ] #5 src/registry.rs contains no filesystem or environment access and no clap, record-writing, or formatting code; build takes only &Config.
- [ ] #6 The diff is limited to src/registry.rs (new), src/lib.rs (module declaration), src/config.rs (two fields plus tests), and task files; papyrus --help, every command output, and the on-disk layout are identical to the previous revision.
- [ ] #7 The two open questions in the plan are answered in a task comment, or explicitly deferred with a named owner, before implementation starts.


## Implementation Notes
# Implementation Plan

## 0. Scope guardrails

- Internal refactor. No command name, output, exit code, or on-disk layout changes.
- New file `src/registry.rs`, plus `pub mod registry;` in `src/lib.rs` and two optional fields on `DocTypeEntry`.
- Nothing consumes the registry in this task. Wiring is TASK-107 (record layer), TASK-108 (command tree), TASK-110 (generic handler), TASK-113 (search), TASK-114 (browser).
- `Config::validate` is untouched. The default `document_types` map is untouched: `deploy` is still seeded and no `statuses` are seeded (TASK-112 owns both). A default build therefore yields twelve document descriptors: the eleven typed defaults plus `deploy` as a generic descriptor.
- `ArtifactRegistry::build` is filesystem-free: no `std::fs`, no `std::path::Path`, no environment read, no `Context`. Its only input is `&Config`.
- No new dependency (ADR-0017 decision 10).

## 1. Module layout (`src/registry.rs`, single file)

Order of items:

1. `ArtifactCategory`
2. `IdShape`
3. `ArchivePolicy`
4. `MAIN_DOCUMENT_STATUSES`, `StatusModel`
5. `Verb`
6. `CreateOptions`, `GroupRequirement`
7. `RequirementKind`, `TypedHandler`, `TypedSchema`, `ContentModel`
8. `DirectorySpec`, `TypeDescriptor` and its accessors
9. `RegistryError` (`Display`, `std::error::Error`, `From<RegistryError> for AppError`)
10. Static built-in tables: `BACKLOG_TEMPLATES`, `DOCUMENT_TEMPLATES`, `DECISION_TEMPLATE`, `DOCUMENT_BASELINE_VERBS`, `GENERAL_FRONTMATTER_KEYS`
11. `ArtifactRegistry`: `build`, `assemble`, `check_invariants`, `descriptor`, `in_category`, `enabled`, `descriptors`, `len`
12. `#[cfg(test)] mod tests`

`assemble` and `check_invariants` are private but unit-testable in the same file: `build` is exactly `assemble(cfg)` -> `check_invariants(&descriptors)` -> index. Tests exercise invariants by mutating an assembled `Vec<TypeDescriptor>`, so no invariant check needs a second code path.

## 2. Exact types

```rust
pub enum ArtifactCategory { Backlog, Decision, Document }

pub enum IdShape {
    /// `<PREFIX>-<sequence>`: the only shape the family claims.
    Plain,
    /// `<PREFIX>-<group>.<sequence>` without group records.
    Dotted,
    /// A grouped family: group records `<PREFIX>-<sequence>` and member
    /// records `<PREFIX>-<group>.<sequence>`. Only `req`/`nfr` use it.
    Grouped,
}

pub enum ArchivePolicy {
    /// The record type is never archived (decisions, releases).
    None,
    /// Archive relocates the file and may carry a front-matter state.
    ArchivedFolder { state_field: Option<&'static str> },
}

pub const MAIN_DOCUMENT_STATUSES: [&'static str; 2] = ["active", "archived"];

pub enum StatusModel {
    /// Tasks only; mirrors top-level `statuses`/`default_status` (REQ-001.002).
    TaskConfigurable { statuses: Vec<String>, default: String },
    /// Milestones, releases, decisions (REQ-007.001, REQ-004.001).
    Fixed { states: &'static [&'static str] },
    /// Every front-matter-backed document type (REQ-007.001, REQ-007.002).
    /// `main` is always `MAIN_DOCUMENT_STATUSES`; `custom` is owned, ordered
    /// data taken from the validated configuration; `default` is
    /// `default_status` or `active`.
    DocumentMain { custom: Vec<String>, default: String },
    /// UI mockups only: state lives in HTML metadata (REQ-005.008).
    HtmlMetadata { states: &'static [&'static str] },
}

pub enum Verb {
    Create, List, View, Edit, Archive, Restore, Delete, Status, CreateGroup, Supersede,
}

impl Verb { pub fn as_str(self) -> &'static str; }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CreateOptions { pub group: GroupRequirement }

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GroupRequirement { #[default] Absent, Required }

pub enum RequirementKind { Business, NonFunctional }

/// Names the module family that owns validation and patch building for a
/// typed descriptor. Dispatch matches on this value, never on a slug string.
pub enum TypedHandler {
    Task, Milestone, Release, Decision,
    Requirement(RequirementKind),
    Glossary, UiSpec, Rule, UiMockup, Audit,
    TechSpec, UseCase, TestCase, TestReport,
}

pub struct TypedSchema {
    pub handler: TypedHandler,
    /// Body sections in written order; empty for HTML mockups.
    pub sections: &'static [&'static str],
    /// Extension verbs this family owns; must equal the descriptor's extensions.
    pub extensions: &'static [Verb],
}

pub enum ContentModel {
    /// Config-defined document type (REQ-007.004, ADR-0018).
    Generic,
    Typed(&'static TypedSchema),
    HtmlMockup(&'static TypedSchema),
}

pub struct DirectorySpec {
    /// `"backlog_directory" | "decision_directory" | "doc_directory"`.
    pub config_key: &'static str,
    /// `"tasks" | "milestones" | "releases" | "" | <entry.directory>`.
    pub relative: String,
}

pub struct TypeDescriptor {
    pub slug: String,
    pub category: ArtifactCategory,
    pub label: String,
    pub prefix: String,
    pub directory: DirectorySpec,
    pub enabled: bool,
    pub status_model: StatusModel,
    pub verbs: Vec<Verb>,
    pub create_options: CreateOptions,
    pub content_model: ContentModel,
    pub frontmatter_keys: Vec<&'static str>,
    pub id_shape: IdShape,
    pub archive_policy: ArchivePolicy,
}

impl TypeDescriptor {
    pub fn has_verb(&self, verb: Verb) -> bool;
    /// `Some("status")` when the type stores a front-matter status.
    pub fn frontmatter_status_key(&self) -> Option<&'static str>;
    pub fn is_enabled(&self) -> bool;
}
```

Derives: `Clone, Copy, Debug, PartialEq, Eq` for the enums and `TypedSchema`; `Clone, Debug, PartialEq, Eq` for `StatusModel`, `DirectorySpec`, and `TypeDescriptor`.

Registry handle:

```rust
pub struct ArtifactRegistry {
    /// Canonical order: task, milestone, release, decision types in key order,
    /// document types in slug order.
    descriptors: Vec<TypeDescriptor>,
    by_slug: BTreeMap<String, usize>,
}

impl ArtifactRegistry {
    pub fn build(cfg: &Config) -> Result<ArtifactRegistry, RegistryError>;
    pub fn descriptor(&self, slug: &str) -> Option<&TypeDescriptor>;   // exact, case-sensitive
    pub fn descriptors(&self) -> impl Iterator<Item = &TypeDescriptor> + '_;
    pub fn in_category(&self, category: ArtifactCategory) -> impl Iterator<Item = &TypeDescriptor> + '_;
    pub fn enabled(&self, category: ArtifactCategory) -> impl Iterator<Item = &TypeDescriptor> + '_;
    pub fn len(&self) -> usize;
}
```

Lookup is exact and case-sensitive (clap matching is case-sensitive; ADR-0016 decision 5 forbids folding). No alias, no prefix, no label lookup: `nfreq`, `non-functional requirement`, `Req`, and `REQ` all return `None`.

## 3. Construction

`assemble(cfg)` produces descriptors in a fixed order, with no checks:

1. `BACKLOG_TEMPLATES` in order task, milestone, release: prefix from `cfg.task_prefix`, `cfg.milestone_prefix`, `cfg.release_prefix`; `enabled = true`.
2. `DECISION_TEMPLATE` once per `cfg.decision_types` entry (BTreeMap order, i.e. `adr`, `bdr`): slug and label are the entry key, prefix and `enabled` from the entry.
3. `DOCUMENT_TEMPLATES` joined against `cfg.document_types`:
   - every entry in `document_types` produces exactly one descriptor (slug = key, prefix/directory/enabled from the entry);
   - when the key matches a template slug, the template supplies category/label/status model/verbs/content model/frontmatter keys/id shape/archive policy/create options, and the entry supplies prefix/directory/enabled and the document status list;
   - when the key matches no template, the generic descriptor of section 6 is used;
   - a template slug missing from `document_types` produces no descriptor (removing an entry stops managing the type, REQ-007.002).

`build(cfg)` = `assemble(cfg)`, then `check_invariants`, then build `by_slug`. `check_invariants` walks `descriptors` in order and fields in the order listed in section 8, so the returned error is deterministic. Errors are not aggregated here; the aggregated report belongs to `Config::validate` (ADR-0019 decision 1).

## 4. Built-in tables

Static templates are `&'static` values that carry no configured data (no prefix, no directory value, no status list). Everything configuration-derived is filled by `assemble`, so a template cannot embed a project path or a stale prefix.

```rust
struct DescriptorTemplate {
    slug: &'static str,
    category: ArtifactCategory,
    label: &'static str,
    prefix: PrefixSource,          // Task | Milestone | Release | Decision | Entry
    directory_key: &'static str,   // "backlog_directory" | "decision_directory" | "doc_directory"
    subpath: Option<&'static str>, // Some("tasks"); None = the document_types entry directory
    id_shape: IdShape,
    status: TemplateStatus,        // TaskConfigurable | Fixed(&[&str]) | DocumentMain | HtmlMetadata(&[&str])
    verbs: TemplateVerbs,          // Fixed(&[Verb]) | DocumentBaseline { extensions: &[Verb] }
    content: ContentModel,
    frontmatter_extra: &'static [&'static str],
    archive: ArchivePolicy,
    create_options: CreateOptions,
}
```

`assemble` derives the final verb list: `Fixed` lists are copied as-is; `DocumentBaseline { extensions }` becomes `DOCUMENT_BASELINE_VERBS`, then `Status` when the status model is `DocumentMain`, then `extensions`, in that order.

## 5. Exact mapping: Backlog and Decision categories

`GENERAL_FRONTMATTER_KEYS = ["id", "title", "created_date", "updated_date", "labels", "references"]`; `frontmatter_keys` = general keys, then `status` when the type stores it in front matter, then the type's other keys (ADR-0022 decision 1).

| slug | label | prefix | directory | status model | verbs | content model (sections) | extra keys | id shape | archive policy |
|---|---|---|---|---|---|---|---|---|---|
| `task` | Task | `cfg.task_prefix` | `backlog_directory` / `tasks` | `TaskConfigurable { cfg.statuses, cfg.default_status }` | Create, List, View, Edit, Archive, Restore, Delete | Typed(Task): `## Description`, `## Acceptance Criteria`, `## Comments` | `status`, `assignee`, `milestone`, `dependencies`, `ordinal`, `documentation`, `type` | Plain | `ArchivedFolder { Some("status") }` |
| `milestone` | Milestone | `cfg.milestone_prefix` | `backlog_directory` / `milestones` | `Fixed(["open", "closed"])` | Create, List, View, Edit, Archive, Delete | Typed(Milestone): `## Description` | `status`, `release` | Plain | `ArchivedFolder { Some("status") }` |
| `release` | Release | `cfg.release_prefix` | `backlog_directory` / `releases` | `Fixed(["planned", "released"])` | Create, List, View, Edit | Typed(Release): `## Description` | `status` | Plain | `None` |
| one per `decision_types` key (`adr`, `bdr`) | the key (`adr` / `bdr`) | entry `prefix` | `decision_directory` / `""` | `Fixed(["proposed", "accepted", "rejected", "deprecated", "superseded"])` | Create, List, View, Edit | Typed(Decision): `## Status`, `## Decision`, `## Context`, `## Consequences` | `superseded_by` (`status`/`type` stay in the document core) | Plain | `None` |
| any configured document slug (section 6) | slug | entry `prefix` | `doc_directory` / entry `directory` | `DocumentMain` | baseline + `Status` | Generic | `status` | Plain | `ArchivedFolder { Some("status") }` |

Notes:

- Task verbs are the current family, including `archive`/`restore` (ADR-0022 decision 5 keeps them for tasks and milestones).
- Milestones keep the verbs the current command family exposes; the milestone restore cycle required by REQ-001.005 is not implemented in today's code and is not added by this refactor.
- Decisions declare no `Archive` verb and no archive policy: REQ-004.002 and ADR-0023 decision 3 state that decisions have no archive command or state.
- Releases declare no archive verb and `ArchivePolicy::None`; BR-007 states releases are never archived.

## 6. Exact mapping: the eleven document types

Every document descriptor carries `category = Document`; `enabled`, `prefix`, and `directory.relative` from the entry; `archive_policy = ArchivedFolder { state_field: Some("status") }` except `ui-mockup`; `frontmatter_keys` = general keys + `status` (empty for `ui-mockup`); `id_shape = Plain` except `req`/`nfr`.

| slug | label | content model | sections | extra keys | status model | verbs | id shape | create options |
|---|---|---|---|---|---|---|---|---|
| `req` | Business requirement | Typed(Requirement(Business)) | `## User Story`, `## Group`, `## Description` | none | DocumentMain | baseline + Status + CreateGroup | Grouped | group Required |
| `nfr` | Non-functional requirement | Typed(Requirement(NonFunctional)) | `## User Story`, `## Group`, `## Description` | none | DocumentMain | baseline + Status + CreateGroup | Grouped | group Required |
| `glossary` | Glossary term | Typed(Glossary) | `## Definition` | none | DocumentMain | baseline + Status | Plain | absent |
| `rule` | Business rule | Typed(Rule) | `## Description`, `## Rationale`, `## Applicability`, `## Requirements` | none | DocumentMain | baseline + Status | Plain | absent |
| `ui-spec` | UI specification | Typed(UiSpec) | `## View`, `## Description` | none | DocumentMain | baseline + Status | Plain | absent |
| `ui-mockup` | UI mockup | HtmlMockup (sections empty, extensions `[Supersede]`) | none | none (no front-matter block) | HtmlMetadata(["pending", "approved", "archived"]) | baseline + Supersede, no Status | Plain | absent |
| `audit` | Audit record | Typed(Audit) | `## Audit Type`, `## Content` | `type` (after `status`) | DocumentMain | baseline + Status | Plain | absent |
| `tech-spec` | Technical specification | Typed(TechSpec) | `## Description` | none | DocumentMain | baseline + Status | Plain | absent |
| `use-case` | Use case | Typed(UseCase) | `## Description`, `## Applicability` | none | DocumentMain | baseline + Status | Plain | absent |
| `test-case` | Test case | Typed(TestCase) | `## Description`, `## Applicability` | none | DocumentMain | baseline + Status | Plain | absent |
| `test-report` | Test report | Typed(TestReport) | `## Description`, `## Applicability` | none | DocumentMain | baseline + Status | Plain | absent |

`DOCUMENT_BASELINE_VERBS = [Create, List, View, Edit, Archive, Delete]` (REQ-007.003). `ui-mockup` archive policy is `ArchivedFolder { state_field: None }` because its state is written in HTML metadata, not front matter (ADR-0022 decision 2).

Document status model rules (`DocumentMain`):

- `MAIN_DOCUMENT_STATUSES` is the fixed `main` list; it is a constant, not a per-descriptor copy (ADR-0017: the registry never stores a second copy of the list).
- entry `statuses: None` gives `custom = []`.
- entry `statuses: Some(list)` gives `custom` = the list minus `active` and `archived`, in declared order.
- `default` = `default_status`, else `active`.
- `frontmatter_keys` = general + `status`; a status value is accepted when it is `active` or a declared custom value; `archived` is produced only by `archive`.

Generic descriptor (no static template):

| field | value |
|---|---|
| `category` | `Document` |
| `label` | the slug (ADR-0018 open question: the slug is the name) |
| `prefix`, `directory.relative`, `enabled` | from the entry |
| `status_model` | `DocumentMain { custom, default }` as above |
| `verbs` | baseline + `Status` |
| `content_model` | `Generic` |
| `frontmatter_keys` | general + `status` |
| `id_shape` | `Plain` |
| `archive_policy` | `ArchivedFolder { state_field: Some("status") }` |
| `create_options` | `group: Absent` |

`deploy` is never a template, so a configuration that declares it (including today's seeded default) resolves to this generic descriptor. `nfreq` is neither a template nor a descriptor slug: no alias exists.

## 7. `DocTypeEntry` extension (`src/config.rs`)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct DocTypeEntry {
    pub prefix: String,
    pub directory: String,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statuses: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_status: Option<String>,
}
```

- `Default for DocTypeEntry` gains `statuses: None`, `default_status: None`.
- Backward compatible: the struct-level `#[serde(default)]` plus the field-level defaults decode a file that has only `prefix`/`directory`/`enabled`; no other field name changes and `DocTypeEntry` has no `deny_unknown_fields`.
- Forward compatible: `skip_serializing_if` keeps an absent declaration absent, so `papyrus init` output and `config list`/`get` text are byte-identical until TASK-112 seeds statuses and TASK-111 exposes the keys.
- `Option` is required, not `Vec::default`, because `statuses: []` (explicitly empty) must stay distinguishable from an absent list: only the former is invalid (REQ-007.002, ADR-0019 decision 7).
- "Ordered" means the YAML sequence order is the semantic order. `serde` preserves it, the registry copies it, duplicate detection uses a set only for membership, and nothing sorts or de-duplicates the list. Order drives the browser status filter (REQ-006.014: `active` plus custom statuses in configured order) and is asserted by a test.
- Only the one struct literal in `Config::default()` and the `Default` impl change. `Config::flattened`, `Config::validate`, `merge_yaml`, `init`, and `commands` are untouched in this task.

## 8. Invariants, errors, and outcomes

`build` returns `Result<ArtifactRegistry, RegistryError>`. `RegistryError` is a plain enum with `Display`, `std::error::Error`, and:

```rust
impl From<RegistryError> for AppError {
    fn from(e: RegistryError) -> Self { AppError::Validation(e.to_string()) } // exit 4
}
```

Messages name the configuration key in ADR-0019 form: `document_types.<slug>.<field>: <detail>`, and `<slug>: <detail>` for non-document descriptors. Checks run in the order below; the first failure is returned.

| # | Invariant | Error variant | Message |
|---|---|---|---|
| I1 | No two descriptors share a slug | `DuplicateSlug { slug }` | `document_types.<slug>: duplicate type slug` |
| I2 | A configured document slug must not equal a built-in registry slug (`task`, `milestone`, `release`, `adr`, `bdr`), compared case-insensitively | `ReservedSlug { slug, builtin }` | `document_types.<slug>: slug conflicts with built-in type <builtin>` |
| I3 | `prefix` is non-empty | `EmptyPrefix { slug }` | `document_types.<slug>.prefix: must not be empty` |
| I4 | `prefix` is unique across all descriptors including disabled ones, compared case-insensitively | `DuplicatePrefix { slug, prefix, other }` | `document_types.<slug>.prefix: <prefix> is already used by <other>` |
| I5 | `directory.relative` is non-empty | `EmptyDirectory { slug }` | `document_types.<slug>.directory: must not be empty` |
| I6 | `directory.relative` is relative: no leading `/`, no `\`, no `.`/`..` segment, no drive prefix | `InvalidDirectory { slug, value }` | `document_types.<slug>.directory: must be relative and inside doc_directory: <value>` |
| I7 | Document `statuses`, when present, is non-empty | `EmptyStatusList { slug }` | `document_types.<slug>.statuses: must not be empty` |
| I8 | Document `statuses` has no duplicate value | `DuplicateStatus { slug, value }` | `document_types.<slug>.statuses: duplicate value <value>` |
| I9 | `statuses` contains `active` exactly once | `MainStatusCount { slug, value: "active", count }` | `document_types.<slug>.statuses: "active" must be declared exactly once` |
| I10 | `statuses` contains `archived` exactly once | `MainStatusCount { slug, value: "archived", count }` | `document_types.<slug>.statuses: "archived" must be declared exactly once` |
| I11 | A type without front-matter status (`ui-mockup`) declares neither `statuses` nor `default_status` | `StatusesNotAllowed { slug }` | `document_types.<slug>.statuses: not allowed for a type without a front-matter status` |
| I12 | `default_status`, when present, is a declared value | `DefaultStatusNotDeclared { slug, value }` | `document_types.<slug>.default_status: <value> is not a declared status` |
| I13 | `default_status` is not `archived` | `ArchivedDefaultStatus { slug }` | `document_types.<slug>.default_status: must not be "archived"` |
| I14 | In the `Document` category, `Status` is present exactly when a front-matter `status` key is declared | `StatusVerbMismatch { slug }` | `document_types.<slug>: verbs and status model disagree about "status"` |
| I15 | A descriptor's extension verbs equal its `TypedSchema::extensions` (set equality) | `ExtensionVerbMismatch { slug, verb }` | `document_types.<slug>: verb <verb> is not declared by the content model` |
| I16 | No `Document` descriptor carries `Restore` | `RestoreVerbOnDocument { slug }` | `document_types.<slug>: documents have no restore verb` |

Asserted at assembly time, not as error paths: every document descriptor is `Document`, every non-document descriptor is `Backlog` or `Decision`, and `ui-mockup` is the only `HtmlMockup`.

Deliberately outside these invariants (owned elsewhere): full slug grammar and reserved command names (`cli::BUILTIN_COMMANDS`, TASK-108/TASK-109); directory containment and symlink resolution (`Context::ensure_within`, TASK-109 and every read/write); task status-list validity (`Config::validate`, already present); and prefix/directory uniqueness across entries absent from the map (TASK-109's validator over the whole map).

## 9. Test plan

`src/registry.rs` (`#[cfg(test)] mod tests`, all filesystem-free, using `Config::default()` or small in-memory configs).

Built-in, typed, and generic descriptors:

1. `backlog_descriptors_match_the_current_command_families`
2. `decision_descriptors_follow_configured_decision_types` (add/remove an `adr`/`bdr` entry and a disabled entry)
3. `decision_descriptors_have_no_archive_policy_or_verb`
4. `default_document_set_has_the_eleven_typed_slugs`
5. `deploy_resolves_to_a_generic_descriptor`
6. `lookup_is_exact_and_case_sensitive` (`req` found; `Req`, `REQ`, `non-functional requirement`, `nfreq` return `None`)
7. `req_and_nfr_declare_grouped_ids_create_group_and_required_group`
8. `ui_mockup_declares_html_states_supersede_and_no_status`
9. `document_status_model_contains_main_statuses_and_configured_order`
10. `document_default_status_falls_back_to_active`
11. `frontmatter_keys_follow_the_field_map_order` (audit: general, `status`, `type`; mockup: empty)
12. `status_models_are_category_specific` (only task configurable; milestone/release/decision fixed; documents document-main; mockup html)
13. `id_shapes_cover_plain_dotted_grouped`
14. `archive_policies_follow_the_layout` (decisions and releases `None`; mockup state field `None`)
15. `disabled_descriptor_resolves_but_is_excluded_from_enabled`
16. `documents_are_enumerated_in_slug_order`
17. `descriptor_total_equals_the_configured_set`
18. `build_uses_no_project_directory` (no filesystem input exists; build succeeds for a config whose paths do not exist)
19. `registry_agrees_with_record_descriptor_for_default_documents` (prefix, directory value, sections, dotted-ness for the eleven slugs; characterization guard for the transition)

Invariant failures, through the `check_invariants` test seam:

20. `duplicate_slug_is_rejected`
21. `reserved_slug_is_rejected_case_insensitively`
22. `empty_prefix_is_rejected`
23. `duplicate_prefix_is_rejected_case_insensitively`
24. `duplicate_prefix_check_includes_disabled_entries`
25. `empty_directory_is_rejected`
26. `absolute_directory_is_rejected`
27. `traversing_directory_is_rejected`
28. `empty_status_list_is_rejected`
29. `duplicate_status_value_is_rejected`
30. `status_list_without_active_is_rejected`
31. `status_list_without_archived_is_rejected`
32. `status_list_with_repeated_main_status_is_rejected`
33. `statuses_on_ui_mockup_is_rejected`
34. `undeclared_default_status_is_rejected`
35. `archived_default_status_is_rejected`
36. `missing_status_verb_is_rejected`
37. `status_verb_on_a_type_without_frontmatter_status_is_rejected`
38. `extension_verb_mismatch_is_rejected`
39. `restore_verb_on_a_document_is_rejected`
40. `registry_error_maps_to_validation_outcome` (`AppError::Validation`, `code() == 4`)

`src/config.rs` (`#[cfg(test)] mod tests`):

41. `doc_type_entry_without_status_fields_deserializes`
42. `doc_type_entry_deserializes_statuses_and_default_status`
43. `status_declaration_order_is_preserved`
44. `empty_status_list_deserializes_as_some_empty`
45. `absent_status_fields_are_omitted_on_serialization`
46. `existing_configuration_files_still_load` (inline YAML with only `prefix`, `directory`, `enabled`)

Whole-suite checks: `cargo test` green, including `tests/no_drift.rs` unchanged (the registry contains none of its forbidden literals: no `deprecated/`, no `backlog/tasks`-style path, no front-matter delimiter, no second writer); `cargo clippy` clean.

## 10. Assumptions and recorded encoding decisions

- **Assumption A1**: the default map still contains `deploy` and no seeded `statuses`; a default build therefore has twelve document descriptors. Tests assert the eleven typed slugs and treat `deploy` as the generic case.
- **Assumption A2**: the registry is not wired into any module here; the only integration point is `pub mod registry;`.
- **Assumption A3**: non-document verb sets transcribe the current command enums; this refactor adds and removes no command.
- **Assumption A4**: `label` is the slug for config-defined document types and for `adr`/`bdr` (ADR-0018: the slug is the name); built-in document labels are the curated values in section 6.
- **Assumption A5**: an explicit empty `statuses: []` is accepted by deserialization and rejected only by ADR-0019 validation (TASK-109); the registry also rejects it as I7 so the behaviour is testable before the validator lands.
- **Assumption A6**: reserved-name scope in this task is the built-in registry slugs only. The command-name set is `cli::BUILTIN_COMMANDS` (ADR-0016 decision 2, ADR-0019 decision 3) and arrives with TASK-108; the registry must not create a second list.
- **Encoding decision D1**: `req`/`nfr` use `IdShape::Grouped`, which covers group records and dotted member records. ADR-0020 decision 4 phrases the same field as `Dotted`; ADR-0018 decision 5 pairs `Dotted`/`Grouped` as req/nfr properties. `Dotted` stays defined as the member-only shape and is asserted by a unit test. No behaviour difference; wording confirmation requested (see open questions).
- **Encoding decision D2**: `ArchivePolicy::None` for releases follows BR-007 ("releases are never archived") even though ADR-0023 decision 2 lists an archive location; no command writes it.
- **Encoding decision D3**: `ArchivePolicy::None` for decisions follows REQ-004.002 and ADR-0023 decision 3.

## 11. Out of scope (handoff)

- `Config::validate` document-type validation, whole-map `document_types`, tolerant repair load: TASK-109.
- Removing `deploy` from `Config::default()` and seeding `statuses`/`default_status`: TASK-112.
- Command tree, `cli::BUILTIN_COMMANDS`, baseline/`status` dispatch, deleting the static enums and `nfreq`: TASK-108.
- Generic handler: TASK-110. Record-layer archive/status and `deprecated/` removal: TASK-107.
- Search (TASK-113) and browser (TASK-114) iteration over `enabled(Document)`; the browser `form: Requirement | Group` of ADR-0020 decision 4 lands there.
- Per-type option metadata in the descriptor (`--view`, `--rationale`, `--audit-type`, `--task`/`--spec`) is deferred to TASK-110, the first consumer that needs it; the descriptor already names the `TypedHandler`, so no slug literal is needed to reach type-specific code.

## 12. Open questions

- **Q1 (encoding wording)**: ADR-0020 decision 4 says `req`/`nfr` have `id_shape: Dotted`, while this plan uses `Grouped` to cover group plus dotted member records (ADR-0018 decision 5 pairs the two). Confirm the encoding, or have `@architect` amend ADR-0020's phrasing at the next touch. Does not block implementation.
- **Q2 (ownership)**: REQ-004.002 states `papyrus decision archive` does not exist, and the descriptor therefore omits `Archive` for `adr`/`bdr`, but the static command still exists in `src/decision.rs` and no downstream task in this milestone lists its removal. Confirm which task owns that removal; until then the registry and the CLI disagree for decisions.


## Comments
author: @task-refiner
created: 2026-11-13T21:57:00Z

---

Refinement open questions from the implementation plan:

- Q1 (encoding, non-blocking): ADR-0020 decision 4 calls the req/nfr id shape `Dotted`, while the plan encodes it as `Grouped` (group records plus dotted member records; ADR-0018 decision 5 pairs the two). Implementation proceeds with `Grouped`; @architect should align ADR-0020 wording at the next touch.
- Q2 (ownership, needs an owner): REQ-004.002 forbids `papyrus decision archive`, so the decision descriptors declare no `Archive` verb, but the static command still exists in `src/decision.rs` and no milestone task lists its removal. Assign the removal to a task, otherwise the registry and the CLI disagree for decisions.

---
