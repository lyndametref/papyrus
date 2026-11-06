# Technical Specification

## Definition

A technical specification is a managed documentation record with a title and description. It uses the `TECH` prefix (for example `TECH-001`), padded to the configured `id_padding`, is stored under `technical/` under `doc_directory`, and follows the document lifecycle active -> deprecated (deprecate/restore preserve the ID). A task may reference zero or more technical specifications, and it may reference non-functional requirements (README line 438 / BR-010). Managed via the `tech-spec` command (BR-017). (REQ-05.009, BR-017, BR-010, README)

## Related Terms

- Task
- Non-functional Requirement
