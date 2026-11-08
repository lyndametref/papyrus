---
id: doc-1
title: Papyrus Feature Roadmap
type: specification
created_date: '2026-11-04T19:22:00Z'
updated_date: '2026-11-08T16:10:00Z'
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
- m-12: Final Cross-Type Integration and Record Metadata — shared record envelope (TASK-63), identity translation (TASK-64), CLI front-matter options (TASK-65), design review (TASK-66), metadata projection (TASK-67), document-record migration (TASK-68), task/milestone migration (TASK-69), legacy removal (TASK-70), Backlog.md round-trip gate (TASK-71), lifecycle page (TASK-72), final cross-type design review (TASK-60), unified search (TASK-18), cross-type browser (TASK-19), packaging (TASK-20).
- m-14: Release 1.0 — release ticket (TASK-73); closes only after TASK-20 is Done, build/tests pass, and the security audit is complete.

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
- m-12 TASK-66 -> TASK-59 (m-11); TASK-60 -> TASK-66; TASK-18 -> search tasks (TASK-31, TASK-35, TASK-39, TASK-44, TASK-48, TASK-56) plus TASK-14 (m-3) and TASK-68; TASK-19 -> TASK-60 plus browser tasks (TASK-23, TASK-24, TASK-25, TASK-26), TASK-67, and TASK-72; TASK-20 -> TASK-18, TASK-19, TASK-60, TASK-59, TASK-69, TASK-70, TASK-71, TASK-72.
- m-14 TASK-73 -> TASK-20 (m-12). No task depends on a later milestone.
- New m-12 internal order: TASK-63 -> TASK-64 -> TASK-65 -> TASK-67; TASK-63/TASK-65 -> TASK-68 -> TASK-69; TASK-67/TASK-69 -> TASK-70 -> TASK-71; TASK-66 -> TASK-67, TASK-72, TASK-60; TASK-60/TASK-67/TASK-72 -> TASK-19; TASK-20 last.

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
