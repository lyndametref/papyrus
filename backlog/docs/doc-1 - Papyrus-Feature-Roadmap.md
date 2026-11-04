---
id: doc-1
title: Papyrus Feature Roadmap
type: specification
created_date: '2026-11-04T19:22:00Z'
updated_date: '2026-11-04T19:40:00Z'
---
# Papyrus Feature Roadmap

Ordered feature increments (milestones). Each milestone delivers a usable, installable increment.

## Milestones

- m-0: Rust CLI Foundation and Requirements (completed) — baseline CLI, project init/config, business/NFR management.
- m-1: Search and Browser for Requirements (completed) — search and offline browser for requirements.
- m-2: Tasks, Milestones, Releases, and Discovery — task/milestone/release CLI, search, browser, packaging. (untouched)
- m-3: Decision Records — decision management, search, browser, packaging.
- m-6: UI Specifications — CLI (TASK-34), search (TASK-35), browser (TASK-24), design review (TASK-36), packaging (TASK-37).
- m-7: UI Mockups — CLI (TASK-38), search (TASK-39), browser (TASK-40), design review (TASK-41), packaging (TASK-42).
- m-8: Business Rules — CLI (TASK-43), search (TASK-44), browser (TASK-25), design review (TASK-45), packaging (TASK-46).
- m-9: Audit Records — CLI (TASK-47), search (TASK-48), browser (TASK-26), design review (TASK-49), packaging (TASK-50).
- m-11: Technical Specifications — domain model (TASK-21), CLI (TASK-55), search (TASK-56), browser (TASK-57), design review (TASK-58), packaging (TASK-59).
- m-5: Glossary Terms — CLI (TASK-30), search (TASK-31), browser (TASK-23), design review (TASK-32), packaging (TASK-33). (moved after m-11 per priority order)
- m-12: Final Cross-Type Integration — unified search (TASK-18), cross-type browser (TASK-19), design review (TASK-60), packaging (TASK-20).

## Removed

- m-4: Other Documentation (archived; TASK-17 obsolete).
- m-10: Deployment Records (removed; TASK-27, TASK-51, TASK-52, TASK-53, TASK-54 deleted).

## Dependency and ordering rules

1. Dependencies first (topological order).
2. User feature priority among ready tasks.
3. No unusable increments: read/view before write/entry.
4. Risk and readiness.
5. Tie-breaker: ask user.

## Cross-milestone dependencies

Each milestone's first task depends on the previous milestone's packaging task: m-3 entry TASK-13 -> TASK-12 (m-2); m-6 entry TASK-34 -> TASK-16 (m-3); m-7 entry TASK-38 -> TASK-37 (m-6); m-8 entry TASK-43 -> TASK-42 (m-7); m-9 entry TASK-47 -> TASK-46 (m-8); m-11 entry TASK-55 -> TASK-50 (m-9); m-5 entry TASK-30 -> TASK-59 (m-11); cross-document browser-view dependency chain decoupled (TASK-24, TASK-25, TASK-26 no longer depend on previous document type's browser task).

## Requirement coverage

All REQ/NFR IDs remain traced via task references. TASK-17 marked obsolete; scope split into per-type CLI tasks. Deployment record types removed from scope (TASK-18/TASK-19 updated).
