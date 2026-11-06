# Feature Specification — Browser Access for UI Specifications

**Ticket:** TASK-24
**Milestone:** m-6 (UI Specifications)
**Status:** Refined (execution-ready)
**UI mockups:** [`MOCK-0005`](ui/mockups/MOCK-0005-browser-ui-specifications-review.html) — approved 2026-11-06 (TASK-36); extends the approved baseline [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html), [`MOCK-0002`](ui/mockups/MOCK-0002-browser-backlog-records-review.html), [`MOCK-0003`](ui/mockups/MOCK-0003-browser-decision-records-review.html), and [`MOCK-0004`](ui/mockups/MOCK-0004-browser-glossary-terms-review.html).

## Purpose

Extend the offline, project-local, read-and-edit web interface so it also serves active UI specifications (title, associated view, description). Reuses TASK-6/TASK-11/TASK-15/TASK-23 baseline (ADR-0005/ADR-0006). Implements REQ-05.004 and REQ-05.007 under REQ-06.002. Deprecated specifications hidden.

## Scope

In scope: sidebar entry?type=ui-spec, list/view/edit, JSON API, validation, atomic writes, boundary, offline. Deprecated hidden. Out of scope: create/deprecate/restore from browser, mockup rendering, search, auth, packaging (TASK-37), cross-type (TASK-19).

## Behavior

1. Category `?type=ui-spec`; default req; other values 400. Sidebar: Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications. Missing dir renders empty state.
2. Records in `<doc_directory>/<document_types.ui-spec.directory>/` (default `docs/specifications/ui/`), prefix `UI`, only `<prefix>-<digits>.md` files. `deprecated/` never listed. Disabled type contributes no records.
3. List: h2 "UI Specifications", rows `ID + title` link `/doc/{id}` plus first paragraph of description preview.
4. View: ID, form UI specification, category UI Specifications, title h1, View and Description sections pre-wrap, breadcrumb UI Specifications › ID, single Edit.
5. Edit: title (required), view (required single-line), description (optional multiline). Hidden version token, 422/409 per baseline.
6. Validation: title/view 1-200 no control, description max 20000, ## line rejected.
7. Persistence: heading `# <ID> <title>`, sections `## View`, `## Description`, unknown sections preserved, atomic, save-then-reread.
8. JSON API: `GET /api/ui-specs` (aliases /api/ui-specifications, /api/ui-spec) array {id,title}; `GET/POST /api/doc/{id}` full {id,kind:ui-spec,form:ui-spec,title,view,description,version,path}.

## Investigation notes (AC #3)

- Storage: active in `<doc_directory>/<document_types.ui-spec.directory>/` default `docs/specifications/ui/`.
- File format: `# <ID> <title>`, `## View`, `## Description`.
- Navigation: sidebar "UI Specifications" after "Glossary" ?type=ui-spec.
- Routes: GET /?type=ui-spec, GET /doc/{id}, GET /doc/{id}/edit, POST /doc/{id}, GET /api/ui-specs, GET/POST /api/doc/{id}.
- Reused: parse_doc, section_value, valid_id, version_token, write_atomic, ensure_within, resolve_id, build_index; render_template, escape, sidebar, breadcrumb, metadata_block_backlog, conflict/error pages.
- New: Category::UiSpec, Form::UiSpec, load_ui_spec_record, apply_ui_spec_edit, sidebar entry, edit_page_ui_spec, api_ui_specs.
