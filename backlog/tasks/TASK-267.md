---
id: TASK-267
title: "Collapse the nine conflict pages and five not-found pages into one generic pair"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-22T16:36:56Z'
labels: []
references:
  - ADR-039
  - REQ-06.014
  - TECH-068
  - ADR-0033
  - TC-184
  - REQ-05.007
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-266
ordinal: 60
documentation:
  - src/browser/render.rs
  - src/browser/routes.rs
  - src/browser/mod.rs
  - src/browser/templates/view.html
  - src/browser/templates/edit.html
type: implementation
---

## Description
## Context
ADR-039 decisions 1 and 23-24 collapse the 9 conflict-page renderers to one and the 5 not-found pages to one. Current code has `render::conflict_page` (the req/nfr/glossary/ui-spec fallback), `document_conflict_page`, `conflict_page_ui_mockup`, `conflict_page_business_rule`, `conflict_page_audit_record`, `conflict_page_tech_spec`, `conflict_page_use_case`, plus the unreachable `conflict_page_test_case` and `conflict_page_test_report`; and `render::tech_spec_not_found_page`, `use_case_not_found_page`, `test_case_not_found_page`, `test_report_not_found_page`, and `document_not_found_page`. `routes::NotFoundPage`, `Presentation.not_found`, `BrowserRoutes::not_found_page`, and `IdRule.not_found` select a per-type 404 by ID shape. `src/browser/mod.rs` has the 404 selection chain in `handle_request` and the stale-version conflict chain in `form_post`. ADR-039 decisions 4, 23, and 24 and TECH-068 sections 4.3 and 6 fix the one-page behavior; TASK-266 delivers the generic edit form the conflict page reuses.

## Scope
- Implement one conflict builder for document types (the existing `document_conflict_page` name may be kept) that renders the TASK-266 edit form with the submitted values, the live version token, and a conflict notice, returns `409`, and writes nothing. The conflict page is not a route. Delete all other conflict renderers and their call sites, including the two unreachable TestCase and TestReport wrappers and the nine-function inventory.
- Implement one not-found builder for documents (the existing `document_not_found_page` name may be kept) that renders through `view.html`, returns `404` for every document type, never echoes the requested ID, and is the same page for an unknown ID, an archived record, and a disabled type. Delete the four per-type not-found pages and the per-type selection machinery: `NotFoundPage`, `Presentation.not_found`, `routes::not_found_slug`, `BrowserRoutes::not_found_page`, and `IdRule.not_found`. `handle_request` uses the one builder for every HTML `/doc/{id}` 404.
- Backlog and Decision 404 output is unchanged. They already use the generic document not-found page, so the one builder must preserve that output while document types adopt it.
- Unknown or disabled document list types continue to return the generic 404 error page through the route table; no per-type 404 route or page remains.
- Do not add a route, template, dependency, asset, client script, or status code. Keep the status-code surface 404 and 409 and the existing headers and `no-store` behavior.

## Grounding
ADR-039 decisions 1, 4, 13, 23, and 24; TECH-068 sections 4.3 and 6; REQ-06.014 and REQ-05.007. Symbols: `src/browser/render.rs` (`conflict_page*`, `*_not_found_page`, `document_conflict_page`, `document_not_found_page`), `src/browser/routes.rs` (`NotFoundPage`, `Presentation.not_found`, `not_found_slug`, `BrowserRoutes::not_found_page`, `IdRule.not_found`, `push_id_rule`), `src/browser/mod.rs` (`handle_request` 404 chain and `form_post` conflict chain), and `src/browser/templates/{view,edit,error}.html`.

## Definition of done
Exactly one conflict builder and one not-found builder remain for documents; the nine conflict functions and five not-found functions are gone; the per-type 404 selection types are gone; unknown or archived document IDs share one 404 page with no ID echo and no write occurs on a conflict; Backlog and Decision 404 output is unchanged; tests, clippy -D warnings, and fmt are clean.

## Acceptance Criteria
- [ ] #1 Exactly one conflict builder remains for document types, and the nine pre-refactor conflict renderers are deleted: conflict_page, document_conflict_page (unless kept as the one builder), conflict_page_ui_mockup, conflict_page_business_rule, conflict_page_audit_record, conflict_page_tech_spec, conflict_page_use_case, conflict_page_test_case, and conflict_page_test_report. No per-type conflict call site remains in form_post.
- [ ] #2 A stale-version POST renders the one conflict page with the TASK-266 edit form, the submitted values, the live version token, and a conflict notice, returns 409, and writes nothing; the conflict page is not a route and cannot be reached by GET.
- [ ] #3 Exactly one not-found builder remains for document types: the four typed pages tech_spec_not_found_page, use_case_not_found_page, test_case_not_found_page, and test_report_not_found_page are deleted, and document_not_found_page is either retained as the one generic builder or replaced by it.
- [ ] #4 Every document 404 returns the same one not-found page with status 404: an unknown ID, an archived record, and a disabled type produce byte-identical bodies, and the page never echoes the requested ID in its title, heading, body, or links.
- [ ] #5 NotFoundPage, Presentation.not_found, routes::not_found_slug, BrowserRoutes::not_found_page, and IdRule.not_found are removed; no per-type ID-shape 404 selection or per-type 404 branch remains.
- [ ] #6 The one not-found builder renders through view.html, and the one conflict builder renders through edit.html; no new template is added and the template set stays shell.html, list.html, view.html, edit.html, and error.html.
- [ ] #7 Backlog and Decision 404 output remains byte-identical to the MS-032 corpus; the one not-found builder preserves the body those surfaces already receive.
- [ ] #8 Unknown or disabled document list types still return the generic 404 error page through the route table, and no per-type 404 route exists.
- [ ] #9 The 404 and 409 response headers and no-store behavior are unchanged, and no write occurs on a stale-token conflict or a not-found request.
- [ ] #10 Tests cover the stale-token conflict and all three 404 triggers, assert no ID echo, assert no write on conflict, and compare the Backlog and Decision 404 output; cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
