# REQ-04.002 Update, Supersede, and Archive Decisions

## User Story
As a project decision-maker, I want to update decision records and mark replaced or inactive decisions so that their current standing is clear.

## Group
REQ-04 Decision Records

## Criticality
Must have

## Description
The user can edit a decision's title, decision text, status, context, and consequences. A decision may be superseded by at most one decision of the same type; the supersession option cannot be combined with status editing. Decisions can be archived.

## Acceptance Criteria
- The documented edit options update title, decision, status, context, and consequences.
- A decision can be marked as superseded by at most one decision of the same type.
- `--superseded-by` cannot be used together with `--status`.
- `papyrus decision archive <id>` archives the selected decision.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → decision edit/archive (lines 370–394)
