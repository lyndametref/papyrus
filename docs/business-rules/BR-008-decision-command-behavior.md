---
id: "BR-008:"
title: "Papyrus decision Command Behavior"
created_date: '2026-11-14T18:39:51Z'
updated_date: '2026-11-16T16:57:06Z'
labels: []
references: []
status: active
---
# BR-008: Papyrus decision Command Behavior

## Description

The `papyrus decision` command creates, lists, views, and edits decision records (ADR/BDR) and expresses their lifecycle only through the Nygard statuses `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`; there is no decision archive command and a decision carries no archive state.

## Rationale

Based on REQ-04.001 — Create and Inspect Decisions and REQ-04.002 — Update and Supersede Decisions, this rule exists because decisions belong to the decision category, whose lifecycle is the five Nygard statuses; an inactive or replaced decision is marked `superseded` or `deprecated`, so documenting an archive operation would invent a state the category does not have and contradict the command set.

## Applicability

Applies to every invocation of `papyrus decision` and to every decision record.

### Command surface

- `create` (`-t/--decision-type`, `-d/--decision`, `--context`, `--consequences`), `list`, `view`, and `edit` (`--title`, `-d`, `--status`, `--context`, `--consequences`, `--superseded-by`).
- `--superseded-by` cannot be combined with `--status`.
- No archive command exists, and a decision is never moved to a document `archived/` folder.

### Status model

- Exactly `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`.
- Decision statuses are distinct from document statuses: a decision is never `active` or `archived`, and a document is never `deprecated`.
- Decision status changes are performed through `edit --status` or supersession, not through a document status command.
