---
id: doc-1
title: Papyrus Feature Roadmap
type: specification
created_date: '2026-11-04T19:22:00Z'
updated_date: '2026-11-15T16:15:00Z'
---
# Papyrus Feature Roadmap

Ordered feature increments (milestones). Each milestone delivers a usable, installable increment.

## Milestones

- m-0: Rust CLI Foundation and Requirements (completed) — baseline CLI, project init/config, business/NFR management.
- m-1: Search and Browser for Requirements (completed) — search and offline browser for requirements.
- m-2: Tasks, Milestones, Releases, and Discovery (completed) — task/milestone/release CLI, search, browser, packaging. (untouched)
- m-3: Decision Records — decision management, search, browser, packaging.
- m-4: CLI — all new record-management CLI commands: glossary terms (TASK-30), UI specifications (TASK-34), UI mockups (TASK-38), business rules (TASK-43), audit records (TASK-47), technical specifications (TASK-55). Ordered by task ordinals 17000–17500, immediately after Decision Records.
- m-5: Glossary Terms — search (TASK-31), browser (TASK-23), design review (TASK-32), packaging (TASK-33).
- m-6: UI Specifications — search (TASK-35), browser (TASK-24), design review (TASK-36), packaging (TASK-37).
- m-7: UI Mockups — search (TASK-39), browser (TASK-40), design review (TASK-41), packaging (TASK-42).
- m-8: Business Rules — search (TASK-44), browser (TASK-25), design review (TASK-45), packaging (TASK-46).
- m-9: Audit Records — search (TASK-48), browser (TASK-26), design review (TASK-49), packaging (TASK-50).
- m-11: Technical Specifications — domain model (TASK-21), search (TASK-56), browser (TASK-57), design review (TASK-58), packaging (TASK-59).
- m-12: Final Cross-Type Integration and Record Metadata — shared record envelope (TASK-63), identity translation (TASK-64), CLI front-matter options (TASK-65), design review (TASK-66), metadata projection (TASK-67), document-record migration (TASK-68), task/milestone migration (TASK-69), legacy removal (TASK-70), Backlog.md round-trip gate (TASK-71), lifecycle page (TASK-72), final cross-type design review (TASK-60), unified search (TASK-18), cross-type browser (TASK-19), packaging (TASK-20), id_padding constraint (TASK-74), interactive init walkthrough (TASK-75), browser skills design review (TASK-76), browser skills page (TASK-77).
- m-15: Use Cases and Test Documentation — manage use cases, test cases, and test reports from the CLI (TASK-78), search (TASK-80), browser exposure (TASK-81), design review mockup (TASK-79), packaging and verification (TASK-82). Should-have document types (REQ-005.012/013/014); lifecycle active → deprecated; front-matter required. Placed before Release 1.0.
- m-14: Release 1.0 — two tasks, both Low priority: the release-level upgrade notes TASK-99 (ordinal 39160) and the release gate TASK-73 (labelled release, ordinal 40000, last). m-14 holds no implementation or specification work and stays open and isolated so more requirements can be added before 1.0 without redefining the gate. Closing it requires the feature milestone m-16 to be fully Done plus TASK-20 (m-12) and TASK-82 (m-15), with build/tests passing and the security audit complete; TASK-104 (m-16) is part of the feature milestone and must be accepted by the gate before it closes.
- m-16: Release 1.0 - Native Layout, Import, and Browser Refinements — the extension feature milestone, 20 tasks, all High priority: specifications TASK-83 to TASK-86, design review TASK-87, native layout TASK-88, task and milestone adoption TASK-89, relocation TASK-90, init/config TASK-91, cursor-driven init walkthrough conformance TASK-104 (ordinal 39085, directly after TASK-91), import TASK-92, browser surfaces TASK-93 to TASK-96, serializer removal TASK-97, backlog kanban board specification TASK-101, mockup revision TASK-102 and implementation TASK-103, verification and packaging TASK-98. Placed before the release milestone; TASK-73 depends on its final tasks.
- m-17: Config-Defined Document Types and Artifact Categories — REQ-007.001/002/003/004 plus REQ-006.014 and the REQ-001.002 list-valued keys; one feature milestone after Release 1.0, thirteen tasks TASK-105 to TASK-117, all High priority, ordinals 41000-41110. Delivers the runtime artifact registry and descriptors, the dynamic `papyrus <slug>` family with `status`, fail-closed `document_types` validation and repair, the generic content model, registry-derived search and browser views, init defaults without `deploy`, and the upgrade documentation. Man-page work is excluded: ADR-0021 is Proposed and NFR-003.003 remains a recorded release risk. No release gate is created; whether this ships as Release 1.1 or folds into an open release is an owner decision.

- m-18: Registry Consolidation and Release Close - follow-ups from the m-17 architecture checkpoint and TASK-117 verification; seven tasks TASK-118 to TASK-124, ordinals 41200-41260. Closes the release-blocking `papyrus release close` gap G-5, refactors the two ADR-0016/0017 deviations in code (descriptor-driven typed dispatch through `document::run`, registry-based record descriptor resolution) and removes the process-global descriptor cache and slug leaks, then clears G-4 and F-117-1 and adds init/registry and archive-terminology drift guards. Every dependency is inside m-18; no release milestone or release ticket is created.
- m-19: Architecture Checkpoint Follow-ups: Mockup Archive and ADR Alignment - owner decisions of 2026-11-15 on the m-18 architecture checkpoint: fix the UI-mockup archive ordering in code, amend ADR-0016/0017/0018/0023 instead of refactoring built-ins, and plan every follow-up. Five tasks TASK-125 to TASK-129, ordinals 41300-41340: TASK-125 routes mockup archive, supersede, and edit-with-move through `record::store::archive_content`; TASK-126 amends the four ADRs and regenerates the ADR index; TASK-127 corrects the browser and init specifications; TASK-128 aligns the domain model and glossary and removes the empty legacy `deprecated/` folders; TASK-129 adds C0/DEL/C1 parity to the path sanitizers. Every dependency is inside m-19; no release milestone or release ticket is created.
- m-20: Milestone Lifecycle Drift Follow-ups - follow-up to m-19 after the m-19 architecture checkpoint found residual milestone drift: ADR-0017 constraint 3 and ADR-0023 decision 2 already record the close-only lifecycle, but the descriptor/record layer and two documents still describe an archived milestone. Three tasks TASK-134 to TASK-136, ordinals 41400-41420: TASK-134 sets `ArchivePolicy::None` for the milestone template, removes the lifecycle archive location from `RecordDescriptor::resolve_archived_directory`, preserves legacy reads and the import write to `backlog/archive/milestones/`, and stops `relocate_if_archived` from moving an `archived` milestone; TASK-135 rewrites the REQ-006.013 archived-milestone clauses and regenerates the business requirement index; TASK-136 corrects `docs/specifications/milestone-and-release-management.md` to the close-only lifecycle and records the superseded wording. TASK-135 and TASK-136 follow TASK-134; every dependency is inside m-20; no release milestone or release ticket is created.
- m-21: Diagnostics, Browser Shell, and Offline Rendering — REQ-001.008 plus REQ-006.015/06.016/06.017 and the 2026-11-15 edits to REQ-001.002, REQ-001.004, REQ-002.003, REQ-005.007, REQ-006.013, REQ-006.014, and REQ-007.002. Fourteen tasks TASK-137 to TASK-150, ordinals 41500-41620: the read-only `check` command and its duplicate-ID repair, the browser shell with a server-side width contract and one shared list layout, collapsible navigation, the vendored asset pipeline with package notices, KaTeX math, Mermaid diagrams, the per-milestone `Clean up all` board action, the specification revision, and packaged plus end-to-end verification. ADR-0025, ADR-0027, ADR-0028, and ADR-0029 are Accepted; ADR-0026 is Proposed, so TASK-146, TASK-148, and TASK-150 are blocked until it is accepted. Every dependency is inside m-21; no release milestone or release ticket is created.

## Removed

- m-13: Other Documentation (archived; TASK-17 obsolete).
- m-10: Deployment Records (removed; TASK-27, TASK-51, TASK-52, TASK-53, TASK-54 deleted).

## Dependency and ordering rules

1. Dependencies first (topological order).
2. User feature priority among ready tasks.
3. No unusable increments: read/view before write/entry.
4. Risk and readiness.
5. Tie-breaker: ask user.

## Cross-milestone dependencies

