---
name: release-orchestration
description: 'Close a release: verify all release tasks are Done, run release-level build and test checks, delegate the mandatory end-of-release security audit to security-officer, compile the release report, and record completion only after explicit user approval.'
argument-hint: 'Optionally provide the release milestone or version (e.g. "Release 1.0"). If omitted, use the active release milestone.'
user-invocable: true
disable-model-invocation: false
---

# Release Orchestration

## Outcome

One release is verified and closed. Every included task is `Done`, release-level checks pass, the `security-officer` audit is complete, the release report is presented, and the release is recorded complete only after explicit user approval. A security `Block` decision stops the release.

## When To Use

- The user asks to "release", "close the release", "ship version X", or "run the release checklist".
- All tasks of a release milestone are expected to be Done and the release must be gated.

Do not use this skill for:

- Planning the release scope or milestones (use `planner`).
- Delivering remaining milestone work (use `milestone-orchestration`).
- Auditing security outside a release (use the `security-officer` agent with `security-audit`).

## Release Model

A release is:

- a backlog milestone named `Release <version>` (for example `Release 1.0`); and
- a release ticket in that milestone, labelled `release`, carrying the release acceptance criteria and recording the audit summary and approval.

The `planner` subagent creates the milestone and release ticket when the orchestrator delegates it. If the project uses a different convention, the user must state the release scope explicitly before orchestration starts. Do not invent a release structure.

## Prerequisites

- **backlog-md skill (REQUIRED)** — Load it before any backlog read or status change.
- **security-audit** is loaded by `security-officer`, not by this skill.
- Available subagents: `security-officer`; `quality-gatekeeper` for the release-ticket gate; `architect` for architecture readiness; development agents for fixes.

## Procedure

1. **Resolve the release scope.**
   - Use the release milestone or version named by the user.
   - Otherwise use the active release milestone from `backlog milestone list --plain`.
   - If several releases are plausible, ask the user and stop.
   - Read the release ticket and its acceptance criteria.
   - If no release ticket exists, report the gap and hand off to `planner`; do not create one yourself.

2. **Verify completion.**
   - List every task assigned to the release milestone with `backlog task list --plain`.
   - Confirm every implementation task is `Done`.
   - If any task is not `Done`, report the remaining work and offer `milestone-orchestration`; stop the release until the user decides.
   - Confirm no task from another milestone is counted as release scope.

3. **Run release-level checks.**
   - Identify the applications in scope (`apps/android/`, `apps/backend/`, `apps/webFrontend/`, `apps/rust/`).
   - Run the full build and test commands available for each stack.
   - Run the project's end-to-end or integration suite when present.
   - Record command, result, and counts.
   - If checks fail, stop and route each failure to the owning development agent through a task; do not declare the release complete.

3b. **Architecture readiness.** Before the security audit, delegate to `@architect` to verify that every ADR for the release scope is Accepted and that NFR architecture constraints are satisfied. Require a short report. If an ADR is missing/Superseded or a constraint is unmet, stop and route the work before the security audit.

4. **Run the mandatory security audit.**
   - Delegate to `security-officer` with the release milestone, release ticket, included tasks, and the application scope.
   - Require the full `security-audit` report: severity-ranked findings, severity counts, NFR coverage, secrets and dependency results, and the release recommendation.
   - `Block` → stop the release, report Critical/High findings, and route remediation to the owning agents and `planner` for follow-up tickets.
   - `Pass with findings` → continue only when no unresolved Critical or High finding remains; record Medium/Low findings with owners.
   - `Pass` → continue.
   - Do not accept a verbal summary without the structured report.

5. **Verify the release ticket.**
   - If the release ticket has acceptance criteria, delegate it to `quality-gatekeeper` with the release scope, completion evidence, check results, and the security audit report.
   - Require a per-criterion evidence table and a decision. If the release ticket has no acceptance criteria, state that explicitly and verify its definition of done directly.
   - `Incomplete` → stop, report the missing evidence, and route the work back to the owning agents.
   - Verified → set the release ticket to `Validated` and continue.

6. **Compile the release report.**
   - Release name, version, milestone, and date with timezone.
   - Included tasks (IDs and titles) and requirement coverage.
   - Build and test results.
   - Security decision, severity counts, accepted risks, and follow-ups.
   - Known limitations and deferred work.
   - Recommended Git tag and command, without executing it.

7. **Request approval.**
   - Print the full release report and the security audit summary in the response.
   - Ask a short approval question only; do not embed the report in the question.
   - If the user requests changes, do not close the release. Return the affected work to tasks through the owning agents, then restart from step 2.
   - If the user withholds approval, stop and report.

8. **Record completion.**
   - Only after explicit approval, set the release ticket status to `Done` through the Backlog.md CLI.
   - Record the release summary and audit decision in the release ticket using `backlog-md`.
   - Do not modify milestone files directly. If all milestone tasks are Done, report the milestone as complete.
   - Do not create Git tags or push unless the user explicitly asks and the environment allows it.

## Stop Conditions

Stop and report when:

- a release task is not `Done`;
- a release ticket or release milestone is missing;
- build or test checks fail;
- the security audit returns `Block` or lacks required evidence;
- the release ticket verification is incomplete or lacks evidence;
- the user requests changes or withholds approval;
- the scope is ambiguous.

## Completion Checks

- Release scope and release ticket confirmed.
- Every included task is `Done`.
- Release-level build and test checks ran with recorded results.
- `security-officer` audit report received with a severity-ranked findings table and decision.
- Release ticket verified by `quality-gatekeeper`, or the absence of acceptance criteria stated explicitly.
- Release ticket set to `Validated` before the approval question.
- No unresolved Critical or High finding remains.
- Release report printed separately from the approval question.
- Release ticket set to `Done` only after explicit user approval.
- Audit summary and approval recorded through the Backlog.md CLI.
- No Git tag created or pushed without explicit user request.
- All backlog operations performed via the Backlog.md CLI.

## Resume / interruption

Release state is derived from the milestone tasks and the release ticket. On re-invocation, re-read both, skip completed steps, and rerun any check whose evidence is stale. Do not mark completion again if the release ticket is already `Done`; report it instead.

## Handoff

- Released → report completion, accepted risks, and follow-up tickets.
- Blocked by incomplete work → `milestone-orchestration` or `planner`.
- Blocked by security findings → owning development agents plus `planner` for remediation tickets; rerun the audit after fixes.
