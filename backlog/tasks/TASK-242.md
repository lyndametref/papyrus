---
id: TASK-242
title: "Record the corrected renderer and route inventory with file paths"
created_date: '2026-11-20T17:40:57Z'
updated_date: '2026-11-21T21:01:02Z'
labels: []
references:
  - ADR-039
  - TECH-068
  - TECH-069
  - REQ-006.014
  - REQ-006.019
status: Done
assignee:
  - code-analyzer
milestone: MS-032
dependencies:
  - TASK-239
ordinal: 50
documentation:
  - docs/audits
  - docs/audits/AUD-135.md
type: audit
---

## Description
Context: earlier planning notes said 6 conflict-page renderers and 4 not-found pages; the audit found 9 and 5. MS-035 deletes the per-type surfaces and must prove every one is gone, so the record must be exact, reproducible and path-addressed, not a count in prose.

Grounding in the current code:
- Request routing lives in `src/browser/mod.rs::route` and `handle_request`: `/assets/<path>`, `/api/requirements`, `/api/docs/<slug>` plus every legacy path in `Presentation::api_paths`, `/api/doc/<ID>` (GET and POST), `/` (inventory when no `type` query, otherwise the list and the task board plus its POST), `/doc/<ID>` (GET and POST), `/doc/<ID>/edit` (GET), `HEAD` variants, 404 for unknown paths and 405 for a known path with a disallowed method.
- Page handlers: `inventory_page`, `list_page`, `board_page`, `view_page`, `edit_page`, `form_post`, `api_requirements`, `api_doc_list`, `api_get`, `api_post` in `src/browser/mod.rs`.
- Renderers live in `src/browser/render.rs`: `inventory_page`, `board_page`, `list_page`, `view_page`/`document_detail_page`, `edit_page` plus the per-type `edit_page_*` functions, the nine `conflict_page*` functions, `error_page`, the five `*_not_found_page` functions, and `Shell`/`Chrome` composition in `src/browser/shell.rs`.
- The not-found selector is `src/browser/routes.rs::BrowserRoutes::not_found_page` over `IdRule`, with the `NotFoundPage` enum (`TechSpec`, `UseCase`, `TestCase`, `TestReport`, `GenericDocument`) and `Presentation::not_found` in the `PRESENTATION` table; it is dispatched by the 404 branch of `src/browser/mod.rs::handle_request` (`src/browser/mod.rs:644` to `661`).
- Current counts, verifiable with the commands recorded in the audit: `rg -n "^pub fn .*conflict_page" src/browser/render.rs` returns 9, and `rg -n "^pub fn .*not_found_page" src/browser/render.rs` returns 5. The conflict functions are `document_conflict_page`, `conflict_page_ui_mockup`, `conflict_page_business_rule`, `conflict_page_audit_record`, `conflict_page_tech_spec`, `conflict_page_use_case`, `conflict_page_test_case`, `conflict_page_test_report` and `conflict_page`; the not-found functions are `tech_spec_not_found_page`, `use_case_not_found_page`, `test_case_not_found_page`, `test_report_not_found_page` and `document_not_found_page`. The stale-version branch of `form_post` selects seven distinct pages (six type branches plus the generic fallback), and `conflict_page_test_case` and `conflict_page_test_report` have no call site and delegate to `conflict_page_use_case`. The audit must state both facts rather than only the count.

Scope:

1. Record the inventory as an audit record created with `papyrus audit create --audit-type code --reference MS-032 --reference ADR-039 --reference TECH-068 --reference TASK-242`, stored under `docs/audits/`, and link it as this task's documentation.

2. Route inventory: one row per route with path pattern, accepted methods, handler function, and file with line number, covering every path in item 1 above, including the `/api/docs/<slug>` and legacy alias rows and the 404 and 405 behaviour.

3. Document-type inventory: one table row per enabled document type (the eleven built-ins plus the config-defined type used by the suite), with slug, DocumentModel (SingleMarkdown, GroupedMarkdown, SingleHtmlMockup), list URL key including the legacy key where declared, view route, edit route, JSON list paths, JSON record route, detail renderer, edit renderer, conflict renderer and not-found renderer, each with file and line.

4. Conflict-page section: the nine functions with file and line, their dispatcher branches in `form_post`, which document types reach each one, and the note that two are unreachable wrappers over `conflict_page_use_case`.

5. Not-found section: the five functions with the `NotFoundPage` variant, the ID shape that selects it, the dispatcher mapping, and the renderer, file and line.

