# REQ-05 Project Documentation Management

## User Story
As a project contributor, I want to manage structured project documentation so that requirements, terms, specifications, rules, and audit records are available alongside backlog information.

## Description
This group covers the document artifact category (REQ-07.001): the create, list, view, edit, status, archive, and delete capabilities of document records, including requirement grouping and type-specific content. Document types are declared under `document_types`, whose keys are the document slugs and CLI command names (REQ-07.002); the set includes the default types seeded at init and any type added by editing the configuration. Every document type has the main `active` or `archived` status, which alone governs storage location, plus optional custom statuses configured per type and held only in front matter (REQ-07.001); archiving has no restore command (REQ-05.007). The group also covers the YAML front-matter metadata carried by every managed Markdown record, excluding UI mockup HTML files, and its automatic maintenance by commands.
