---
id: REQ-06.001
title: "Search Project Records"
created_date: '2026-11-13T20:19:47Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-06.001 Search Project Records

## User Story
As a project contributor, I want to search backlog and documentation records together so that relevant project information can be found without searching stores separately.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
The user can run a free-text search across tasks, milestones, releases, decisions, and documents in one pass. Documents include every type declared under `document_types`, including types added by configuration. The user may restrict results to one or more record types and limit the maximum number of results. The documented type values are task, milestone, release, decision, and doc; the default is all types.

## Acceptance Criteria
- A query searches tasks, milestones, releases, decisions, and documents by default.
- A quoted phrase containing spaces is accepted as one query.
- Repeatable `--type` options restrict results to the selected documented record types.
- `--limit <n>` caps the number of returned results.
- `--type doc` includes records of every document type declared under `document_types`, including config-defined types (REQ-07.002).

## Traceability
- README.md, `papyrus(1)` → COMMANDS → search (lines 720–733)
