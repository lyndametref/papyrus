---
id: ADR-039
title: "One descriptor-driven document view and edit surface"
created_date: '2026-11-20T19:58:18Z'
updated_date: '2026-11-22T08:55:32Z'
labels: []
references:
  - RE-010
  - ADR-0020
  - ADR-0028
  - ADR-0033
  - ADR-0035
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0005
  - ADR-0006
  - ADR-0013
  - ADR-0034
  - REQ-006.004
  - REQ-006.012
  - REQ-006.014
  - REQ-006.019
  - REQ-005.007
  - REQ-007.004
  - NFR-001.003
  - TECH-066
  - TECH-067
  - TECH-068
  - TECH-074
  - TECH-076
  - TECH-079
superseded_by: null
---
# ADR-039 One descriptor-driven document view and edit surface

## Status
accepted
## Decision
### Supersession and amendments

1. This ADR supersedes ADR-0020 decision 5 ("Two generic view families cover every document type; typed types keep field-specific forms") and only that decision. It also supersedes the alternative that ADR-0020 rejected under "Server-side rendering from a data-driven generic page for **all** types, dropping typed forms"; that alternative is no longer rejected. ADR-0020 keeps its accepted status because the supersession is by decision, not in full.

2. **Dated amendment to ADR-0020 decision 1 (2026-11-20).** The registry remains the only source of the browser's type set, and the descriptor now also owns every presentation fact of a document type. `browser::routes::PRESENTATION` may not hold a document entry; navigation, list, view, edit, conflict, and not-found for documents are produced by one document surface whose only variation is descriptor data. The backlog and decision entries of the table may remain, because those surfaces stay bespoke.

3. **Dated amendment to ADR-0020 decision 3 (2026-11-20).** Routing is unchanged: `/?type=<slug>`, `/api/docs/<slug>`, the universal `/doc/{id}` and `/doc/{id}/edit`, exact matching, and `404` for an unknown or disabled type. What changes is that the legacy API aliases and legacy `?type=` keys are seed data (`api_aliases`, `type_alias`) rather than a static table, and every document route is served by one document handler instead of a per-type handler.

4. **Dated amendment to ADR-0028 decision 1 (2026-11-20).** The template set stays `shell.html`, `list.html`, `view.html`, `edit.html`, and `error.html`. `document_detail.html` and `document_edit.html` are deleted. `view.html` and `edit.html` are single descriptor-driven templates for every document model: this ADR supersedes the clause that "`view.html`/`edit.html` keep their typed field sets" and the statement that collapsing detail/edit forms is out of scope. The conflict page renders through `edit.html` with a conflict notice, and the not-found page renders through `view.html`; neither gains a template.

5. **Dated amendment to ADR-0028 decision 2 (2026-11-20).** The shared list layout and its fixed column order are unchanged. Family slot labels and presence come from the descriptor's seed entry alone, not from "the descriptor plus the presentation table (ADR-0020)". For a document type no field becomes a list column: document slots are declared seed slots, and the seed declares none. Reference rendering in a row is decided by the descriptor's seed `reference_visibility` (ADR-0035 as amended here), not by the content model.

6. **Dated amendment to ADR-0028 downstream constraint 10 (2026-11-20).** The structural column set stays `id`, `title`, `status`, family slot(s), `created`, `updated`, `actions`. A slot key may not be a structural key (`id`, `title`, `status`, `created`, `updated`, `actions`) and now may not be any reserved field key of ADR-036 decision 23 (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, `group`) or repeat; the union of the two lists is the prohibited set. Slots are declared only in the seed entry, so a document field, a general metadata key, or a handler cannot introduce a column.

7. **Dated amendments to ADR-0033 (2026-11-20).** Decision 1 now reads: one shared rendering path produces the detail view for every document record type, and the artifact content comes from the descriptor field map. Decision 2 keeps the fixed head and the reference, dependency, and dependent disclosures; the artifact content between them is the field sections of decision 19 below. Decision 6 consumes the shared `RecordView` projection **and** the field map. Decision 7 now includes `SingleHtmlMockup`, whose view uses the same head and chrome with the fixed metadata block instead of a field sequence. Decisions 3, 4, 5, and 8 stand unchanged.

