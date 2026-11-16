# Registry-Derived Browser Views for Config-Defined Document Types

**Ticket:** TASK-114
**Milestone:** m-17 (Config-Defined Document Types and Artifact Categorization)
**Status:** Refined — execution-ready; `MOCK-0015` approved (2026-11-14)
**UI mockups:** [`MOCK-0015`](ui/mockups/MOCK-0015-browser-config-defined-document-types-review.html) (approved 2026-11-14, revised for the 2026-11-14 owner request) — UI description [`ui/browser-config-defined-document-types-ui.md`](ui/browser-config-defined-document-types-ui.md); reuses the approved shell [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html) and the generic document patterns of [`MOCK-0009`](ui/mockups/MOCK-0009-browser-technical-specifications-review.html)/[`MOCK-0013`](ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html)
**Requirements:** REQ-06.014, REQ-06.003, REQ-06.004, REQ-06.012, REQ-05.007, REQ-05.011, REQ-07.001, REQ-07.002, REQ-07.004, NFR-02.002
**ADRs:** ADR-0020, ADR-0017, ADR-0022, ADR-0005, ADR-0006, ADR-0013, ADR-0018

## Purpose

The browser still resolves document types through `browser::records::Category`
(fifteen variants, per-variant key/config-key/descriptor-name mappings) and a
hand-written route chain in `src/browser/mod.rs`. A document type added in
`document_types` therefore has no navigation, list, detail, edit, or API
surface, and a disabled type remains browsable because directory resolution
does not consult `enabled` for most types. This increment makes the startup
`ArtifactRegistry` (ADR-0017) the only source of the browser's document-type
set and renders every enabled type through the registry's descriptors, while
preserving the existing built-in page field sets (REQ-06.014, ADR-0020).

## Scope

In scope:

- One startup registry instance and one startup route/navigation table derived
  from it; `Category` and the per-type routing/API chain are removed.
- Registry-derived navigation grouped by the three artifact categories,
  inventory cards, list pages, detail routing, edit forms, not-found selection,
  JSON API endpoints, and lifecycle page.
- The shared list layout from `browser-shell-ui.md` §3 (ID, Title, Status, family slots, Created, Updated, Actions), with no family slot for a config-defined type, and read-only front-matter `created_date` display.
- Two generic view families (ADR-0020 decision 5): generic config-defined
  document pages and unchanged typed built-in pages selected by descriptor.
- Read-only status display and a validated status filter on document list
  views (REQ-06.014, NFR-02.002).
- Archived/unknown equivalence and disabled-type invisibility (REQ-05.007).
- Unit and integration tests; a characterization fixture for one built-in type.

Out of scope:

- CLI command surface, registry model, record storage, and patch primitives
  (TASK-105/107/110/113 are complete).
- Browser create or delete. The browser still does not create records; delete
  stays CLI-only (REQ-05.007).
- Browser status writes. The browser never edits status (REQ-05.011).
- Status filtering on the JSON API and search (deferred to a future release;
  OQ-2, owner `@planner`).
- Changing the record content model (ADR-0018) or adding new dependencies,
  templates engines, or assets (ADR-0005).
- Changing the layout of non-document lists (the task board and the
  milestone, release, and decision list layouts) beyond the status behavior
  already scoped.
- Repository-wide navigation redesign; only the document-type sidebar is
  grouped as section 2.1 defines.

## Behavior

### 1. Registry and startup route table

1. The registry is the one already built and validated by `cli::dispatch` at
   startup (the same instance TASK-113 uses); `dispatch` passes it to
   `browser::run`, which derives one immutable `BrowserRoutes` from it before
   binding the listener. The browser never calls `ArtifactRegistry::build`
   again. A registry build error is already a validation outcome (exit 4) and
   the server does not start.
2. `ServerState` stores the registry (a clone of the startup handle; add
   `Clone` to `ArtifactRegistry`) and the route table for the process
   lifetime. Handlers resolve descriptors from the stored registry and never
   build a second registry or enumerate document slugs themselves.
3. The route table remains fixed for the process lifetime (ADR-0020 decision 2,
   ADR-0005): exact string comparison after single-pass percent-decoding, no
   path join, no filesystem lookup during route matching.
4. Document-type configuration is therefore a startup snapshot. A change to
   `document_types` (add, remove, enable, disable, rename, re-point) takes
   effect on the next browser start. The `/config` page states this next to
   the document-type keys (A1).
5. The table is composed of:
   - fixed routes unchanged: `/assets/app.css`, `/assets/app.js`, `/`,
     `/skills`, `/lifecycle`, `/config`, `/config/edit`, `/doc/{id}`,
     `/doc/{id}/edit`, `/api/doc/{id}`;
   - one canonical list API route per **enabled** `Document` descriptor:
     `/api/docs/<slug>`;
   - the legacy API aliases in section 5.3, generated only for descriptors that
     exist and are enabled;
   - one canonical `?type=<slug>` list value per enabled descriptor in **all**
     three categories, plus the HTML aliases in section 2.2;
   - one navigation item and one inventory card per enabled `Document`
     descriptor;
   - one prefix/ID-shape rule per enabled descriptor for 404 page selection.
