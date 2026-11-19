---
id: TASK-197
title: "Remove dead lifecycle code and stale route comment"
created_date: '2026-11-18T21:35:27Z'
updated_date: '2026-11-19T10:54:25Z'
labels: []
references:
  - AUD-107
  - AUD-108
  - ADR-034
  - TECH-080
  - REQ-06.018
status: Done
assignee:
  - rust-dev
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 54
documentation:
  - src/browser/routes.rs
type: implementation
---

## Description
Correction for AUD-107 finding 4 and AUD-108 finding 5 (low): delete routes::lifecycle_description, which has no remaining caller; remove the Presentation.lifecycle field and its 17 curated strings, now unreachable, relocating that content only if outside-browser documentation still needs it; correct the stale doc comment at line 56 of the browser routes module that still lists config and skills. Leave no remaining reference to the removed items. Done when cargo fmt, cargo clippy --all-targets -- -D warnings, and cargo test --lib pass.
## Comments
- **rust-dev**: Evidence (rust-dev): AUD-107 finding 4 and AUD-108 finding 5 corrected. Removed routes::lifecycle_description (no caller), the Presentation.lifecycle field and its curated lifecycle strings (unreachable data; no relocation needed because outside-browser lifecycle documentation already lives in UI-010 and UI-012), and corrected the stale PageRoute::Other doc comment that still listed config and skills; also dropped stale lifecycle wording from the NavGroup and heading docs. No remaining reference to lifecycle_description or the lifecycle field in the source or tests. Behavior unchanged: PRESENTATION keeps slugs, labels, headings, short nouns, notes, legacy keys, order, API aliases, not-found templates, aggregate groups, and list slots. Files changed: src browser routes module only. Checks: cargo fmt pass; cargo clippy --all-targets -D warnings pass with 0 warnings; cargo test --lib 426 passed 0 failed; cargo test --test browser_cli 181 passed 0 failed. Branch: task TASK-197. Commit: da85fa1.
