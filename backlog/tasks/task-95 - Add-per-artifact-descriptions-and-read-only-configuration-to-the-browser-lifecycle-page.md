---
id: TASK-95
title: >-
  Add per-artifact descriptions and read-only configuration to the browser
  lifecycle page
status: To Do
assignee: []
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-11T20:57:00Z'
labels:
  - browser
  - release-1.0
milestone: m-16
dependencies:
  - TASK-87
  - TASK-88
  - TASK-90
references:
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.008.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
priority: high
type: feature
ordinal: 39120
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
REQ-06.003 now requires each lifecycle section to describe its artifact and to present the configuration attached to it, read-only, with edits confined to the configuration view. The current page explains lifecycles but neither describes the artifact nor shows its enabled state, prefix, folder, or statuses, so a reader cannot tell how a type is configured in this project.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 Each artifact section carries curated description text shipped with the binary that works offline and is not derived from requirement files, record content, or configuration.
- [ ] #2 Each section shows the effective enabled state, ID prefix, and storage folder, and the task section additionally shows the configured statuses.
- [ ] #3 Configuration values on the page are read-only, and the page points to the configuration view for changes.
- [ ] #4 The fixed covered type set and the configured-status derivation of the task lifecycle are unchanged.
- [ ] #5 Tests or fixtures cover a disabled type, a renamed prefix or folder, and custom task statuses.
<!-- AC:END -->
