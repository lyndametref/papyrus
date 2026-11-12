---
name: domain-modeling
description: 'Create and maintain the semantic domain model of the project: Business Object Models, context maps, and business object documentation.'
argument-hint: 'Generate or update domain model from business requirements.'
user-invocable: true
disable-model-invocation: false
---

# Domain Modeling

## Purpose
Create and maintain the semantic domain model of the project: Business Object Models, context maps, and business object documentation.

## When to use
Use this skill when you need to:
- Generate or refine a full domain model from business requirements
- Validate an existing domain model against requirements
- Create a context map with bounded contexts
- Document business objects and their relationships

## Inputs
- Business requirements from `docs/requirements/`
- Architecture documentation from `docs/architecture/`
- Existing domain model files from `docs/domain-model/`
- Technical specifications from `docs/specs/`

## Workflow

1. **Clarify scope** — Understand the specific domain modeling task. Ask clarifying questions if the task is ambiguous.
2. **Read source of truth** — Read business requirements, architecture docs, specs, and the existing domain model.
3. **Check for existing model** — If a domain model already exists in `docs/domain-model/`, review it to understand the current state and identify necessary changes.
4. **Identify bounded contexts** — Group requirements into coherent bounded contexts (e.g. Individual Management, Genealogy, Observations & Events, Medication, Calendar, References).
5. **Define business objects** — For each context, identify the main business objects, their attributes, and relationships. Respect the domain rules in the requirements.
6. **Produce the output** — Write or update the domain model documentation using the output format below.

## Output Format

The output is split into 2 files: `context-map.md` in `docs/domain-model/` and `business-object-model.md` in `docs/domain-model/object-model/`.

### `context-map.md`
- A title "Context Map"
- One very short paragraph stating the purpose and scope of the model.
- A Mermaid `graph LR` diagram showing bounded contexts and their relationships.
- A "Context Descriptions" section with one subsection per bounded context (name, short description).

### `business-object-model.md` (stored in `docs/domain-model/object-model/`)
- A title "Business Object Model"
- A Mermaid `graph LR` covering all Business Objects and their relationships (no attributes, show context grouping).
- A "Business Object Descriptions" section with for each object: name, detailed description, bounded context and optionally business examples (no technical implementation details).

## Constraints
- DO NOT generate implementation code (no Kotlin, SQL, Java, etc.).
- DO NOT introduce persistence or framework concerns — models must remain semantic.
- ONLY derive entities and relationships from the provided business requirements. Do not invent business rules.
- If information is missing or ambiguous, list open questions for stakeholder clarification.