6. No request path is ever joined to a filesystem path. A descriptor's
   configured directory is resolved against the project directory and passes
   `Context::ensure_within` before any read (NFR-04.002, ADR-0005).

### 2. Navigation and list route

#### 2.1 Sidebar and inventory

1. The sidebar is generated from `BrowserRoutes` and is grouped by the three
   artifact categories, with group headings in this order: `Backlog`,
   `Decisions`, `Documents`. Each group contains one entry per enabled
   descriptor of that category; a config-defined document type appears under
   `Documents`; a disabled type has no entry in any group.
2. The `Backlog` group keeps the existing single `Backlog` entry (the task
   board). The `Decisions` group keeps the existing single `Decisions` entry.
   Milestone and release records gain no new navigation entry in this
   increment.
3. The `Documents` group keeps the existing built-in document entries in their
   current relative order: Business requirements, Business Rules, Audit
   Records, Technical Specifications, Use Cases, Test Cases, Test Reports,
   Non-functional requirements, Glossary, UI Specifications, UI Mockups.
   Config-defined enabled entries follow UI Mockups in registry order (slug
   order) and before the trailing utility entries. Labels are unchanged.
4. Configuration, Managed artifacts &amp; lifecycles, and Agent skills &amp;
   workflows keep their current labels and order after the three groups; they
   are utility entries and are not part of a category group.
5. A selected entry carries `aria-current="page"` and the existing non-colour
   cue. The selected value is the descriptor slug; legacy `?type=` values
   select the same entry. A group heading is presentational (not a link, not
   selectable).
6. The inventory page is unchanged by this grouping: it renders one card per
   enabled document type, generated from the same route table. Built-in cards
   keep their existing titles, notes, and ordering; a config-defined card uses
   the title-cased label and the note
   `Config-defined document records in this project.` A disabled type has no
   card. The lifecycle card count is the registry descriptor count, not the
   literal `12 types`.
7. The approved mockup draws the utility entries under a fourth `Project`
   heading. That heading is presentational only: this specification is
   authoritative, the sidebar has exactly the three category groups, and the
   utility entries follow them without a fourth group heading.

#### 2.2 List route and type aliases

1. `GET|HEAD /?type=<slug>` serves the list view for every enabled descriptor
   in every category. `GET /` without a `type` still serves the inventory.
2. `<slug>` is the registry slug (`req`, `nfr`, `glossary`, `rule`, `ui-spec`,
   `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`,
   task, milestone, release, decision, or a configured slug).
3. The existing HTML aliases remain accepted and select the same descriptor:
   `business-rule` → `rule`, `audit-record` → `audit`. Generated built-in links
   keep their current alias (`/?type=business-rule`, `/?type=audit-record`) so
   approved page output is unchanged; config-defined links use the slug.
4. The virtual values `skills`, `lifecycle`, and `config` are handled before
   registry lookup and keep their current pages.
5. An unknown, empty, or disabled `type` value returns `404` with the generic
   error page, not `400` (ADR-0020 decision 3). The status-code change from
   `400` to `404` is intentional and documented in the task.
6. A known route with a disallowed method is `405` with an `Allow` header; an
   unknown path is `404` for every method.

### 3. List views

#### 3.1 Shared list contract

1. Every document list view renders the enabled records of exactly one type,
   read from that descriptor's active directory only (section 6).
2. Every document list view uses the shared list contract in
   `browser-shell-ui.md` §3: the fixed order **ID**, **Title**, **Status**,
   family slot(s), **Created**, **Updated**, **Actions**. A generic
   config-defined type declares no family slot; a typed built-in family
   declares its slots as data. Grouped requirement lists keep their
   group/member structure inside the shared table.
   - `Status` renders the stored status read-only, exactly as stored.
   - `ID` renders the record ID as text.
   - `Title` links to `/doc/<ID>`.
   - `Created` renders the front-matter `created_date` as stored, or `(none)`
     when the key is absent; it is never derived or rewritten by the browser.
   - `Updated` renders the stored `updated_date`, or `(none)`.
3. Rows are ordered by ID ascending. The Title cell carries the record labels and references; the legacy type-specific row metadata is not carried over by the shared contract.
4. The page shows a count of the records displayed (after filtering) and the
   existing empty state when there are none.
5. Built-in typed list pages render through the shared contract of (2) and
   the shared empty state of `browser-shell-ui.md` §3.6; the status filter
   (3.2) is added to them. Requirement lists keep their group/member
   structure (req/nfr), and the task route keeps the board page.

#### 3.2 Status filter

1. The list view for every document type exposes a status filter built from
   the descriptor's status model:
   - `DocumentMain { custom, .. }` → the filterable values are `active`
     followed by `custom` in configured order. `archived` is never a value.
   - `HtmlMetadata` (UI mockups) → `pending`, `approved` in lifecycle order.
     `archived` is never a value.
