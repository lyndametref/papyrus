---
id: REQ-06.019
title: "Do not Show the Reference List in Document-Type Browser List Views"
created_date: '2026-11-18T19:28:43Z'
updated_date: '2026-11-18T19:28:51Z'
labels: []
references:
  - RE-006
---
# REQ-06.019 Do not Show the Reference List in Document-Type Browser List Views

## User Story
As a project contributor, I want document-type browser list views to omit the reference list so rows stay compact.

## Group
REQ-06 Search and Local Web Interface

## Description
In browser list views for config-defined document types (generic documents), the reference list is not displayed in any row, keeping rows compact. References remain visible in the single-document view for those document types (REQ-06.012) and in CLI list and view output (REQ-06.004). The change is limited to document-type list views: tasks, decisions, milestones, releases, requirements, NFRs, audit records, business rules, UI mockups, and the glossary landing page (REQ-06.011) are unchanged. The reference list is hidden entirely, not collapsed or expandable. No underlying record or front-matter value is changed. Criticality: Should have. Acceptance Criteria: (1) Browser list views for config-defined document types do not render the reference list in any row. (2) References remain visible in the single-document view for those document types. (3) References remain shown in CLI list and view output. (4) Other artifact families list views are unchanged, including their reference display. (5) The glossary landing page behavior is unchanged. (6) No underlying record or front-matter value is changed. (7) The views require no network access (NFR-01.003). Traceability: Source: RE-006. Product decision 2026-11-18: references are hidden entirely, not collapsed or expandable, in document-type list views. Product decision 2026-11-18: scope is config-defined document types only; tasks, decisions, milestones, releases, and other families are unchanged. REQ-06.004 - Use Front-Matter Metadata in CLI and Browser Views (CLI and detail views still show references). REQ-06.011 - Adjust Glossary Landing Page Display (precedent for omitting references on a list page; unchanged here). REQ-06.012 - Define Single Record View Layout (references remain collapsible in single-document view). REQ-06.014 - Display Config-Defined Document Types in the Browser (document-type list views in scope). REQ-06.016 - Present a Uniform List View Across Artifact Families (shared list layout; title-cell reference display qualified for document types). NFR-01.003 - Use the Browser Command and Interface Without Internet.