8. **Dated amendment to ADR-0035 (2026-11-20).** Its discriminator is replaced. Reference visibility is seed data per type and, for a grouped type, per role: `reference_visibility` resolves to `Visible` or `Hidden` from the descriptor, and a list row renders its reference list only when the value for that row's type and role is `Visible`. The rule keyed on `ContentModel::Generic` no longer exists. Everything else in ADR-0035 stands: labels render for every family, the single-document view keeps its References disclosure, and CLI list, CLI view, JSON, record files, and the shared list contract are unchanged.

9. Nothing else is superseded or amended. ADR-0005 (transport, routes, security, no cookies), ADR-0006 (optimistic concurrency, atomic writes), ADR-0009 and ADR-0013 (record envelope and single write path), ADR-0022 (lifecycle), ADR-0034 (removed browser sections), ADR-0036 (descriptor model and field schema), ADR-0037 (serving snapshot), and ADR-0038 (asset caching) remain in force unchanged.

### One document surface

10. The browser has exactly three category handlers: `Backlog`, `Decision`, and `Document`. Backlog and decision records keep their bespoke board, list, view, edit, and conflict code. Every document record type is served by one document surface for list, view, edit, conflict, not-found, and form submission. "Document record type" means every descriptor in the `Document` category under any `DocumentModel`, including seed types and user-defined types.

11. The surface is fixed and exhaustive:

    | Surface | Route or trigger | One renderer or handler |
    |---|---|---|
    | List | `GET /?type=<slug>`, `GET /api/docs/<slug>` | `ListPlan::for_descriptor` for the HTML list; the existing JSON list projection |
    | View | `GET /doc/{id}` | one document view builder |
    | Edit | `GET /doc/{id}/edit` | one document edit builder |
    | Submit | `POST /doc/{id}` | one `form_post` |
    | Conflict | `409` response of a submit with a stale version token | one conflict builder, rendering the edit form with the submitted values |
    | Not found | any document `GET` whose ID does not resolve | one not-found builder |
    | Group create | `GET`/`POST` for a grouped type's group form | the same edit builder and `form_post` with the group role |

12. The browser document path contains no per-type dispatch and no slug-name matching. It may not contain a `match` on a slug, a `TypedHandler` variant, a `ContentModel` variant, or a document-type lookup table. The only variation comes from the descriptor and the field map of decision 14. Backlog and decision handlers may keep their `TypedHandler` dispatch because they are bespoke and unchanged.

13. The following are deleted with the refactor: `browser::records::TypedHandler` use for documents, the per-type document content fields on `Record` (see decision 17), `routes::NotFoundPage` and the per-type not-found page selection for documents, the `PRESENTATION` document entries, `routes::list_slots(slug)` for documents, and the per-document edit, conflict, and detail renderers.

### Record envelope and field map

14. The loaded document record carries one model-neutral field map in addition to the existing envelope (`id`, `title`, `status`, `labels`, `references`, `created_date`, `updated_date`, body, version, path):

    ```
    DocumentFieldMap {
        role: FieldRole,                 // Single | Group | Member
        fields: Vec<ResolvedField>,      // descriptor sequence order
        fixed_metadata: Vec<(String, String)>, // SingleHtmlMockup only
        extra_metadata: Vec<(String, String)>, // undeclared stored keys, read-only
        reference_visibility: Visibility,      // resolved for this role
    }
    ResolvedField {
        key: String, label: String, help: Option<String>,
        kind: FieldKind,                 // text | multiline | list
        required: bool, editable: bool,
        value: FieldValue,               // Text(String) | Items(Vec<String>) | Absent
    }
    ```

15. The map is built once per loaded record by the record layer from the descriptor: `SingleMarkdown` uses the top-level `fields` sequence with role `Single`; `GroupedMarkdown` uses `group.fields` with role `Group` for a group record and `member.fields` with role `Member` for a member record; `SingleHtmlMockup` has no field sequence (`fields: []`) and carries `fixed_metadata`. No second field table, per-type schema, or browser-side field list exists.

16. Value resolution is deterministic. A field with only `heading` reads the body section with that heading; a field with only `frontmatter_key` reads that front-matter key; a field with both reads the body section as the display value and does not merge the two. On write both locations are set from the same submitted value. A divergence already present on disk is left as stored and is not repaired or merged by the browser. An absent optional value is `Absent`; an absent required value is `Absent` and is flagged on the edit form.

