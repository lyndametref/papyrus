---
id: TECH-068
title: "Descriptor-Driven Document Browser Surface"
created_date: '2026-11-16T17:46:54Z'
updated_date: '2026-11-21T07:44:31Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0013
  - ADR-0017
  - ADR-0019
  - ADR-0020
  - ADR-0022
  - ADR-0028
  - ADR-0033
  - ADR-0034
  - ADR-0035
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0039
  - MOCK-0015
  - NFR-01.003
  - NFR-02.002
  - REQ-05.007
  - REQ-05.011
  - REQ-06.004
  - REQ-06.012
  - REQ-06.014
  - REQ-06.016
  - REQ-06.018
  - REQ-06.019
  - REQ-07.001
  - REQ-07.002
  - REQ-07.004
status: verified
---
# TECH-068 Descriptor-Driven Document Browser Surface

## Purpose

The browser has exactly three category handlers: `Backlog`, `Decision`, and
`Document`. Backlog and decision records keep their bespoke board, list, view,
edit, and conflict code. Every document record type — seed or user-defined,
under any `DocumentModel` — is served by one document surface for list, view,
edit, conflict, not-found, and form submission.

The document surface has one list builder, one view builder, one edit builder,
one form submission handler, one conflict builder, and one not-found builder.
The browser document path contains no per-type dispatch, no slug matching, no
per-type handler variant, and no document entry in the static presentation
table. Its only variation comes from the descriptor and the field map built for
the record's model and role.

The browser serves from the immutable `ServingSnapshot` of ADR-037: one
validated configuration, one registry, and one route table, rebuilt atomically
when the configuration file changes. HTML and JSON responses stay
`Cache-Control: no-store` (ADR-0038).

## Scope

In scope:

- One startup registry and route table derived from it; one document handler for
  every document descriptor.
- Registry-derived navigation grouped by the three artifact categories,
  inventory cards, list pages, detail routing, edit forms, conflict, not-found,
  and JSON API endpoints.
- The shared list layout with its fixed columns and seed-declared slots.
- Model-neutral field-map rendering for every document model and role, including
  `GroupedMarkdown` group/member rendering and `SingleHtmlMockup` fixed metadata.
- Read-only status display and a validated status filter on document list views.
- Archived, unknown, and disabled equivalence.
- Unit and integration tests, including the three-category surface boundary.

Out of scope:

- Browser create or delete. The browser does not create records; delete stays
  CLI-only.
- Browser status writes. The browser never edits status.
- Status filtering on the JSON API and search.
- Changing the backlog or decision surfaces, their templates, or their output.
- Adding a dependency, template engine, client script, or asset.

## Behavior

### 1. Registry, snapshot, and route table

1. The browser uses one `ServingSnapshot` built after the listener binds: the
   effective configuration, the `ArtifactRegistry`, the `BrowserRoutes` table,
   and the input fingerprint. No handler calls `load_effective_config` per
   request; every configuration, descriptor, and route lookup in a request
   resolves through one snapshot handle.
2. `BrowserRoutes::build(&ArtifactRegistry)` produces the canonical list API
   routes, legacy API aliases, `?type=` values and HTML aliases, navigation
   items, inventory entries, and ID rules for enabled descriptors. The seed
   supplies `api_aliases`, `type_alias`, labels, order, and presentation facts;
   no static table supplies a document entry.
3. Routing is exact: a single-pass percent-decoded path compared against the
   table, with no path join, no prefix match, and no filesystem lookup during
   matching. An unknown or disabled type is `404`; a known route with a
   disallowed method is `405` with `Allow`.
4. A configuration change is picked up by the next request's fingerprint check
   and published as one new snapshot; pages, navigation, lists, and routes move
   together. A failed reload keeps the previous snapshot serving.
5. No request path is ever joined to a filesystem path. A descriptor's
   directory is resolved against the project directory and passes
   `Context::ensure_within` before any read.
6. The route table covers the fixed asset and document routes plus one canonical
   list API route per enabled descriptor and the seed-declared aliases. A
   disabled type has no navigation entry, inventory card, list route, detail
   route, or API route.

