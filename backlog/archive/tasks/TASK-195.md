---
id: TASK-195
title: "Strengthen TC-036, TC-037, TC-038 removal tests"
created_date: '2026-11-18T21:35:19Z'
updated_date: '2026-11-19T10:54:24Z'
labels: []
references:
  - AUD-107
  - AUD-108
  - REQ-06.018
  - RE-005
  - TC-036
  - TC-037
  - TC-038
status: Done
assignee:
  - test-manager
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 52
documentation:
  - tests/browser_cli.rs
type: implementation
---

## Description
Correction for AUD-107 findings 2-3 and AUD-108 finding 2 (medium): TC-036, TC-037, and TC-038 do not exercise their scenarios. tc036 and tc037 only assert that documentation directories exist; assert instead the concrete outside-browser artifacts that carry the content (the managed-artifact lifecycle sections of UI-010 and UI-012, the agent-skills description in UI-011 and UI-012, the TECH revision banners, and the .opencode skills entries) and that their content is unchanged. tc038 compares the configuration file only; snapshot record files under the backlog, decisions, and docs directories by path and content hash before and after the browser session and assert equality. Keep the one-to-one TC-036, TC-037, TC-038 to test mapping. Done when cargo test --test browser_cli passes with the strengthened assertions.
## Comments
- **test-manager**: test comment
- **test-manager**: Strengthened TC-036 TC-037 TC-038 in tests browser_cli.rs
- **test-manager**: TC-036 asserts concrete outside-browser lifecycle content in UI-010 section 6 and UI-012 section 7, the ADR-034 outside-browser statement, and the TECH-050 revision banner. TC-037 asserts agent-skills content in UI-011 section 4 and UI-012 section 8, three .opencode skills SKILL.md entries (test-design, manage-test-cases, managed-records) with names and descriptions, and the TECH-051 revision banner. TC-038 creates requirement, task, and decision records, snapshots every file under backlog, decisions, and docs by relative path and content hash plus papyrus.config.yml before and after browser requests including the removed routes, asserts equality, and checks record paths, IDs, prefixes, and directory settings. Verified cargo test --test browser_cli: 181 passed, 0 failed. cargo fmt --check clean.
