---
id: TASK-233
title: "Update TECH-077, TECH-070, TECH-071, TECH-072, TECH-068, TECH-069, and TECH-007 for the descriptor engine"
created_date: '2026-11-20T17:40:05Z'
updated_date: '2026-11-20T18:15:42Z'
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
  - NFR-007.001
  - NFR-007.002
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
## Refinement inputs (TASK-290)

### TECH section map (real paths)
- TECH-077 — `docs/specifications/technical/TECH-077.md` — §1 Config::validate, §2 slug/prefix/directory/status rules, §3 duplicate YAML keys, §4 loaders, §5 repair/tolerant commands, §6 init --force, §7 error aggregation, §8 registry.
- TECH-070 — `docs/specifications/technical/TECH-070.md` — §1 descriptor option metadata, §2 command construction, §3 invocation contract, §4 document::run dispatch, §5 per-type migration matrix, §6 compatibility contract, §11 traceability.
- TECH-071 — `docs/specifications/technical/TECH-071.md` — §1 record data on the registry template, §2 one accessor, §3 field-source matrix, §4 call-site migration, §5 legacy behavior, §6 single-source invariant tests.
- TECH-072 — `docs/specifications/technical/TECH-072.md` — §1 ownership model, §2 clap owned command names, §3 call sites, plus `Behavior preservation` and `Test plan`.
- TECH-068 — `docs/specifications/technical/TECH-068.md` — §1 route table, §2 navigation/list, §3 list views, §4 detail/edit, §5 JSON API, §6 visibility, §7 lifecycle page, §8 presentation table.
- TECH-069 — `docs/specifications/technical/TECH-069.md` — §1 isolation/fixtures, §2 test suites, §3 CLI matrix, §4 browser matrix, §5 upgrade scenario, §6 Debian package, §7 no-drift checks, §8 carried gaps, §9 evidence.
- TECH-007 — `docs/specifications/technical/TECH-007.md` — §3 HTTP server behavior, §6 assets and rendering, §7 editing contract, §8 configuration changes; plus `Acceptance criteria mapping`.

### Refinement assumptions / definition of done
All seven TECH files are `verified` records; this task updates their bodies only and attaches requirement/decision links with `--reference`. No TASK, MS, or REL id appears in a body. Done means all acceptance criteria pass and no source or test file changes (MS-031 AC-7, AC-10).
## Acceptance Criteria
- [ ] TECH-077 replaces the flat validation contract with the nested entry shape, field vocabulary, reserved keys, default-fill and collection-replace rules, and the `document_types.<slug>.<path>` diagnostic form (MS-031 AC-7).
- [ ] TECH-070 replaces hand-written typed dispatch with model-derived verbs and options, states the verbs each DocumentModel exposes, and states that the engine emits only PatchOps (MS-031 AC-7).
- [ ] TECH-071 describes descriptors as owned values derived from the seed and configuration with no 'static requirement, and TECH-072 keeps the process-global cache removed while describing the ADR-037 mtime-invalidated snapshot (MS-031 AC-7).
- [ ] TECH-068 describes one descriptor-driven list, view, edit, conflict, and not-found surface with seed presentation facts and role-aware group and member rendering; TECH-069 updates packaging and verification to the three-category handler set and single release boundary (MS-031 AC-7).
- [ ] TECH-007 states that offline requirement browsing uses the unified document surface and schema-driven field map; no TASK, MS, or REL ids appear in any body, and the user has verified the seven updates (MS-031 AC-7, AC-8).
