---
name: attribute-list-definition
description: 'Create a structured attribute list for a business object or event, documenting each attribute with name, type, optionality, constraint, and requirement traceability. Stored in docs/domain-model/object-model/<entity-name>-attributes.md.'
argument-hint: 'define attribute list for Individual; list attributes for the FLOCK_ENTRY observation event'
user-invocable: true
disable-model-invocation: false
---

# Attribute List Definition

## Outcome

Produce a structured attribute list for a business object or event, documenting every attribute with its name, data type, optionality (mandatory/optional), constraint or format rule, and traceability back to a requirement. Stored in `docs/domain-model/object-model/<entity-name>-attributes.md`.

## When To Use

- Define the complete attribute list for a domain business object (e.g. Individual, Observation, FutureEvent).
- Define the payload attributes for a domain event (e.g. FLOCK_ENTRY, FLOCK_EXIT, BIRTH).
- Document attribute-level constraints, formats, and default values.
- Establish a single source of truth for attribute definitions referenced in specs and implementation.

Do not use this skill for:
- Database schema or table column definitions.
- State diagrams or lifecycle transitions (use `state-diagram-creation` skill).
- Business rules (use `business-rules-management` skill).

## Location

- All attribute list files live in `docs/domain-model/object-model/`.
- File naming convention: `<kebab-case-entity-name>-attributes.md`.
  - Examples: `individual-attributes.md`, `flock-entry-event-attributes.md`, `future-event-attributes.md`

## Format

Each attribute list file MUST follow this structure:

```markdown
# <Entity Name> — Attributes
WARNING: the attribute list is about the business object description and do not necessary represent a persistance structure or a technical implementation.

| Attribute | Type | Required | Constraints / Format | Requirement |
|-----------|------|----------|----------------------|-------------|
| `attributeName` | <type> | Yes/No | <constraint or N/A> | REQ-XX.XXX |
```

### Conventions

- **Attribute** column: the canonical attribute name in camelCase.
- **Type** column: the conceptual data type (String, Int, LocalDate, Timestamp, Boolean, Enum, etc.).
- **Required** column: `Yes` if mandatory, `No` if nullable or optional.
- **Constraints / Format** column: any format rules, value ranges, regex patterns, or length limits. Use `N/A` if none.
- **Requirement** column: the requirement ID that defines or justifies this attribute. Use REQ-XX.XXX format. If derived from a business rule, use BR-XXX.
- Attributes are ordered by logical grouping (identity first, then core data, then metadata).
- If an attribute is an enum, list the valid values in the Constraints column.

## Procedure

1. **Identify the entity** — Determine which business object or event needs an attribute list.
2. **Read source docs** — Read the domain model (`docs/domain-model/object-model/business-object-model.md`), relevant requirements in `docs/requirements/`, and specs to extract all attributes, their types, and constraints.
3. **Inventory attributes** — List every attribute of the object or event, noting:
   - Name and type
   - Whether it is mandatory or optional
   - Any format constraints, allowed values, or default values
   - The requirement that defines it
4. **Save the file** — Write to `docs/domain-model/object-model/<entity-name>-attributes.md`.

## Resume / interruption

If interrupted, re-read the source docs and continue from the last completed step; the attribute file is the source of truth once saved.

## Completion Checks

- [ ] File exists at `docs/domain-model/object-model/<entity-name>-attributes.md`.
- [ ] Every attribute from the domain model and requirements is listed.
- [ ] Each attribute has a type, required indicator, and requirement reference.
- [ ] Constraints are documented where applicable (formats, allowed values, ranges).
- [ ] No implementation code or database column definitions are included.

## Output Template

After creation, summarize:

- **Entity**: <entity name>
- **File**: `docs/domain-model/object-model/<entity-name>-attributes.md`
- **Attributes**: <count> attributes documented
- **Source references**: <REQs and BRs used>
