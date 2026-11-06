# Context Map

This context map groups the Papyrus business objects documented in the [Business Object Model](object-model/business-object-model.md) by business capability and shows their inter-context relationships.

```mermaid
graph LR
    BacklogManagement[Backlog Management]
    DecisionManagement[Decision Management]
    RequirementsManagement[Requirements Management]
    DocumentationManagement[Documentation Management]

    BacklogManagement -->|tasks may reference| RequirementsManagement
    BacklogManagement -->|tasks may reference| DocumentationManagement
    DecisionManagement -->|decisions may reference| RequirementsManagement
    DocumentationManagement -->|business rules may reference| RequirementsManagement
    DocumentationManagement -->|technical specifications may reference| RequirementsManagement
```

## Context Descriptions

### Backlog Management

Manages tasks, milestones, and releases. Tasks may depend on other tasks, may be assigned to a milestone, and may reference requirements, UI specifications, technical specifications, audit records, and business rules.

### Decision Management

Manages decisions with a decision type, status, context, decision, consequences, and optional superseding-decision reference. Decisions may reference business and non-functional requirements.

### Requirements Management

Manages business requirements, non-functional requirements, and their type-specific requirement groups. Each requirement belongs to exactly one group, and a group contains requirements of one type.

### Documentation Management

Manages glossary terms, UI specifications, technical specifications, audit records, and business rules. Technical specifications are a managed documentation type (REQ-05.009, BR-017) with a required title and description, ID format `TECH-xxx` (padded to `id_padding`), storage under `technical/` under `doc_directory`, and an active -> deprecated lifecycle; they are managed via the `tech-spec` command and may reference non-functional requirements (README line 438 / BR-010).
