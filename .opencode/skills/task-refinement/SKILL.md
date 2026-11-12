---
name: task-refinement
description: 'Audit, clarify, and complete a single backlog ticket by removing ambiguity, filling missing structure, and aligning it with specifications. Uses the backlog CLI for all ticket modifications.'
argument-hint: 'Provide a ticket path and goal (e.g. "backlog/tasks/task-20 - P3-API-for-Individuals-and-Observations.md: clarify scope and finish acceptance criteria").'
user-invocable: true
disable-model-invocation: false
---

# Backlog Refinement

## Outcome

A single backlog ticket becomes execution-ready with a clear description, explicit assumptions, tracked dependencies, concrete acceptance criteria, and a well-defined definition of done — all aligned with relevant project documentation.

## Prerequisites

- **backlog-md skill (REQUIRED)** — This skill relies on the backlog CLI tool for ALL ticket operations. You MUST load the `backlog-md` skill before performing any backlog read or write operations.

## When To Use

- A ticket is vague, ambiguous, or missing key sections (description, AC, assumptions, dependencies, DoD).
- A ticket needs to be checked for alignment with `docs/` specifications before implementation.
- A feature needs an implementation specification and an execution-ready ticket that link the applicable requirements and specifications.
- You need to make a single ticket execution-ready without decomposing it into sub-tasks.

Do not use this skill for:
- Creating new tickets from scratch (use the backlog CLI directly).
- Decomposing a ticket into implementation sub-tasks or sprint planning.
- Refining multiple tickets in one pass — focus on one ticket at a time.
- Editing tickets without using the backlog CLI (never edit ticket files directly) — you MUST use the `backlog-md` skill for all CLI operations.

## Procedure

1. **Read the target ticket** — Use the `backlog-md` skill to read the full ticket content. Identify missing or weak sections: scope, assumptions, dependencies, acceptance criteria, and definition of done.

2. **Cross-check relevant requirements** — Review `docs/` files and any linked backlog artifacts referenced by the ticket. Note any specifications the ticket must conform to.

   For feature work, use `docs/specifications/` as the canonical location for the feature specification. Link relevant business rules and domain attribute lists/state diagrams in their canonical documentation locations.

3. **Check definition of done** — Determine whether the standard DoD applies or if the ticket needs a customized DoD. If the latter, clarify and define specific done criteria. If unsure, ask the user.

4. **Interact with the user** — Present findings and ask targeted questions to clarify intent, assumptions, and ambiguous wording. Do not mark the ticket execution-ready until material behavior and verification questions are answered:
   - What exactly needs to be built or done?
   - What are the boundaries (in scope / out of scope)?
   - What dependencies exist?
   - How will completion be verified?

5. **Update the ticket via the backlog CLI** — Apply all refinements using the backlog CLI tool with commands from the `backlog-md` skill. Never edit ticket files directly. Ensure the refined ticket has:
   - A clear, unambiguous description
   - Explicit assumptions (if applicable)
   - Tracked dependencies (if applicable)
   - Concrete, testable acceptance criteria
   - A clear definition of done
   - All implementation-critical scope and behavior in the ticket itself; links to requirements and specifications explain context and rationale and are not substitutes for ticket content
   - Direct references to the relevant requirement IDs and specification artifacts

6. **Return a summary** — Provide a summary of changes made, any assumptions that were documented, open questions that remain, and suggested next actions.

## Resume / interruption

If interrupted, re-read the target ticket and linked docs, then continue from the last completed step; do not re-apply already-made edits.

## Example

See `references/` for a before/after refinement demonstration:
- `task-rough.md` — a vague, ambiguously scoped ticket before refinement
- `task-refined.md` — the same ticket after refinement with clear scope, AC, assumptions, and dependencies

## Completion Checks

- The target ticket has a clear, unambiguous description with concrete scope.
- Acceptance criteria are specific, testable, and numbered.
- Assumptions are explicitly stated (or noted as "none").
- Dependencies are tracked (or noted as "none").
- Definition of done is present and appropriate for the ticket.
- All changes were made via the backlog CLI — no direct file edits.
- The `backlog-md` skill was loaded (step 0) and used for all backlog operations.
- Any requirements invented without traceable documentation sources are flagged as open questions.
- No material ambiguity remains that would force a developer to seek product clarification before beginning implementation.

## Decision Rules

- If a ticket's scope is too broad for a single execution unit, flag it rather than expanding it — recommend splitting but do not perform the split.
- If a ticket references specifications that no longer exist or have changed, flag the inconsistency and ask the user how to proceed.
- If critical information cannot be found in documentation or user input, add it as an open question — never invent untraceable requirements.
- If the standard DoD is insufficient for the ticket type, customize it and explain the custom criteria.
- If the user asks for implementation decomposition, decline and explain this skill only refines the ticket structure.

## Quality Bar

- Descriptions are concise, unambiguous, and self-contained.
- Acceptance criteria are concrete and verifiable (pass/fail), not vague aspirations.
- Assumptions are explicit and noted as assumptions (prefixed with "Assumption:").
- Every claim in the refined ticket is traceable to either project docs, user input, or is marked as an open question.
- No implementation details, code snippets, or technical design decisions appear in the ticket.
- The ticket is in a state where a developer could pick it up and start working without further clarification.

## Output Template

After refining a ticket, provide this summary:

- **Action**: refine
- **Target ticket**: <ticket path or ID>
- **Summary of changes**: <concise list of edits made>
- **Assumptions documented**: <list>
- **Open questions**: <list of unresolved items needing stakeholder input>
- **Suggested next actions**: <e.g., "Assign stakeholder to answer open questions before sprint planning">
