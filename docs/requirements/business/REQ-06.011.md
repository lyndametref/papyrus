---
id: REQ-06.011
title: "Adjust Glossary Landing Page Display"
created_date: '2026-11-11T19:58:13Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-06.011 Adjust Glossary Landing Page Display

## User Story
As a project contributor, I want the glossary landing page to show labels as tags without dates or references so that it stays concise.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Could have

## Description
In the glossary landing page, created and updated dates are removed, labels are shown as a list of tags, and references are not shown. "Tags" means the taglist presentation used for labels in single record views (REQ-06.012). The change is limited to the glossary landing (list) page; glossary detail views follow REQ-06.012, and other document types are unchanged.

## Acceptance Criteria
- The glossary landing page omits created and updated dates.
- Labels are shown as a list of tags on the glossary landing page, using the same taglist presentation as REQ-06.012.
- References are not shown on the glossary landing page.
- Glossary detail views are unchanged by this requirement and follow REQ-06.012.
- Other document-type landing pages are unchanged.

## Traceability
- Source: customer-feedback handoff — maintainer, Item 13.
- Maintainer decision 2026-11-11: scope is the glossary landing page only; tags use the taglist presentation of REQ-06.012; detail views and other document types are unaffected.
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views (labels and references come from front matter)
- REQ-06.012 — Define Single Record View Layout (taglist presentation)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
