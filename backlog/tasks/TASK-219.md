---
id: TASK-219
title: "Write ADR-B: in-server configuration and registry snapshot with mtime invalidation"
created_date: '2026-11-20T17:36:52Z'
updated_date: '2026-11-20T18:15:36Z'
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
  - TECH-070
  - TECH-077
  - REQ-07.002
  - NFR-007.001
  - NFR-007.002
status: ToDo
assignee:
  - architect
milestone: MS-031
dependencies:
  - TASK-218
ordinal: 30
documentation:
  - decisions/ADR-037-in-server-configuration-and-registry-snapshot.md
  - decisions/ADR-0017-artifact-category-registry-and-type-descriptors.md
  - decisions/ADR-0016-runtime-config-derived-command-dispatch.md
  - decisions/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - decisions/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - decisions/ADR-0020-registry-derived-browser-views-for-document-types.md
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
## Refinement inputs (TASK-290)

### ADR inventory and amendment targets (`decisions/`)
Inputs and exact paths: ADR-0017 d10 (`decisions/ADR-0017-artifact-category-registry-and-type-descriptors.md`), ADR-0016 d3/d9 (`decisions/ADR-0016-runtime-config-derived-command-dispatch.md`), ADR-0005 (`decisions/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md`), ADR-0006 (`decisions/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md`), ADR-0020 d2 (`decisions/ADR-0020-registry-derived-browser-views-for-document-types.md`).
Front-matter ids are four-digit; filenames for ADR-032 to ADR-0035 are three-digit. Output path: `decisions/ADR-037-in-server-configuration-and-registry-snapshot.md`.

### Refinement assumptions / definition of done
The source of truth is RE-010 D1-D14 (accepted, not reopenable) and MS-031 AC-2/AC-3. No decision is renumbered, softened, split, or reopened. Done means every acceptance criterion passes, the record is accepted and user-verified, and no source, test, or packaging file changes (MS-031 AC-10).
## Acceptance Criteria
- [ ] ADR-037 exists at `decisions/ADR-037-in-server-configuration-and-registry-snapshot.md` with status accepted (MS-031 AC-2B).
- [ ] ADR-037 supersedes the no-cache clause of ADR-0017 decision 10 and records dated amendments to ADR-0016 d3 and d9, the no-cache risk bullets in ADR-0005 and ADR-0006, and ADR-0020 d2 (MS-031 AC-2B).
- [ ] ADR-037 defines mtime-based invalidation across the config file, registry inputs, and route table; one atomic snapshot swap; and fail-closed reload that keeps serving the last known good snapshot while reporting the error through the existing diagnostics path (MS-031 AC-2B, RE-010 D11).
- [ ] ADR-037 defines a request-scoped record snapshot for title and reference resolution and states that no write content or version token is derived from the snapshot, so optimistic concurrency still reads the live record (MS-031 AC-2B, RE-010 D11).
- [ ] ADR-037 explains why the cache does not reintroduce the staleness risk ADR-0017 avoided and leaves all source, test, and packaging files unchanged (MS-031 AC-10).