6. Shared-duplication section: for each conflicted or not-found page, the shared helper or markup it re-implements (the conflict notice block and the per-type field reconstruction; the page skeleton, sidebar and error copy), naming the functions that duplicate it. This is the set MS-035 must collapse.

7. Reproducibility: record the exact commands used to enumerate routes and renderers, so a later re-run produces the same nine and five sets.

8. Note explicitly that the 1,156-test count is an informational count, not a gate.

Out of scope: deleting or refactoring renderers, routes or tests; changing `src/`; deciding MS-035's target shape.

Definition of done: the audit record exists, is linked from this task, lists every route and every enabled document type with file and line, contains the nine-and-five confirmation with the enumeration commands, and re-running those commands reproduces the recorded sets.
## Acceptance Criteria

- [ ] An audit record of type code is created through the CLI, references MS-032, ADR-039, TECH-068 and TASK-242, is linked as task documentation, and its ID is stated in a task comment.
- [ ] The audit lists every browser route handled by the request router with path pattern, accepted methods, handler function and file with line number, including the per-slug JSON list, every legacy JSON alias, the per-record JSON route, the inventory, the board and its POST, and the 404 and 405 behaviour.
- [ ] The audit lists every enabled document type with its DocumentModel, list URL key, view route, edit route, JSON list paths, JSON record route, and the detail, edit, conflict and not-found renderer for each, with file and line.
- [ ] The conflict section lists exactly nine renderer functions with file and line: document_conflict_page, conflict_page_ui_mockup, conflict_page_business_rule, conflict_page_audit_record, conflict_page_tech_spec, conflict_page_use_case, conflict_page_test_case, conflict_page_test_report and conflict_page (MS-032 AC-3).
- [ ] The conflict section states that conflict_page_test_case and conflict_page_test_report have no call site and delegate to conflict_page_use_case, and that the stale-version dispatcher selects seven distinct pages, so the defined count is not confused with the dispatch surface.
- [ ] The not-found section lists exactly five pages with variant, selecting ID shape, renderer function, file and line: tech_spec_not_found_page, use_case_not_found_page, test_case_not_found_page, test_report_not_found_page and document_not_found_page (MS-032 AC-3).
- [ ] The exact enumeration commands are recorded and rerunning them reproduces the same nine and five sets.
- [ ] The shared-duplication section names the reused conflict notice and field reconstruction and the shared not-found page skeleton, with the functions that duplicate each.
- [ ] The audit states that the 1,156-test count is an informational count, not a gate.
- [ ] No source file changes: the MS-032 diff for this task contains only the audit record and task evidence.

## Comments
- **task-refiner**: Refined by task-refiner for MS-032: description grounded in the current routes, renderers, harness conventions and root-cause call sites; acceptance criteria added; references and documentation set. No dependency, ordinal or milestone change. Status left ToDo because papyrus.config.yml declares statuses [ToDo, In-Progress, Done].
- **code-analyzer**: Code audit AUD-135 issued (type code, status issued; references MS-032, ADR-039, TECH-068, TECH-069, TASK-242, REQ-006.014, REQ-006.019). Linked as task documentation. Confirmed: 9 conflict-page renderers and 5 not-found pages; 23 declared legacy API aliases; 12 document types (11 seed plus the config-defined note); 14 TypedHandler variants; 13 ops builders; 7 route-reachable conflict pages. Reachability correction: conflict_page_test_case and conflict_page_test_report have no call site and delegate to conflict_page_use_case, so 7 pages are selectable (1 generic branch, 5 type-specific branches, 1 fallback) and 2 are dead wrappers. Discrepancy: the TASK-242 wording six type branches plus the generic fallback is imprecise; a prefix-only regex returns 8 rather than 9 because document_conflict_page lacks the conflict_page prefix, so use the .* regex recorded in the audit. Checks: papyrus check unchanged with the same 3 pre-existing frontmatter conflicts; cargo fmt, clippy and lib tests pass. Findings 1 medium and 3 low; verdict pass for the inventory scope. TASK-242 remains In-Progress.
- **orchestrator**: Acceptance met: AUD-135 records the corrected pre-refactor inventory (9 active routes, 23 legacy API aliases, 9 conflict and 5 not-found renderers with 7 reachable conflict paths, 14 TypedHandler variants and 13 slug comparisons), with exact file locations and reproducibility commands.
