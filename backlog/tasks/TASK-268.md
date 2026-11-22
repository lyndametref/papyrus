---
id: TASK-268
title: "Implement seed-declared presentation and reference visibility per type and role"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-22T16:36:06Z'
labels: []
references:
  - ADR-039
  - ADR-035
  - ADR-036
  - ADR-041
  - REQ-06.019
  - REQ-06.014
  - TECH-068
  - TC-184
  - REQ-06.004
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-267
ordinal: 70
documentation:
  - src/registry.rs
  - src/config.rs
  - src/browser/routes.rs
  - src/browser/list.rs
  - src/browser/mod.rs
type: implementation
---

## Description
## Context
`BrowserRoutes::build` and `routes::{presentation, list_slots, label, heading, short, noun, inventory_note, legacy_url_key, api_aliases, inventory_order, presentation_order}` still read the static `PRESENTATION` table for documents. `TypeDescriptor` already carries `label`, `slots`, `reference_visibility` for a single-role type, and `group` and `member` `RoleFields` each carry their own `reference_visibility`, but it does not carry the seed `plural`, `summary`, `order`, `type_alias`, or `api_aliases`, although `config::SEED_DOCUMENT_TYPES` and `DocTypeEntry` hold them. `ListPlan::for_descriptor` takes a `presentation_slug`, calls `routes::list_slots`, and sets one plan-wide `show_references = !descriptor.is_config_defined_document()` (the superseded ADR-0035 content-model rule). ADR-039 decisions 29-33, ADR-036 decision 29, ADR-041 decision 4, and TECH-068 section 8 define the replacement.

## Scope
- Make `TypeDescriptor` carry or expose the seed presentation facts `plural`, `summary`, `order`, `type_alias`, and `api_aliases` in addition to the existing `label`, `slots`, and `reference_visibility`, resolved from `resolved_document_entry` and `SEED_DOCUMENT_TYPES`. A user-defined type keeps the documented defaults of ADR-036 decision 33: slug-derived label and plural, no summary, an order after the seed types, empty aliases, empty slots, and `Visible` reference visibility.
- `BrowserRoutes::build` derives document navigation labels, inventory titles and notes, inventory order, `?type=` legacy keys, and API aliases from the descriptor. Remove the hard-coded document list `inventory_order()` and its slug order; derive the order from the descriptor's seed `order` with slug tie-break. Fixed layout constants such as the aggregate card position are not per-type data and may remain.
- `routes::presentation`, `list_slots`, `label`, `heading`, `short`, `noun`, `inventory_note`, and `legacy_url_key` must not return a document entry or a slug fallback for a document type; `PRESENTATION` holds only backlog and decision entries (plus their aggregate entries). Backlog and decision presentation stays static and unchanged.
- `ListPlan::for_descriptor` uses `descriptor.slots` for document types and no `presentation_slug` or `routes::list_slots` lookup for documents; backlog and decision types keep their static presentation slots and reference display. Document list row content comes from the field map (TASK-265).
- `show_references` becomes a per-row property resolved from the descriptor, not a plan-wide flag: `SingleMarkdown` and `SingleHtmlMockup` use `descriptor.reference_visibility`; `GroupedMarkdown` uses `descriptor.group.reference_visibility` for a group row and `descriptor.member.reference_visibility` for a member row. `Hidden` omits the reference list entirely (no empty list, no placeholder, no collapsed control) while labels still render in the Title cell. Seed values: `req` and `nfr` are `Hidden` for both roles; every other seed type is `Visible`; a user-defined type defaults to `Visible`.
- No fallback branch keyed on a document slug or on `is_config_defined_document` supplies a presentation string or a visibility value. Reference visibility affects list rows only; the single-document view keeps its References disclosure and the CLI, JSON, and stored front matter are unchanged.
- Do not add a route, dependency, asset, template, client script, or configuration key. Do not change the Backlog or Decision surfaces.

## Grounding
ADR-039 decisions 29-33; ADR-036 decisions 29 and 32-33; ADR-041 decision 4; REQ-06.014, REQ-06.019, and REQ-06.004; TECH-068 sections 2.1, 3.1, and 8. Symbols: `src/registry.rs` (`TypeDescriptor`, `document_descriptor`, `resolved_document_entry`), `src/config.rs` (`SEED_DOCUMENT_TYPES`, `SeedDocumentType`, `DocTypeEntry`, `merge_entry`, `user_defined_entry`), `src/browser/routes.rs` (`PRESENTATION`, `BrowserRoutes::build`, `presentation`, `list_slots`, `label`, `heading`, `short`, `noun`, `inventory_note`, `legacy_url_key`, `api_aliases`, `inventory_order`), `src/browser/list.rs` (`ListPlan::for_descriptor`, `Row`, `render_row`), and `src/browser/mod.rs` (`inventory_page`).

## Definition of done
Every document presentation fact and every document reference-visibility value comes from the descriptor seed; `PRESENTATION` holds no document entry; document list slots and rows use descriptor data; req and nfr hide references in both roles while labels still render; other seed types and user-defined types behave per ADR-039; Backlog and Decision output is unchanged; tests, clippy -D warnings, and fmt are clean.

## Acceptance Criteria
- [ ] #1 TypeDescriptor carries or exposes the seed presentation facts plural, summary, order, type_alias, and api_aliases in addition to label, slots, and reference_visibility, resolved from resolved_document_entry and SEED_DOCUMENT_TYPES; a user-defined type receives the documented defaults of ADR-036 decision 33.
- [ ] #2 Every document navigation label, list heading, page title, count noun, empty state, inventory title and note, and API alias equals the descriptor seed value (apart from a fixed template-level capitalization rule); inventory order follows descriptor order with slug tie-break; every ?type= legacy key equals the descriptor type_alias. No per-type string is held in the browser.
- [ ] #3 PRESENTATION holds no document entry; routes::presentation, list_slots, label, heading, short, noun, inventory_note, and legacy_url_key never serve a document slug from the static table; the hard-coded document inventory_order list is removed and inventory order comes from descriptor order with slug tie-break.
- [ ] #4 Backlog and Decision presentation, navigation, inventory cards, list slots, and reference display remain static and unchanged; document types use descriptor slots and do not call routes::list_slots.
- [ ] #5 ListPlan::for_descriptor uses descriptor.slots for document types and field-map row content; no presentation_slug lookup or static slot lookup remains for documents, and the structural column set and order are unchanged.
- [ ] #6 Reference visibility resolves per type and role from the descriptor: SingleMarkdown and SingleHtmlMockup use the single value; GroupedMarkdown uses group.reference_visibility for a group row and member.reference_visibility for a member row; a group row and a member row can differ.
- [ ] #7 In the seed req and nfr resolve Hidden for both roles, every other seed type resolves Visible, and a user-defined SingleMarkdown type defaults to Visible; no rule keyed on a former content model or on is_config_defined_document remains.
- [ ] #8 When the resolved value is Hidden the reference list is absent in every row of that type and role: no ul.row-refs, no empty list, no placeholder, and no collapsed control; labels still render in the Title cell.
- [ ] #9 Reference visibility affects list rows only; the single-document view keeps its References disclosure for every document type, and CLI list, CLI view, JSON output, and stored front matter are unchanged.
- [ ] #10 A user-defined type is listed, navigable, and rendered with slug-derived presentation defaults without a code change, an empty slots set, and visible references.
- [ ] #11 Tests assert the seed matrix for req, nfr, and the other seed types, both grouped roles, a user-defined type, labels always rendered, hidden references absent, and no document entry in PRESENTATION; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