- m-3 entry TASK-13 -> TASK-12 (m-2, completed).
- CLI entry tasks TASK-30, TASK-34, TASK-38, TASK-43, TASK-47, TASK-55 -> TASK-13 (m-3). CLI tasks no longer depend on later-milestone package/search/browser tasks.
- m-5 search/design/browser/packaging -> TASK-30 (CLI); TASK-23 also -> TASK-15 (m-3).
- m-6 search/design/browser/packaging -> TASK-34 (CLI).
- m-7 search/design/browser/packaging -> TASK-38 (CLI).
- m-8 search/design/browser/packaging -> TASK-43 (CLI).
- m-9 search/design/browser/packaging -> TASK-47 (CLI).
- m-11 search/design/browser/packaging -> TASK-55 (CLI); TASK-21 -> TASK-1 (foundation, completed).
- m-12 TASK-66 -> TASK-59 (m-11); TASK-60 -> TASK-66 and TASK-76; TASK-18 -> search tasks (TASK-31, TASK-35, TASK-39, TASK-44, TASK-48, TASK-56) plus TASK-14 (m-3) and TASK-68; TASK-19 -> TASK-60 plus browser tasks (TASK-23, TASK-24, TASK-25, TASK-26), TASK-67, TASK-72, and TASK-77; TASK-74 -> (none); TASK-75 -> TASK-74; TASK-76 -> TASK-66; TASK-77 -> TASK-76, TASK-72, TASK-67; TASK-20 -> TASK-18, TASK-19, TASK-60, TASK-59, TASK-69, TASK-70, TASK-71, TASK-72, TASK-74, TASK-75, TASK-77.
- m-15 TASK-78 -> TASK-20 (m-12); TASK-80 -> TASK-78; TASK-79 -> TASK-78; TASK-81 -> TASK-78 and TASK-79; TASK-82 -> TASK-78, TASK-80, TASK-81.
- m-16 all 20 tasks depend only on tasks inside m-16; the entry tasks TASK-83 to TASK-86 and TASK-101 follow. TASK-101 -> TASK-86; TASK-102 -> TASK-101; TASK-103 -> TASK-97, TASK-101, TASK-102; TASK-98 -> TASK-97, TASK-103; TASK-104 -> TASK-91. No m-16 task depends on m-12, m-15, or m-14 work, and none depends on a later milestone.
- m-14 TASK-99 -> TASK-83 (m-16) and TASK-92 (m-16); TASK-73 -> TASK-99 (m-14), TASK-98 (m-16), TASK-91 (m-16), TASK-20 (m-12), TASK-82 (m-15). The release milestone depends on the feature milestone; no m-16 task depends on m-14 or on a later milestone.
- New m-12 internal order: TASK-63 -> TASK-64 -> TASK-65 -> TASK-67; TASK-63/TASK-65 -> TASK-68 -> TASK-69; TASK-67/TASK-69 -> TASK-70 -> TASK-71; TASK-66 -> TASK-67, TASK-72, TASK-60, TASK-76; TASK-74 -> TASK-75; TASK-76 -> TASK-77; TASK-60/TASK-67/TASK-72/TASK-77 -> TASK-19; TASK-20 last.

- m-18 all seven tasks depend only on tasks inside m-18; TASK-118, TASK-119, TASK-122, and TASK-123 have no dependency, TASK-120 follows TASK-119, TASK-121 and TASK-124 follow TASK-120. No m-18 task depends on m-14, m-16, m-17, or a later milestone.
- m-19 TASK-125, TASK-126, and TASK-129 have no dependency; TASK-127 and TASK-128 follow TASK-126. All dependencies are inside m-19; no m-19 task depends on m-14, m-16, m-17, m-18, or a later milestone.
- m-20 TASK-134 has no dependency; TASK-135 and TASK-136 follow TASK-134 so the corrected requirement and specification record the landed contract. All dependencies are inside m-20; no m-20 task depends on m-14, m-16, m-17, m-18, m-19, or a later milestone.
- m-21 TASK-137, TASK-141, TASK-143, and TASK-147 have no dependency; TASK-138 follows TASK-137; TASK-139 follows both; TASK-142 and TASK-144 follow TASK-141; TASK-145 follows TASK-141 and TASK-143; TASK-146 follows TASK-145 and is gated on ADR-0026 acceptance; TASK-140 follows TASK-139; TASK-148 follows TASK-142, TASK-144, TASK-145, and TASK-146; TASK-149 follows TASK-142, TASK-143, TASK-144, TASK-145, and TASK-147; TASK-150 follows TASK-140, TASK-146, TASK-148, and TASK-149. All dependencies are inside m-21; no m-21 task depends on m-14, m-16, m-17, m-18, m-19, m-20, or a later milestone.

## Requirement coverage

All REQ/NFR IDs remain traced via task references. TASK-17 marked obsolete; scope split into per-type CLI tasks. Deployment record types removed from scope (TASK-18/TASK-19 updated).

### Front-matter and Backlog.md interoperability (2026-11-08)

- REQ-005.010 — Record Metadata in YAML Front Matter: TASK-63 (parser, descriptors, key registry), TASK-65 (options and timestamps), TASK-68 (document records), TASK-69 (tasks/milestones and `release`), TASK-70 (no legacy metadata carriers).
- REQ-005.011 — Maintain Record Metadata Through All Commands: TASK-65 (create/update options), TASK-67 (browser save and `updated_date`), TASK-68 and TASK-69 (lazy per-record migration, managed types only, frozen bodies, legacy conversion with per-file diff), TASK-70 (single write path).
- REQ-006.003 — Describe Managed Artifacts and Their Lifecycles in the Browser: TASK-66 (design and approval), TASK-72 (implementation).
- REQ-006.004 — Use Front-Matter Metadata in CLI and Browser Views: TASK-66 (design and approval), TASK-67 (one projection, browser save, read-only timestamps).
- REQ-001.005 — Stable Record IDs and two ID spaces: TASK-64 (translation, reference translation, input rejection), TASK-67 (display ID in every projection).
- NFR-004.003 — Backlog.md task and milestone interoperability: TASK-64 (identity), TASK-69 (layout and additive keys), TASK-71 (two-direction round-trip gate), TASK-20 (packaged verification).
- REQ-005.008 — UI mockup metadata stays in HTML: TASK-63 and TASK-68 exclude HTML/mockups; no front matter is written to mockups.

### Project setup and configuration (2026-11-09)

- REQ-001.001 — Initialize a Papyrus Project: TASK-2 (baseline, done), TASK-75 (interactive walkthrough).
- REQ-001.005 — Stable Record IDs and two ID spaces: TASK-64 (translation), TASK-67 (display ID in every projection), TASK-74 (id_padding constraint).
- REQ-001.006 — Interactive Init Walkthrough: TASK-75 (first implementation), TASK-85 (specification), TASK-91 (space-toggle walkthrough, nonconforming presentation), TASK-104 (cursor-driven conformance).
- REQ-001.007 — ID Padding Constraint for File IDs: TASK-74.
- REQ-006.005 — Browser Skills Page for Agent Workflow Configuration: TASK-76 (design review), TASK-77 (implementation).

### Use cases and test documentation (2026-11-10)

- REQ-005.012 — Manage Use Cases: TASK-78 (CLI management), TASK-80 (search), TASK-81 (browser exposure), TASK-79 (design review mockup), TASK-82 (packaging and verification). Should-have; lifecycle active → deprecated; front-matter required (REQ-005.010).
- REQ-005.013 — Manage Test Cases: TASK-78, TASK-80, TASK-81, TASK-79, TASK-82.
- REQ-005.014 — Manage Test Reports: TASK-78, TASK-80, TASK-81, TASK-79, TASK-82.
- Open question: REQ-005.012/013/014 state management via the `req` command, which conflicts with BR-009 (req is business-requirements only). BR-018/019/020 recommend a dedicated per-type command (`use-case`, `test-case`, `test-report`) following the tech-spec pattern (BR-017). Resolution is tracked as AC #5 of TASK-78 and must be decided before implementation.

## Requirement change update (2026-11-06)

- REQ-005.009 now defines technical specifications as a managed documentation type with the `tech-spec` command (BR-017, README, domain model); REQ-005.007 includes technical specifications in the document lifecycle.
- TASK-55 unblocked: `blocked` label removed; documented command contract now exists. No milestone, dependency, or ordering change: TASK-55 remains in m-4 at ordinal 17500 depending on TASK-13.
- TASK-21 (m-11, Done) remains the domain-model/requirements definition task; TASK-55 does not depend on it to avoid a backward cross-milestone dependency.