2. The filter control is a no-JavaScript GET form: a `Status` select listing
   exactly the filterable values of (1) in order, an `Apply filter` submit, and
   a `Show all` reset link. The select has no `All records`/empty-value option:
   the no-filter state is the absence of a `status` parameter, reached with
   `Show all`. When no filter is active the select shows a disabled
   placeholder option selected by default, so an untouched submit omits
   `status` entirely. The offered `status=` values are exactly the set in (1). `?type=<slug>&status=<value>` remains directly
   addressable without the form.
3. With no `status` parameter, every record in the active directory is listed,
   including records carrying a custom status. Custom statuses never hide
   records (ADR-0022 decision 10).
4. With `status=<value>`, only records whose stored status equals `<value>`
   exactly are listed. Comparison is byte-exact and case-sensitive; no
   normalization, aliasing, or fuzzy match.
5. For grouped requirement lists, filtering is applied to records before
   grouping: only matching requirement records are listed; a group heading is
   shown when the group record matches or when at least one of its members
   matches; a group with neither is omitted. Orphan-member rows are recomputed
   from the filtered set. Synthesised NFR group headings are display-only and
   are shown only when they retain at least one matching member.
6. The selected option is rendered selected in the select and is also a
   directly addressable URL; the filter remains usable with JavaScript
   disabled.
7. A type that declares no custom statuses offers only `active`; no filter and
   `active` therefore list the same set unless the directory contains a
   record with an undeclared status value (3.3).
8. A `status` value outside the set in (1) — including `archived`, an
   undeclared value, a wrong-case value, or an empty value on an explicit
   `status=` parameter — returns `400` with the standard error page and no
   listing. An absent `status` parameter is the no-filter state and is never
   rejected; only an explicitly supplied empty `status=` is invalid. More than
   one `status` parameter is rejected the same way (NFR-02.002).

#### 3.3 Tolerant status values

1. A stored status outside the type's declared vocabulary is displayed verbatim
   and never rewritten by a read (ADR-0022 decision 9).
2. Such a record is listed only when no `status` parameter is present. It is
   not offered as a filter value and is never matched by `active` or a custom
   value it does not carry exactly.
3. A write to such a record keeps the value unless the edit explicitly changes
   another field; the browser never writes `status`.

### 4. Detail and edit views

#### 4.1 Detail

1. `GET|HEAD /doc/<ID>` resolves the ID to exactly one enabled descriptor by
   configured prefix and `id_shape` (`Grouped` accepts group and member IDs;
   `Dotted` member IDs; `Plain` sequence IDs), then reads the record from that
   descriptor's active directory. Detail addressing stays ID-based; the URL
   carries no type.
2. A generic document detail page renders:
   - breadcrumb `<type label> > <ID>`;
   - `h1` `<ID> <title>`, the labels taglist under the title, and a read-only
     status line;
   - a summary card containing only the record ID, created date, and updated
     date (REQ-06.012);
   - the description section (`No description.` when empty);
   - a separate read-only metadata list with the remaining `RecordView` values
     (including status verbatim and any unknown front-matter keys);
   - references as a native `<details>` disclosure at the end, shown only when
     the record has at least one reference (REQ-06.012);
   - an `Edit` action linking to `/doc/<ID>/edit`.
3. A typed built-in detail page keeps its existing sections, metadata rows,
   related-record links, and actions. Status is visible read-only where the
   existing `RecordView`/type-specific section already shows it, and is added
   as a read-only row for types without it.
4. The sidebar is selected for the record's type; the breadcrumb's type link
   is the type's list route.

#### 4.2 Edit

1. `GET|HEAD /doc/<ID>/edit` renders the edit form selected by descriptor:
   generic config-defined types get the generic form; typed built-ins get
   their existing per-type form.
2. The generic form offers exactly `title` (required, 1–200 characters),
   `description` (optional, multi-line, no line beginning with `##`),
   `labels` (repeatable tag list), and `references` (repeatable tag list).
   It renders `created_date` and `updated_date` disabled/read-only, carries
   the hidden version token, and offers Save and Cancel.
3. The generic form never renders a `status` control, a `status_change` control,
   or any hidden field that can write status. Typed built-in forms keep their
   existing field sets and likewise contain no status control.
4. Saving a generic record (`POST /doc/<ID>`, form-encoded):
   - requires a valid version token present in the form; a missing or malformed
     token is `400`;
   - resolves the record and descriptor exactly as a detail request;
   - returns `415` for a non-form content type and `400` for malformed bodies;
   - validates title, description, labels, and references with the shared
     validators; a validation failure is `422` and nothing is written;
   - patches the front-matter `title` together with the heading line, the
     `## Description` section, and the `labels`/`references` lists through
     `record::patch`, then writes once through `record::store::write_atomic`
     (ADR-0006, ADR-0013); unknown sections and unknown front-matter keys are
     preserved;
   - never writes `status` and never changes the file location;
   - refreshes `updated_date` only when content changed and performs no write
     when nothing changed;
   - returns `409` with a conflict page that preserves the submitted values
     when the version token is stale.
