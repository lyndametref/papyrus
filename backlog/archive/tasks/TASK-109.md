---
id: TASK-109
title: Serve registry-derived browser views for every enabled document type
status: Done
assignee:
  - rust-dev
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-14T18:06:00Z'
labels:
  - rust
  - browser
milestone: MS-018
dependencies:
  - TASK-104
  - TASK-105
documentation:
  - >-
    docs/architecture/adr/ADR-0020-registry-derived-browser-views-for-document-types.md
  - docs/requirements/business/REQ-006.014.md
  - docs/specifications/browser-config-defined-document-types.md
  - >-
    docs/architecture/adr/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
  - docs/requirements/business/REQ-006.003.md
  - docs/requirements/business/REQ-005.007.md
modified_files:
  - src/browser/routes.rs
  - src/browser/mod.rs
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/templates/document_list.html
  - src/browser/templates/document_detail.html
  - src/browser/templates/document_edit.html
  - src/cli/dispatch.rs
  - tests/browser_config_types_cli.rs
  - tests/browser_cli.rs
  - docs/specifications/browser-config-defined-document-types.md
priority: high
ordinal: 41080
---


## Description
The browser still resolves document types through browser::records::Category and a hand-written per-type route/API chain, so a type added in `document_types` has no browser presence and a disabled type can still be browsed. This task makes the startup ArtifactRegistry the only source of the browser's document-type set and serves every enabled type through its descriptor, while preserving the built-in typed pages.

Scope: build one ArtifactRegistry and one route/navigation table at browser startup; generate a sidebar grouped by the three artifact categories (Backlog, Decisions, Documents), inventory cards, `?type=<slug>` list pages, `/api/docs/<slug>` endpoints, not-found selection, and the lifecycle page from the registry; delete Category and the per-type API functions; add generic list/detail/edit views (template plus handlers) for config-defined types; keep typed built-ins selected by ContentModel/TypedHandler; render status read-only in list/detail and never offer or write it in edit; use one shared document-list column order Status, ID, Title, Created, Updated (Created from front-matter `created_date`, as stored); add a validated list status filter (active plus declared custom statuses, configured order, never archived; invalid values 400); make archived/ unread and unindexed, return the same 404 for archived and unknown IDs across detail, edit, and API, exclude archived records from every list and API response, and give disabled types no views/API while still listing them on the lifecycle page with enabled: false.

Behavior contract: docs/specifications/browser-config-defined-document-types.md (route table, sidebar groups, view contracts, API alias table, visibility rules, test plan T-114-01..14).

Out of scope: CLI/command changes, registry model changes, record storage/patch changes, browser create/delete, status writes, API status filtering, a layout change to non-document lists (task board and milestone/release/decision lists beyond the already-scoped status behavior), and new dependencies or template engines.

Assumptions: the route table is a startup snapshot, so document_types changes need a browser restart (ADR-0020 decision 2); the sidebar is grouped by the three artifact categories, with built-in document entries in their existing relative order and config-defined entries under Documents after UI Mockups (2026-11-14 owner decision, replacing the earlier flat-sidebar assumption); the shared list column order Status, ID, Title, Created, Updated applies to every document list view, while the task board and milestone/release/decision list layouts are unchanged; the filter resting state is a `Show all` reset that omits `status` and lists every active-directory record so custom statuses never hide records; the select offers only the allowed status values (no `All records`/empty option), while an explicit empty `status=` remains 400; UI mockups filter by pending/approved; canonical API output equals the legacy body; browser label/reference editing saves the edited lists; a missing stored status shows `(none)` and is never rewritten; MOCK-0015's revised layout is approved (2026-11-14) and the specification remains authoritative on disagreement; the approved mockup's fourth `Project` utility heading is presentational only, and utilities follow the three category groups.

Open questions: OQ-2 (API `?status=` filter) is deferred to a future release, owner `@planner` for triage; OQ-3 (invalid filter 400 vs 404) is owned by `@architect` for confirmation, with current behavior 400 per NFR-002.002.