## Requirement change update (2026-11-08)

- New requirements REQ-005.010, REQ-005.011, REQ-006.003, REQ-006.004, and NFR-004.003 are validated; ADR-0007, ADR-0008, and ADR-0009 define the staged delivery.
- Placement decision: the work is merged into m-12 rather than added as a post-final milestone. All m-12 tasks were To Do, m-12 is the declared final increment, and front matter changes every record file and both presentation surfaces. A new milestone after m-12 would invalidate TASK-20's packaging and TASK-60's final design review and would require a second packaging pass.
- Reorder within m-12: TASK-60 (final cross-type design review) moves from ordinal 30000 to 30095 and now depends on TASK-66, so the final review covers the front-matter and lifecycle UI. TASK-18 gains TASK-68; TASK-19 gains TASK-67 and TASK-72; TASK-20 gains TASK-69, TASK-70, TASK-71, and TASK-72. No task was moved across a milestone and no in-progress task was reordered.
- Release decision: m-14 `Release 1.0` with release ticket TASK-73, depending on TASK-20. Version 1.0 is a planning assumption; rename the milestone and ticket if the user selects another version.
- Specification prerequisite: `business-analyst`/`tech-writer` correct the browser specification pages that currently claim front-matter preservation before TASK-67/TASK-70 complete; this is noted inside those tasks, not a separate backlog task.

## Requirement change update (2026-11-09)

- New requirements REQ-001.006, REQ-001.007, and REQ-006.005 are validated; REQ-001.001, REQ-001.005, and the REQ-006 group were updated; REQ-007/08/09 were removed (canonicalized).
- Placement decision: merge into m-12 rather than a post-final milestone, consistent with the 2026-11-08 front-matter decision. Adding a milestone after m-12 would invalidate TASK-20 packaging and TASK-60 final design review and require a second packaging pass; TASK-20 and TASK-73 already gate the release.
- REQ-001.007 (TASK-74) and REQ-001.006 (TASK-75) are foundational and scheduled early in m-12 (ordinals 30005, 30006); TASK-75 depends on TASK-74. They do not reopen archived TASK-2 scope beyond the new behavior.
- REQ-006.005 is a browser surface: TASK-76 (design review, ordinal 30093) precedes TASK-60 so the final design review covers it; TASK-77 (implementation, ordinal 30098.5) depends on TASK-76, TASK-72, and TASK-67; TASK-19 gains TASK-77 and TASK-60 gains TASK-76.
- Deferred TASK-69 gaps (active relocation, no-drift, interop) remain tracked by TASK-70 and TASK-71 and are not reopened; new tasks do not duplicate them.
- TASK-67 is refined and in progress on branch task/TASK-67 and is not modified; new tasks depend on it rather than changing it.
- Release decision: m-14 `Release 1.0` (TASK-73) remains gated on TASK-20, which now includes TASK-74, TASK-75, and TASK-77; no new release ticket is required.

## Requirement change update (2026-11-10)

- New requirements REQ-005.012, REQ-005.013, and REQ-005.014 are validated (Should have; document types under REQ-005; lifecycle active → deprecated; front-matter required). BR-018, BR-019, and BR-020 define per-type behavior.
- Placement decision: unlike the 2026-11-08/09 front-matter and skills work, these are new document types not yet implemented anywhere, so they cannot be merged into the completed m-12. A new milestone m-15 "Use Cases and Test Documentation" is created and placed before Release 1.0. m-14 (TASK-73) now also depends on TASK-82 (m-15 packaging) in addition to TASK-20.
- The three types share one combined milestone and one task per pattern category (CLI management, search, browser exposure, design review mockup, packaging/verification) because they share the lifecycle and front-matter contract; per-type CLI command naming remains an open question (see above).
- No in-progress task is reordered; TASK-73 gains a dependency only.
## Requirement change update (2026-11-11) — Release 1.0 extension

- Settled inputs: requirements commit 5fd5046 and architecture commit 1d4914c. NFR-004.004 adopted and NFR-004.003 superseded; REQ-001.005 single Papyrus ID space; REQ-005.010 and REQ-005.011 Backlog.md clauses removed; REQ-001.001, REQ-001.002, and REQ-001.003 changed; REQ-001.006 now a single space-toggle TTY walkthrough; REQ-006.003 extended; REQ-006.006 to REQ-006.012 added. ADR-0010 to ADR-0013 accepted; ADR-0007, ADR-0008, and ADR-0009 superseded.
- Roadmap correction: the earlier roadmap closed Release 1.0 without the native record layout, the one-way import, and the new browser surfaces, so it no longer covered the requirement set. The work is added to the Release 1.0 delivery, not to a separate later release: ADR-0010 decision 11 and ADR-0011 decision 9 require the layout and the import in the same release, and no completed packaging or audit is invalidated.
- Milestone restructure (user decisions 2026-11-11): the release milestone and the feature work are separated. m-14 Release 1.0 holds only release-level work — the release-level upgrade notes TASK-99 and the release gate TASK-73, both Low priority — and stays open and isolated, because more requirements may still arrive before 1.0. The implementation and specification work lives in the new m-16 Release 1.0 - Native Layout, Import, and Browser Refinements (TASK-83 to TASK-98, all High). The release milestone depends on the feature milestone, never the reverse.
- Order inside m-16: specifications TASK-83 to TASK-86, design review TASK-87, native layout TASK-88, task and milestone adoption TASK-89, relocation TASK-90 and init/config TASK-91, import TASK-92, browser surfaces TASK-93 to TASK-96, serializer removal TASK-97, verification and packaging TASK-98. TASK-99 follows the feature work at ordinal 39160 and TASK-73 closes the release at ordinal 40000. TASK-73 depends on TASK-99, TASK-98, TASK-91, TASK-20, and TASK-82; TASK-91 is listed explicitly because nothing else depends on it.
- Hard constraints encoded: the native layout and the import ship in the same release (TASK-88 and TASK-92 are both in m-16, feed TASK-98, and are verified together); specifications and ADR alignment precede implementation; no task depends on a later milestone or on the release gate.
- Priorities: release-milestone tasks (TASK-73, TASK-99) are Low per the user decision; every m-16 task is High. Relative order is expressed by ordinal, not by priority.
- User decision (2026-11-11, revision): this repository own backlog store is NOT imported in Release 1.0. The migration is deferred to after the release. TASK-100 was removed from the feature scope, left unassigned, and archived as deferred; it is not a dependency of TASK-73. The release still meets the ADR-0010 decision 11 and ADR-0011 decision 9 shipment coupling: the native layout and papyrus import backlog ship in the same release, TASK-98 verifies the import against an isolated sample Backlog.md project, and TASK-99 publishes the one-time import in the upgrade notes for users. Accepted consequence: after the release, this repository own backlog keeps the Backlog.md layout and is therefore not visible to the new Papyrus binary until the deferred import is run.

### Coverage of the changed and new requirements

- NFR-004.004 — TASK-83 (specification), TASK-88 (no translation, no backlog/config.yml), TASK-89 (no Backlog body conversion), TASK-92 (one-way import), TASK-97 (removal and enforcement), TASK-98 (verification on a sample project), TASK-99 (upgrade notes), TASK-73 (gate and re-audit). Own-store migration deferred: TASK-100, archived.
- NFR-004.003 — superseded, no work planned; the historical round-trip suites and fixtures are deleted by TASK-89 and TASK-97.
- REQ-001.005 — TASK-83, TASK-84, TASK-88 (single ID, allocation, resolution), TASK-92 (imported IDs), TASK-93 (browser ID surfaces), TASK-97 (storage-ID JSON removal), TASK-98 (verification).
- REQ-005.010 — TASK-84, TASK-88 (front-matter id), TASK-89 (task and milestone adoption, key set), TASK-92 (imported record format).
- REQ-005.011 — TASK-84, TASK-89 (lazy front-matter-only adoption), TASK-97 (single write path).
- REQ-001.001 — TASK-85 (specification), TASK-91 (non-empty directory, --force replacement).
- REQ-001.002 — TASK-85, TASK-90 (relocation), TASK-94 (browser delegates to config set).
- REQ-001.003 — TASK-85, TASK-91 (missing-configuration runtime error).
- REQ-001.006 — TASK-85, TASK-91 (space-toggle TTY walkthrough), TASK-104 (cursor-driven presentation, navigation, clamping, Enter/EOF confirmation).
- REQ-006.003 — TASK-86, TASK-87, TASK-95.
- REQ-006.006 — TASK-86, TASK-93.
- REQ-006.007 — TASK-86, TASK-96.
- REQ-006.008 — TASK-86, TASK-87, TASK-94.
- REQ-006.009 — TASK-86, TASK-93.
- REQ-006.010 — TASK-86, TASK-93.
- REQ-006.011 — TASK-86, TASK-93.
- REQ-006.012 — TASK-86, TASK-87, TASK-93.
- REQ-006.013 — TASK-101 (specification for the per-milestone kanban board and the card status control), TASK-102 (MOCK-0002 revision and explicit approval), TASK-103 (rendering, release grouping, card order and links, drag-and-drop and non-drag status change, persistence and failure path).
- Unchanged requirements that run through rewritten code paths stay covered by the existing Done tasks and are re-verified by TASK-98: REQ-002, REQ-003, REQ-004, REQ-005.001 to REQ-005.009, REQ-005.012 to REQ-005.014, REQ-006.001, REQ-006.002, REQ-006.004, REQ-006.005.

