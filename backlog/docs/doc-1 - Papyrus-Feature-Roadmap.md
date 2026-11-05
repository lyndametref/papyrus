---
id: doc-1
title: Papyrus Feature Roadmap
type: specification
created_date: '2026-11-04T19:22:00Z'
updated_date: '2026-11-05T18:48:00Z'
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
- m-12: Final Cross-Type Integration — unified search (TASK-18), cross-type browser (TASK-19), design review (TASK-60), packaging (TASK-20).

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
- m-12 TASK-60 -> TASK-59 (m-11); TASK-18 -> search tasks (TASK-31, TASK-35, TASK-39, TASK-44, TASK-48, TASK-56) plus TASK-14 (m-3); TASK-19 -> TASK-60 plus browser tasks (TASK-23, TASK-24, TASK-25, TASK-26); TASK-20 -> TASK-18, TASK-19, TASK-60, TASK-59.
- TASK-55 no longer depends on TASK-21 (m-11) to avoid a backward cross-milestone dependency; TASK-21 remains in m-11 per user decision.

## Requirement coverage

All REQ/NFR IDs remain traced via task references. TASK-17 marked obsolete; scope split into per-type CLI tasks. Deployment record types removed from scope (TASK-18/TASK-19 updated).