5. A generic save request that carries a `status` or `status_change` field is
   rejected with `422` and writes nothing. `status_change` is a task-board
   control and is accepted only for descriptors whose status model is
   `TaskConfigurable`.
6. Typed built-in save behavior is unchanged; each handler is selected by the
   descriptor's `ContentModel`/`TypedHandler`, not by a slug or `Category`.

### 5. JSON API

#### 5.1 Canonical endpoints

1. Every enabled document descriptor has `GET|HEAD /api/docs/<slug>`.
2. The canonical endpoint returns exactly the JSON its legacy endpoint returns
   for that type, so an alias is indistinguishable from the canonical form:
   - generic config-defined type: an array sorted by ID of
     `{"id", "kind": "<slug>", "title", "status", "labels": [..],
     "references": [..], "version", "path", "metadata"}`;
   - `req`/`nfr`: the existing `{"groups": [...], "requirements": [...]}`
     object;
   - `ui-mockup`: the existing mockup array;
   - every other built-in: its existing array shape.
3. `GET|HEAD /api/doc/<ID>` resolves an enabled descriptor exactly as a detail
   request and returns the existing record object; a generic record returns
   `{"id", "kind": "<slug>", "form": "document", "title", "description",
   "status", "labels", "references", "version", "path", "metadata"}`.
4. Status filtering is not part of the API in this increment (OQ-2, deferred
   to a future release; owner `@planner`).

#### 5.2 Errors

1. An unknown or disabled `<slug>` returns `404` with the JSON error object.
2. Any method other than GET/HEAD on a canonical or alias list route returns
   `405` with `Allow: GET, HEAD`.
3. An unknown path returns `404` for every method.

#### 5.3 Legacy aliases (all generated only for an enabled descriptor)

| Canonical | Legacy paths kept |
|---|---|
| `/api/docs/req`, `/api/docs/nfr` | `/api/requirements` (`?type=req` / `?type=nfr`) |
| `/api/docs/glossary` | `/api/glossary` |
| `/api/docs/ui-spec` | `/api/ui-specs`, `/api/ui-specifications`, `/api/ui-spec` |
| `/api/docs/ui-mockup` | `/api/ui-mockups`, `/api/ui-mockup`, `/api/mockups` |
| `/api/docs/rule` | `/api/business-rules`, `/api/business-rule` |
| `/api/docs/audit` | `/api/audit-records`, `/api/audit-record` |
| `/api/docs/tech-spec` | `/api/tech-specs`, `/api/tech-spec` |
| `/api/docs/use-case` | `/api/use-cases`, `/api/use-case` |
| `/api/docs/test-case` | `/api/test-cases`, `/api/test-case` |
| `/api/docs/test-report` | `/api/test-reports`, `/api/test-report` |
| `/api/docs/<configured-slug>` | none |

### 6. Visibility: archived, unknown, disabled

1. `build_index` reads only the descriptor's active directory, top level,
   non-recursively. `archived/`, a legacy `deprecated/` folder, and any other
   subdirectory are never read (REQ-05.007, ADR-0022 decision 6).
2. A candidate file is a record only when its extension follows
   `ContentModel` (`.html` for `HtmlMockup`, `.md` otherwise) and its stem
   matches the descriptor's `prefix` and `id_shape`. No slug selects the rule.
3. On the browser list, detail, and edit paths, a file in the active directory
   whose front-matter status is `archived` is auto-relocated through
   `record::store::relocate_if_archived` before the record is returned
   (ADR-0022 decision 12). A relocated record is omitted from the list and is
   no longer resolvable; a relocation failure is a `500`/validation outcome
   with the source intact and the record not listed.
4. An archived ID and an unknown ID of the same type return the identical
   `404` page body for list, detail, and edit requests, and the identical JSON
   error body for `/api/doc/<ID>`. The not-found page never echoes the
   requested ID, so the two responses are byte-identical.
5. Archived records never appear in any canonical or legacy list API
   response: `/api/docs/<slug>` and its aliases are built from the active
   index. An archived ID is also never resolvable through `/api/doc/<ID>`.
6. Not-found page selection is presentation-only: an ID matching an enabled
   built-in descriptor's prefix and shape renders the existing not-found page
   of its descriptor's `TypedHandler` (`tech-spec`, `use-case`, `test-case`,
   `test-report`); every other document type (including config-defined types)
   renders one generic document not-found page. Selection follows the
   descriptor and its handler; it is not a slug branch in list, detail, or
   edit logic. Non-document records keep the generic error page.
7. A disabled type produces no navigation entry in any group, no inventory
   card, no list page (`404`), no detail or edit page (its prefix is not
   resolvable), and no canonical or legacy API response (`404`).
8. A disabled type still appears on the lifecycle page with `enabled: false`
   (section 7).

### 7. Lifecycle page

