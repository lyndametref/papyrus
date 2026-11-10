---
id: TASK-77
title: Add browser skills page for agent workflow configuration
status: Done
assignee: []
created_date: '2026-11-09T18:51:00Z'
updated_date: '2026-11-10T17:51:00Z'
labels:
  - browser
  - agent-workflow
milestone: m-12
dependencies:
  - TASK-76
  - TASK-72
  - TASK-67
references:
  - docs/requirements/business/REQ-06.005.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
documentation:
  - docs/specifications/browser-agent-skills-and-workflows.md
  - docs/specifications/ui/browser-agent-skills-ui.md
  - docs/specifications/ui/mockups/MOCK-0011-browser-agent-skills-review.html
priority: medium
type: feature
ordinal: 30098.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Add a read-only browser page listing agent skills and workflows for configuring and operating Papyrus, per REQ-06.005 and approved MOCK-0011. The page is agent-facing reference material; its values are parameterized by the effective papyrus.conf.yaml (prefixes, id_padding, statuses, enabled document/decision types, directories, paths, date format). Route: /skills (alias /?type=skills). Sidebar entry 'Agent skills & workflows' is placed after 'Managed artifacts & lifecycles' and is always present. The page renders eight fixed skill groups, a 'Configuration in effect' block, and copyable snippet blocks. It reuses the shared shell/CSP/escaping/responsive rules; no records are listed, viewed, edited, or mutated.

Scope and behavior are fully specified in docs/specifications/browser-agent-skills-and-workflows.md. Open questions OQ-1..OQ-5 (UI description) and REQ-06.005 open questions are resolved to their recommended defaults (user confirmation 2026-11-10).

Assumptions:
- Assumption: The page is read-only and renders no project records.
- Assumption: Route is /skills with /?type=skills as an alias.
- Assumption: The sidebar entry is always present, independent of document_types.*.enabled.
- Assumption: Content format is structured sections with copyable snippets (OQ-1 resolved).
- Assumption: Snippets are templates parameterized by effective config values, with static prose for narrative (OQ-2 resolved).
- Assumption: Custom task statuses have no per-state descriptions; only their names and order are rendered.

Dependencies: TASK-76 (Done), TASK-72 (Done), TASK-67 (Done). Related: TASK-19, TASK-20 (depend on this ticket).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The local web interface serves a page at /skills listing all available Papyrus agent skills and workflows, with exactly one h1 titled 'Agent skills and workflows'.
- [ ] #2 The page is reachable via the alias route GET /?type=skills and serves the same content as GET /skills.
- [ ] #3 The page renders a 'Configuration in effect' block (dl) summarizing task/milestone/release prefixes, id_padding with example, date format, statuses, enabled document and decision types, and backlog/decision/doc directories, all derived from the effective configuration.
- [ ] #4 Skill content is parameterized by the effective configuration: prefixes, id_padding, the status chain, enabled document/decision types, and directories appear in the rendered page and change with a custom configuration.
- [ ] #5 The sidebar contains an entry labelled 'Agent skills & workflows' placed after 'Managed artifacts & lifecycles', always present (not gated by document_types), linking to /skills and carrying aria-current='page' when active.
- [ ] #6 The page contains the eight fixed skill groups in order: Project setup and init; Backlog, tasks, milestones, releases; Requirements and documentation management; Search; Browser usage; Release and audit workflows; Configuration management; Decisions.
- [ ] #7 Skills cover, at minimum: new project setup/init, backlog/task/milestone/release management, requirements and documentation management, search, browser usage, and release/audit workflows.
- [ ] #8 Non-GET methods on /skills return 405 with an Allow: GET, HEAD header; unknown paths return 404.
- [ ] #9 The page works without an internet connection: no http:// or https:// references and no external resource references.
- [ ] #10 All dynamic configuration values are HTML-escaped; the page works with JavaScript disabled and the Copy affordance is progressive enhancement.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Feature specification docs/specifications/browser-agent-skills-and-workflows.md created and marked execution-ready
- [ ] #2 Implementation follows the approved MOCK-0011 layout and the parameterized config values in the specification
- [ ] #3 Unit/integration tests added for the page route, alias route, method handling, config summary, group headings and order, custom configuration values, and offline (no external references)
- [ ] #4 No code, dependency, or ADR changes beyond those required by the specification
<!-- DOD:END -->
