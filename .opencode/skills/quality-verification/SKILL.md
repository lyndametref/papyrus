---
name: quality-verification
description: 'Verify a completed backlog ticket against its acceptance criteria, definition of done, and implementation plan. Detects scope creep, gathers evidence from the repository, and produces a completion report with a close/incomplete decision. Use when validating that an implemented ticket is ready to close.'
argument-hint: 'Provide a ticket name or path (e.g. "task-0029" or "backlog/tasks/task-0020").'
user-invocable: true
disable-model-invocation: false
---

# Quality Verification

## Outcome

One backlog ticket receives an explicit verification decision: `Ready to close` or `Incomplete`, backed by a per-criterion completion table with evidence and a scope-creep check. Verification is read-only: the caller (`orchestrator`) obtains explicit user approval and closes the ticket via the backlog CLI.

## When To Use

- A ticket claims to be implemented and must be validated before closing.
- The user asks to verify, gate, or review a specific ticket's completion state.
- Scope creep must be checked against the ticket's implementation plan and acceptance criteria.

Do not use this skill for:

- Verifying multiple tickets in one pass — focus on one ticket at a time.
- Producing implementation, code, or configuration changes.
- Editing files or closing tickets — verification is read-only; report to the caller.

## Prerequisites

- **backlog-md skill (REQUIRED)** — Load the `backlog-md` skill before any backlog read operation. All ticket reads go through the backlog CLI.

## Inputs

- Ticket name or path provided by the user.
- Repository files where the ticket's changes should have been made.

## Constraints

- DO NOT invent requirements or acceptance criteria not traceable to the target ticket.
- DO NOT produce any implementation, code, or configuration.
- DO NOT modify any file, ticket status, checklist, or content; verification is read-only.
- DO NOT edit ticket files directly — read via the backlog CLI only.
- **Scope creep detection**: Flag any feature, component, or code change that goes beyond what is documented in the ticket's implementation plan, acceptance criteria, and Definition of Done.
- Do not ask the user for approval and do not close the ticket; the caller owns approval and closure.
- Do not create persistent review reports or other long-lived files; provide the report in the response. If a temporary artifact is necessary, use only project-local `./.tmp/` and delete it before finishing, including when verification is blocked.

## Procedure

1. **Read the target ticket** — Use the backlog CLI to read the full ticket. Extract scope, acceptance criteria, dependencies, definition of done, and the implementation plan.
2. **Gather evidence** — Search the repository for the files, features, and configuration the ticket's changes should have produced. Cross-map the ticket's acceptance criteria and implementation to relevant linked requirement and feature-specification behavior. The ticket's acceptance criteria remain the closure criteria; if a linked source requires behavior absent from the ticket, report the mismatch and mark verification incomplete for task-refiner alignment. Do not silently expand scope or invent acceptance criteria.
3. **Scope creep check** — Cross-reference the implementation plan against the actual changes. Verify that every file, feature, component, and configuration change is traceable to a documented AC, DoD item, or implementation-plan entry. Flag any additions not justified by the plan.
4. **Validate each acceptance criterion** — Mark each AC pass/fail/inconclusive with evidence references. If a criterion cannot be assessed unambiguously, mark it inconclusive and report the ambiguity to the caller.
5. **Rerun tests** — Execute the relevant test suite (via bash) for the changed code (e.g., `./gradlew test`, `npm test`, `pytest`) and record pass/fail results. Do not rely on reported evidence alone. If tests fail, cannot run, or no tests exist, mark the ticket Incomplete and report the failure/gap; do not accept "manual verification" as sufficient unless the ticket explicitly allows it.
6. **Verify NFR coverage** — If the ticket references non-functional requirements, verify each is addressed (performance, security, usability, etc.) with evidence; flag any unverified NFR.
7. **Validate definition of done** — Mark each DoD item pass/fail/inconclusive with evidence references. If a point cannot be assessed unambiguously, mark it inconclusive and report it to the caller.
8. **Report** — Produce the Decision and Completion summary table (see Output Format).
9. **Report to caller** — Return the verification report to the caller (`orchestrator`). Do not ask the user for confirmation and do not close the ticket; the caller requests approval and performs closure.

## Output Format

- **Decision**: `Ready to close` (pending user approval) or `Incomplete`.
- **Completion summary table**: one row per acceptance criterion with columns:
  - acceptance criteria id and title
  - completion: ok or not ok (with color)
  - Evidences/Findings: completion evidence / what is missing
- **Scope creep findings**: any unjustified changes (or "none").
- **Open questions**: ambiguous criteria or missing evidence requiring caller input before approval.

The verifier only reports. The caller prints this report separately, requests approval with a short question, and closes the ticket only after explicit approval.

## Completion Checks

- Every AC marked pass/fail/inconclusive with evidence.
- Tests rerun and passing (or marked Incomplete with failure/gap details).
- NFRs referenced by the ticket verified (if any).
- DoD items marked pass/fail/inconclusive with evidence.
- Scope creep checked; unjustified changes flagged.
- Decision reported; no status change or closure performed; the caller obtains user approval before closure.

## Resume / interruption

Verification is a single pass. If interrupted, re-run from the ticket read; no persistent state is kept.

## Examples

No example files are currently bundled. Follow the Output Format above.
