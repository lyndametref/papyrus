---
id: TASK-182
title: "Implement removal of reference list from document-type browser list views"
created_date: '2026-11-18T19:30:34Z'
updated_date: '2026-11-19T07:59:34Z'
labels: []
references:
  - REQ-06.019
  - RE-006
status: Done
assignee:
  - rust-dev
milestone: MS-028
dependencies:
  - TASK-181
ordinal: 50
documentation:
  - src/browser/render.rs
  - src/browser/list.rs
  - tests/list_characterization_replay.rs
  - tests/browser_config_types_cli.rs
  - tests/list_shared_layout.rs
type: implementation
---

## Description
Code and automated tests for removing the reference list from document-type browser list views.
## Comments
- **rust-dev**: Implementation evidence. Acceptance criteria: Generic rows omit the reference block entirely; Typed and HtmlMockup families keep it; labels stay in the Title cell; Row.references keeps the RecordView projection; shared layout, status filter, grouped rows, actions, shell, and page width unchanged; Generic detail keeps the collapsible References section; CLI list and view, JSON API, RecordView, and front matter unchanged; glossary landing page gains no reference section; views and assets are local. Implementation: list.rs ListPlan gains show_references derived in for_descriptor from !matches!(descriptor.content_model, ContentModel::Generic); render_table and render_row skip the row-refs block only when false. No slug match, presentation override, per-family template, or presentation-table field. render.rs unchanged. Tests: TC-055, TC-056, TC-057, TC-058, TC-059, TC-060, TC-063, TC-064, TC-065 in browser_config_types_cli.rs and list_shared_layout.rs; TC-055, TC-057, TC-061, TC-062 in the characterization replay split (Generic note family has no row-refs; Typed families with fixture refs and HtmlMockup MOCK-002 keep row-refs); list.rs unit tests cover Generic omission with retained Row.references and Typed and HtmlMockup retention. Files changed: list.rs, browser_config_types_cli.rs, list_characterization_replay.rs, list_shared_layout.rs. Checks: cargo fmt --check pass; cargo clippy --all-targets pass with no warnings; cargo test --lib 433 passed; browser_config_types_cli 22 passed; list_shared_layout 4 passed; list_characterization_replay populated replay passes, every_empty_fixture_replays_the_shared_empty_state fails on the pre-existing project-empty decisions ADR-001 test fixture (decision page is no longer empty) unrelated to this change; cargo test --no-fail-fast has only two pre-existing failures, that empty replay fixture and no_drift no_id_formatting_or_prefix_parsing_outside_identity at import.rs line 1202. Security: no unsafe; rendered values stay escaped; no new input, authorization, secret, or network surface. Gaps: none in scope; the two full-suite failures are pre-existing and outside TASK-182.