1. `GET|HEAD /lifecycle` (and `/?type=lifecycle`) renders three category
   groups in this order: `Backlog`, `Decisions`, `Documents` (REQ-06.003).
2. Each group contains one section per registry descriptor, in registry order:
   backlog = tasks, milestones, releases; decisions = every decision descriptor;
   documents = every document descriptor, **enabled and disabled**. The section
   count is derived, never fixed.
3. Each section shows:
   - a curated description for a built-in slug from the static presentation
     table, or the shared generic description for a config-defined slug;
   - the lifecycle: task configured status chain; milestone `open → closed`;
     release `planned → released`; decision
     `proposed → accepted | rejected; accepted → deprecated | superseded`;
     document `active → archived` plus the declared custom statuses in
     configured order, described as front-matter values that do not move files
     (`(none)` when empty); UI mockup `pending → approved → archived`;
   - read-only effective configuration: `enabled`, ID prefix, storage folder
     (`doc_directory` + configured directory for documents), and, for
     front-matter documents, the declared custom statuses;
   - a link to `/config` for changes; no form or input control.
4. Configuration values are HTML-escaped and display-only. The page loads no
   external resource and works without JavaScript.
5. The existing page statements about the three categories and the excluded
   Backlog.md `docs`, `drafts`, and project configuration files are retained.
6. The fixed twelve-section contract of the earlier lifecycle specification is
   superseded by this section for this milestone.

### 8. Presentation table and compatibility rules

1. A static built-in table maps a built-in slug to its label, list heading and
   subheading, curated lifecycle description, nav position, legacy `?type=`
   values, and legacy API path aliases. It is presentation only: it never
   decides whether a type exists, and a descriptor missing from the table
   still renders with derived text.
2. A test asserts that the rendered sidebar document entries across the three
   category groups, inventory card set, canonical API route set, and `?type=`
   canonical set each equal
   `registry.enabled(ArtifactCategory::Document)` for a fixture configuration,
   so the table cannot become a second authoritative list.
3. `src/browser/` keeps exactly one document-type source of truth: the startup
   `ArtifactRegistry`. No second document-type registry, count, or slug list
   may decide which types exist or which list/detail/edit behavior runs.
   Behavior branches on descriptor fields only — `ArtifactCategory`,
   `ContentModel`, `TypedHandler`, `IdShape`, and `StatusModel` — so a typed
   built-in page or save handler is selected through its descriptor's
   `TypedHandler`. Slug literals may appear only in the static presentation
   table of (1), where they map legacy `?type=` values and API path aliases,
   labels, nav order, and curated text; they never select list, detail, edit,
   or dispatch behavior.

## Implementation plan (task level)

1. **New `src/browser/routes.rs`**
   - `BrowserRoutes::build(&ArtifactRegistry)` produces:
     `api_lists: BTreeMap<&'static str, String>` (canonical + legacy path →
       slug), `type_values: BTreeMap<String, String>` (canonical slug and
       HTML aliases → slug), `nav: Vec<NavItem { slug, label, url_key,
       group }>` (`group` = the `Backlog`/`Decisions`/`Documents` heading),
       `inventory: Vec<InventoryEntry>`, and
       `id_rules: Vec<IdRule { prefix, id_shape, not_found: NotFoundPage }>`
       for enabled descriptors.
   - Keep the static presentation table here (built-in labels, list headings,
     descriptions, nav order, legacy URL keys).
   - Unit tests: table sets equal the enabled registry set; no disabled slug;
     no `Category`.
2. **`ServerState` and startup** (`src/browser/mod.rs`)
   - Build `ArtifactRegistry` and `BrowserRoutes` in `run()`, store both in
     `ServerState`, and pass `&BrowserRoutes` to `route`.
   - Replace `category_from_query` with `type_from_query(&routes, request)`:
     resolve aliases, return `404` for unknown/disabled, and keep the virtual
     `skills`/`lifecycle`/`config` handling.
   - Replace the hand-written per-type API chain with:
     one lookup for canonical/legacy list routes → `api_doc_list(slug)`;
     `api_doc_list` loads `state.registry.descriptor(slug)`, builds the index,
     and serializes by `ContentModel`/`TypedHandler` with the legacy shapes.
   - Add a `/config` note that document-type changes require a browser restart.
3. **`src/browser/records.rs`**
   - Delete `Category` and `Form`. `Record` stores the owning descriptor
     (slug, category, content model, status model, prefix, id shape) plus
     `is_group` derived from `IdShape::Grouped` and the resolved ID.
   - Replace `descriptor_name`, `active_prefixes`, `uri_kind`,
     `parse_id_for`, `canonical_key`, `matches_stem`, `form_for_stem`, and
     `record_directory` with descriptor-driven helpers:
     `resolve_directory(descriptor, cfg, project_dir)` from
     `DirectorySpec`, `matches_stem(descriptor, stem)` from
     `prefix`/`id_shape`, and `form(descriptor, id)`.
   - `build_index(ctx, cfg, descriptor, filter)`: empty for a disabled
     descriptor; active directory only; extension by `ContentModel`; skip a
     candidate that auto-relocates out of the active directory; filter by the
     exact stored status before sorting; derive groups/members after filtering.
   - `resolve_id(ctx, cfg, registry, id)`: iterate enabled descriptors, match
     prefix/shape (`Grouped` tries member then group; `Dotted` member only;
     `Plain` sequence), resolve in that descriptor's active directory, and
     return the record with its descriptor. No cross-type ambiguous scan.
   - `load_record`: read the record through
     `read_document_normalized` (already auto-relocates), project title,
     description, labels, references, `created_date`, `updated_date`, and
     status from the shared `RecordView`/front matter for every document type;
     keep the HTML mockup loader for `HtmlMockup`.
