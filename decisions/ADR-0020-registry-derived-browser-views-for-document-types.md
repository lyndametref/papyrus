---
id: ADR-0020
title: "Registry-Derived Browser Views for Config-Defined Document Types"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-26T18:24:52Z'
labels: []
references: []
---
# Registry-Derived Browser Views for Config-Defined Document Types

## Status
deprecated
## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-006.014 requires the local web interface to serve list, detail, and edit views for **every enabled document type declared in `document_types`**, not only the types shipped with Papyrus. A type added by configuration must become available with no code change, using the same navigation, list, detail, and edit behavior as the default types; views use the type's configured folder and prefix, the metadata projection of REQ-006.004, and the layout of REQ-006.012. Archived records are hidden and unreachable and return the same page as an unknown address (REQ-005.007); a disabled type has no views and its records are neither listed nor reachable. Every listed record shows its `status` as stored — including a custom status declared for the type — and the list view offers a status filter whose options are exactly `active` plus the type's declared custom statuses in configured order, with `archived` never offered because archived records are not listed (REQ-006.014). `status` is not editable in the browser (REQ-005.011). REQ-006.003 additionally requires the lifecycle page to describe every document type declared in configuration, enabled or not, grouped by the three artifact categories.

Current code shape, which is entirely closed against new types:

- `src/browser/records.rs` defines `enum Category` with fifteen variants, each with `key()`, `config_key()`, and `descriptor_name()`, plus `parse_*`/`form_for_stem` branches. `record_directory` resolves a type's folder from `descriptor_for(name)`, which returns `None` for any slug that is not one of the eleven compiled document descriptors, and does not consult `enabled` for non-mockup types, so a disabled type is still browsable today.
- `src/browser/mod.rs` routes with a hand-written chain: one `if path == "/api/<plural>"` block per built-in type, `/?type=<key>` for list pages through `category_from_query` (unknown value → `400`), and one universal detail route `/doc/{id}` plus `/doc/{id}/edit`. Per-type "not found" pages are selected by `matches_doc_type_id` for four tech-spec/use-case/test-case/test-report keys.
- `src/browser/render.rs` is 4.4k lines with per-category page titles, sidebar sections, list pages, detail pages, and ten `apply_*_edit`-style form handlers in `records.rs`; `build_sidebar` hard-codes the three requirement entries, the backlog group, and a fixed tuple list of document types.
- `src/browser/templates/` holds the shared page fragments; there is no generic document template.
- ADR-0005 fixes the transport, the "route against a fixed route table after single-pass percent-decoding", the rule that request paths are never joined to filesystem paths, exact-match ID resolution, the security headers, and the embedded-template renderer. ADR-0006 fixes optimistic concurrency and atomic writes for browser saves. ADR-0013 decision 3 requires the browser to use `record::document` + `record::patch` + `record::identity`.

Forces: no new dependency, template engine, or asset (ADR-0005); no per-type code for config-defined types (REQ-006.014); identical 404 for archived and unknown (REQ-005.007); the existing CLI metadata projection must be the same one the browser renders (REQ-006.004); existing built-in pages must not regress; and the route table must stay auditable and traversal-safe.

## Decision
1. **The registry (ADR-0017) is the only source of the browser's type set.** Navigation, list pages, detail routing, edit forms, the lifecycle page, and the JSON API iterate `ArtifactRegistry`: `enabled(Document)` for navigable views, `all(Document)` for the lifecycle and configuration pages. No module may hold a second list of document-type keys.

   *Revision (2026-11-20, amended by ADR-039): the descriptor is also the only source of a document type's presentation facts. The static presentation table may not hold a document entry, and the document list, view, edit, conflict, and not-found surfaces are one descriptor-driven path. Backlog and decision entries may remain because those surfaces are bespoke.*