### 2. Navigation and list route

#### 2.1 Sidebar and inventory

1. The sidebar is generated from the route table and grouped by the three
   categories in the order `Backlog`, `Decisions`, `Documents`. Each group has
   one entry per enabled descriptor of that category; a disabled type has no
   entry.
2. The `Backlog` group keeps the existing board entry and the `Decisions` group
   keeps the existing decisions entry. Document entries follow in seed `order`
   with ties broken by slug; each label comes from the seed.
3. A selected entry carries `aria-current="page"` and the existing non-colour
   cue. The selected value is the descriptor slug; a legacy `?type=` value
   selects the same entry.
4. The inventory renders one card per enabled document type from the same route
   table. A card's title, note, and order come from the seed; a disabled type
   has no card.
5. The retired configuration, managed-artifacts, lifecycle, and agent-skills
   entries are absent; the sidebar has exactly the three category groups.

#### 2.2 List route and type aliases

1. `GET|HEAD /?type=<slug>` serves the list view for every enabled descriptor
   in every category. `GET /` without a `type` serves the inventory.
2. `<slug>` is the registry slug. The seed `type_alias` values are accepted and
   select the same descriptor; generated built-in links keep their seed alias,
   and a user-defined type uses its slug.
3. An unknown, empty, or disabled `type` value returns `404` with the generic
   error page, not `400`.
4. A known route with a disallowed method is `405` with an `Allow` header; an
   unknown path is `404` for every method.

### 3. List views

#### 3.1 Shared list contract

1. Every document list view renders the enabled records of exactly one type,
   read from that descriptor's active directory only (section 6).
2. Every document list view uses the shared list layout: the fixed order
   **ID**, **Title**, **Status**, family slot(s), **Created**, **Updated**,
   **Actions**. A document descriptor's slots come from its seed entry; no field
   becomes a list column.
3. `Status` renders the stored status read-only. `ID` renders the record ID as
   text. `Title` links to `/doc/<ID>` and carries the record labels for every
   family. `Created` and `Updated` render the stored front-matter values as
   stored, or `(none)` when absent; the browser never derives or rewrites them.
4. Rows are ordered by ID ascending. A grouped type renders the group row and its
   member rows through the shared hierarchical row variant, in group order and
   then member order. Role is a property of the row, not a second table.
5. A row renders its reference list only when the resolved
   `reference_visibility` for its type and, for a grouped type, its role is
   `Visible`. `Hidden` omits the reference list entirely, with no collapsed or
   empty placeholder. Labels still render in the Title cell.
6. The page shows a count of the displayed records and the existing empty state
   when there are none.

#### 3.2 Status filter

1. The filter values come from the descriptor's status model:
   `SingleMarkdown` and `GroupedMarkdown` offer `active` followed by the declared
   custom statuses in configured order; `SingleHtmlMockup` offers `pending` and
   `approved` in lifecycle order. `archived` is never offered.
2. The filter is a no-JavaScript GET form with a `Status` select, an
   `Apply filter` submit, and a `Show all` reset link. The select has no
   `All records` or empty-value option; the no-filter state is the absence of a
   `status` parameter.
3. With no `status` parameter, every record in the active directory is listed,
   including records carrying a custom status. Custom statuses never hide
   records.
4. With `status=<value>`, only records whose stored status equals the value
   exactly are listed. Comparison is byte-exact and case-sensitive.
5. For a grouped type, filtering applies before grouping: a group row is shown
   when the group record matches or when at least one member matches; a group
   with neither is omitted. Orphan-member rows are recomputed from the filtered
   set.
6. A value outside the offered set — `archived`, an undeclared value, a
   wrong-case value, an explicit empty `status=`, or duplicate parameters —
   returns `400` with the standard error page and no listing. An absent
   `status` parameter is the no-filter state.

#### 3.3 Tolerant status values

1. A stored status outside the type's declared vocabulary is displayed
   verbatim and is never rewritten by a read.
2. Such a record is listed only when no `status` parameter is present. It is not
   offered as a filter value and is never matched by a value it does not carry
   exactly.
3. The browser never writes `status`, so a read or edit does not repair such a
   value.

