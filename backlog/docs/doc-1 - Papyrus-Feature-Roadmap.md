---
id: doc-1
title: Papyrus Feature Roadmap
type: specification
created_date: '2026-11-04T19:22:00Z'
updated_date: '2026-11-10T20:17:00Z'
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
- m-15: Use Cases and Test Documentation — manage use cases, test cases, and test reports from the CLI (TASK-78), search (TASK-80), browser exposure (TASK-81), design review mockup (TASK-79), packaging and verification (TASK-82). Should-have document types (REQ-05.012/013/014); lifecycle active → deprecated; front-matter required. Placed before Release 1.0.
- m-14: Release 1.0 — release ticket (TASK-73); closes only after TASK-20 (m-12) and TASK-82 (m-15) are Done, build/tests pass, and the security audit is complete.

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
- m-14 TASK-73 -> TASK-20 (m-12) and TASK-82 (m-15). No task depends on a later milestone.
- New m-12 internal order: TASK-63 -> TASK-64 -> TASK-65 -> TASK-67; TASK-63/TASK-65 -> TASK-68 -> TASK-69; TASK-67/TASK-69 -> TASK-70 -> TASK-71; TASK-66 -> TASK-67, TASK-72, TASK-60, TASK-76; TASK-74 -> TASK-75; TASK-76 -> TASK-77; TASK-60/TASK-67/TASK-72/TASK-77 -> TASK-19; TASK-20 last.

## Requirement coverage

All REQ/NFR IDs remain traced via task references. TASK-17 marked obsolete; scope split into per-type CLI tasks. Deployment record types removed from scope (TASK-18/TASK-19 updated).

### Front-matter and Backlog.md interoperability (2026-11-08)

- REQ-05.010 — Record Metadata in YAML Front Matter: TASK-63 (parser, descriptors, key registry), TASK-65 (options and timestamps), TASK-68 (document records), TASK-69 (tasks/milestones and `release`), TASK-70 (no legacy metadata carriers).
- REQ-05.011 — Maintain Record Metadata Through All Commands: TASK-65 (create/update options), TASK-67 (browser save and `updated_date`), TASK-68 and TASK-69 (lazy per-record migration, managed types only, frozen bodies, legacy conversion with per-file diff), TASK-70 (single write path).
- REQ-06.003 — Describe Managed Artifacts and Their Lifecycles in the Browser: TASK-66 (design and approval), TASK-72 (implementation).
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views: TASK-66 (design and approval), TASK-67 (one projection, browser save, read-only timestamps).
- REQ-01.005 — Stable Record IDs and two ID spaces: TASK-64 (translation, reference translation, input rejection), TASK-67 (display ID in every projection).
- NFR-04.003 — Backlog.md task and milestone interoperability: TASK-64 (identity), TASK-69 (layout and additive keys), TASK-71 (two-direction round-trip gate), TASK-20 (packaged verification).
- REQ-05.008 — UI mockup metadata stays in HTML: TASK-63 and TASK-68 exclude HTML/mockups; no front matter is written to mockups.

### Project setup and configuration (2026-11-09)

- REQ-01.001 — Initialize a Papyrus Project: TASK-2 (baseline, done), TASK-75 (interactive walkthrough).
- REQ-01.005 — Stable Record IDs and two ID spaces: TASK-64 (translation), TASK-67 (display ID in every projection), TASK-74 (id_padding constraint).
- REQ-01.006 — Interactive Init Walkthrough: TASK-75.
- REQ-01.007 — ID Padding Constraint for File IDs: TASK-74.
- REQ-06.005 — Browser Skills Page for Agent Workflow Configuration: TASK-76 (design review), TASK-77 (implementation).

### Use cases and test documentation (2026-11-10)

- REQ-05.012 — Manage Use Cases: TASK-78 (CLI management), TASK-80 (search), TASK-81 (browser exposure), TASK-79 (design review mockup), TASK-82 (packaging and verification). Should-have; lifecycle active → deprecated; front-matter required (REQ-05.010).
- REQ-05.013 — Manage Test Cases: TASK-78, TASK-80, TASK-81, TASK-79, TASK-82.
- REQ-05.014 — Manage Test Reports: TASK-78, TASK-80, TASK-81, TASK-79, TASK-82.
- Open question: REQ-05.012/013/014 state management via the `req` command, which conflicts with BR-009 (req is business-requirements only). BR-018/019/020 recommend a dedicated per-type command (`use-case`, `test-case`, `test-report`) following the tech-spec pattern (BR-017). Resolution is tracked as AC #5 of TASK-78 and must be decided before implementation.