2. **The route table is assembled from the registry; its semantics are unchanged from ADR-0005.** The table is fixed for the lifetime of one serving snapshot (ADR-037), matched by exact string comparison after single-pass percent-decoding, never joined to a filesystem path, and served through the same handler/security/chrome pipeline. What changes is only that the table's document entries are produced by iterating the registry instead of being written by hand. The universal `/doc/{id}` and `/doc/{id}/edit` routes are kept: detail addressing stays ID-based, and the ID's owner is resolved by prefix through `registry::resolve_id`.

   *Revision (2026-11-20, amended by ADR-037): this decision previously said the table is assembled once at startup and is fixed for the process lifetime. ADR-037 derives the table in the same atomic rebuild as the configuration and registry, so it is fixed for the lifetime of one serving snapshot; a configuration change that alters the type set produces a new table with the new snapshot and never a partially updated one. Matching, path rules, the handler and security pipeline, and the universal routes are unchanged.*

3. **One canonical list route and one canonical API route per type, plus the existing built-in aliases.** `/?type=<slug>` remains the list route for every type. The JSON API gains one canonical endpoint `/api/docs/<slug>` for every type. The existing `/api/requirements`, `/api/glossary`, `/api/tech-specs`, and the other built-in endpoint names remain as aliases so no existing consumer breaks; new code must use the canonical form. An unknown or disabled `type`/`<slug>` is `404`, not `400`: a type that has no views is indistinguishable from an address that does not exist, which is the same rule REQ-005.007 applies to archived records. Changing the current `400` is a deliberate, documented status-code change.

   *Revision (2026-11-20, amended by ADR-039): the canonical routes, exact matching, the universal document routes, and the 404 rule are unchanged. The legacy alias set and the legacy `?type=` key are seed data (`api_aliases`, `type_alias`) rather than a static table, and one document handler serves the route of every document type.*

4. **`Category` is deleted; `Form` becomes a descriptor property.** `ArtifactCategory` (`Backlog | Decision | Document`) comes from the registry. The requirement/group distinction that `Category::Req`/`Category::Nfr` and `Form` encode today moves into the `req`/`nfr` descriptors as `form: Requirement | Group` and `id_shape: Grouped` (group records plus dotted member records); `Dotted` remains the member-only shape. This agrees with ADR-0018 decision 5 and TASK-105 encoding decision D1, so the list page still renders groups and requirements separately without branching on a slug.

5. **Two generic view families cover every document type; typed types keep field-specific forms.**
   - Generic (`content_model = Generic`, ADR-0018): one `document_list.html`, one `document_detail.html`, one `document_edit.html` rendering title, description, labels, and references from the shared `RecordView`. The list and detail views render `status` read-only for every document type; the edit form never offers a status field. The list view's status filter is built from the descriptor's status model (ADR-0017): `active` plus the declared custom statuses in configured order, with no `archived` option.
   - Typed built-ins: keep their existing field sets and forms, but their handlers are selected by descriptor, not by a `match` on the id's prefix or on a category enum.
   - Presentation strings (page titles, curated descriptions, field labels) are looked up by slug from a static built-in table; a config-defined slug gets a title-cased label from its slug and the shared document description required by REQ-006.003.

   *Superseded (2026-11-20, by ADR-039): this decision is replaced as a whole. There is one descriptor-driven document surface for every document type, not two generic view families with typed forms; the field-specific forms of the typed built-ins are deleted rather than kept, and presentation strings come from the descriptor's seed entry, not from a static table keyed by slug.*

6. **Status filtering is a validated projection, never a free-form query.** A list request's `status` filter accepts only `active` or a custom status declared for that type; any other value (including `archived` and an undeclared value) is rejected rather than applied as an unfiltered or guessed match, and a type with no custom statuses offers only `active`. Filtering compares the stored front-matter value exactly, so a record whose value is no longer declared remains visible under `active`-only listing rules but is never silently rewritten.

7. **Indexing is enabled-only and active-only.** `build_index` receives a descriptor and returns an empty index when the descriptor is disabled; it reads only the descriptor's active directory, never `archived/`, and claims files by the descriptor's prefix and id shape. A disabled type therefore has no views and its files are neither listed nor reachable, satisfying REQ-006.014 without a separate check in every handler.

