# REQ-06.001 Search Project Records

## User Story
As a project contributor, I want to search backlog and documentation records together so that relevant project information can be found without searching stores separately.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
The user can run a free-text search across tasks, milestones, releases, decisions, and documents in one pass. The user may restrict results to one or more record types and limit the maximum number of results. The documented type values are task, milestone, release, decision, and doc; the default is all types.

## Acceptance Criteria
- A query searches tasks, milestones, releases, decisions, and documents by default.
- A quoted phrase containing spaces is accepted as one query.
- Repeatable `--type` options restrict results to the selected documented record types.
- `--limit <n>` caps the number of returned results.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → search (lines 720–733)
