---
id: TASK-233
title: "Update TECH-077, TECH-070, TECH-071, TECH-072, TECH-068, TECH-069, and TECH-007 for the descriptor engine"
created_date: '2026-11-20T17:40:05Z'
updated_date: '2026-11-20T17:40:05Z'
labels: []
references:
  - TECH-077
  - TECH-070
  - TECH-071
  - TECH-072
  - TECH-068
  - TECH-069
  - TECH-007
  - ADR-036
  - ADR-037
  - ADR-039
  - REQ-07.002
  - REQ-07.003
  - REQ-07.004
status: ToDo
assignee:
  - tech-writer
milestone: MS-031
dependencies:
  - TASK-218
  - TASK-219
  - TASK-221
ordinal: 170
documentation:
  - docs/specifications/technical/TECH-077.md
  - docs/specifications/technical/TECH-070.md
  - docs/specifications/technical/TECH-071.md
  - docs/specifications/technical/TECH-072.md
  - docs/specifications/technical/TECH-068.md
  - docs/specifications/technical/TECH-069.md
  - docs/specifications/technical/TECH-007.md
type: docs
---

## Description
Context: these sections describe the flat document-type entry, the 'static descriptor registry, the per-type command builders, and the per-type registry-derived browser views. All four premises change.

Scope:
- TECH-077: replace the flat validation contract with the nested entry shape, the field item vocabulary, reserved keys, the default-fill and collection-replace rules, and the document_types.<slug>.<path> diagnostic form.
- TECH-070: replace hand-written typed dispatch with model-derived verb and option generation; state which verbs each DocumentModel exposes and that the engine emits only PatchOps.
- TECH-071: descriptors become owned values derived from the seed and the configuration, with no 'static requirement.
- TECH-072: keep the process-global cache removed, but describe the mtime-invalidated snapshot from ADR-037 instead of per-request reloading.
- TECH-068: one descriptor-driven list, view, edit, conflict, and not-found surface; presentation facts from the seed; role-aware group and member rendering.
- TECH-069: update packaging and verification steps to the three-category handler set and the single release boundary.
- TECH-007: state that offline requirement browsing uses the unified document surface and the schema-driven field map.
Do not reference TASK, MS, or REL IDs in the body; attach requirement and decision references with --reference.