4. **`src/browser/render.rs`**
   - Replace every `match kind`/`Category::` branch with
     `match record.content_model`/`TypedHandler` (requirement, glossary,
     ui-spec, rule, audit, tech-spec, use-case, test-case, test-report,
     ui-mockup, task, milestone, release, decision) and keep the existing
     functions and markup.
   - `sidebar(selected_slug, routes)` and inventory rendering use the route
     table grouped by artifact category; remove the hard-coded enabled checks
     and `category_name` matches; derive labels from the presentation table or
     title-case the slug.
   - Add `list_page`, the `document_detail_page`, `document_edit_page`,
     and `document_conflict_page` renderers for `ContentModel::Generic`; every
     list route builds a `ListPlan` and renders the shared `list.html` body
     fragment.
   - Add one shared list builder (`ListPlan`) and the declared presentation
     slots for every family, so every list renders the same seven-column order
     with its declared slots, plus one shared
     `status_filter_html(descriptor, selected)` helper and a shared
     `status_text(record)` helper.
   - Reduce the not-found pages to: the four existing built-in pages selected
     by handler, one generic document not-found page, and the existing generic
     error page.
   - Rewrite `lifecycle_page(routes, registry, cfg, chrome)` to render the
     three groups and one section per descriptor from registry data.
5. **`src/browser/mod.rs` handlers**
   - `list_page`: descriptor from `type_from_query`; task → board; other
     backlog/decision → existing list; document → typed list (existing
     function) or generic list, both with the filter applied.
   - `view_page`: descriptor from `resolve_id`; generic → detail page; typed →
     existing per-type view, selected by handler.
   - `edit_page`: descriptor + record; generic → generic edit; typed →
     existing per-type edit.
   - `form_post`: dispatch by descriptor content model/handler; add
     `save_generic_document` using `records::validate_*` +
     `record::patch` + `save_ops`; reject `status`/`status_change` for
     non-task descriptors with `422`; generic conflict page on stale version.
   - `handle_request` 404 mapping: use `routes.id_rules` to choose the
     built-in not-found page or the generic document page; no ID echo.
6. **Templates**
   - The legacy generic list template is superseded by the shared `list.html`
     body fragment; `document_detail.html` and `document_edit.html` remain body
     fragments, embedded with `include_str!` and using `render_template`
     escaping.
7. **Tests**
   - New `tests/browser_config_types_cli.rs` for T-114-01..13.
   - Update `tests/browser_cli.rs` only where this specification deliberately
     changes output: unknown `?type=` 400→404, grouped sidebar structure,
     lifecycle section structure and count, inventory count, the shared list
     column order, and the additive status filter.
8. **Docs**
   - This specification; UI mockup/description from the delegated design;
     ticket updated with the route table and test IDs.

## Test plan

