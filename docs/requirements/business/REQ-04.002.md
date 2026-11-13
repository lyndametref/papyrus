# REQ-04.002 Update and Supersede Decisions

## User Story
As a project decision-maker, I want to update decision records and mark replaced decisions so that their current standing is clear.

## Group
REQ-04 Decision Records

## Criticality
Must have

## Description
The user can edit a decision's title, decision text, status, context, and consequences. A decision may be superseded by at most one decision of the same type; the supersession option cannot be combined with status editing. Decision records have no archive command and no archive state: their lifecycle is expressed only by the five Nygard statuses (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`), and `papyrus decision archive` is not part of the command set. An earlier decision or an inactive decision is marked `superseded` or `deprecated` instead of being archived.

## Acceptance Criteria
- The documented edit options update title, decision, status, context, and consequences.
- A decision can be marked as superseded by at most one decision of the same type.
- `--superseded-by` cannot be used together with `--status`.
- `papyrus decision archive` does not exist; decisions have no archive state.
- An inactive or replaced decision is expressed as status `superseded` or `deprecated`, not as an archived record.
- Decision status values are exactly `proposed`, `accepted`, `rejected`, `deprecated`, and `superseded`.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → decision edit (lines 370–394)
- Project-owner decision 2026-11-13: `papyrus decision archive` is dropped; decisions are managed only through their five Nygard statuses, and document `archived` does not apply to decisions.
- REQ-04.001 — Create and Inspect Decisions
- REQ-05.007 — Manage Document Archive State and Deletion (documents only)
- BR-008 — decision Command Behavior (needs revision: archive removed)