### Specification revisions

- Superseded in intent and retired by TASK-83: migrate-tasks-and-milestones-to-the-backlog-md-layout, translate-papyrus-and-backlog-md-ids-at-the-storage-boundary, verify-backlog-md-round-trip-interoperability.
- Revised by TASK-84: shared-record-envelope-and-front-matter-patcher, migrate-document-record-types-to-front-matter-lazily-on-first-read-and-first-write, expose-front-matter-metadata-options-on-all-record-commands, render-one-metadata-projection-in-cli-and-browser-views, remove-legacy-record-serializers-and-enforce-the-no-drift-rule, task-management, milestone-and-release-management, and the search specifications.
- Revised by TASK-85: project-initialization-and-configuration, interactive-init-walkthrough-for-artifact-activation.
- Revised by TASK-86: browser-requirements, browser-managed-artifacts-and-lifecycles, browser-glossary-terms, browser-tasks-milestones-and-releases, browser-complete-access (shared header), plus a configuration view specification; TASK-87 produces the mockup package for the configuration view and the record view layout before browser implementation starts.
- Revised by TASK-101, after TASK-86 has revised the shared backlog specification: browser-backlog-records-ui.md section 4 and the backlog list section of browser-tasks-milestones-and-releases.md, plus the mockup register and approval status; TASK-102 revises MOCK-0002 and obtains explicit user approval before TASK-103 implements the board. TASK-101 depends on TASK-86 so the two specification revisions do not edit the browser specification set at the same time.

### Deferred work

- TASK-100 (import this repository own backlog store into the native layout once) — deferred by user decision on 2026-11-11 until after Release 1.0; archived, outside m-14 and m-16, and not a release dependency. Restore it when the migration is scheduled, before this repository starts relying on Papyrus for its own records.

## Requirement change update (2026-11-11) — Backlog kanban board (REQ-006.013)

- Changed requirement: REQ-006.013 Present the Backlog as a Per-Milestone Kanban Board (Should have, validated, 12 acceptance criteria), from project-owner feedback; the REQ-006 group description now includes the backlog kanban presentation. Source requirement IDs: REQ-006.013, with REQ-001.002 (configured task statuses), REQ-002.001 and REQ-002.002 (task records and lifecycle), REQ-006.002, NFR-001.002, and NFR-005.001.
- Placement (user directive): milestone m-16, priority High, no work in m-14. m-16 is the release feature milestone and is still fully To Do, so the change extends the Release 1.0 delivery without reopening completed work and without redefining the release gate.
- Ordering rule applied: the board is a browser list surface with a write path, so the specification and the approved mockup precede the implementation (no unapproved mockup is implemented), and the implementation follows TASK-97, which deletes the legacy browser write helpers and leaves one shared write path. Building the board earlier would mean writing against the path TASK-97 removes.
- Tasks: TASK-101 (specification revision, ordinal 39035, depends on TASK-86) -> TASK-102 (MOCK-0002 revision and explicit user approval, ordinal 39045) -> TASK-103 (implementation, ordinal 39145, depends on TASK-97, TASK-101, TASK-102).
- Release gate: no change to TASK-73's dependency list. TASK-98 gains TASK-103, so the packaged build and the verification cover the board, and TASK-73's existing dependency on TASK-98 covers the new work transitively. TASK-99 is unaffected.
- Hard constraints encoded: the status change uses the existing browser write contract (ADR-0006 version token, ADR-0013 single writer) and introduces no new module, stack, or non-functional requirement; the specification drift in browser-backlog-records-ui.md section 4 and browser-tasks-milestones-and-releases.md section 3 is fixed before implementation; a materially revised mockup requires explicit user approval before implementation starts.
- Requirement coverage: REQ-006.013 -> TASK-101, TASK-102, TASK-103; REQ-001.002 (configured statuses drive the columns) -> TASK-101, TASK-103; REQ-002.002 (status change and field preservation) -> TASK-103; REQ-006.002 and NFR-001.002 (served in the local browser) -> TASK-103; NFR-005.001 (named browsers) -> TASK-103 tests.

## Requirement change update (2026-11-13) — REQ-001.006 cursor-driven init walkthrough

- Changed requirement: REQ-001.006 Interactive Init Walkthrough at `papyrus init <project_name>` (Should have, validated). The artifact-activation step must be a single cursor-driven list: exactly one highlighted selectable row starting on the first selectable artifact, Up/Down or `k`/`j` navigation, Space toggles the highlighted artifact, Enter confirms and continues, redraw after every keypress, navigation clamped at both ends, task/milestone/release shown as always active and never highlighted or toggled, typed space-separated tokens explicitly rejected, and end of input on the list confirms and continues. No NFR or integration impact; no new flag or environment variable.
- Classification: conformance gap, not a new requirement. TASK-91 delivered the space-toggle list as a static render plus a one-line token prompt (`src/init.rs`, `build_config_interactive`), so REQ-001.006 was only partly met when TASK-91 was closed Done. TASK-91 is not reopened and not re-scoped; the correction is a separate task so its other delivered behavior (non-empty-directory init, `--force`, missing-config failure) is not disturbed.
- Placement (no new milestone): TASK-104 goes to m-16, the feature milestone that delivered the init work, and reopens that milestone from 14/14 to 14/15 Done. Creating a milestone after m-16 would sit between the feature work and the release gate for a single CLI defect, and m-14 stays release-only per the 2026-11-11 decision.
- Ordering: ordinal 39085, immediately after TASK-91 (39080) and before TASK-92 (39090), so the init surface is corrected before the import and browser surfaces that were ordered after it. Dependencies: TASK-104 -> TASK-91 only. Priority High, matching every m-16 task; REQ-001.006 criticality (Should have) is not changed and order inside the milestone is expressed by ordinal.
- Verification: TASK-98 (packaging and verification) is already Done and is not re-scoped. TASK-104 carries its own pseudo-TTY test coverage and a real-TTY check, and the release gate TASK-73 must accept TASK-104 before closing. TASK-73 is currently archived in To Do; restoring it and adding the dependency is a status change owned by the user through `orchestrator`, so the dependency is recorded here rather than edited into the archived ticket.
- Specification drift found while checking coverage: `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` section 2 states only a space-toggle list without the cursor contract, section 1.2 says prompts are read line-by-line, and section 4.4 keeps `y`/`yes`/`n`/`no` activation prompts. Correlating the specification with the requirement is an acceptance criterion of TASK-104 rather than a separate specification task, because the text changes only describe the one implementation being corrected.
- Requirement coverage: REQ-001.006 -> TASK-85 (specification), TASK-91 (delivered walkthrough), TASK-104 (cursor-driven conformance). REQ-001.001 and REQ-001.007 remain covered by TASK-91 and are unchanged; BR-016 behavior (only activated types enabled) is unchanged.
- No other roadmap item changes: milestones, priorities, ordinals of existing tasks, and all other requirement coverage are unaffected.

## Requirement change update (2026-11-13) — artifact categories and config-defined document types