## Acceptance Criteria
- [x] One ArtifactRegistry and one BrowserRoutes table are built once at browser startup and stored in ServerState; document route/navigation entries, inventory cards, ?type list values, and /api/docs/<slug> endpoints derive from registry.enabled(Document); sidebar entries are grouped by the three artifact categories in the order Backlog, Decisions, Documents, with config-defined types under Documents and no entry for a disabled type; group membership comes from the descriptor category, not from the presentation table; src/browser/ keeps no second document-type registry and no Category or Form dispatch, and list/detail/edit/dispatch logic branches only on descriptor fields (ArtifactCategory, ContentModel, TypedHandler, IdShape, StatusModel), never on a slug literal; slug literals appear only in the static presentation table, which maps legacy ?type values and API path aliases, labels, nav order, and curated text and never decides type existence or which list/detail/edit behavior runs.
- [x] ?type=<slug> serves the list view for every enabled type in all three categories; the canonical slug and the legacy business-rule/audit-record aliases resolve to the same page; an unknown or disabled type value returns 404 instead of 400; generated built-in links and labels are unchanged, and within each sidebar group built-in entries keep their existing relative order; Configuration, Managed artifacts & lifecycles, and Agent skills & workflows follow the three groups (the approved mockup's fourth `Project` heading is presentational only).
- [x] A config-defined type gets list and detail views that use its configured folder and prefix; every document list row uses the column order Status, ID, Title, Created, Updated, with Status immediately left of ID; the title links to /doc/<ID>; Created renders front-matter created_date as stored ((none) when absent) and Updated renders updated_date; the detail page shows description, labels, references, status, created/updated, and the shared RecordView metadata.
- [x] Every built-in document type keeps its existing detail/edit field sets, sections, and actions, selected by ContentModel and TypedHandler rather than by slug or Category; its list view adopts only the shared row-field order Status, ID, Title, Created, Updated and the additive status filter (grouped requirement lists keep their grouping), and the task board and milestone/release/decision list layouts are unchanged.
- [x] List and detail views render the stored status read-only for every document type (front-matter value, or HTML metadata for ui-mockup), with the list Status field immediately left of ID; no edit form renders a status or status_change control; a generic document POST carrying either field returns 422 and writes nothing.
- [x] Every document list view offers exactly active plus the type's declared custom statuses in configured order (HtmlMetadata types offer pending/approved), never archived; a type with no custom statuses offers only active; the no-filter state lists every record in the active directory so custom-status records stay visible; selecting a value lists only records whose stored value matches it exactly.
- [x] A status value outside the offered set (including archived, an undeclared value, a wrong-case value, an explicit empty `status=` parameter, or duplicate status parameters) returns 400 with an error page and no listing (NFR-002.002); the no-filter state is an absent `status` parameter reached with `Show all`, and the filter select offers only the allowed values with no empty/`All records` option.
- [x] Archived records never appear: archived/ is never indexed or resolvable; a record in the active folder whose stored status is archived is auto-relocated and omitted; archived records are excluded from every document list and from /api/docs/<slug> and its aliases; an archived ID and an unknown ID of the same type return an identical 404 body for detail, edit, and /api/doc/<ID>; a disabled type has no sidebar entry in any group, no inventory card, no list, detail, edit, or canonical/legacy API response, while the lifecycle page still lists it with enabled: false.
- [x] The lifecycle page renders three category groups (Backlog, Decisions, Documents) and one registry-derived section per descriptor, including disabled document types; each document section shows active -> archived, declared custom statuses, enabled, prefix, folder, and curated or shared description; values are read-only with a /config link and no fixed section count remains.
- [x] Every enabled document type has GET|HEAD /api/docs/<slug> returning the same JSON body as its legacy endpoint (generic: id, kind, title, status, labels, references, version, path, metadata in an ID-sorted array); legacy paths remain aliases; an unknown or disabled slug returns 404; other methods return 405 with Allow; /api/doc/<ID> resolves a generic record and returns 404 for an archived or unknown ID.
- [x] The generic edit form offers only title, description, labels, and references; save patches front matter and the Description section through record::patch and writes once through record::store::write_atomic, preserves unknown keys/sections, never writes status or moves the file, refreshes updated_date only on change, performs no write when unchanged, and returns 409 with preserved values on a stale version.
- [x] Tests cover T-114-01..14: a user-added type end to end, canonical versus legacy API bodies, read-only status and status POST rejection, the Status/ID/Title/Created/Updated column order, filter values/order/default listing/rejection, archived-versus-unknown identical 404 for detail/edit/API and archived exclusion from lists and API, archived-in-active relocation, disabled invisibility, grouped sidebar, lifecycle groups, route/table invariants, path safety, and a characterized built-in page.

## Definition of Done
- [x] #1 cargo build succeeds.
- [x] #2 cargo fmt --check, cargo clippy -D warnings, and cargo test are clean.
- [x] #3 No enum Category, no Category:: dispatch, no second document-type registry, and no slug-literal branch in list/detail/edit/dispatch logic remain in src/browser/; slug literals occur only in the static presentation table (legacy URL aliases, labels, nav order, curated text), and behavior is selected by descriptor ContentModel/TypedHandler (grep/review check).
- [x] #4 A test asserts that sidebar, inventory, canonical API, and canonical ?type sets each equal registry.enabled(Document).
- [x] #5 A configured directory outside the project renders no page and no request path is joined to a filesystem path.
- [x] #6 The existing browser suite is green except the documented intentional changes (unknown type 404, grouped sidebar, shared list column order, lifecycle structure/count, additive status filter).
- [x] #7 Open questions and deferred items are recorded on the ticket with owners.
- [x] #8 Delivered behavior matches docs/specifications/browser-config-defined-document-types.md including the grouped sidebar and the Status/ID/Title/Created/Updated list columns; MOCK-0015's revised layout is approved by the owner (2026-11-14), and the specification remains authoritative on disagreement.


## Implementation Notes
1. Add src/browser/routes.rs: BrowserRoutes::build(&ArtifactRegistry) creates the canonical and legacy API path map, the `?type` value map (canonical slugs plus business-rule/audit-record aliases), the nav/inventory tables, and the ID prefix/shape rules for 404 selection. Keep the static built-in presentation table here (labels, headings, curated descriptions, nav order, legacy URL keys) and prove it never decides existence.
2. In src/cli/dispatch.rs and src/browser/mod.rs run(): pass the existing startup ArtifactRegistry by reference to browser::run (no second ArtifactRegistry::build); derive BrowserRoutes before binding; add Clone to ArtifactRegistry, store the registry clone and routes in ServerState; pass routes into route(). Replace category_from_query with type_from_query returning 404 for unknown/disabled and keep the virtual skills/lifecycle/config values. Replace the per-type API chain with a map lookup into api_doc_list(slug), serialized by ContentModel/TypedHandler with the existing shapes. Add the /config restart note.
3. In src/browser/records.rs: delete Category and Form. Record carries its descriptor (slug, category, content model, status model, prefix, id shape) and is_group. Replace descriptor_name/active_prefixes/uri_kind/parse_id_for/canonical_key/matches_stem/form_for_stem/record_directory with descriptor-driven directory (DirectorySpec + ensure_within), stem matching (prefix/id_shape), and form derivation. build_index(ctx, cfg, descriptor, filter) returns empty for disabled, reads only the active directory, uses the ContentModel extension, drops records auto-relocated out of the active directory, applies the exact status filter before sorting, and derives req/nfr groups after filtering. resolve_id(ctx, cfg, registry, id) iterates enabled descriptors and resolves in the owning active directory.
4. In src/browser/render.rs: replace every Category match with ContentModel/TypedHandler dispatch and keep the existing per-type page functions and markup. Generate sidebar and inventory from routes. Add document_list_page, document_detail_page, document_edit_page and document_conflict_page for ContentModel::Generic, one shared status_filter_html (Show all reset; the select lists only the allowed values) and status_text helper. Reduce 404 pages to the four named built-in pages plus one generic document not-found page with no ID echo. Rewrite lifecycle_page(routes, registry, cfg, chrome) into Backlog/Decisions/Documents groups with one registry-derived section per descriptor.
5. In src/browser/mod.rs handlers: list_page resolves the descriptor and dispatches task -> board, other backlog/decision -> existing list, document -> typed list or generic list (both filtered); view_page/edit_page dispatch by content model/handler; form_post adds save_generic_document (validate title/description/labels/references, patch title+heading, Description section, labels/references, write once through save_ops, no status write, 409 on stale version, 422 when status or status_change is submitted); handle_request maps 404s through routes.id_rules.
6. Add templates document_list.html, document_detail.html, document_edit.html with include_str!, reusing the existing placeholder shapes and single-pass escaping; leave the four existing templates untouched.
7. Tests: add tests/browser_config_types_cli.rs for T-114-01..14 (user-added note type end to end; canonical vs legacy API bodies; status read-only and status POST rejection; filter values/order/default listing/rejection; archived vs unknown identical 404; archived-in-active relocation; disabled invisibility with lifecycle still showing the type; generic edit round trip and 409; built-in characterization; lifecycle groups; route/table invariants and grep; method/path safety). Update tests/browser_cli.rs only for the documented changes (unknown type 400 -> 404, lifecycle structure and count, additive status filter).
8. Run cargo fmt, cargo clippy -D warnings, cargo test; record the spec link and MOCK id on the ticket.
9. UI revision (2026-11-14 owner request): render the sidebar grouped under Backlog, Decisions, and Documents headings (config-defined types under Documents after UI Mockups; disabled types absent; utility entries after the groups); render every document list with the shared Status, ID, Title, Created, Updated column order (Created from front-matter created_date, (none) when absent); keep the task board and milestone/release/decision list layouts unchanged; exclude archived records from every list and API response and return the identical 404 for archived and unknown IDs on detail, edit, and /api/doc/<ID>. Update tests/browser_cli.rs and the new tests for the grouped sidebar and column order. MOCK-0015 revision is approved (2026-11-14); the specification is authoritative on disagreement.


## Comments
author: @task-refiner
created: 2026-11-14T15:19:00Z

---

Refinement 2026-11-14: made the ticket self-contained and execution-ready against ADR-0020/0017/0022 and REQ-006.014/06.003. Added the behavior contract at docs/specifications/browser-config-defined-document-types.md with the startup route table, nav/inventory rules, typed-versus-generic dispatch, status filter semantics, API alias table, visibility rules, lifecycle page contract, and test plan T-114-01..14.

Resolved: unknown/disabled ?type is 404 (was 400, deliberate per ADR-0020 decision 3); the status filter resting state is an All reset (custom statuses never hide records per ADR-0022 decision 10); generated built-in links keep the approved ?type=business-rule / ?type=audit-record values; the flat approved sidebar is preserved and config entries append after UI Mockups; UI mockups filter by pending/approved; canonical /api/docs/<slug> returns the legacy body for the same type.

Non-blocking: OQ-1 sidebar category grouping (ADR open question; flat preserved), OQ-2 API status filter (deferred, not required by REQ-006.014), OQ-3 invalid filter returns 400 (confirm at review). Assumption A1: document_types changes require a browser restart because the route table is a startup snapshot; /config will state this.

Dependencies TASK-105/109/110/113 are complete. UI design delegated; MOCK id to be added to the spec and ticket when returned.

---

author: @task-refiner
created: 2026-11-14T15:26:00Z

---

UI design recorded: MOCK-0015 (docs/specifications/ui/mockups/MOCK-0015-browser-config-defined-document-types-review.html) with UI description docs/specifications/ui/browser-config-defined-document-types-ui.md. Draft, awaiting owner approval; covers list/filter/empty, detail, edit/validation/conflict, lifecycle delta, and sidebar insertion. Spec remains authoritative on disagreement. Corrections applied: exact built-in sidebar order/labels including Use Cases/Test Cases/Test Reports, optional description in edit, references as a collapsible details section.

---

author: @task-refiner
created: 2026-11-14T16:08:00Z

---

Refinement 2026-11-14 (owner UI revision): updated the behavior contract and AC1/AC2/AC3/AC4/AC5/AC8/AC10/AC12. Resolved OQ-1: the sidebar is now grouped by the three artifact categories (Backlog, Decisions, Documents), config-defined types appear under Documents, and disabled types have no entry; the earlier flat-sidebar assumption A2 is withdrawn. Document lists now use Status, ID, Title, Created, Updated (Status immediately left of ID; Created from front-matter created_date), applying to generic and typed built-in document lists; task board and milestone/release/decision list layouts are unchanged. Archived records are excluded from all lists and API responses, and detail/edit/API return the identical 404 for archived and unknown IDs. MOCK-0015's revised layout is pending re-approval; the specification remains authoritative. No new material questions; ticket remains execution-ready.

---

author: @task-refiner
created: 2026-11-14T16:56:00Z

---

Quality-review alignment 2026-11-14 (UTC): reworded AC1 and DoD3, and spec sections 8.1/8.3/6.6 and test T-114-13, so the no-slug-branch rule is testable and consistent with the approved design. The rule now forbids a second document-type registry and slug-based branching in list/detail/edit/dispatch logic, while explicitly allowing the static presentation table to map legacy ?type values and API path aliases, labels, nav order, and curated text, and allowing descriptor ContentModel/TypedHandler dispatch. Slug literals may appear only in the static presentation table. AC4 and the milestone/release/decision list layout remain unchanged.

---

author: @task-refiner
created: 2026-11-14T17:26:00Z

---

Bookkeeping 2026-11-14 (UTC): MOCK-0015 is approved (2026-11-14) per docs/specifications/ui/browser-config-defined-document-types-ui.md and ui/mockups/README.md; the pending re-approval wording in comments #2 and #3, the description, DoD 8, and plan step 9 is superseded by this comment and the updated sections. OQ-2 (API `?status=` filter) is deferred to a future release, owner @planner (triage). OQ-3 (invalid filter 400 vs 404) is owned by @architect for confirmation; current behavior is 400 per NFR-002.002. Filter form: the select has no `All records`/empty option; `Show all` is the no-filter reset (absent `status` parameter), while an explicit empty `status=` remains 400 (AC7, spec 3.2). The approved mockup's fourth `Project` utility heading is presentational only; utilities follow the three category groups.

---

author: @task-refiner
created: 2026-11-14T17:27:00Z

---

Correction 2026-11-14 (UTC): comment #1's 'All reset' filter resting state and 'OQ-3 ... confirm at review' wording are also superseded: the resting state is the 'Show all' link with no 'status' parameter, and OQ-3 is owned by @architect. Backlog.md comments are append-only, so this comment and #5 are the current record.

---

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Browser views are registry-derived: one startup ArtifactRegistry + BrowserRoutes, grouped sidebar (Backlog/Decisions/Documents), one list/detail/edit and /api/docs/<slug> per enabled type plus legacy aliases, lifecycle page covering all descriptors incl. disabled. Document lists use Status|ID|Title|Created|Updated with read-only status; filter offers active + declared custom statuses (pending/approved for mockups), never archived, with a disabled placeholder and Show all reset; explicit invalid/empty status= returns 400. Archived and unknown IDs return identical 404; archived records are never listed or reachable; disabled types are invisible outside the lifecycle page. Generic edit preserves labels/references and unknown keys, performs no write when unchanged, and rejects status fields with 422. Evidence: cargo test 773 passed / 0 failed / 22 ignored; browser_cli 165; browser_config_types_cli 19; clippy and fmt clean.
