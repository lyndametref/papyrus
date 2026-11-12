---
name: reverse-engineering
description: 'Read existing source code and produce the business (REQ) and non-functional (NFR) requirements that correspond to the implemented behavior.'
argument-hint: 'reverse engineer code to requirements; extract REQ and NFR from the flock entry implementation; derive requirements from this module'
user-invocable: true
disable-model-invocation: false
---

# Reverse Engineering

## Outcome

Analyze existing source code and produce traceable business requirements (REQ) and non-functional requirements (NFR) that describe the behavior, rules, and quality attributes actually implemented by that code. All artifacts are written under `docs/requirements/` using the canonical REQ-XX.XXX and NFR-XX.XXX formats.

## When To Use

- Extract requirements from an existing implementation that lacks documentation.
- Reconstruct requirements for partially documented or legacy code.
- Detect implemented behavior that has no corresponding requirement, and propose new requirements to cover it.
- Validate that documented requirements match the code actually delivered.

Do not use this skill for:
- Generating or modifying implementation code.
- Refactoring, debugging, or reviewing code quality.
- Writing technical specs or architecture documents.

## Prerequisites

- Read access to the target source code (and any supporting `docs/` context).
- The requirements-management skills listed under `## Leveraged Skills` for creating requirement files.

## Procedure

1. **Scope the analysis** — Confirm with the user:
   - Which code to analyze (path, module, screen, or feature).
   - Which requirement types to produce: business requirements (REQ), non-functional requirements (NFR), or both.
   - Whether to propose new requirements or only validate existing ones against the code.
   If any of this is ambiguous, ask before starting.

2. **Read the code** — Explore the target implementation:
   - Entry points and user-visible flows (screens, actions, navigation).
   - Domain logic (view models, use cases, services).
   - Data layer (models, repositories, persistence, schemas, enums).
   - Configuration and resources (permissions, themes, network settings).
   - Error handling, validations, and edge cases.

3. **Extract behaviors** — List every user-visible feature and functional flow observed in the code (e.g., "record an individual's color pattern", "filter the flock list").

4. **Extract rules** — List business rules, validations, constraints, default values, and state transitions enforced in the code (e.g., "tag number must be unique", "a field cannot be empty").

5. **Extract data** — Inventory the attributes, fields, and enums handled by the code. This feeds acceptance criteria and may be used later with the `attribute-list-definition` skill.

6. **Extract quality attributes** — Identify non-functional characteristics implied by the code: performance, security, usability, reliability, maintainability (e.g., timeouts, data encryption, offline behavior, permission handling).

7. **Map to existing requirements** — Read the group and atomic requirement files under `docs/requirements/business/` and `docs/requirements/non-functional/`. These files are the source of truth. Classify each extracted item as:
   - **Covered** — an existing requirement already describes it.
   - **Partially covered** — extend the existing requirement.
   - **Uncovered** — a new requirement is needed.
   Report the uncovered and partially covered items before writing files.

8. **Formulate requirements** — For each uncovered or partially covered item:
   - Use the `business-requirements-management` skill to create or update business requirements (REQ) under `docs/requirements/business/`.
   - Use the `non-functional-requirements-management` skill to create or update NFRs under `docs/requirements/non-functional/`.
   - Assign the next free REQ-XX.XXX / NFR-XX.XXX number by inspecting existing requirement filenames.
   - Write acceptance criteria that are directly testable against the observed code behavior.
   - In the description, reference the source code element(s) the requirement derives from (file path and class/function) so traceability is preserved.

   Do not create or update separate Markdown indexes. After changing requirement files, invoke `requirements-index-visualization` to regenerate `docs/requirements/business/index.html` and `docs/requirements/non-functional/index.html`.

9. **Validate traceability** — Confirm every extracted behavior, rule, and quality attribute is reflected in at least one requirement, and that every new requirement has a source code anchor.

10. **Report outcome** — Summarize using the `## Output Template`.

## Leveraged Skills

This skill runs inside the Requirements Engineering agent and may invoke the skills available to it:

- `business-requirements-management` — create, update, and validate REQ-XX.XXX business requirement files. Always used to produce business requirements.
- `non-functional-requirements-management` — create, update, and validate NFR-XX.XXX files. Always used to produce non-functional requirements.
- `requirements-index-visualization` — regenerate the HTML requirements index after requirement files change.

Additional artifacts implied by the code (business rules, attribute lists, domain model, state diagrams, process diagrams, glossary terms) are **not** produced here; hand them off to `business-analyst` (business rules), `domain-modeler` (attribute lists, state diagrams, process diagrams), `terminologist`, and `architect` (via the `business-rules-management`, `attribute-list-definition`, `domain-modeling`, `state-diagram-creation`, `business-process-diagramming`, `business-glossary-management`, and `business-terminology-consistency-check` skills).

## Resume / interruption

Reverse engineering is multi-file. If interrupted, re-run the validation pass on the affected requirements and regenerate the index before continuing.

## Handoff

After producing requirements, report the new/updated REQ/NFR IDs and any handoff items (business rules, attribute lists, domain model, state/process diagrams, glossary terms) to the caller.

## Decision Rules

- If the code is too large to analyze fully, narrow scope to the module/feature requested and state what was excluded.
- If an extracted behavior is already covered by an existing requirement, do not create a duplicate; propose an update instead.
- If the code and the existing requirements conflict, flag the discrepancy and ask the user which is authoritative.
- If a quality attribute is observed but its target value is not measurable from the code, write the NFR with a verification method and mark the missing target as an open item.
- Do not invent behaviors that are not supported by the code; list observed behavior only.

## Completion Checks

- [ ] Every extracted behavior maps to a REQ with user story, priority, description, and acceptance criteria.
- [ ] Every extracted quality attribute maps to an NFR with description, rationale, and verification method.
- [ ] No duplicate requirements created for already-covered behavior.
- [ ] New requirement numbers are the next free numbers based on existing requirement filenames.
- [ ] `docs/requirements/business/index.html` and `docs/requirements/non-functional/index.html` are regenerated by `requirements-index-visualization` after requirement changes.
- [ ] Each new requirement references its source code element(s) for traceability.
- [ ] No implementation code (Kotlin, SQL, Java, etc.) in requirements files.
- [ ] No files modified outside `docs/requirements/` (unless explicitly requested).

## Output Template

Use this summary after analysis:

- **Scope analyzed**: <paths and modules covered>
- **Extracted behaviors**: <count>
- **Extracted rules**: <count>
- **Extracted quality attributes**: <count>
- **Coverage**: <covered / partially covered / uncovered per item>
- **Created requirements**: <list of REQ/NFR IDs>
- **Updated requirements**: <list of REQ/NFR IDs>
- **Source anchors**: <file paths referenced>
- **Open items / assumptions**: <missing targets, excluded code, conflicts>
