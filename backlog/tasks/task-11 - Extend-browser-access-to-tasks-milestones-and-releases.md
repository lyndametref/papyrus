---
id: TASK-11
title: Extend browser access to tasks, milestones, and releases
status: Refined
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-04T20:22:00Z'
labels:
  - browser
  - backlog
milestone: m-2
dependencies:
  - TASK-6
  - TASK-8
  - TASK-9
references:
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-02.001.md
  - docs/requirements/business/REQ-02.002.md
  - docs/requirements/business/REQ-03.001.md
  - docs/requirements/business/REQ-03.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/specifications/ui/mockups/MOCK-0002-browser-backlog-records-review.html
  - docs/specifications/browser-tasks-milestones-and-releases.md
documentation:
  - docs/specifications/ui/browser-backlog-records-ui.md
  - docs/specifications/browser-tasks-milestones-and-releases.md
type: feature
ordinal: 10000
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Extend the offline browser (TASK-6 baseline, ADR-0005/ADR-0006) to serve active task, milestone, and release records from the backlog store, with read and edit support, under the same transport, asset, rendering, security, and configuration baseline.

Scope:
- Categories: extend the persistent sidebar and ?type= selector to task, milestone, release (default req; any other value 400).
- List/view/edit active tasks, milestones, releases; archived tasks/milestones hidden and unreachable (404).
- Task editable fields: title, description, status, milestone, assignee, labels, dependencies, acceptance criteria. Milestone: title, description, release. Release: title, description. Comments read-only.
- JSON API endpoints for tasks/milestones/releases and generic /api/doc/{id}.
- Reuse baseline escaping, ID resolution, boundary checks, Host/Origin validation, request limits, a11y.

Out of scope: create/archive/restore/delete from browser; other record types; search in browser; new deps/ADRs.

Spec: docs/specifications/browser-tasks-milestones-and-releases.md
UI: MOCK-0002 (approved) extends MOCK-0001.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The local browser displays and edits active task, milestone, and release records in addition to all previously supported requirement content.
- [ ] #2 Category selection supports type=task|milestone|release (default req); any other value, including all, returns 400.
- [ ] #3 Archived tasks and milestones are not listed and return 404 by URL.
- [ ] #4 Task view shows ID, status, milestone, assignee, labels, dependencies, acceptance criteria, comments, and description; milestone/dependency links resolve when possible.
- [ ] #5 An edit round-trip persists and is visible through both the browser and the CLI.
- [ ] #6 Field validation returns 422 and does not write for empty title, unknown status, unknown milestone/release, self-dependency, and lines starting with ## .
- [ ] #7 A stale version token returns 409 and does not write.
- [ ] #8 Writes are atomic, project-local, leave no temporary file, and symlink escapes return 404.
- [ ] #9 Browser operations remain offline-capable and use no paths outside the project directory except during init.
- [ ] #10 The expanded interface renders in Firefox, Microsoft Edge, and Safari version 100 or later.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Unit and integration tests added/updated and pass.
- [ ] #2 cargo build, cargo test, cargo clippy --all-targets -- -D warnings, and cargo fmt --check pass.
- [ ] #3 Feature spec and UI description updated with MOCK-0002 reference.
- [x] #4 MOCK-0002 explicitly approved by the user before implementation.
- [ ] #5 No new dependencies, crates, or ADRs introduced.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
UI mockup MOCK-0002 approved by the user on 2026-11-04 (TASK-11). Extends MOCK-0001 shell: single combined backlog view, kanban by status, expandable milestones, framed tags, bullet-list acceptance criteria, tag-based dependency editing, searchable milestone dropdown. Registry and UI description updated to approved.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: ui-designer
created: 2026-11-04T20:19:00Z

---

MOCK-0002 revised (still pending): (1) unified single 'Backlog' sidebar entry instead of separate task/milestone/release views; (2) task status changed by drag-and-drop between kanban columns; (3) labels edited via add/remove tag workflow with search box (same as dependencies). UI description doc and registry updated.

---

author: ui-designer
created: 2026-11-04T20:22:00Z

---

MOCK-0002 explicitly approved by the user on 2026-11-04 (TASK-11). Registry and UI description updated to approved; DoD #4 checked.

---
<!-- COMMENTS:END -->
