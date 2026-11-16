---
id: TASK-013
title: Manage decision records from the CLI
status: Done
assignee: []
created_date: '2026-10-31T22:26:00Z'
updated_date: '2026-11-05T14:17:00Z'
labels: []
milestone: MS-004
dependencies:
  - TASK-012
references:
  - docs/requirements/business/REQ-04.md
  - docs/requirements/business/REQ-04.001.md
  - docs/requirements/business/REQ-04.002.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/business-rules/BR-008-decision-command-behavior.md
documentation:
  - docs/specifications/decision-records-management.md
ordinal: 13000
---


## Description
Deliver the `decision` commands so a project contributor can create, list, view, edit, supersede, and archive decision records, with stable IDs, documented Nygard fields (Title, Status, Decision, Context, Consequences), fixed status vocabulary and transitions, and text-file persistence inside the project directory. Implements REQ-04, REQ-04.001, REQ-04.002 and the BR-008 command contract. Provides decision records later searched (TASK-14) and exposed in the browser (TASK-15). Full behavior and acceptance criteria are in docs/specifications/decision-records-management.md.

## Acceptance Criteria
- [ ] papyrus decision create <title> [-t <type>] [-d <decision>] [--context <text>] [--consequences <text>] creates a decision with Nygard fields and prints its generated ID.
- [ ] Default decision type is adr; built-in types adr and bdr; additional configured types supported; unknown or disabled type fails with exit code 4.
- [ ] Newly created decisions have initial status proposed.
- [ ] Decision IDs are stable <prefix>-<number> using configured prefix and id_padding; IDs are never reused across active and archived folders.
- [ ] papyrus decision list [-t <type>] lists active decisions, filtered by type when provided, ordered by ID ascending; archived and disabled-type decisions are hidden.
- [ ] papyrus decision view <id> prints the full persisted text; archived decisions are viewable; unknown ID exits 3.
- [ ] papyrus decision edit <id> updates only explicitly supplied fields among title, decision, status, context, and consequences.
- [ ] Status values and transitions match the documented graph (proposed->accepted/rejected, accepted->deprecated/superseded); invalid transition exits 4; setting the current status again succeeds.
- [ ] --superseded-by <id> marks the decision superseded by a same-type decision, sets status to Superseded by <id>, and cannot be combined with --status (exit 2); unknown target exits 3, type mismatch or self-reference exits 4.
- [ ] papyrus decision archive <id> moves the file to archived/; already-archived exits 4; unknown ID exits 3; archived decisions remain viewable and editable.
- [ ] Persistence is UTF-8 Markdown text in decision_directory (active) or its archived/ subfolder; archived/ is created on first archive.
- [ ] All user-controlled input is sanitized; unnormalizable input exits 4.
- [ ] No reads or writes occur outside the project directory; IO or boundary failure exits 1.
- [ ] Common CLI controls (--json, --no-color/NO_COLOR, help/version, exit statuses 0/1/2/3/4) behave as delivered by TASK-2.

## Definition of Done
- [ ] #1 Implementation compiles and unit/integration tests pass.
- [ ] #2 Tests cover create, list, view, edit, archive, status transitions, supersession, type filtering, sanitization, and project-directory boundary.
- [ ] #3 Feature specification docs/specifications/decision-records-management.md reviewed and aligned.
- [ ] #4 No direct backlog file edits; all changes via backlog CLI.
