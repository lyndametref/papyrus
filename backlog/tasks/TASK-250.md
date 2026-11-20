---
id: TASK-250
title: "Remove per-record configuration and registry construction from the read path"
created_date: '2026-11-20T17:41:37Z'
updated_date: '2026-11-20T17:41:37Z'
labels: []
references:
  - ADR-037
  - ADR-036
  - TECH-043
  - TECH-040
  - NFR-007.001
status: ToDo
assignee:
  - rust-dev
milestone: MS-033
dependencies:
  - TASK-249
ordinal: 70
documentation: []
type: implementation
---

## Description
Context: migrate::ensure_front_matter calls managed_descriptor and load_cfg_for, which read the config, parse YAML, validate, and build an ArtifactRegistry at least twice per record; load_record then resolves titles through resolve_id, which rescans descriptor directories.

Scope: pass &Config and &ArtifactRegistry into the read path; require callers to supply them; delete the per-record load entry points; make title resolution read from the supplied registry instead of rescanning. Keep the mtime cache as the only reload point (owner decision 11). Do not introduce the request-scoped snapshot yet; that belongs to MS-036. Behavior and output bytes must be unchanged.
Do not start until MS-031 governance is accepted.
