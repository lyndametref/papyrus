---
id: TASK-274
title: "Implement the mtime-invalidated config, registry, and route snapshot with fail-closed reload"
created_date: '2026-11-20T17:43:07Z'
updated_date: '2026-11-20T17:43:07Z'
labels: []
references:
  - ADR-037
  - ADR-0016
  - ADR-0020
  - ADR-0005
  - ADR-0006
status: ToDo
assignee:
  - rust-dev
milestone: MS-036
dependencies:
  - TASK-273
ordinal: 40
documentation: []
type: implementation
---

## Description
Context: ServerState::config() reloads configuration per request, and ADR-0017 decision 10 avoided caching entirely. Owner decision 11 keeps an mtime cache.

Scope: one snapshot holds config, registry, and routes; its inputs are the config file and registry sources with mtimes. Reload builds a complete new snapshot and swaps atomically; readers never see a partial reload. A reload error fails closed: the last known good snapshot keeps serving and the error is reported through diagnostics. No per-record config read, YAML parse, or validation remains. Preserve behavior when the config is invalid at startup.
