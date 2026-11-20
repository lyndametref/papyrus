---
id: TASK-219
title: "Write ADR-B: in-server configuration and registry snapshot with mtime invalidation"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T17:36:52Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0016
  - ADR-0017
  - ADR-0020
  - NFR-07.001
  - NFR-07.002
  - TECH-068
  - TECH-071
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 30
documentation:
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
type: architecture
---

## Description
Context: configuration is currently re-read and re-validated per record and per request, which makes inventory latency O(n^2)-ish and view latency seconds at a few hundred records. ADR-0017 decision 10 chose no in-server cache; owner decision 11 keeps an mtime cache instead.

Scope: author ADR-037, "In-server configuration and registry snapshot".
- Supersede the no-cache clause of ADR-0017 decision 10.
- Amend ADR-0016 decisions 3 and 9, the no-cache risk bullets in ADR-0005 and ADR-0006, and ADR-0020 decision 2.
- Define mtime-based invalidation across the config file, the registry inputs, and the route table.
- Define one atomic swap of a snapshot containing config, registry, and routes; readers never observe a partially reloaded snapshot.
- Define fail-closed reload: a reload error keeps the last known good snapshot serving and reports the error through the existing diagnostics path.
- Define a request-scoped record snapshot (id to Record) that serves title and reference resolution for the lifetime of one request, so no resolution rescans descriptor directories.
- State that no write content or version token is derived from the snapshot; optimistic concurrency still reads the live record.
- Justify why the cache does not reintroduce the staleness risk that ADR-0017 avoided.