17. No browser document code reads a typed content field. The content-model fields that `Record` carries today (`story`, `description`, `rationale`, `applicability`, `requirements`, `audit_type`, `definition`, `view`, `specs`, `mockup_task`, and the equivalent) are not consulted for a document record; the field map is the only content projection. Backlog and decision records keep the fields their bespoke surfaces use.

18. `labels`, `references`, `created_date`, and `updated_date` are general record metadata, not fields. `title` is fixed and is not a field. `status` is reserved and read-only. These exclusions are the same for every document model (ADR-0036 decisions 23 and 24). Undeclared front-matter keys that already exist on disk are rendered read-only in `extra_metadata`, so a hand-added key is not hidden and CLI/browser metadata parity (REQ-006.004) is not lost.

### Rendering rules

19. **View.** The view is the ADR-0033 head and reference, dependency, and dependent disclosures, with the artifact content built from the field map: one `<section>` per non-empty field, in descriptor sequence order, its visible heading taken from the field `label`, and its value rendered through the existing prose pipeline (escaping, math, diagrams) or as a list for `kind = list`. A required field that is empty is rendered as an empty section, not omitted. `extra_metadata` follows the field sections as read-only metadata rows. There is no per-type summary card, no type-specific metadata block, and no repetition of the document type (ADR-0033).

20. **Role-aware rendering.** A `GroupedMarkdown` group record renders the `group.fields` sections, then an engine-owned, read-only member list (member ID, title, link, in ID order). A member record renders the `member.fields` sections and a read-only link to its group. Membership is never a form control and never a field (ADR-0036 decision 9). The list renders the group row and its member rows through the shared hierarchical row variant (ADR-0028 decision 2); role is a property of the row, not a second table.

21. **Fixed metadata for `SingleHtmlMockup`.** The mockup view uses the same head, chrome, and reference disclosure as every other document, and a fixed metadata block instead of field sections: `status` (`pending`, `approved`, `archived`), `task`, `specs`, `references`, and `superseded_by`, all read from the HTML metadata comment. The mockup edit form offers title and the fixed metadata controls only; it has no field sequence and no `status` control. Supersede remains a separate verb, not a form field. The HTML body is rendered under the same rules as today.

22. **Edit.** There is one form builder. It renders, in order: the fixed `title` control; one control per `ResolvedField` whose `editable` is true, in descriptor sequence order, with control type from `kind`, the `label` as the control label, `help` as the hint text bound with `aria-describedby`, and the required marker from `required`; the general `labels` and `references` controls; read-only `status`, `created_date`, and `updated_date`; the hidden version token. A field whose `verbs` exclude `edit` is rendered read-only or omitted from the form; it is never writable. `status` is never a control (ADR-0020 downstream constraint 4).

23. **Conflict.** One conflict builder renders the same edit form with the submitted values, the current version token, and a conflict notice, and returns `409`. Nothing is written. Submitting the form again is a deliberate user action against the current token, exactly as today. The conflict page is not a route.

24. **Not found.** One not-found builder returns `404` for every document type, never echoes the requested ID, and is the same page for an unknown ID, an archived record, and a disabled type (ADR-0020 decision 8, REQ-005.007). Backlog and decision not-found pages are unchanged.

25. **List.** The list is the shared layout of ADR-0028 with its structural columns and declared slots. A document descriptor's slots come from its seed entry (currently none), the heading, count noun, empty state, and inventory note come from the descriptor's seed presentation facts, and the status filter is the validated projection of ADR-0020 decision 6. Grouped document types render the group row plus member rows in merged order. A row's reference list is rendered when its resolved `reference_visibility` is `Visible`.

### One form_post

26. One handler serves `POST /doc/{id}` for every document type and both grouped roles. Its order is fixed:

    1. Reject a media type other than `application/x-www-form-urlencoded` with `415`.
    2. Parse the form; read `version`. A missing or malformed version token is `400`.
    3. Resolve the ID against the request's index; an unknown, archived, or disabled ID is the one not-found page with `404`.
    4. Read the live record for its version token (ADR-0006, ADR-0037 decision 24).
    5. Compare the submitted token with the live token. A mismatch renders the one conflict page with `409`, with the submitted values and the live token, and writes nothing.
    6. Build the field map for the record's model and role. A submitted key that is not `title`, `labels`, `references`, or a declared field key of that map is rejected as `422`, including `status`; no write occurs.
    7. Validate every value and collect every error: the title contract (single line, non-empty, length limit), `kind` rules, and the required rule (a required field may be changed but not cleared; an optional field may be cleared). Any error re-renders the edit page with `422`, the submitted values, and the same token; no write occurs.
    8. Build the patch with the document engine (ADR-0036 decisions 35 to 37) and write once through `record::store`; `updated_date` is refreshed once. Answer `303` to `/doc/{id}`; the redirect target proves the save by re-reading the file.