| ID | Scenario | Assertion |
|---|---|---|
| T-114-01 | User-added type `note` (prefix `NOTE`, directory `notes/`, statuses `["active","review","approved"]`, default `active`), records created with the CLI | `GET /?type=note` 200 lists both records in the shared seven-column contract with no family slot; sidebar shows `Notes` under the `Documents` group; `Created` shows the front-matter `created_date`; detail 200; edit 200; `GET /api/docs/note` 200 array with `id/title/status/labels/references/version/path/metadata` |
| T-114-02 | Canonical vs legacy API | `/api/docs/glossary` body equals `/api/glossary`; `/api/docs/rule` equals `/api/business-rules`; `/api/docs/audit` equals `/api/audit-records`; `/api/docs/ui-mockup` equals `/api/mockups`; `/api/docs/req` equals `/api/requirements?type=req` |
| T-114-03 | Status read-only | List shows the structural Status column in the fixed order and the detail shows `review`; `Created` equals the stored `created_date`; edit form contains no `name="status"`/`name="status_change"`; `POST /doc/NOTE-001` with `status=archived` and a valid version returns 422 and the file is byte-identical |
| T-114-04 | Filter values and order | The filter select offers exactly `active`, `review`, `approved` in that order and no empty/`All records` option; `Show all` is the only reset and omits `status`; no `status=archived` option; `?status=review` lists only the review record; `?status=approved` shows the empty state; a type with no custom statuses offers only `active` |
| T-114-05 | Filter rejection | `?status=archived`, `?status=bogus`, `?status=Review`, `?status=` and duplicate `status` parameters return 400 with no record list |
| T-114-06 | Default listing keeps custom statuses | No `status` parameter (including the `Show all` link) lists active-, review-, and approved-status records together; `All records` is not an offered status value |
| T-114-07 | Archived vs unknown | Archive `NOTE-001`; `GET /doc/NOTE-001`, `GET /doc/NOTE-001/edit`, and `GET /api/doc/NOTE-001` are 404, and the two page bodies for the archived and unknown `NOTE-999` IDs are identical; list, `/api/docs/note`, and the legacy alias exclude it; archived/ is never indexed |
| T-114-08 | Archived-in-active auto-relocation | Hand-edit `notes/NOTE-002.md` to `status: archived`; browser list moves it to `notes/archived/`, omits it, and detail, edit, and API requests 404 |
| T-114-09 | Disabled type invisibility | Start with `document_types.note.enabled false`: no sidebar entry under `Documents`, no inventory card, `/?type=note` 404, `/api/docs/note` 404, `/doc/NOTE-001` 404; lifecycle still shows `Notes` with `enabled: false` |
| T-114-10 | Generic edit round trip | POST title/description/labels/references: front matter and heading updated, description section updated, labels/references replace the edited lists, unknown `## Notes` section and unknown key preserved, status unchanged, updated_date refreshed, second identical save writes nothing; stale version returns 409 |
| T-114-11 | Built-in characterization | Glossary list/detail/edit fixture matches the recorded baseline apart from the shared seven-column list contract and the additive status filter; tech-spec, rule, ui-mockup, req/nfr list/detail/edit keep their existing sections and field sets with the shared row fields |
| T-114-12 | Lifecycle page | Default config renders `Backlog`, `Decisions`, `Documents` groups, one section per registry descriptor (16 by default), custom statuses and prefix/folder/enabled per document, shared description for `note`, and no fixed `12` count |
| T-114-13 | Route/table invariants | Unit test: sidebar document entries across the three groups = inventory slugs = canonical API slugs = `type` canonical values = `registry.enabled(Document)`; the sidebar renders exactly the `Backlog`, `Decisions`, `Documents` groups; grep test: no `enum Category`, no `Category::`, no second document-type registry, and no per-slug list/detail/edit/dispatch branch in `src/browser/`; slug literals occur only in the static presentation table |
| T-114-14 | Method and safety | `POST /api/docs/note` 405 + `Allow: GET, HEAD`; unknown API slug 404; coded directory escaping the project yields no page and no traversal; `?type=business-rule` and `?type=rule` render the same list |

## Acceptance criteria mapping

- AC 1 (startup registry/route table, no Category) → T-114-13, review.
- AC 2 (list/detail/edit/sidebar/API per enabled type, aliases) → T-114-01, T-114-02, T-114-14.
- AC 3 (generic list/detail content) → T-114-01, T-114-03.
- AC 4 (typed built-ins preserved) → T-114-11.
- AC 5 (status read-only, no status write) → T-114-03, T-114-10.
- AC 6 (filter set and default listing) → T-114-04, T-114-06.
- AC 7 (filter rejection) → T-114-05.
- AC 8 (archived/unknown/disabled) → T-114-07, T-114-08, T-114-09.
- AC 9 (lifecycle) → T-114-12.
- AC 10 (API) → T-114-02, T-114-14.
- AC 11 (generic edit) → T-114-10.
- AC 12 (tests) → all of the above.

## Assumptions

- **A1 — Startup snapshot.** ADR-0020 decision 2 fixes the table for the
  process lifetime. `document_types` edits made through `/config` or externally
  take effect on the next browser start; `/config` says so. Tests change
  configuration and start a new server.
- **A2 — Grouped sidebar.** The sidebar is grouped by the three artifact
  categories (`Backlog`, `Decisions`, `Documents`) rather than presented as one
  flat list (2026-11-14 owner decision, replacing the earlier flat-sidebar
  assumption). Built-in labels stay unchanged, document entries keep their
  relative order inside `Documents`, and config-defined entries follow UI
  Mockups. This matches the category model already required for the lifecycle
  page (REQ-06.003) without changing entry labels or page routes.
- **A3 — Filter default is "all active-directory records".** A custom status
  must never hide a record (ADR-0022 decision 10), so the resting state is a
  `Show all` reset link that omits `status`; selecting `active` is a real
  filter that excludes custom values. `All records` is not an offered status
  value, and an explicit empty `status=` is rejected like any other invalid
  value.
- **A4 — UI mockups use their HTML status model.** Their filter values are
  `pending`/`approved`, `archived` excluded, because their status is not
  front-matter and REQ-06.014's `active`-plus-custom list does not apply.
- **A5 — Canonical API equals legacy output.** "Alias" means the same body and
  status code for the same type; the canonical route is not a new schema.