8. **Archived and unknown share one 404.** Because `archived/` is never indexed and is never a resolvable directory, `view`/`edit` of an archived ID and of an unknown ID render the same response for their type. The per-type not-found pages are reduced to one generic document not-found page plus the existing built-in pages that a requirement names.

9. **The lifecycle page is generated, not enumerated.** REQ-006.003's page renders three category groups; backlog and decision sections are built from their descriptors; one section per `document_types` entry (enabled and disabled) shows the main statuses `active -> archived`, any custom statuses the entry declares as front-matter values that do not change location (UI mockups `pending -> approved -> archived`), the entry's effective `enabled`, prefix, and folder, and curated text for built-ins or the shared generic text otherwise. Configuration values remain read-only there.

10. **Writes keep the ADR-0006 and ADR-0013 contracts unchanged.** A browser save for a config-defined type parses and patches through `record::document`/`record::patch`, writes through `record::store::write_atomic`, carries the version token, returns `409` on a stale token, and refreshes `updated_date` once per write.

11. **No new dependency, asset, or template engine.** The generic templates use the existing `{{name}}` single-pass renderer and escaping, are embedded with `include_str!`, and are served from the same binary (ADR-0005).

### Amendment (2026-11-20) — ADR-039

- **Amended:** decision 1 (the descriptor is the only source of a document type's presentation facts) and decision 3 (seed-declared aliases and one document handler).
- **Superseded:** decision 5, and the rejected alternative "Server-side rendering from a data-driven generic page for **all** types, dropping typed forms". The alternative is no longer rejected.
- **Not changed:** decisions 2, 4, and 6 to 11, every downstream constraint, the route semantics, the security and write contracts, and this record's accepted status.

### Clarification (2026-11-23) — the seed table is the only production naming of document slugs

- **Decision 1.** `src/init.rs` holds `DOCUMENT_ARTIFACTS`, an 11-slug literal list of the built-in document types (`req`, `nfr`, `glossary`, `rule`, `ui-spec`, `ui-mockup`, `audit`, `tech-spec`, `use-case`, `test-case`, `test-report`). It is a second production naming of document slugs and is covered by decision 1's rule that no module may hold a second list of document-type keys; ADR-036 decision 28 makes the seed table the only definition of built-in document types. The interactive-init selectable document list must derive its slugs from `SEED_DOCUMENT_TYPES` in `src/config.rs`, in seed order. No exception is recorded. Tests and documentation remain exempt.
- **Not changed:** decisions 2-11, every downstream constraint, the route semantics, the security and write contracts, and this record's accepted status.
## Consequences

**Positive:**

- REQ-006.014 becomes structural: an enabled `document_types` entry gains list, detail, edit, navigation, and API access with no code change.
- Deleting `Category` and the per-type not-found chains removes a large class of "forgot to add the new type here" defects.
- The browser and CLI consume the same registry and the same `RecordView`, so REQ-006.004's "same metadata values" stays structural.
- Disabled and archived visibility rules are enforced in one place (index construction) rather than in each handler.

**Negative:**

- The route table is no longer readable as a literal list in `mod.rs`; auditing it requires reading the registry builder.
- The `400 → 404` change for unknown `?type=` values is a small behaviour change for any external consumer that relied on the status code.
- Generic document pages are plainer than built-in ones (no type-specific sections); this is the intended consequence of REQ-007.004's generic model.
- The status filter is a second per-type surface: it must be built from the descriptor rather than a template literal, or a type could offer a filter value it does not declare.
- `status` being read-only in the browser while a CLI `status` verb exists is intentional but asymmetric; the browser stays a metadata viewer for lifecycle fields.
- Refactoring 4.4k lines of `render.rs` carries regression risk for pages that have nothing to do with config-defined types.

**Risks and mitigations:**

- *Regression in existing pages*: characterize each built-in page (status, headings, form fields) before refactoring, then produce the same output through the descriptor path; keep the per-type output tests.
- *Path safety regression*: routing still never joins a path; the descriptor's folder comes from configuration and is re-checked with `Context::ensure_within`; a test asserts a `directory` that escapes the project cannot render a page.
- *Duplicate/ambiguous slugs*: impossible after ADR-0019 validation; the registry builder asserts it.
- *Edit-form drift*: one conformance test runs create → list → detail → edit → archive for a generic type and asserts the record file, the metadata projection, and the archived visibility.
- *Status filter drift*: a test per document type asserts that the offered filter values equal `active` plus the declared custom statuses, that `archived` is absent, and that an out-of-set value is rejected.
- *Status value edited in the browser by mistake*: the edit form has no status control, and a save that carried one would be rejected as an unknown field rather than written.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Registry-derived route table and two generic view families (this ADR) | Selected | The only way to satisfy REQ-006.014 with no per-type code while keeping ADR-0005's transport and security decisions. |
| Keep the static route table and add a `Generic(String)` category | Rejected | Still one navigation entry, one list path, and one form per type; the browser set would drift from the CLI set. |
| Serve config-defined types only through the JSON API, not HTML | Rejected | REQ-006.014 requires list, detail, and edit views. |
| Server-side rendering from a data-driven generic page for **all** types, dropping typed forms | Rejected | Would regress the built-in field sets that REQ-005.x and REQ-006.004 require. |
| Client-side routing/JS for dynamic types | Rejected | Violates the no-JS-required posture of ADR-0005 and adds an asset. |
| Generate one template per type at startup | Rejected | Runtime template synthesis outside the fixed renderer, with no benefit over one generic template. |

## Open Questions

- Is the browser navigation grouped by the three categories, with document types flat inside the document group (current decision), or fully flat? REQ-006.014 leaves this open.
- Should an unknown `?type=` be `404` (current decision) or keep the current `400`?
- Should a config-defined type be able to declare a display name or icon for navigation? Current requirement defines none.
- Should the canonical API form be `/api/docs/<slug>` (current decision) or `/api/<slug>`? The former avoids colliding with built-in endpoint names.

## Downstream constraints

1. `browser::records::Category` must not survive this ADR; no handler may branch on a document slug.
2. List and detail views must render from `RecordView`; a browser-only metadata projection is prohibited (REQ-006.004, ADR-0013 decision 5).
3. `archived/` must never be indexed or resolvable; disabled types must produce no navigation entry, no list, no detail, no edit, and no API response.
4. `status` is displayed but never edited in the browser; no form field and no save path may write it.
5. Routing keeps ADR-0005's exact-match, no-path-join, single-request-per-connection model; no new asset or dependency is introduced.
6. Built-in page output must be preserved; the refactor is judged by characterization tests, not by visual similarity.

## References

- `docs/requirements/business/REQ-006.014.md` — browser views for config-defined types.
- `docs/requirements/business/REQ-006.003.md` — lifecycle/artifact description page.
- `docs/requirements/business/REQ-006.004.md`, `REQ-006.012.md` — metadata projection and record layout.
- `docs/requirements/business/REQ-005.007.md` — archived visibility and 404 equivalence.
- `docs/requirements/business/REQ-007.001.md`, `REQ-007.002.md` — main status, custom statuses, status-list configuration.
- `docs/requirements/business/REQ-005.011.md` — `status` exemption from the option rule.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — server, routes, templates, security posture.
- `docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md` — version tokens and atomic writes.
- `docs/architecture/adr/ADR-0013-shared-record-envelope-without-a-storage-layer.md` — shared reader/patcher.
- `docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md` — the registry consumed here.
- `docs/architecture/adr/ADR-0018-generic-document-content-model.md` — generic field set.
- Implementation: `src/browser/mod.rs`, `src/browser/records.rs`, `src/browser/render.rs`, `src/browser/templates/`, `src/registry.rs` (new).