27. The concurrency and write contract is unchanged. The token comes from the live bytes, never from a request snapshot; a stale token is `409`; the write is atomic; a validation failure writes nothing; and a successful save is proven by the fresh read at the redirect target (ADR-0006, ADR-0013, ADR-0037 decision 24).

28. The status-code surface is exactly `415`, `400`, `404`, `409`, `422`, `303`, and the existing `500` for an unexpected failure. No status is added, removed, or renumbered. Headers, CSP, security headers, and `Cache-Control: no-store` on HTML responses are unchanged (ADR-0005, ADR-0038). No route, endpoint, asset, cookie, client script, or JSON member is added.

### Presentation facts live in the seed

29. Every presentation fact of a document type is seed data: the descriptor `label`, `plural`, `order`, `summary`, `api_aliases`, `type_alias`, `slots`, and `reference_visibility`, and for each field its `label`, `help`, sequence position, `required`, `kind`, and `verbs`. The browser may format a value it is given (escape it, capitalise a fixed literal at the template level), but it may not hold a per-type string or a per-type list. This is RE-010 D10 applied to the browser.

30. Concretely: the list heading, count noun, empty state, inventory note, page title, navigation label, sidebar entry, API alias, legacy `?type=` key, form label, form hint, view section heading, and required marker are all derived from the descriptor. The visible view section heading is the field `label`; the storage `heading` (for example `## User Story`) is never rendered as a display heading. No document entry remains in `PRESENTATION`, and no `match` on a slug supplies a string.

31. Backlog and decision presentation may stay in the static table. Those surfaces are bespoke and their output must not change (RE-010 D4). The refactor's characterization tests assert that their pages are byte-identical to today's output.

### Reference visibility

32. Reference visibility is resolved from the descriptor: one `Visible` or `Hidden` value for `SingleMarkdown` and `SingleHtmlMockup`, and one value for each of the group and member roles for `GroupedMarkdown`. The seed values of ADR-036 decision 29 apply: `req` and `nfr` are `Hidden` for both roles; every other seed type is `Visible`. A user-defined type defaults to `Visible`. The list renderer reads the resolved value for the row's type and role; no other surface does.

33. Reference visibility affects list rows only. The single-document view keeps its References disclosure (ADR-0033), and the CLI list, CLI view, JSON list, JSON view, and stored front matter are unchanged (REQ-006.004, REQ-006.019, ADR-0035). Labels are unaffected and render in the Title cell for every family.

### What does not change

34. The transport, route matching, security headers, CSP, cache classes, `no-store` on HTML and JSON, the absence of cookies and sessions, the no-JavaScript posture, the single writer, the optimistic-concurrency token, the serving snapshot, and the absence of a new dependency, asset, or template engine are all unchanged (ADR-0005, ADR-0006, ADR-0013, ADR-0037, ADR-0038).

35. The JSON list and view projections, the CLI output, the record file bytes, and the on-disk layout are unchanged. The refactor changes how the browser renders a document, not what a document is or what any other surface emits (RE-010 D9).

36. This milestone makes no source, test, requirement, specification, or packaging change (MS-031 AC-10). This ADR fixes the contract the refactor milestone implements.

### Downstream constraints

