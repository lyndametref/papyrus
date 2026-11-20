---
id: TASK-235
title: "Update TECH-008, TECH-013, TECH-024, and TECH-027 packaging and verification sections"
created_date: '2026-11-20T17:40:19Z'
updated_date: '2026-11-20T17:40:19Z'
labels: []
references:
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - NFR-007.001
  - NFR-007.002
  - NFR-007.003
status: ToDo
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-232
ordinal: 190
documentation:
  - docs/specifications/technical/TECH-008.md
  - docs/specifications/technical/TECH-013.md
  - docs/specifications/technical/TECH-024.md
  - docs/specifications/technical/TECH-027.md
type: docs
---

## Description
Context: these four sections still describe per-milestone packaging and verification with per-type document code. The refactor has one release boundary and no intermediate release or RC.

Scope:
- Replace milestone-level packaging steps with verification builds that are discarded; only the final milestone produces a retained release package.
- State the gates that every milestone verification runs: full suite, clippy, no_drift, no new crates, zero document TypedHandler variants, no document slug dispatch outside the seed, strict record/JSON/status-code equivalence, semantic HTML equivalence, and the NFR-007.001 to NFR-007.003 measurements.
- Keep the binary size gate at 12 MiB and record the production LOC target of 28,000 as an internal target, explicitly not an acceptance criterion.
- Remove references to per-type document modules from the packaging inventory.
Do not reference TASK, MS, or REL IDs in the body.