- Changed requirement set (validated, all Pass): REQ-007.001 Artifact categories and status model, REQ-007.002 Configure document types and slugs, REQ-007.003 Dispatch config-defined document commands, REQ-007.004 Generic content model, REQ-006.014 Browser views for config-defined types, and the REQ-001.002 list-valued configuration keys. Architecture: ADR-0016 to ADR-0020 and ADR-0022 to ADR-0024 accepted 2026-11-13. ADR-0021 (man pages) is Proposed and deferred by the owner; no man-page work is planned, and NFR-003.003 is recorded as a release risk.
- Roadmap entry (owner top priority): **Config-defined document types**, one ordered feature spanning registry/descriptors, command dispatch, configuration validation and repair, the generic document handler, search, browser, init defaults, tests, and documentation. Source requirement IDs: REQ-007.001, REQ-007.002, REQ-007.003, REQ-007.004, REQ-006.014, REQ-001.002. Rationale: the four current partial models (record descriptors, per-type clap enums, browser Category, hard-coded search keys) make a configured type unreachable; one registry-backed model is the only way to satisfy "add a type without code" consistently across every surface.
- Milestone decision: a single feature milestone, **m-17 Config-Defined Document Types and Artifact Categories**. It is not merged into m-14 (release-only) or m-16 (Release 1.0 feature scope, delivered): this is an artifact-model change after Release 1.0, and a second milestone would only split one coherent increment. No release milestone or release ticket is created; the release vehicle is an owner decision (see Open Questions).
- Task set (13 tasks, all High priority, all To Do): TASK-105 registry and descriptors; TASK-106 ADR-0019 correction; TASK-107 record-layer archive/status refactor; TASK-110 generic document handler; TASK-108 command-tree cutover, built-in table, help, static command and `nfreq` removal; TASK-109 fail-closed configuration validation and repair mode; TASK-111 list-valued configuration keys; TASK-112 init defaults and status seeding; TASK-113 search integration; TASK-114 browser registry-derived views and filters; TASK-115 business-rule updates; TASK-116 upgrade notes and README; TASK-117 cross-surface and packaged verification.

### Task order

| Ordinal | Task | Depends on | Assignee |
|---|---|---|---|
| 41000 | TASK-105 Introduce the artifact registry and type descriptors | — | rust-dev |
| 41005 | TASK-106 Correct the stale stricter-than-requirement notes in ADR-0019 | — | architect |
| 41010 | TASK-107 Refactor the record layer for document archive and status | TASK-105 | rust-dev |
| 41020 | TASK-110 Implement the generic document handler for every document type | TASK-105, TASK-107 | rust-dev |
| 41030 | TASK-108 Replace static document commands with registry-derived dispatch | TASK-105, TASK-110 | rust-dev |
| 41040 | TASK-109 Validate document-type configuration fail-closed with a repair mode | TASK-105, TASK-108 | rust-dev |
| 41050 | TASK-111 Support list-valued configuration keys in config set, get, and list | TASK-109 | rust-dev |
| 41060 | TASK-112 Update init defaults and seed document statuses | TASK-105, TASK-109 | rust-dev |
| 41070 | TASK-113 Integrate config-defined document types into search | TASK-107, TASK-110 | rust-dev |
| 41080 | TASK-114 Serve registry-derived browser views for every enabled document type | TASK-109, TASK-110 | rust-dev |
| 41090 | TASK-115 Update business rules for artifact categories and config-defined document types | TASK-108, TASK-109, TASK-110, TASK-111, TASK-112 | business-analyst |
| 41100 | TASK-116 Write upgrade notes and align the README with dynamic document commands | TASK-108, TASK-109, TASK-111, TASK-112, TASK-114, TASK-115 | tech-writer |
| 41110 | TASK-117 Verify the config-defined document type release end to end and in the package | TASK-110, TASK-112, TASK-113, TASK-114, TASK-116 | rust-dev |

### Ordering rationale

- The registry (TASK-105) is first because every later surface consumes its descriptor set. The ADR-0019 correction (TASK-106) is independent and can run in parallel; it is not allowed to change an architecture decision.
- The record layer (TASK-107) changes the storage and status contract before any handler, browser, or search code consumes it. The generic handler (TASK-110) then implements the baseline family against that contract.
- The command-tree cutover (TASK-108) follows the handler so `document::run` exists before the static variants are deleted. Static commands and the `nfreq` alias are removed in the same task; keeping them beside the dynamic tree would create duplicate names and an ambiguous help surface.
- Validation (TASK-109) follows the cutover because its reserved-name set is `cli::BUILTIN_COMMANDS`. List values (TASK-111) and init defaults (TASK-112) follow validation because both must validate the configuration they produce.
- Search (TASK-113) and browser (TASK-114) follow the record layer and handler. The browser also follows validation because it cannot start on an invalid configuration. They can proceed in parallel.
- Business rules (TASK-115) follow the delivered CLI/config/handler behavior; upgrade notes and README (TASK-116) follow the rules and the browser surface. Verification (TASK-117) is last and re-runs the cross-surface and packaged scenarios.

### Dependency and ordering rules

- All dependencies are inside m-17; no m-17 task depends on m-14, m-16, or a later milestone. The Release 1.0 gate and its archived release ticket are unaffected.
- The `--help` surface must not change before TASK-108. No earlier task may add, hide, or rename a top-level command; TASK-105 to TASK-107 and TASK-110 are internal.
- TASK-108 is the single cutover: the dynamic tree, the built-in table, generated help, the removal of the eleven static document variants, and the removal of `nfreq` land together.
- TASK-109 is the only validator; TASK-111 and TASK-112 may not add a second validation path. TASK-114 and the startup path consume its result.
- No task may reference `deprecated/`, a document restore, or a static per-type command after TASK-107 and TASK-108; TASK-117 enforces the no-drift rule.

### Coverage

- REQ-007.001 — TASK-105 (category and status models), TASK-107 (main status, archive operation, no restore, tolerant read), TASK-110 (status command), TASK-114 (status display and filter), TASK-115 (rules).
- REQ-007.002 — TASK-105 (document_types model), TASK-109 (slug/prefix/directory/status validation, duplicate keys, whole-map, repair), TASK-111 (list values), TASK-112 (default set without deploy), TASK-108 (slug dispatch and help), TASK-115 (BR-016).
- REQ-007.003 — TASK-108 (dynamic command family, help, exit 2, static removal), TASK-110 (baseline verbs and extensions), TASK-112 (default entries usable as commands), TASK-115 (BR-001), TASK-117 (CLI verification).
- REQ-007.004 — TASK-105 (generic descriptor), TASK-110 (generic content model and ID/path rules), TASK-114 (generic browser views), TASK-117 (end-to-end).
- REQ-006.014 — TASK-114 (list/detail/edit, status filter, visibility), TASK-113 (search visibility), TASK-117 (browser verification).
- REQ-001.002 — TASK-105 (config fields), TASK-109 (load/repair), TASK-111 (list-valued set/get), TASK-112 (init output).
- NFR-002.002 and NFR-004.002 — TASK-109 (reject-only validation and containment); NFR-001.003 — TASK-114 offline views; NFR-003.003 — no task, release risk recorded by TASK-116 and TASK-117.

### Risks and sequencing notes

- **Help surface:** changes only in TASK-108, in one commit sequence. Any documentation or test that asserts the old static command list must change there, not earlier.
- **NFR-003.003:** config-defined slugs have no man page. ADR-0021 is Proposed and deferred, so the release notes must not claim compliance for those slugs; TASK-116 records the gap and TASK-117 verifies the package without asserting such a page.
- **Upgrade data:** `deprecated/` folders become inert and are not migrated; users must move files and edit metadata by hand. This is accepted by REQ-005.007/ADR-0022 and must be prominent in the upgrade notes.
- **Configuration break:** whole-map `document_types` removes the implicit default merge, and `deploy` disappears from new and existing effective sets unless declared. The aggregated validator message and `init --force` are the recovery path.
- **Parallel work:** after TASK-109, TASK-111 and TASK-112 can run in parallel; TASK-113 and TASK-114 can run in parallel; TASK-115 can run alongside TASK-113/TASK-114 once TASK-108/109/110/111/112 are done.
- **Split candidates during refinement:** TASK-108 (tree + help + removal), TASK-110 (handler + typed delegation), TASK-114 (4.4k-line browser refactor), and TASK-117 (five test surfaces + packaging). TASK-109 can split validation from repair mode if the change set is too large for one review.

### Open questions