1. One document list, view, edit, conflict, not-found, and submit path serves every document type; no per-type or per-slug branch exists in the browser document path.
2. Document content is read and written only through the field map and the descriptor; no typed content field and no second field table survives.
3. The field map is built once per record from the model and role; the browser never selects fields itself.
4. Every presentation string of a document type comes from the seed; no document entry remains in the static presentation table.
5. `title` is fixed, `status` is read-only, and `labels` and `references` are the general metadata block for every document model.
6. One `form_post` implements the order of decision 26 and the status-code surface of decision 28; ADR-0006 token semantics are preserved exactly.
7. Reference visibility comes from the seed per type and role; it affects list rows only; labels, the detail view, and every non-browser surface are unchanged.
8. Backlog and decision surfaces, including their templates, dispatch, and 404 pages, are byte-compatible with today's output.
9. Five templates remain, no new asset or dependency is added, and the no-JavaScript and no-store postures are unchanged.
10. No source, test, requirement, specification, or packaging file changes in MS-031.

### Clarification (2026-11-20) — fixed metadata keys, empty fields, and scope

- **Decision 11, "Group create":** there is no separate group route. A group record and a member record both use `/doc/{id}` and `/doc/{id}/edit`, with the role resolved from the record. A browser create form is out of scope for this ADR; if one is added, it reuses the same builder and role, and the submit handler's resolve, live-read, and version steps apply only after the record exists.
- **Decision 19:** an optional field whose value is `Absent` is omitted; a required field whose value is `Absent` renders as an empty section. The phrase "one section per non-empty field" is read with that exception.
- **Decision 21:** the mockup form's editable controls are the title and the two list-valued fixed metadata keys `specs` and `refs`; `status`, `task`, `references`, and `superseded_by` are read-only.
- **Decision 22:** a field whose `verbs` exclude `edit` is rendered read-only (label and value, no control) and is never writable; it is not omitted, so the view and the form show the same field sequence.
- **Decision 26 step 6:** the accepted keys are `title`, `labels`, `references`, a declared field key of the record's field map, and a fixed-metadata key the record's model offers (`specs` and `refs` for `SingleHtmlMockup`). Every other key, including `status`, is `422`.
- **Decision 29:** the presentation facts are the descriptor `label`, `plural`, `order`, `summary`, `api_aliases`, `type_alias`, `slots`, and `reference_visibility`, and each field's `label`, `help`, and sequence position. A field's `required`, `kind`, and `verbs` are data too, but they drive validation and control type rather than supplying a string.
- No other decision, downstream constraint, or status changes.

### Clarification (2026-11-22) — seed list slots (decisions 5, 6, and 25)

- **Decision 5:** the sentence "the seed declares none" is replaced in effect by ADR-041 decision 4. Every seed document type declares the exact slot set of that table, and seven of them declare an empty `slots` collection.
- **Decision 6:** `group` is a permitted slot key for a `GroupedMarkdown` seed type (`req`, `nfr`), where it renders the engine-owned group link. The rest of the reserved-key rule stands: `group` remains reserved and prohibited as a field key and a `frontmatter_key`, the other structural and reserved keys remain prohibited as slot keys, and a slot key may not repeat.
- **Decision 25:** "currently none" now reads: the slots are the seed sets of ADR-041 decision 4. Browser list columns are unchanged, and a slot key is not a field key (ADR-041 decision 7).
- No other decision, downstream constraint, or status changes.
## Context
RE-010 fixes the target architecture: three category handlers, and one schema-driven document engine with `DocumentModel` = `SingleMarkdown`, `GroupedMarkdown`, `SingleHtmlMockup`. ADR-036 gives every document type one descriptor, one seed table, config-declared field sequences, and one engine that emits only PatchOps. ADR-0036 settles the data model; this ADR settles the browser surface that consumes it.

The browser is not there yet. Document types are still rendered by per-type code:

- `src/browser/render.rs` (4,333 lines) has ten document edit renderers (`edit_page`, `edit_page_glossary`, `edit_page_ui_spec`, `edit_page_ui_mockup`, `edit_page_business_rule`, `edit_page_audit_record`, `edit_page_tech_spec`, `edit_page_use_case`, `edit_page_test_case`, `edit_page_test_report`), nine conflict renderers (`conflict_page`, `conflict_page_ui_mockup`, `conflict_page_business_rule`, `conflict_page_audit_record`, `conflict_page_tech_spec`, `conflict_page_use_case`, and the test-case and test-report wrappers around it, plus `document_conflict_page`), and five not-found pages (`tech_spec_not_found_page`, `use_case_not_found_page`, `test_case_not_found_page`, `test_report_not_found_page`, `document_not_found_page`). `view_page` is one function with a `match` on `TypedHandler` and eight separate document branch clusters; `document_detail_page` is a second, generic-only detail renderer.
- `src/browser/records.rs` (1,284 lines) holds a single `Record` struct with a union of every type's content fields (`story`, `description`, `rationale`, `applicability`, `requirements`, `audit_type`, `definition`, `view`, `specs`, `mockup_task`, and more), populated by a `match` on `TypedHandler` in `load_record`. `Record::handler()` selects behaviour from `ContentModel`.
- `src/browser/mod.rs` (4,448 lines) dispatches `GET` edit and `POST` by `TypedHandler`, and `form_post` contains a nine-branch conflict chain plus a long per-type validation and save chain.
- `src/browser/routes.rs` (903 lines) holds a static `PRESENTATION` table keyed by slug that supplies labels, headings, count nouns, inventory notes, legacy URL keys, API aliases, and not-found page selection for documents, plus `list_slots(slug)`.
- `src/browser/list.rs` already builds one shared `ListPlan`, but selects `show_references` from `ContentModel::Generic` (ADR-0035) and takes slots and labels from the static table.
- `src/browser/render.rs` has two document templates (`document_detail.html`, `document_edit.html`) next to the shared `view.html` and `edit.html`, and ADR-0028 decision 1 explicitly kept typed field sets for view and edit.

The result is the class of defect the refactor exists to remove: a new field, a new built-in type, a heading change, a label change, or a help-text change is a code edit in several places, and a document type can be correct in the CLI and wrong in the browser. ADR-0035 already had to qualify a list rule by content model because there was no per-type visibility data; RE-010 D8 replaces that with seed data.

Forces:

- One browser surface for every document model, including user-defined types, with no per-type code (RE-010 D4, D5, D10; REQ-006.014, REQ-007.004).
- Presentation facts in the config seed, so a label, heading, help text, or order change is a file edit (RE-010 D10).
- A model-neutral read projection: view and edit must not know whether a field lives in a body section or in front matter, and must not know the type (RE-010 D9, ADR-0013).
- Role awareness for `GroupedMarkdown`: group and member field sets differ, membership is engine-owned, and the merged list order is fixed (REQ-007.004, ADR-0036 decisions 9 and 27).
- `SingleHtmlMockup` has no field sequence and fixed metadata; its view and edit must fit the same chrome without a second template (ADR-0033, REQ-005.009).
- Reference visibility is seed data per type and role, not a content-model rule (RE-010 D8, ADR-0035).
- Optimistic concurrency, the atomic single write, and the existing error surface must not regress (ADR-0006, ADR-0013, REQ-005.007).
- Backlog and decision surfaces stay bespoke and byte-compatible (RE-010 D4).
- No new dependency, template engine, asset, client script, or JavaScript requirement (ADR-0005, ADR-0027, ADR-0038).
- No source, test, requirement, specification, or packaging change in this milestone (MS-031 AC-10).

Alternatives considered:

| Option | Outcome | Reason |
|---|---|---|
| One descriptor-driven document surface with a model-neutral field map, seed presentation facts, one conflict and one not-found renderer, and one `form_post` (this ADR) | Selected | Satisfies RE-010 D4, D5, D8, D9, and D10 with one rendering path and no per-type code, while keeping ADR-0005 and ADR-0006 intact. |
| Keep typed view/edit forms per built-in type and add the descriptor path only for user-defined types | Rejected | Exactly ADR-0020 decision 5 as superseded: built-ins would still need per-type modules, so the nine conflict renderers, five not-found pages, and ten edit renderers would survive, and a seed field change would still be a code edit. |
| Render a generic page from front matter only, ignoring body sections | Rejected | `heading` fields are body sections and required by the seed; ignoring them would drop `## User Story`, `## Definition`, and every other section from the browser. |
| Render from the body only, ignoring `frontmatter_key` fields | Rejected | Metadata-only fields (`audit_type` stored as `type`) and list fields have no section, and CLI/browser parity (REQ-006.004) would be lost. |
| Branch the renderer on `DocumentModel` | Rejected | It moves per-model code into the browser and leaves the browser responsible for role selection; the record layer can resolve model and role once and hand the renderer one field map. |
| Keep `PRESENTATION` as the label and heading source | Rejected | A second presentation source per document type contradicts RE-010 D10 and reintroduces the drift the refactor removes; presentation must come from the descriptor. |
| Keep the per-type not-found and conflict pages and merge only edit | Rejected | The task's equivalence goal and AC require one conflict page and one not-found page; leaving the variants keeps the per-type branches alive. |
| One generic `form_post` that ignores undeclared keys | Rejected | Fail-closed validation is the project posture (ADR-0019); silently dropping a submitted `status` or an unknown field hides a broken form, and the existing `422` surface already rejects `status`. |
| Add a per-type template or a template engine for variation | Rejected | ADR-0005 and ADR-0028 forbid a second template engine and per-family templates; the only variation is data. |
## Consequences
**Positive:**