### 4. Detail and edit views

#### 4.1 Detail

1. `GET|HEAD /doc/<ID>` resolves the ID to exactly one enabled descriptor by
   configured prefix and ID shape, then reads the record from that descriptor's
   active directory. Detail addressing stays ID-based; the URL carries no type.
   An unknown, archived, or disabled ID returns the one document not-found page
   with `404`.
2. One document view builder renders every document type:
   - the shared head and chrome and breadcrumb;
   - a small record ID above the title, the title, and read-only status beside
     the ID;
   - the labels taglist;
   - created and updated dates under the title;
   - one `<section>` per non-empty field in descriptor sequence order, headed by
     the field `label`, with the value rendered through the prose pipeline or as
     a list for `kind = list`;
   - a required field that is empty renders as an empty section, not omitted;
   - read-only extra metadata rows after the field sections;
   - references, dependencies, and dependents as three separate native
     `<details>` disclosures, each rendered only when non-empty;
   - an `Edit` action linking to `/doc/<ID>/edit`.
3. A field's storage `heading` is never rendered as a display heading; the
   visible section heading is the field `label`. There is no per-type summary
   card and no repetition of the document type.
4. A `GroupedMarkdown` group record renders the `group.fields` sections followed
   by an engine-owned, read-only member list (member ID, title, link, in ID
   order). A member record renders the `member.fields` sections followed by a
   read-only link to its group. Membership is never a form control and never a
   field.
5. A `SingleHtmlMockup` record uses the same head, chrome, and reference
   disclosure with a fixed metadata block instead of field sections: `status`,
   `task`, `specs`, `references`, and `superseded_by`, read from the HTML
   metadata comment.

#### 4.2 Edit

1. `GET|HEAD /doc/<ID>/edit` uses one document edit builder for every document
   type. It renders, in order: the fixed `title` control; one control per
   declared field whose `editable` is true, in descriptor sequence order, with
   the control type from `kind`, the `label` as its label, `help` as the hint
   text bound with `aria-describedby`, and the required marker from `required`;
   the general `labels` and `references` controls; read-only `status`,
   `created_date`, and `updated_date`; and the hidden version token.
2. A field whose `verbs` exclude `edit` is rendered read-only and is never
   writable. `status` is never a control.
3. A `SingleHtmlMockup` form offers the title and the list-valued `specs` and
   `refs` controls only; `labels`, `references`, `status`, `task`, and
   `superseded_by` are read-only there, and there is no field sequence and no
   `status` control.
4. A grouped page resolves the role from the record: a group record uses
   `group.fields`, a member record uses `member.fields`. The group link is
   shown read-only and is never a form control.

#### 4.3 Submit, conflict, and not found

1. One `POST /doc/{id}` handler serves every document type and both grouped
   roles, in this fixed order:
   1. reject a media type other than `application/x-www-form-urlencoded` with
      `415`;
   2. parse the form and read `version`; a missing or malformed token is `400`;
   3. resolve the ID through the request index; an unknown, archived, or
      disabled ID is the one not-found page with `404`;
   4. read the live record for its version token;
   5. compare the submitted token with the live token; a mismatch renders the
      one conflict page with `409`, the submitted values, and the live token,
      and writes nothing;
   6. build the field map for the record's model and role; a submitted key that
      is not `title`, `labels`, `references`, a declared field key, or a fixed
      metadata key of the record's model is rejected as `422`, including
      `status`;
   7. validate every value and collect every error: the title contract, `kind`
      rules, and the required rule; any error re-renders the edit page with
      `422`, the submitted values, and the same token, and writes nothing;
   8. build the patch with the document engine and write once through
      `record::store`; `updated_date` is refreshed once; answer `303` to
      `/doc/{id}`.
2. The conflict page is not a route. Nothing is written on a conflict.
3. One not-found builder returns `404` for every document type, never echoes the
   requested ID, and is the same page for an unknown ID, an archived record, and
   a disabled type.
4. The status-code surface is exactly `415`, `400`, `404`, `409`, `422`, `303`,
   and the existing `500` for an unexpected failure. No status is added,
   removed, or renumbered.