- Release vehicle: whether m-17 ships as Release 1.1 or folds into an open release. No release milestone or ticket is created until the owner decides.
- ADR-0021: man-page generation and the NFR-003.003 interpretation remain undecided. No man-page task is created; TASK-116 and TASK-117 only record the risk.
- Specification pages that describe static per-type commands and per-category browser pages will be stale after TASK-108 and TASK-114. No specification task was in the requested scope; decide whether to add one before implementation or accept the ADRs and business rules as the contract.
- REQ-006.014 leaves navigation grouping and display names open; TASK-114 follows ADR-0020's grouped default and slug-derived labels.

## Requirement change update (2026-11-14) - m-17 follow-ups and registry consolidation

- Inputs: the m-17 architecture checkpoint and the TASK-117 verification dispositions. Owner decisions: open a follow-up for G-5; refactor the code rather than amend ADR-0016/ADR-0017 for typed-module dispatch and the descriptor cache; plan the follow-ups and continue. No requirement, business rule, or ADR changed, so this is a planning update, not a requirement change.
- Roadmap entry: **Registry consolidation and release close**, one feature increment after m-17. Source requirement and rule IDs: BR-007, REQ-003.002, REQ-007.001 (release close); ADR-0016, ADR-0017 (dispatch and registry); REQ-001.002 and BR-016 (configuration surface); NFR-002.002 (input validation); ADR-0022 (archive terminology).
- Milestone decision: new **m-18 Registry Consolidation and Release Close**, not an extension of the completed m-17. m-17 is Done and its 13 tasks are not reopened; the work ranges over release, CLI dispatch, the record layer, configuration, and sanitizers, and splitting it would separate the release blocker from the refactor that makes the same surfaces consistent.
- Release coupling: no release milestone and no release ticket are created. There is no gate to add G-5 to, and the m-17 release vehicle (Release 1.1 or an open release) is still an owner decision. Until that decision, `release-ready` stays no for the m-17 increment and m-18 is the work that clears it.

### Task order

| Ordinal | Task | Priority | Depends on | Assignee |
|---|---|---|---|---|
| 41200 | TASK-118 Implement `papyrus release close` for the planned to released transition | High | - | rust-dev |
| 41210 | TASK-119 Route typed document commands through `document::run` with descriptor-driven dispatch | High | - | rust-dev |
| 41220 | TASK-120 Resolve record descriptors from the artifact registry as the single source | High | TASK-119 | rust-dev |
| 41230 | TASK-121 Remove the process-global descriptor cache and slug leaks | Medium | TASK-120 | rust-dev |
| 41240 | TASK-122 Include document type `default_status` in `config list` (G-4) | Medium | - | rust-dev |
| 41250 | TASK-123 Reject C0 control characters in titles (F-117-1) | Low | - | rust-dev |
| 41260 | TASK-124 Guard init defaults and registry drift and finish archive terminology | Low | TASK-120 | rust-dev |

### Ordering rationale

- Dependencies first. TASK-118, TASK-119, TASK-122, and TASK-123 have no dependency and are the first eligible set. TASK-120 follows TASK-119 because both rework the same command path and the record lookup is defined against the descriptor-driven dispatch. TASK-121 removes the leaked constructor that TASK-120 leaves behind. TASK-124 follows TASK-120 because the rename touches the same seven typed modules.
- User priority and release risk. G-5 is the only release-blocking item, so it holds ordinal 41200. The refactor pair follows because the owner directed a code change rather than an ADR amendment, and the remaining findings are ordered Medium then Low.
- Reviewable split. The refactor is split at a real boundary - CLI dispatch before record descriptor resolution - so each change set stays reviewable and behavior-preserving. TASK-119 explicitly leaves `descriptor_for_doc_type` and the cache in place; TASK-120 then removes the second lookup; TASK-121 removes the leak.
- No unusable increment. Nothing in m-18 depends on a later-milestone capability; every task is runnable against the delivered m-17 code.

### Parallel work

- TASK-118, TASK-119, TASK-122, and TASK-123 touch disjoint surfaces (release command and registry verb list; CLI tree, dispatch, document, typed modules; config flattening; sanitizers and shared validation tests) and can run concurrently.
- TASK-120 starts after TASK-119; TASK-121 after TASK-120; TASK-124 after TASK-120. TASK-121 and TASK-124 can then run in parallel.

### Coverage

- BR-007, REQ-003.002, REQ-007.001 - TASK-118 (the `close` verb, the fixed `planned -> released` transition, browser text).
- ADR-0016 - TASK-119 (every document dispatch through `document::run`, descriptor-built commands). ADR-0017 - TASK-119 (descriptor-driven commands), TASK-120 (single registry source for record descriptors), TASK-121 (registry-owned descriptors, no leaked identity).
- REQ-001.002, BR-016 - TASK-122 (`document_types.<slug>.default_status` inspectable through `config list`, validation and repair unchanged).
- NFR-002.002 - TASK-123 (C0 and DEL rejected in single-line input).
- ADR-0022 and the document lifecycle - TASK-124 (archive terminology, no behavior change) plus the init/registry default-set guard.

### Accepted and deferred items

- G-1 - `config set` cannot bootstrap a document type. Accepted as documented behavior; the direct-edit workaround and the error naming the missing field stand. No task.
- G-2, G-3 - upgrade-note and README wording findings already recorded by TASK-117. No task; they are documentation corrections outside this increment.
- NFR-003.003 man-page gap - remains owned by the deferred ADR-0021 decision. No man-page task is created, and no m-18 task may claim compliance for config-defined slugs.

### Open questions

- Release vehicle: whether m-17 plus m-18 ship as Release 1.1 or fold into an open release, and therefore whether a release gate must depend on TASK-118. No release ticket exists to record that dependency.
- Priority assumption: the owner gave a directive but no priority for the refactor and cache tasks. TASK-119 and TASK-120 are recorded as High to match the owner-directed ADR-0016/0017 conformance; TASK-121 is Medium because it has no user-visible effect; TASK-124 is Low as a drift guard. Adjust on request.

## Planning update (2026-11-15) - m-18 architecture checkpoint follow-ups

- Inputs: the m-18 architecture checkpoint and the owner decisions of 2026-11-15: **fix the ui-mockup code**, **amend ADR-0016 rather than refactor built-ins**, and **plan all follow-ups**. TASK-118..TASK-124 are Done. No requirement changed, so this is a planning update, not a requirement change.
- Roadmap entry: **Architecture checkpoint follow-ups: mockup archive and ADR alignment**, one correction increment after m-18. Source references: ADR-0022 decisions 2 and 8, ADR-0018 decision 3, ADR-0016, ADR-0017, ADR-0023 decision 2, REQ-005.007, REQ-005.008, REQ-007.001, REQ-001.001, NFR-002.002.
- Milestone decision: new **m-19 Architecture Checkpoint Follow-ups: Mockup Archive and ADR Alignment**. m-18 is Done and is not reopened; the work spans the mockup module, four ADRs, two specification pages, the domain model/glossary, and the sanitizers, so it does not belong in a completed milestone.
- Release coupling: no release milestone and no release ticket are created. The release vehicle for the m-17/m-18 work is still an owner decision, and these are corrections, not a shippable feature increment.

### Task order

| Ordinal | Task | Priority | Depends on | Assignee |
|---|---|---|---|---|
| 41300 | TASK-125 Route UI-mockup archive, supersede, and edit-with-move through the shared archive write-then-unlink path | High | - | rust-dev |
| 41310 | TASK-126 Amend ADR-0016, ADR-0017, ADR-0018, and ADR-0023 for the derive/builder split, cross-category verbs, and release archive policy | High | - | architect |
| 41320 | TASK-127 Correct browser and init specifications to the settled archive lifecycles and directory creation | Medium | TASK-126 | business-analyst |
| 41330 | TASK-128 Align the business object model and glossary lifecycles and remove the empty legacy deprecated folders | Medium | TASK-126 | domain-modeler |
| 41340 | TASK-129 Reject C0, DEL, and C1 control characters in sanitize_path_component and sanitize_path | Low | - | rust-dev |

### Ordering rationale

- Dependencies first. TASK-127 and TASK-128 mirror the amended ADRs, so they follow TASK-126; no other dependency exists.
- User priority among ready tasks. The owner listed the UI-mockup fix first and the ADR amendments second; both are High, so TASK-125 holds ordinal 41300 and TASK-126 ordinal 41310. The spec and domain-model corrections are Medium and follow their ADR dependency; the sanitizer parity is Low.
- No unusable increment. Every task is runnable against the delivered m-18 code; the ADR amendment records the exception the code fix preserves rather than enabling it, so the two High tasks can run concurrently.
- Reviewable split. Code, decision records, specifications, domain model, and sanitizer are separate ownership areas and separate reviews; no task mixes them.