- **A6 — Browser label/reference editing replaces the edited lists.** The CLI
  `edit` appends; the browser form shows the current tags and saves the
  resulting lists so a user can remove a label. Both use the same patch and
  writer; status is untouched.
- **A7 — Status is displayed as stored.** A front-matter record without a
  `status` key (legacy/incomplete) shows `(none)` read-only and is never
  rewritten by the browser.
- **A8 — The filter applies to every document list view,**
  including typed built-ins, because ADR-0020 decision 5 says list and detail
  render status read-only "for every document type"; typed pages keep their
  field sets and sections, and the filter is additive.
- **A9 — List columns are a document-list contract.** The fixed order
  ID, Title, Status, family slot(s), Created, Updated, Actions applies to
  every document list view (generic and typed built-in), with the family
  slots declared as data and no slot declared for a config-defined type
  (`browser-shell-ui.md` §3). The task board and the milestone,
  release, and decision list layouts are unchanged. `Created` and `Updated`
  are read-only front-matter values shown as stored (`(none)` if absent), never
  derived or rewritten; `created_date` is already part of the shared metadata
  projection (REQ-06.004).


## UI notes

- `MOCK-0015` is one self-contained review package with nine views: Notes list
  (default), filtered list, empty list, detail, edit, validation error,
  stale-version conflict, lifecycle delta, and the sidebar treatment. The
  2026-11-14 revision (grouped sidebar and Status/ID/Title/Created/Updated
  list columns) was approved by the project owner on 2026-11-14; the
  specification remains authoritative if they disagree.
- The designed list filter is a no-JavaScript GET form. The approved mockup
  draws an `All records` select option; because a no-JavaScript select submits
  an empty `status=` for that option and an explicit empty `status=` is
  rejected (3.2.8), this specification removes the option: the select lists
  only the allowed values and `Show all` is the no-filter reset (3.2.2).
- The list is the shared seven-column table; Status is a structural column and
  is read-only in list and
  detail; the edit view has no status control.
- The sidebar is grouped under `Backlog`, `Decisions`, and `Documents`
  headings. Every built-in entry keeps its approved label and relative order
  inside its group; the config-defined `Notes` entry follows `UI Mockups` in
  the `Documents` group. Configuration, Managed artifacts &amp; lifecycles,
  and Agent skills &amp; workflows follow the three groups. The approved
  mockup draws those utility entries under a fourth `Project` heading; that
  heading is presentational only and the three-group structure is
  authoritative (2.1.7).
- The lifecycle delta shows `Notes` (enabled) and `Deploy logs` (disabled) in
  the `Documents` group with the shared lifecycle, custom statuses, prefix,
  folder, and enabled state.
- The package and its 2026-11-14 revision were explicitly approved by the
  project owner on 2026-11-14. The layout requirements in this specification
  remain authoritative if the mockup and the specification disagree.

## Open questions

- **OQ-1 (resolved 2026-11-14 by the project owner)** — The sidebar is
  grouped by the three artifact categories with document types listed inside
  `Documents`; the earlier flat-sidebar assumption is withdrawn (A2). See
  ADR-0020's open question on navigation grouping.
- **OQ-2 (deferred to a future release; owner `@planner` for triage)** — Should
  `/api/docs/<slug>` accept `?status=<value>`? Not required by REQ-06.014
  ("list view"); recorded for a later API increment. No implementation in
  TASK-114.
- **OQ-3 (owner `@architect` for confirmation)** — Invalid filter values return
  `400` per NFR-02.002 ("invalid filter values rejected"), which is the current
  behavior. The alternative `404` would conflate a bad filter with a missing
  type; the specification keeps `400` for filters and `404` for unknown or
  disabled types.

## Traceability

- REQ-06.014 — Display config-defined document types in the browser.
- REQ-06.003 — Describe managed artifacts and their lifecycles.
- REQ-06.004 — Front-matter metadata projection in CLI and browser views.
- REQ-06.012 — Single record view layout.
- REQ-05.007 — Archive state, `archived/` only, auto-relocation, archived
  invisibility.
- REQ-05.011 — `status` is not editable.
- REQ-07.001 / REQ-07.002 / REQ-07.004 — three categories, configured types,
  generic content model.
- NFR-02.002 — invalid filter values are rejected.
- ADR-0020 — registry-derived route table, two view families, canonical API,
  one 404, generated lifecycle page.
- ADR-0017 — artifact registry and descriptors (single type model).
- ADR-0022 — document lifecycle state and archive location.
- ADR-0005 / ADR-0006 / ADR-0013 — transport, atomic writes, and the shared
  record layer.
- ADR-0018 — generic document content model.
- TASK-105 (registry), TASK-107 (record layer), TASK-110 (generic handler),
  TASK-113 (search) — completed dependencies.
- Project-owner decision 2026-11-14 — sidebar grouped by the three artifact
  categories; document list columns ID, Title, Status, family slot(s), Created, Updated, Actions;
  archived records never listed, reachable, or returned by the API. Withdraws
  the earlier flat-sidebar assumption and resolves OQ-1.
