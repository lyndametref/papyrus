---
id: TASK-233
title: "Update TECH-077, TECH-070, TECH-071, TECH-072, TECH-068, TECH-069, and TECH-007 for the descriptor engine"
created_date: '2026-11-20T17:40:05Z'
updated_date: '2026-11-21T07:45:39Z'
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
status: Done
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
## Comments
- **tech-writer**: Updated TECH-077, TECH-070, TECH-071, TECH-072, TECH-068, TECH-069, and TECH-007 to the descriptor engine: nested entry validation with document_types diagnostics, model-derived verbs and options with a PatchOp-only engine, owned seed-derived descriptors, the mtime-invalidated serving snapshot with a request-scoped record snapshot, one document browser surface with seed presentation and role-aware rendering, three-category release verification and a single packaging boundary, and unified offline requirement browsing. All seven sections are draft pending architect conformance. papyrus check reports only the three pre-existing archived task frontmatter conflicts.
- **architect**: Architect conformance review. Conformant and set to verified: TECH-070 (model-derived verbs and options, all three DocumentModel verb sets, PatchOp-only engine), TECH-071 (owned seed-derived descriptors, no static requirement), TECH-072 (ADR-037 ServingSnapshot and request-scoped record snapshot). Not conformant, left draft: TECH-077 section 6 wrongly treats statuses, api_aliases, and slots as comma list keys; ADR-0024 as amended classifies only statuses; api_aliases and slots are declarative file data and not config-set or config-unset targets. TECH-068 section 4.1 keeps the retired summary card; use ADR-033 decision 2 order and add references, dependencies, and dependents disclosures, each only when non-empty; section 4.2.3 and test plan make mockup references editable, but ADR-039 clarification makes specs and refs editable and references plus labels read-only. TECH-069 section 4 row B-02 gives Status, ID, Title, Created, Updated; ADR-0028 and ADR-039 fixed order is ID, Title, Status, family slots, Created, Updated, Actions. TECH-007 sections 4.12 and 4c keep the retired summary card and only a References disclosure; align with ADR-033 and ADR-039 including the three disclosures and required-empty rule; section 5.5 last-value-wins must accumulate list fields; sections 4.13 and 5.5 omit SingleHtmlMockup fixed metadata specs and refs editable with labels, references, status, task, and superseded_by read-only; section 4.6 sidebar label Document types conflicts with the three-category sidebar. No TASK, MS, or REL ids and no exact ContentModel identifier appear in any body.
- **tech-writer**: Corrections not applied: blocked by tooling. `papyrus tech-spec edit` patches only the `## Description` block or launches the configured interactive editor, and this session has no editor and no general shell access, so the target body sections cannot be rewritten without duplicating content. The attempt on TECH-077 through `--description` appended the corrected body instead of replacing it, so TECH-077 now holds a duplicated section set and must be restored from version control before the next attempt. TECH-068, TECH-069, and TECH-007 were left untouched to avoid the same corruption. Pending corrections: TECH-077 section 6 (comma syntax and collection reset limited to `statuses`; `slots` and `api_aliases` are declarative file data and not `config set` or `config unset` targets); TECH-068 section 4.1 (current metadata order with references, dependencies, and dependents disclosures, each only when non-empty) plus 4.2.3 and the test plan (mockup `specs` and `refs` editable; `references` and `labels` read-only); TECH-069 row B-02 (fixed order ID, Title, Status, family slots, Created, Updated, Actions); TECH-007 sections 4.6, 4.12, 4.13, 4c, and 5.5 (three-category sidebar, metadata order with three disclosures and the required-empty rule, mockup fixed metadata edit contract, list-valued repeated fields that accumulate). All four sections remain draft. TASK-291 raised for a non-interactive body edit path and the TECH-077 repair.
- **architect**: -
- **architect**: Architect conformance re-review after corrections: all four sections conformant and set to verified. TECH-077: duplicate section set removed; nested entry shape, field vocabulary, reserved keys, scalar fill and collection replace, statuses-only comma list, declarative slots and api_aliases, and document_types diagnostics match ADR-036 and ADR-0019 as amended. TECH-068: ADR-033 metadata order with the three non-empty disclosures and no summary card; mockup form offers title, specs, and refs only, with labels, references, status, task, and superseded_by read-only per ADR-039 clarification; role-aware rendering and seed presentation facts present. TECH-069: row B-02 now gives ID, Title, Status, family slots, Created, Updated, Actions; three-category suites, document model matrix, single release boundary, and no-drift checks conform. TECH-007: three-category sidebar; sections 4.12 and 4c use the ADR-033 layout with three non-empty disclosures and the required-empty rule; sections 4.13 and 5.5 give the mockup fixed metadata contract and list accumulation; REQ-06.003 citation removed. Structural: one section set per file, no orphan heading, no duplicated block, no TASK, MS, or REL ids in bodies, no archived requirement citations. Statuses set: TECH-077 verified, TECH-068 verified, TECH-069 verified, TECH-007 verified. Open observation: REQ-06.014 still describes the mockup references control as editable while ADR-039 clarification and these specifications make references read-only; requirements-owner follow-up, not a TECH block.
- **orchestrator**: Acceptance met: TECH-077, TECH-070, TECH-071, TECH-072, TECH-068, TECH-069 and TECH-007 updated to ADR-036..039 and architect-verified as conformant; TECH-077 duplication repaired; no archived requirement citation in the touched sections.
