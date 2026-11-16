---
id: REQ-06.012
title: "Define Single Record View Layout"
created_date: '2026-11-11T19:58:28Z'
updated_date: '2026-11-16T20:56:06Z'
labels: []
references:
  - RE-004
---
# REQ-06.012 Define Single Record View Layout

## User Story
As a project contributor, I want single record views to use one uniform layout for all document types so that key metadata is easy to find and consistent across the browser.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
In all single record views, the metadata elements are placed under the artifact content in this order: a small record ID above the title; the title in big bold text; the status beside the ID in a label format that can be colored in a future change; the created and updated dates as small text directly under the title and before references; and references, dependencies, and dependents as separate collapsible sections under the title. The document type is not repeated on the view. Each collapsible section appears only when the record has at least one value. Collapsible sections use native HTML disclosure elements so they work without JavaScript and in the supported browsers (NFR-05.001). The layout applies to every managed Markdown record type that has a single record view; UI mockup views are HTML records and are outside this requirement.

## Acceptance Criteria
- In every single record view, the record ID appears small above the title.
- The title is rendered big and bold.
- The status appears beside the ID in a label format, with color support deferred to a future change.
- Created and updated dates appear as small text directly under the title, before references.
- References, dependencies, and dependents appear as separate collapsible sections under the title.
- Each collapsible section appears only when the record has at least one value.
- Collapsible sections use native HTML disclosure elements and work with JavaScript disabled.
- The document type is not repeated on the view.
- The metadata elements are placed under the artifact content.
- The layout applies to every managed Markdown record type with a single record view; UI mockup HTML views are excluded.

## Traceability
- Source: customer-feedback handoff - maintainer, Item 14; RE-004 uniform single-document browser layout.
- Maintainer decision 2026-11-11: the layout applies to all managed Markdown record types (UI mockup HTML views excluded), and disclosure uses native HTML elements.
- Product decision 2026-11-16: replace the previous summary-card layout with the uniform single-document layout (id small above title, status label beside id, created/updated under title before references, separate collapsible references/dependencies/dependents, no document-type repetition, elements under content) (RE-004).
- REQ-05.010 - Record Metadata in YAML Front Matter (id, dates, labels, references, status)
- REQ-06.002 - Serve the Local Web Interface
- REQ-06.004 - Use Front-Matter Metadata in CLI and Browser Views
- REQ-06.010 - Present Dependencies as Links in the Browser
- REQ-06.011 - Adjust Glossary Landing Page Display (taglist presentation)
- NFR-01.002 - Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-05.001 - Display the Browser Interface in Named Browsers
- RE-004 - Uniform single-document browser layout for all document types
