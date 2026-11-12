---
name: business-process-diagramming
description: 'Document end-to-end business workflows as Mermaid flowcharts in docs/business-processes/, covering actors, steps, decisions, and business rules without technical detail.'
user-invocable: true
disable-model-invocation: false
---

# Business Process Diagramming

## Outcome

Produce business process diagrams that document end-to-end workflows using Mermaid flowchart syntax. Stored in `docs/business-processes/` with naming `<process-name>-process.md`. The business process do not document any technical aspects, only the business view of the users or the business rules which are applied in the process.

## When to use

For documenting cross-actor workflows, business processes, user journeys, approval flows, data movement between systems, or any multi-step business activity. NOT for object lifecycle states (use state-diagram-creation for that) or architecture diagrams.

## Example

See `references/example-mating-cycle-process.md` for a complete worked example covering all required sections.

## Format

- File location: `docs/business-processes/`
- Naming: `<kebab-case-process-name>-process.md`
- Structure per file:
  - H1: `<Process Name> — Business Process`
  - H2: Overview paragraph
  - H2: Participants table (Actor | Role )
  - H2: Process Steps table (Step | Actor | Action | Notes)
  - H2: Diagram section with Mermaid flowchart 
  - H2: References 
- Use `flowchart LR` for left-to-right flow direction.
- Each participant (actor, role) gets one `subgraph` (lane).
- Every process has exactly one start event and at least one end event.
- Gateways must be labeled with the condition or question.
- Sequence flows between activities must be labeled with action verbs in imperative form (e.g. "Submit request", "Approve", "Notify").
- Use `-.->` for message flows that cross participant boundaries.
- Pool/Lane order should follow the flow: leftmost/topmost is the initiator.
- Use subgraph nesting for pools containing lanes where applicable.

## Procedure

1. Identify the business process from user request or documentation in `docs/` 
2. Read relevant requirements, specs, domain model, and business rules
3. Identify all participants (actors, systems, roles)
4. Map the flow: start event → activities/tasks (with gateways for decisions/forks) → end event(s)
5. Draft the Mermaid flowchart 
6. Present the draft to the user to ask if ok.
7. Iterate on user feedback, if needed ask questions.
8. Save to `docs/business-processes/<name>-process.md`

## Resume / interruption

If interrupted, re-read the source docs and continue from the last completed step; the process file is the source of truth once saved.

## Completion checks

- File exists at `docs/business-processes/<name>-process.md`
- All participants are identified and assigned to pools/lanes (`subgraph`)
- All steps are documented with actor, action, and system columns filled
- There is exactly one start event and at least one end event
- Every gateway has a labeled condition
- Cross-participant interactions use message flows (`-.->`)
- Sequence flows are labeled with imperative action verbs
- The diagram covers the full end-to-end process
- Relevant business rules are referenced
- Traceable to requirements
- Make sure no system or interface is cited.
