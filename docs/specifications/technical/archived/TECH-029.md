---
id: TECH-029
title: "Browser Access for UI Specifications"
created_date: '2026-11-16T17:46:46Z'
updated_date: '2026-11-16T17:47:12Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - MOCK-0001
  - MOCK-0002
  - MOCK-0003
  - MOCK-0004
  - MOCK-0005
  - REQ-005.004
  - REQ-005.007
  - REQ-006.002
  - TASK-11
  - TASK-15
  - TASK-19
  - TASK-23
  - TASK-24
  - TASK-36
  - TASK-37
  - TASK-6
  - TASK-86
status: archived
---
# TECH-029 Browser Access for UI Specifications

## Description
## Purpose

Extend the offline, project-local, read-and-edit web interface so it also serves active UI specifications (title, associated view, description). Reuses TASK-6/TASK-11/TASK-15/TASK-23 baseline (ADR-0005/ADR-0006). Implements REQ-005.004 and REQ-005.007 under REQ-006.002. Deprecated specifications hidden.

## Scope

In scope: sidebar entry?type=ui-spec, list/view/edit, JSON API, validation, atomic writes, boundary, offline. Deprecated hidden. Out of scope: create/deprecate/restore from browser, mockup rendering, search, auth, packaging (TASK-37), cross-type (TASK-19).

## Behavior

1. Category `?type=ui-spec`; default req; other values 400. Sidebar: Business requirements, Non-functional requirements, Backlog, Decisions, Glossary, UI Specifications. Missing dir renders empty state.
2. Records in `<doc_directory>/<document_types.ui-spec.directory>/` (default `docs/specifications/ui/`), prefix `UI`, only `<prefix>-<digits>.md` files. `deprecated/` never listed. Disabled type contributes no records.
3. List: h2 "UI Specifications", through the shared list layout in `browser-shell-ui.md` §3: `ID + title` link in the Title cell with the record labels/references below it, no family slot, and no description preview.
4. View: ID, form UI specification, category UI Specifications, title h1, View and Description sections pre-wrap, breadcrumb UI Specifications › ID, single Edit.
5. Edit: title (required), view (required single-line), description (optional multiline). Hidden version token, 422/409 per baseline.
6. Validation: title/view 1-200 no control, description max 20000, ## line rejected.
7. Persistence: heading `# <ID> <title>`, sections `## View`, `## Description`, unknown sections preserved, atomic, save-then-reread.
8. JSON API: `GET /api/ui-specs` (aliases /api/ui-specifications, /api/ui-spec) array {id,title}; `GET/POST /api/doc/{id}` full {id,kind:ui-spec,form:ui-spec,title,view,description,version,path}.

## Shared layout and references (TASK-86)

Record views in this specification follow the shared single record view layout defined in section 4c of `docs/specifications/browser-requirements.md` (summary card with ID, created date, updated date; labels as a taglist under the title; dependencies and references at the end as native disclosure sections), in addition to the type-specific fields above. Record references that resolve to Papyrus records follow the shared reference-link rule defined in section 4b of `docs/specifications/browser-requirements.md`.

## Investigation notes (AC #3)

- Storage: active in `<doc_directory>/<document_types.ui-spec.directory>/` default `docs/specifications/ui/`.
- File format: `# <ID> <title>`, `## View`, `## Description`.
- Navigation: sidebar "UI Specifications" after "Glossary" ?type=ui-spec.
- Routes: GET /?type=ui-spec, GET /doc/{id}, GET /doc/{id}/edit, POST /doc/{id}, GET /api/ui-specs, GET/POST /api/doc/{id}.
- Reused: parse_doc, section_value, valid_id, version_token, write_atomic, ensure_within, resolve_id, build_index; render_template, escape, sidebar, breadcrumb, metadata_block_backlog, conflict/error pages.
- New: Category::UiSpec, Form::UiSpec, load_ui_spec_record, apply_ui_spec_edit, sidebar entry, edit_page_ui_spec, api_ui_specs.