### 5. JSON API

1. Every enabled document descriptor has `GET|HEAD /api/docs/<slug>`, and the
   canonical endpoint returns exactly the JSON its seed-declared legacy alias
   returns for that type. An alias is indistinguishable from the canonical form.
2. `GET|HEAD /api/doc/<ID>` resolves an enabled descriptor exactly as a detail
   request and returns the existing record object. A user-defined document
   returns the generic document object with its metadata and field values.
3. Status filtering is not part of the API.
4. An unknown or disabled `<slug>` returns `404` with the JSON error object. Any
   method other than GET/HEAD on a list route returns `405` with
   `Allow: GET, HEAD`. An unknown path returns `404` for every method.

### 6. Visibility: archived, unknown, disabled

1. `build_index` reads only the descriptor's active directory, top level,
   non-recursively. `archived/`, a legacy `deprecated/` folder, and any other
   subdirectory are never read.
2. A candidate file is a record only when its extension follows the model
   (`.html` for `SingleHtmlMockup`, `.md` otherwise) and its stem matches the
   descriptor's prefix and ID shape. No slug selects the rule.
3. On the browser list, detail, and edit paths, a file in the active directory
   whose front-matter status is `archived` is auto-relocated before the record
   is returned. A relocated record is omitted from the list and is no longer
   resolvable; a relocation failure is a `500`/validation outcome with the source
   intact and the record not listed.
4. An archived ID and an unknown ID of the same type return the identical `404`
   page for list, detail, and edit requests, and the identical JSON error body
   for `/api/doc/<ID>`.
5. A disabled type produces no navigation entry, inventory card, list page,
   detail or edit page, and no canonical or legacy API response.
6. The one not-found builder serves every document type; backlog and decision
   not-found pages are unchanged.

### 7. Retired lifecycle and configuration surfaces

1. `GET /lifecycle` and `GET /?type=lifecycle` return `404`. The former
   Backlog/Decisions/Documents lifecycle page, its descriptor count, and its
   links are retired; no page links to it.
2. The browser serves no configuration view. `GET /config`, `GET /config/edit`,
   `POST /config`, and `GET /?type=config` return `404`; configuration editing is
   CLI-only.
3. The retired managed-artifacts and agent-skills surfaces return `404` and have
   no sidebar entry.
4. The asset and document routes remain unchanged; the retired routes add no
   redirect and no fallback.

### 8. Presentation facts and compatibility

1. Every presentation fact of a document type is seed data: the descriptor
   `label`, `plural`, `order`, `summary`, `api_aliases`, `type_alias`, `slots`,
   `reference_visibility`, and each field's `label`, `help`, and sequence
   position. A field's `required`, `kind`, and `verbs` drive validation and
   control type rather than supplying a display string.
2. The list heading, count noun, empty state, inventory note, page title,
   navigation label, sidebar entry, API alias, legacy `?type=` value, form
   label, form hint, view section heading, and required marker are derived from
   the descriptor. No document entry remains in the static presentation table,
   and no slug match supplies a string.
3. Backlog and decision presentation may stay in the static table. Those
   surfaces are bespoke and their output is byte-compatible with the existing
   pages.
4. The browser document path contains no `match` on a slug, no `TypedHandler`
   variant, no superseded content model variant, and no document-type lookup table. The
   only variation is the descriptor and the field map.

## Implementation contract

1. **Routes.** `BrowserRoutes::build(&ArtifactRegistry)` produces the API
   aliases, type values, navigation, inventory, and ID rules. The static
   presentation table may hold backlog and decision entries only.
2. **Record projection.** The loaded document record carries the envelope
   (`id`, `title`, `status`, `labels`, `references`, `created_date`,
   `updated_date`, body, version, path) plus one `DocumentFieldMap` with the
   role, the resolved fields in descriptor order, fixed metadata for
   `SingleHtmlMockup`, undeclared stored keys as read-only extra metadata, and
   the resolved reference visibility.