### Parallel work

- TASK-125, TASK-126, and TASK-129 touch disjoint surfaces (`src/ui_mockup.rs` and its tests; `docs/architecture/adr`; `src/sanitize.rs`) and can run concurrently as the first eligible set.
- TASK-127 and TASK-128 start after TASK-126 and can then run in parallel.
- TASK-126 is a prerequisite, not a bottleneck: it changes only four ADR files and the ADR index.

### Coverage

- ADR-0022 decisions 2 and 8, ADR-0018 decision 3, REQ-005.007, REQ-005.008 - TASK-125 (mockup archive, supersede, edit-with-move, delete; HTML metadata and exit codes preserved). TASK-126 records the exception that TASK-125 keeps.
- ADR-0016, ADR-0017, ADR-0023 decision 2 - TASK-126 (derive/builder split and constraint 2, verbs across all categories, `ArchivePolicy::None` for releases, regenerated index).
- REQ-005.007, REQ-007.001, REQ-001.001 - TASK-127 (browser and init specifications, OQ-1 resolved).
- REQ-005.007, REQ-007.001 - TASK-128 (business object model and glossary lifecycles, empty legacy folder removal).
- NFR-002.001, NFR-002.002 - TASK-129 (path sanitizer control-character parity).

### Accepted and deferred items

- **NFR bounded-resource coverage stays with ADR-0017 decision 10.** The checkpoint recorded no new bounded-resource obligation and no new task is created; the registry remains plain in-process data with no cache, index file, background scan, or serialization format.
- Built-in command refactor - rejected by the owner in favor of amending ADR-0016. TASK-126 records the derive/builder split; no code task reworks built-ins.
- The `deprecated/` folders are removed only where empty; no record file is deleted and no migration or inventory command is introduced (ADR-0022 decision 12).

### Open questions

- Release vehicle: whether the m-17/m-18/m-19 work ships as Release 1.1 or folds into an open release remains an owner decision. No release ticket exists to record it.
- ADR-0021 and the NFR-003.003 man-page gap remain deferred; no task in m-19 claims man-page compliance.

## Planning update (2026-11-15) - m-19 architecture checkpoint follow-ups

- Inputs: m-19 is Done. Its architecture checkpoint found residual milestone drift: the ADRs amended by TASK-132 are correct, but the descriptor/record layer and two documents still describe an archived milestone. No new requirement is introduced; this is a correction and planning update.
- Roadmap entry: **Milestone Lifecycle Drift Follow-ups**, one correction increment after m-19. Source references: ADR-0017 constraint 3, ADR-0023 decision 2, ADR-0022 decision 5, REQ-003.001, REQ-006.013.
- Milestone decision: new **m-20 Milestone Lifecycle Drift Follow-ups**. m-19 is Done and is not reopened; the work spans code, a requirement, and a specification page.
- Release coupling: no release milestone and no release ticket are created. The release vehicle for the m-17/m-18/m-19/m-20 work remains an owner decision.

### Task order

| Ordinal | Task | Priority | Depends on | Assignee |
|---|---|---|---|---|
| 41400 | TASK-134 Align the milestone descriptor and record layer with the close-only milestone lifecycle | High | - | rust-dev |
| 41410 | TASK-135 Rewrite REQ-006.013 archived-milestone clauses to the close-only milestone lifecycle | Medium | TASK-134 | requirements-engineer |
| 41420 | TASK-136 Correct the milestone and release specification to the close-only milestone lifecycle | Medium | TASK-134 | business-analyst |

### Ordering rationale

- Dependencies first. TASK-135 and TASK-136 record the contract that TASK-134 makes true, following the m-19 pattern of documents after the authority they describe; no other dependency exists.
- User priority among ready tasks. The owner set the code correction High and the two document corrections Medium, so TASK-134 runs first and the two Medium tasks follow it.
- No unusable increment. TASK-134 alone prevents the invalid milestone auto-archive and keeps every legacy read path; the documents then describe the delivered contract rather than a pending one.
- Reviewable split. Code, requirement text, and specification text are separate ownership areas and separate reviews; no task mixes them.

### Parallel work

- TASK-134 is the first and only initially eligible task.
- TASK-135 and TASK-136 become eligible together after TASK-134 and touch disjoint documents, so they can run in parallel.

### Coverage

- ADR-0017 constraint 3, ADR-0023 decision 2, ADR-0022 decision 5, REQ-003.001 - TASK-134 (descriptor `ArchivePolicy::None`, no lifecycle archive location, legacy read/import preservation, relocation guard, regression test).
- REQ-006.013 - TASK-135 (browser board wording and business requirement index); REQ-003.001 is added to its traceability.
- REQ-003.001, REQ-003.002 - TASK-136 (milestone and release specification correction).

### Accepted and deferred items

- No new milestone feature, verb, configuration key, or record type is introduced; `archived` remains legacy data only.
- No legacy archived milestone file is migrated or deleted; it stays readable, editable, and deletable in place.
- The release vehicle for m-17 through m-20 remains an owner decision; no release ticket exists to record it.

### Open questions

- Release vehicle: whether m-17, m-18, m-19, and m-20 ship as Release 1.1 or fold into an open release remains an owner decision.

## Requirement change update (2026-11-15) - diagnostics, browser shell, and offline rendering

- Inputs: the validated requirement set for **REQ-001.008** (`papyrus check`), **REQ-006.015** (content width and collapsible navigation), **REQ-006.016** (uniform list layout), and **REQ-006.017** (offline math and Mermaid diagrams), plus the 2026-11-15 edits to REQ-001.002, REQ-001.004, REQ-002.003, REQ-005.007, REQ-006.013, REQ-006.014, and REQ-007.002. Architecture gate: ADR-0025 (KaTeX), ADR-0027 (vendored assets), ADR-0028 (browser shell, list, width, nav), and ADR-0029 (`check`) are Accepted; ADR-0026 (Mermaid) is **Proposed** and PlantUML is excluded.
- Roadmap entry: **Diagnostics, Browser Shell, and Offline Rendering**, one feature increment after m-20. Source requirement IDs: REQ-001.002, REQ-001.004, REQ-001.008, REQ-002.003, REQ-005.007, REQ-006.013, REQ-006.014, REQ-006.015, REQ-006.016, REQ-006.017, REQ-007.002; NFR-001.002, NFR-001.003, NFR-002.002, NFR-003.001, NFR-005.001, NFR-006.002.
- Milestone decision: new **m-21 Diagnostics, Browser Shell, and Offline Rendering**. m-20 is Done and is not reopened; the work adds a CLI command, a browser shell refactor, vendored assets, two offline renderers, and a package contract, so it cannot be attached to a completed correction milestone.
- Release coupling: no release milestone and no release ticket are created. The release vehicle for m-17 to m-21 remains an owner decision, and this increment changes package contents, notices, and the binary-size budget, so the eventual release gate must depend on TASK-149 and TASK-150.

### Requirement impact

- REQ-001.008 (new) - **covered**: TASK-137 scanners, tolerant self-load, `CommandOutput.exit_code`, exit 4, `--json`; TASK-138 `--fix`; TASK-139 reserved slug and help; TASK-140 README and business rules; TASK-150 end-to-end.
- REQ-006.015 (new) - **covered**: TASK-141 server-side `PageWidth`; TASK-144 nav collapse, persistence, `.app--full`, `.bleed`; TASK-149 and TASK-150 verification.
- REQ-006.016 (new) - **covered**: TASK-141 shared list layout and characterization fixtures; TASK-142 migration of every family, deletion of the second list path, marker test; TASK-148 specification.
- REQ-006.017 (new) - **covered**: TASK-143 asset pipeline; TASK-145 KaTeX; TASK-146 Mermaid (blocked on ADR-0026); TASK-148 specification; TASK-150 verification.
- REQ-006.013 (edited) - **covered**: TASK-147 adds the per-milestone `Clean up all` action, which the board does not implement today; TASK-144 makes the board uncapped. The existing board, release separators, and status control are unchanged.
- REQ-001.002, REQ-001.004, REQ-007.002 (edited) - **covered**: TASK-137 adds `check` to the tolerant repair surfaces and exit 4 as a findings outcome; TASK-139 reserves the `check` slug; TASK-140 documents both.
- REQ-005.007 (edited) - **covered**: TASK-137 never reads or inventories a `deprecated/` folder and reports no finding for it.
- REQ-006.014 (edited) - **covered**: TASK-142 gives every configured document type the shared list layout, including the status filter and archived exclusion.
- REQ-002.003 (edited) - **no change needed**: `papyrus task list` has no interactive mode and no `--plain` flag today, so the removal is already the delivered state; dependency readiness semantics are unchanged by this set.
- Partial or obsolete items: none.

