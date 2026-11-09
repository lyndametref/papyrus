---
id: TASK-77
title: Add browser skills page for agent workflow configuration
status: To Do
assignee: []
created_date: '2026-11-09T18:51:00Z'
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
priority: medium
type: feature
ordinal: 30098.5
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Implement the browser page required by REQ-06.005 listing agent skills and workflows for configuring Papyrus, with instructions derived from the effective papyrus.config.yml so they match the configured project.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 The local web interface serves a page listing all available Papyrus agent skills and workflows.
- [ ] #2 Each skill entry includes agent-facing instructions on how to use Papyrus (not end-user instructions).
- [ ] #3 Skill content is derived from or parameterized by the effective project configuration, including prefixes, id_padding, statuses, enabled document and decision types, and directories.
- [ ] #4 The page is accessible via browser mode and has a navigation entry within the local web interface.
- [ ] #5 Skills cover, at minimum: new project setup and init, backlog/task/milestone/release management, requirements and documentation management, search, browser usage, and release and audit workflows.
- [ ] #6 The page works without an internet connection.
- [ ] #7 The page route, navigation entry, and covered workflows are documented; content format is a design decision.
<!-- AC:END -->