- A document type cannot be correct in the CLI and wrong in the browser: one descriptor supplies fields, labels, order, help, presentation, and reference visibility to both, and one field map feeds view, edit, conflict, and submit.
- The per-type surface count collapses from ten edit renderers, nine conflict renderers, five not-found pages, and a multi-branch view into one of each. A seed field change, a label change, or a new document type is a configuration edit, not a code change.
- The browser no longer knows where a value is stored. A field that moves from a body heading to a front-matter key (or back) changes the seed, not the renderer.
- Role handling is resolved once per record, so `GroupedMarkdown` needs no separate list, view, edit, or conflict path, and membership cannot be edited through a field control.
- `SingleHtmlMockup` gains the uniform head and chrome without a second template, and keeps its fixed metadata contract.
- Reference visibility becomes data, so a type can change its list-row reference display without an ADR or a code branch, and the list rule stops depending on a content-model enum.
- The change is bounded by the existing transport and write contracts: no new dependency, asset, script, route, or header, and the concurrency and atomicity guarantees are untouched.

**Negative:**

- A generic document page is structurally uniform: a type with a bespoke editorial layout loses it. That is intended, but it is a visible change for built-in document types whose pages were hand-shaped.
- Presentation moves into the seed, so a copy change to a heading or help text is now a configuration/seed change with its own review path, not a local string edit in a renderer.
- The field map is a new intermediary: debugging a field requires reading the descriptor and the map rather than the renderer's struct literal.
- `extra_metadata` keeps undeclared front-matter keys visible, which preserves parity but means a hand-added key still appears on the view without an edit control.
- Deleting the typed fields from the document path is a broad mechanical refactor of `render.rs`, `records.rs`, and `mod.rs`, with regression risk on every document page.
- Seed data becomes the browser's presentation contract, so changing a seed label or order is no longer a harmless cosmetic edit; it is a user-visible change that this ADR governs.

**Risks and mitigations:**

- *A per-type branch survives behind a helper*: a contract test enumerates every enabled document descriptor and asserts that its view, edit, conflict, and not-found responses come from the one builder, and a static check rejects `TypedHandler`, `ContentModel`, and slug literals in the browser document path.
- *Field map and form drift apart*: both are built from the same `ResolvedField` list; a conformance test runs one type per model and role through list, view, edit, invalid submit, conflict, and save, and asserts the rendered controls equal the descriptor sequence.
- *Body/metadata divergence on read*: decision 16 fixes the precedence and forbids merging; a fixture with a divergent pair asserts the body value is displayed and the write sets both.
- *Role mix-up*: a grouped fixture asserts that a group renders `group.fields` and a member renders `member.fields`, that a member cannot post group-only keys, and that membership is absent from both forms.
- *Presentation drift*: a test asserts that every document label, heading, form label, help text, and empty-state string in the rendered output equals the seed value, and that no document slug appears in the presentation table.
- *Reference behaviour regression*: per-type assertions cover `Visible` and `Hidden`, both grouped roles, labels always rendered, and the detail view and CLI unchanged.
- *Backlog/decision regression*: characterization tests capture the board, task/milestone/release/decision lists, views, edits, and 404s before the refactor and replay them byte-for-byte afterwards.
- *Concurrency regression*: the existing 409 suite is kept and extended to a field-driven form; a stale token renders the one conflict page with the submitted values and writes nothing, and a valid submit answers 303 and is proven by the redirect target's fresh read.
- *Fail-closed form regression*: a test posts `status`, a group key on a member, an undeclared key, and an over-length value, and asserts 422 and no write for each.
- *Undeclared metadata hidden*: a fixture with a hand-added front-matter key asserts it is still displayed read-only.