## Requirement change update (2026-11-06)

- REQ-05.009 now defines technical specifications as a managed documentation type with the `tech-spec` command (BR-017, README, domain model); REQ-05.007 includes technical specifications in the document lifecycle.
- TASK-55 unblocked: `blocked` label removed; documented command contract now exists. No milestone, dependency, or ordering change: TASK-55 remains in m-4 at ordinal 17500 depending on TASK-13.
- TASK-21 (m-11, Done) remains the domain-model/requirements definition task; TASK-55 does not depend on it to avoid a backward cross-milestone dependency.

## Requirement change update (2026-11-08)

- New requirements REQ-05.010, REQ-05.011, REQ-06.003, REQ-06.004, and NFR-04.003 are validated; ADR-0007, ADR-0008, and ADR-0009 define the staged delivery.
- Placement decision: the work is merged into m-12 rather than added as a post-final milestone. All m-12 tasks were To Do, m-12 is the declared final increment, and front matter changes every record file and both presentation surfaces. A new milestone after m-12 would invalidate TASK-20's packaging and TASK-60's final design review and would require a second packaging pass.
- Reorder within m-12: TASK-60 (final cross-type design review) moves from ordinal 30000 to 30095 and now depends on TASK-66, so the final review covers the front-matter and lifecycle UI. TASK-18 gains TASK-68; TASK-19 gains TASK-67 and TASK-72; TASK-20 gains TASK-69, TASK-70, TASK-71, and TASK-72. No task was moved across a milestone and no in-progress task was reordered.
- Release decision: m-14 `Release 1.0` with release ticket TASK-73, depending on TASK-20. Version 1.0 is a planning assumption; rename the milestone and ticket if the user selects another version.
- Specification prerequisite: `business-analyst`/`tech-writer` correct the browser specification pages that currently claim front-matter preservation before TASK-67/TASK-70 complete; this is noted inside those tasks, not a separate backlog task.

## Requirement change update (2026-11-09)

- New requirements REQ-01.006, REQ-01.007, and REQ-06.005 are validated; REQ-01.001, REQ-01.005, and the REQ-06 group were updated; REQ-07/08/09 were removed (canonicalized).
- Placement decision: merge into m-12 rather than a post-final milestone, consistent with the 2026-11-08 front-matter decision. Adding a milestone after m-12 would invalidate TASK-20 packaging and TASK-60 final design review and require a second packaging pass; TASK-20 and TASK-73 already gate the release.
- REQ-01.007 (TASK-74) and REQ-01.006 (TASK-75) are foundational and scheduled early in m-12 (ordinals 30005, 30006); TASK-75 depends on TASK-74. They do not reopen archived TASK-2 scope beyond the new behavior.
- REQ-06.005 is a browser surface: TASK-76 (design review, ordinal 30093) precedes TASK-60 so the final design review covers it; TASK-77 (implementation, ordinal 30098.5) depends on TASK-76, TASK-72, and TASK-67; TASK-19 gains TASK-77 and TASK-60 gains TASK-76.
- Deferred TASK-69 gaps (active relocation, no-drift, interop) remain tracked by TASK-70 and TASK-71 and are not reopened; new tasks do not duplicate them.
- TASK-67 is refined and in progress on branch task/TASK-67 and is not modified; new tasks depend on it rather than changing it.
- Release decision: m-14 `Release 1.0` (TASK-73) remains gated on TASK-20, which now includes TASK-74, TASK-75, and TASK-77; no new release ticket is required.

## Requirement change update (2026-11-10)

- New requirements REQ-05.012, REQ-05.013, and REQ-05.014 are validated (Should have; document types under REQ-05; lifecycle active → deprecated; front-matter required). BR-018, BR-019, and BR-020 define per-type behavior.
- Placement decision: unlike the 2026-11-08/09 front-matter and skills work, these are new document types not yet implemented anywhere, so they cannot be merged into the completed m-12. A new milestone m-15 "Use Cases and Test Documentation" is created and placed before Release 1.0. m-14 (TASK-73) now also depends on TASK-82 (m-15 packaging) in addition to TASK-20.
- The three types share one combined milestone and one task per pattern category (CLI management, search, browser exposure, design review mockup, packaging/verification) because they share the lifecycle and front-matter contract; per-type CLI command naming remains an open question (see above).
- No in-progress task is reordered; TASK-73 gains a dependency only.