3. **Field resolution.** A field with only `heading` reads the body section; a
   field with only `frontmatter_key` reads that key; a field with both reads the
   body section as the display value and does not merge the two locations. An
   absent optional value is `Absent`; an absent required value is `Absent` and
   is flagged on the edit form.
4. **Templates.** The template set is `shell.html`, `list.html`, `view.html`,
   `edit.html`, and `error.html`. `view.html` and `edit.html` are single
   descriptor-driven templates for every document model; the document-specific
   templates are deleted. The conflict page renders through `edit.html` with a
   conflict notice, and the not-found page renders through `view.html`.
5. **Backlog and decision handlers.** They keep their `TypedHandler` dispatch
   and their existing templates because they are bespoke and unchanged.

## Test plan

| Scenario | Assertion |
|---|---|
| User-defined `SingleMarkdown` type | List, detail, edit, conflict, not-found, and JSON work with no per-type code; labels render, reference visibility follows the seed, and the field map drives the form |
| Canonical vs legacy API | Each canonical list body equals its seed-declared legacy alias body for the same type |
| Status read-only | List and detail show the stored status; the edit form has no `name="status"` control; a POST with `status` returns `422` and writes nothing |
| Filter values | The select offers exactly `active` plus the declared custom statuses in order; `archived` is absent; `Show all` omits `status` |
| Filter rejection | `archived`, an unknown value, a wrong-case value, an empty value, and duplicate parameters return `400` with no listing |
| Archived vs unknown | A missing, archived, or disabled ID returns the identical `404` page with no ID echo; archived records are absent from lists and the API |
| Grouped roles | A group record renders `group.fields` and a member list; a member renders `member.fields` and its group link; neither form offers membership |
| HTML mockup | The view uses the fixed metadata block; the form offers title, `specs`, and `refs` only; `labels`, `references`, `status`, `task`, and `superseded_by` are read-only |
| Presentation facts | Every rendered document label, heading, form label, help text, empty-state string, and alias equals the seed value; no document slug appears in the presentation table |
| Surface boundary | A source scan finds no `TypedHandler`, the superseded content model, or slug literal in the document browser path; no document entry remains in `PRESENTATION` |
| Reload | A configuration edit changes navigation, lists, and routes on the next request; a failed reload keeps the previous page serving |
| Method and safety | A wrong method returns `405` with `Allow`; an unknown API slug returns `404`; a coded directory escaping the project yields no page and no traversal |

## Assumptions

- **A1 — Snapshot.** The browser uses one `ServingSnapshot`; a configuration
  edit takes effect on the next request, while host, port, and open are fixed at
  startup.
- **A2 — Grouped sidebar.** The sidebar is grouped by the three artifact
  categories; document entries follow seed order inside `Documents`.
- **A3 — Filter default.** A custom status never hides a record; the resting
  state is a `Show all` link that omits `status`, and an explicit empty `status=`
  is invalid.
- **A4 — HTML mockups.** Their filter values are `pending` and `approved`;
  `archived` is excluded because they are not listed.
- **A5 — Canonical API equals legacy output.** "Alias" means the same body and
  status code for the same type; the canonical route is not a new schema.
- **A6 — Labels and references.** The browser form saves the resulting lists so
  a user can remove a label; the CLI edit appends. Both use the same patch and
  writer; status is untouched.
- **A7 — Status as stored.** A record without a `status` key shows `(none)`
  read-only and is never rewritten by the browser.
- **A8 — Shared list contract.** The fixed column order applies to every
  document list view; the task board and the backlog/decision lists are
  unchanged.

## UI notes

- `MOCK-0015` is the browser reference package. The specification remains
  authoritative if the mockup and the specification disagree.
- The filter is a no-JavaScript GET form. The select lists only allowed values;
  `Show all` is the no-filter reset.
- The list is the shared table; Status is a structural, read-only column.
- The sidebar has exactly the three category groups; retired surfaces have no
  entry.

## Open questions

- Whether a document type can declare an icon or a display name; the seed
  presentation facts define none.
- Whether the JSON API should accept a status filter; not required by the
  document list contract.
- Invalid filter values return `400` per the input-sanitization rule; an
  unknown or disabled type returns `404`. This distinction is intentional.
