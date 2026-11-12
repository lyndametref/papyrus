---
name: architecture-decision-record
description: 'Create and manage Architecture Decision Records (ADRs) in the architecture documentation folder.'
argument-hint: 'ADR title or description of the architectural decision to record.'
user-invocable: true
disable-model-invocation: false
---

# Architecture Decision Record (ADR) Management

## Outcome
Create, revise, and manage Architecture Decision Records (ADRs), stored in `docs/architecture/adr/`. Once an ADR reaches a final status (Accepted, Refused), its body becomes immutable — only status transitions to Deprecated or Superseded are permitted. After every ADR change, the visual ADR index (`docs/architecture/adr/index.html`) is regenerated via the `adr-index-visualization` skill.

## When To Use
- Record a significant architectural decision with rationale
- Propose a new architectural approach for discussion
- Supersede or deprecate a previously accepted/refused decision
- Document the context, trade-offs, and consequences of a decision

Do not use this skill for:
- Business or project management decisions (use `backlog/decisions/`)
- Implementation notes or task-level choices
- Casual discussions that do not impact architecture

## ADR Location
- All ADRs live in `docs/architecture/adr/`
- File naming convention: `ADR-<NNNN>-<kebab-case-title>.md`
  - Example: `docs/architecture/adr/ADR-0001-use-neo4j-for-genealogy.md`
- The browsable index is the generated artifact `docs/architecture/adr/index.html`. It is owned by the `adr-index-visualization` skill — never create or hand-edit a markdown ADR index, and never hand-edit `index.html`.

## ADR Discovery and Path Handling (Decision Rules)
- The canonical ADR directory is `docs/architecture/adr/`. All reads, writes, numbering, and index generation operate on that directory.
- If ADR files are found in a different path (e.g. `docs/architecture/` root or another directory), do not move them automatically. Stop and ask the user to move them into `docs/architecture/adr/` before proceeding.
- Never relocate, rename, or delete ADR files without explicit user instruction.

## ADR Format (Mandatory Template)
Every ADR MUST follow this structure. See [the example](references/example-ADR-0001-use-neo4j-for-genealogy.md) for a complete worked example.

```markdown
# <Title>

## Status

<Proposed | Accepted | Refused | Deprecated | Superseded>

## Decision

<State the decision clearly. If superseding, reference the superseded ADR.>

## Context

<Describe the problem, forces at play, background, and options considered.>

## Consequences

<List positive and negative consequences, trade-offs, and migration notes.>
```

- **Status** values and their lifecycle:
  - `Proposed` — initial state, open for discussion
  - `Accepted` — decision adopted (immutable body)
  - `Refused` — decision rejected (immutable body)
  - `Deprecated` — still valid but no longer recommended
  - `Superseded` — replaced by another ADR (must reference the new ADR)
- Status transitions allowed:
  - `Proposed` → `Accepted` | `Refused`
  - `Accepted` → `Deprecated` | `Superseded`
  - `Refused` → `Deprecated` | `Superseded`
  - `Deprecated` → `Superseded`
- Any other status change is prohibited without creating a new ADR.

## Procedure

Every procedure below ends with the same mandatory step: **regenerate the visual index** by invoking the `adr-index-visualization` skill. Do not run the generator script or build an index yourself — that skill is the single owner of index generation.

### 1. Create a new ADR (Proposed)

1. **Determine the next ADR number** — find the highest existing `ADR-NNNN-` prefix in `docs/architecture/adr/`, increment by 1.
2. **Define the title** — short, descriptive noun phrase (e.g., "Use Neo4j for Genealogy").
3. **Write the ADR** using the template above with status `Proposed`.
4. **Save** to `docs/architecture/adr/ADR-<NNNN>-<kebab-case-title>.md`.
5. **Regenerate the visual index** via the `adr-index-visualization` skill.

### 2. Accept or Refuse a Proposed ADR

1. **Validate** that current status is `Proposed`.
2. **Ask for user validation** — present the ADR (title, context, decision, consequences) and ask the user to confirm acceptance or refusal. Do not change the status without explicit user approval.
3. **Change status** to `Accepted` or `Refused` only after user confirmation.
4. **Immutable lock** — once set to `Accepted` or `Refused`, the body (Context, Decision, Consequences) MUST NOT be edited further.
5. **Regenerate the visual index** via the `adr-index-visualization` skill.

### 3. Supersede a Final ADR

1. **Validate** that current status is `Accepted` or `Refused`.
2. **Supersede**: create a new ADR that replaces the old one. The old ADR's `Status` section MUST reference the new ADR number (`Superseded by ADR-NNNN`). The new ADR's `Context` MUST reference the old ADR (`Supersedes ADR-NNNN`).
3. **Change status** of the old ADR to `Superseded`.
4. Only status is editable — body remains frozen.
5. **Regenerate the visual index** via the `adr-index-visualization` skill.

### 4. Deprecate an ADR

1. **Validate** that current status is `Proposed`, `Accepted` or `Refused`.
2. **Change status** of the ADR to `Deprecated` and provide rationale in the `Status` section (e.g., "Deprecated due to change in REQ-01.003" or "Deprecated because of new architectural guidelines").
3. Only status is editable — body remains frozen.
4. **Regenerate the visual index** via the `adr-index-visualization` skill.

### 5. Revise a Proposed ADR (Before Final Status)

1. Any field (Context, Decision, Consequences, Status) may be edited freely as long as the status is `Proposed`.
2. **Regenerate the visual index** via the `adr-index-visualization` skill.

## Immutability Rules

| Status | Body Editable | Status Editable To |
|--------|:---:|:---:|
| Proposed | Yes | Accepted, Refused |
| Accepted | **No** | Deprecated, Superseded |
| Refused | **No** | Deprecated, Superseded |
| Deprecated | **No** | — |
| Superseded | **No** | — |

- "Body" = the `## Context`, `## Decision`, and `## Consequences` sections.
- The `## Status` line is always editable (within allowed transitions).
- The `# Title` line is editable only while status is `Proposed`.

## Completion Checks
- [ ] ADR file exists at `docs/architecture/adr/ADR-<NNNN>-<kebab-case-title>.md`
- [ ] Template follows the defined format with all required sections
- [ ] Status is valid and transitions are legal
- [ ] Visual index regenerated via `adr-index-visualization` (`docs/architecture/adr/index.html` is newer than the edited ADR files)
- [ ] No markdown ADR index file was created or updated
- [ ] If superseding, the old ADR links to the new one and vice versa

## Output Template
After any operation, summarize:

- **Action**: create / accept / refuse / deprecate / supersede
- **ADR**: `ADR-NNNN` – Title
- **Status**: OldStatus → NewStatus
- **File**: `docs/architecture/adr/ADR-NNNN-title.md`
- **Transitions validated**: yes/no
- **Visual index regenerated**: yes/no (`docs/architecture/adr/index.html`)