### Task order

| Ordinal | Task | Priority | Depends on | Assignee |
|---|---|---|---|---|
| 41500 | TASK-137 Implement the read-only `check` diagnostic command with descriptor-driven scanners | High | - | rust-dev |
| 41510 | TASK-138 Implement `check --fix` duplicate-ID repair with refusals and confirmation | High | TASK-137 | rust-dev |
| 41520 | TASK-139 Reserve the `check` slug and publish the check command surface | High | TASK-137, TASK-138 | rust-dev |
| 41530 | TASK-140 Document `papyrus check` in the README and BR-001, BR-005, BR-016 | High | TASK-139 | tech-writer |
| 41540 | TASK-141 Introduce the browser shell, the PageWidth contract, and the shared list layout | High | - | rust-dev |
| 41550 | TASK-142 Migrate every artifact family list to the shared layout with characterization tests | High | TASK-141 | rust-dev |
| 41560 | TASK-143 Add the vendored browser asset table, digest manifest, and package notices | High | - | rust-dev |
| 41570 | TASK-144 Add collapsible navigation and content-width classes to the browser shell | High | TASK-141 | frontend-dev |
| 41580 | TASK-145 Render math formulas with vendored KaTeX in curated prose fields | High | TASK-141, TASK-143 | rust-dev |
| 41590 | TASK-146 Render Mermaid diagrams with strict security and source-text fallback | High | TASK-143, TASK-145; ADR-0026 accepted | rust-dev |
| 41595 | TASK-147 Add the per-milestone `Clean up all` board action | High | - | rust-dev |
| 41600 | TASK-148 Revise packaging and browser specifications for bundled assets and the shell contract | Medium | TASK-142, TASK-144, TASK-145, TASK-146 | business-analyst |
| 41610 | TASK-149 Verify packaged assets, offline rendering, and the binary budget | High | TASK-142, TASK-143, TASK-144, TASK-145, TASK-147 | rust-dev |
| 41620 | TASK-150 Verify the diagnostics, shell, and rendering release end to end | High | TASK-140, TASK-146, TASK-148, TASK-149 | rust-dev |

### Ordering rationale

- Dependencies first. `check` repairs only what the scanners report, so TASK-138 follows TASK-137 and the help text follows both. The family migration needs the shell and its fixtures, math needs the asset table, and diagrams need the math pre-pass and the ADR-0026 acceptance.
- User feature priority. The owner listed the `check` foundation first; both the `check` and browser tracks are High, so the `check` track holds the earlier ordinals and the browser and asset tracks start in parallel rather than waiting.
- No unusable increment. The shell and the shared layout land on the task family before the other families migrate; the asset table lands before any renderer; a family never ships through half a layout.
- Risk and readiness. The characterization fixtures are captured before the layout changes, `check` is read-only before it writes, and the ADR-0026 work is isolated behind an explicit gate instead of being scheduled optimistically.
- Tie-breaker. Not needed: every adjacent pair is separated by a dependency, a stated priority, or the milestone narrative above.

### Parallel work

- First eligible set: TASK-137 (CLI), TASK-141 (browser shell), TASK-143 (assets), and TASK-147 (`Clean up all`) touch disjoint surfaces and can start together.
- Second wave: TASK-138 and TASK-142 after their foundations; TASK-144 after TASK-141; TASK-145 after TASK-141 and TASK-143.
- Third wave: TASK-139 and TASK-146 (gated) ; TASK-149 once the lists, assets, nav, math, and board action land.
- Final wave: TASK-140 after TASK-139; TASK-148 after the browser and rendering work; TASK-150 last.
- Practical parallel streams: `check` (TASK-137 to TASK-140), browser (TASK-141, TASK-142, TASK-144, TASK-147), assets and rendering (TASK-143, TASK-145, TASK-146), documentation (TASK-148), verification (TASK-149, TASK-150).

### UI mockup approval gates

- TASK-141 intends no visual change; if refinement turns it into one, a mockup is approved or a waiver recorded.
- TASK-142 changes every list view, TASK-144 changes page chrome and the navigation control, and TASK-147 changes the board action and warning. Each requires an approved `ui-designer` mockup before merge or an explicit no-mockup waiver recorded in the task.

### Risks

- ADR-0026 gate. TASK-146, TASK-148, and TASK-150 are blocked until acceptance, and acceptance may change the fence alias, the `style-src` decision, the caps, or the sandboxed-iframe fallback. Re-plan the affected tasks after acceptance; no diagram code, asset, notice, or CSP change lands before it.
- `render.rs` regression surface. The shell and list refactor touches about 4.7k lines and every list route. Mitigation: characterization fixtures per family captured first and replayed after each migration step.
- `check --fix` semantic risk. References are never rewritten, so a repaired corpus can leave a reference pointing at the keeper. Mitigation: the prompt and the report list every affected reference, and the keeper is the oldest record, which references most likely target.
- `check` performance. No index or cache, so it is the slowest command on a large corpus. Accepted by ADR-0029; any cache is a separate decision.
- Dollar-sign heuristic. A record containing prices can be read as inline math. Mitigation: `\$` escape, non-space and no-newline rules, and source fallback.
- CSP relaxation. Mermaid needs `style-src 'unsafe-inline'`; a security review may refuse it and require the sandboxed-iframe fallback, which needs its own ADR.
- Package contract. Notices, digests, and the 12 MiB budget are release-gating; Mermaid alone is about 3.4 MiB. TASK-149 measures before TASK-150 closes.
- Documentation latency. TASK-148 waits on TASK-146. If the owner wants the accepted-state documentation earlier, split it at refinement into asset/shell and diagram halves.

### Coverage

- ADR-0025 decisions 2 to 10 - TASK-143 (assets), TASK-145 (pre-pass, placeholders, CSP, rendering call), TASK-148 (specification wording), TASK-149 (package verification).
- ADR-0026 decisions 1 to 9 - TASK-146, gated on acceptance; TASK-148 (diagram clauses after acceptance); TASK-150 (end-to-end).
- ADR-0027 decisions 1 to 10 - TASK-143 (asset table, digests, notices), TASK-146 (Mermaid entry), TASK-149 (package, budget, offline reference tests).
- ADR-0028 decisions 1 to 8 - TASK-141 (shell, list, `PageWidth`), TASK-142 (migration), TASK-144 (nav, CSS, accessibility), TASK-148 (specification).
- ADR-0029 decisions 1 to 11 - TASK-137 (tolerant load, scanners, exit code, `--json`, read-only primitive), TASK-138 (repair, refusals, prompt, concurrency), TASK-139 (reserved name, help), TASK-140 (documentation).
- REQ-001.008, REQ-006.015, REQ-006.016, REQ-006.017 - mapped task by task in the requirement impact section above; no unexplained gap.

### Accepted, deferred, and out-of-scope items

- No mockup preview route: ADR-0028 defines only the full-width class; a route needs its own ADR.
- No PlantUML: excluded by owner decision and shown as source with a note.
- No new Rust crate, no npm or Node step, and no asset fetched at build time.
- No record-body view: rendering is limited to the curated prose fields of existing record views.
- No `deprecated/` migration, no reference rewriting by `--fix`, and no browser surface for `check`.
- No release milestone, release ticket, release notes, or formal security audit in this milestone.
- Milestone `closed` records stay visible on the board; the close-only milestone lifecycle from m-20 is unchanged.

### Open questions

- Release vehicle: whether m-17 to m-21 ship as a single release and where its gate lives. No release ticket exists to record that decision.
- ADR-0026 acceptance and its open questions: the pinned Mermaid minor version, the promised diagram types, `style-src 'unsafe-inline'`, and whether the input caps stay fixed.
- Priority assumption: the owner supplied no priority for this set. Implementation tasks are recorded as High and the specification revision as Medium, matching the Should-have criticality of the four requirements and the m-15 to m-19 precedent. Adjust on request.
- TASK-148 split: keep it whole behind the ADR-0026 gate, or split it into accepted-state and diagram halves for earlier documentation.
