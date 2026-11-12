---
name: backlog-md
description: 'Manage project tasks, docs, and decisions using the Backlog.md CLI tool. Create, edit, search, and track tasks with full metadata support.'
argument-hint: 'Optional: create task, edit task, search, or manage docs.'
user-invocable: true
disable-model-invocation: false
---

# Backlog.md Task Management

## Outcome
Efficiently manage all project tasks, status, and documentation using the Backlog.md CLI, ensuring all project metadata remains fully synchronized and up-to-date.

## When To Use
- Create, view, list, or search tasks
- Update task status, assignees, labels, or priority
- Manage Acceptance Criteria or Definition of Done checklists
- Navigate tasks (Kanban board, web UI)
- Manage project docs and architectural decision records (ADRs)
- Search across tasks, docs, and decisions

## Core Capabilities
- **Task Management**: Create, edit, assign, prioritize, and track tasks with full metadata
- **Search**: Fuzzy search across tasks, documents, and decisions with `backlog search`
- **Priority ordering**: List tasks sorted by priority with `backlog task list --sort priority --plain`
- **Ordinal ordering**: List tasks sorted by ordinal with `backlog task list --sort ordinal --plain`
- **Ready tasks**: Filter to dependency-ready tasks with `backlog task list --ready --plain` (tasks whose dependencies are all completed)
- **Execution sequences**: Compute dependency-aware execution order with `backlog sequence list --plain` (respects task dependencies)
- **Paging/counting**: `--max-count <n>`, `--skip <n>`, `--count`, `--limit <n>`; `--json --watch` for live updates
- **Archive/restore/delete**: `backlog task archive|restore|delete <id>` (archive first, then delete)
- **Task view**: `backlog task <id> --plain` shows dependency graph, readiness, checklists, comments
- **Docs/decisions**: `backlog doc ...`, `backlog decision ...` (create/update/archive/restore/delete)
- **Maintenance**: `backlog doctor`, `backlog cleanup`, `backlog overview`, `backlog browser`
- **Agent instructions**: `backlog instructions ...`, `backlog agents --update-instructions`
- **Acceptance Criteria**: Granular control with add/remove/check/uncheck by index
- **Definition of Done checklists**: Per-task DoD items with add/remove/check/uncheck
- **Board Visualization**: Terminal-based Kanban board (`backlog board`) and web UI (`backlog browser`)
- **Git Integration**: Automatic tracking of task states across branches
- **Dependencies**: Task relationships and subtask hierarchies
- **Documentation & Decisions**: Structured docs and architectural decision records
- **Export & Reporting**: Generate markdown reports and board snapshots
- **AI-Optimized**: `--plain` flag provides clean text output for AI processing

## Key Understanding
- Tasks live in `backlog/tasks/` as `task-<id> - <title>.md` files
- Interact via CLI only: `backlog task create`, `backlog task edit`, etc.
- Use `--plain` flag for AI-friendly output when viewing/listing
- Default task list order: ordinal, then priority, then ID. `--sort priority` prints one flat list; `--ready` filters to dependency-complete tasks; `--json` returns a flat array.
- Never bypass the CLI — it handles Git, metadata, file naming, and relationships

---

# CRITICAL: NEVER EDIT TASK FILES DIRECTLY

| DO | DON'T |
|----|-------|
| Use `backlog task edit` | Edit markdown files directly |
| Use `backlog task create` | Manually change checkboxes |
| Use `backlog task edit <id> --check-ac <n>` | Add/modify text in task files directly |
| Use CLI for all operations | Use slash commands like `/create-task` (not supported) |

Direct file editing breaks metadata synchronization, Git tracking, and task relationships.

---

## 1. Source of Truth & File Structure

- Task files: `backlog/tasks/task-<id> - <title>.md`
- Drafts: `backlog/drafts/`
- Docs: `backlog/docs/`
- Decisions: `backlog/decisions/`
- Assets: `backlog/assets/` (images, diagrams)

Always use `--plain` flag for AI-friendly output. Create/update docs via `backlog doc create` / `backlog doc update`.

---

## 2. Task Format (Read-Only Reference)

```markdown
---
id: task-42
title: Add GraphQL resolver
status: To Do
assignee: [@sara]
labels: [backend, api]
modified_files:
  - src/server/api.ts
---

## Description
Brief explanation of the task purpose.

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 First criterion
- [x] #2 Second criterion (completed)
- [ ] #3 Third criterion
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Tests pass
- [ ] #2 Docs updated
<!-- DOD:END -->

## Implementation Plan
1. Research approach
2. Implement solution

## Implementation Notes
Progress notes captured during implementation.

## Final Summary
PR-style summary of what was implemented.
```

---

## 3. Defining Tasks

### Title (one line)
Clear brief title summarizing the task.

### Description (the WHY)
Concise summary of purpose and goal. No implementation details.

### Acceptance Criteria (the WHAT)
Outcome-oriented, testable, clear, complete, user-focused. Mark as complete via `backlog task edit <id> --check-ac <n>`.

### Definition of Done
Per-task DoD checklist. Configurable defaults in project config.

### Task Requirements
- Atomic (single PR scope) and testable
- Never reference future tasks (id < current only)
- Independent — no dependencies on future work
- Structure for AI coding agents — clear language, unambiguous ACs, self-contained context

### Breakdown Strategy
1. Identify foundational components first
2. Create in dependency order
3. Each task delivers independent value
4. Avoid blocking dependencies


---

## 7. Search

```bash
backlog search "topic" --plain
backlog search "topic" --type task --plain
backlog search "api" --status "To Do" --plain
backlog search --modified-file src/api.ts --plain
```

Fuzzy matching across titles, descriptions, content, and modified_files. Always use `--plain`.

## 7b. Execution sequences (dependency-aware order)

Compute the order in which tasks can be executed given their dependencies:

```bash
backlog sequence list --plain
```

Use this to determine the next task to implement when dependencies matter. Combine with `backlog task list --sort priority --plain` for priority ranking.

---

## 8. Quick Reference: DO vs DON'T

| Action | DO | DON'T |
|--------|----|-------|
| View task | `backlog task 42 --plain` | Open .md directly |
| List tasks | `backlog task list --plain` | Browse folder |
| Highest priority | `backlog task list --sort priority --plain` | Browse folder |
| Ordinal order | `backlog task list --sort ordinal --plain` | Guess order |
| Ready tasks (deps done) | `backlog task list --ready --plain` | Guess order |
| Execution sequence | `backlog sequence list --plain` | Guess order |
| Check AC | `backlog task edit 42 --check-ac 1` | Change checkbox in file |
| Add notes | `backlog task edit 42 --notes "..."` | Type into .md file |
| Add summary | `backlog task edit 42 --final-summary "..."` | Type into .md file |
| Change status | `backlog task edit 42 -s Done` | Edit frontmatter |
| Add AC | `backlog task edit 42 --ac "New"` | Add `- [ ]` to file |

---

## Remember: The Golden Rule

**If you want to change ANYTHING in a task, use `backlog task edit`.**
**Use CLI to read tasks, exceptionally READ directly, never WRITE.**

See [the reference guide](references/REFERENCE.md) for the complete command tables, examples, formatting guide, and multi-line input patterns.
