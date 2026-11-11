# REQ-06.012 Define Single Record View Layout

## User Story
As a project contributor, I want single record views to have a consistent summary card layout so that key metadata is easy to find.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
In all single record views, the summary card at the beginning shows only the record ID, created date, and updated date; a sub-requirement additionally shows its owning group there. Dependencies and references are listed at the end as collapsible sections when relevant, meaning a section appears only when the record has at least one such value. Labels are shown as a taglist under the title. Collapsible sections use native HTML disclosure elements so they work without JavaScript and in the supported browsers (NFR-05.001). The layout applies to every managed Markdown record type that has a single record view; UI mockup views are HTML records and are outside this requirement.

## Acceptance Criteria
- The summary card shows only the record ID, created date, and updated date.
- A sub-requirement shows its owning group in the summary card, and nothing else is added to the card.
- Dependencies and references appear at the end of the view as collapsible sections, each shown only when the record has at least one such value.
- Labels are shown as a taglist under the title.
- Collapsible sections use native HTML disclosure elements and work with JavaScript disabled.
- The layout applies to every managed Markdown record type with a single record view; UI mockup HTML views are excluded.

## Traceability
- Source: customer-feedback handoff — maintainer, Item 14.
- Maintainer decision 2026-11-11: the layout applies to all managed Markdown record types (UI mockup HTML views excluded), the owning group is confirmed in the summary card, and disclosure uses native HTML elements.
- REQ-05.010 — Record Metadata in YAML Front Matter (id, dates, labels, references)
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.004 — Use Front-Matter Metadata in CLI and Browser Views
- REQ-06.010 — Present Dependencies as Links in the Browser
- REQ-06.011 — Adjust Glossary Landing Page Display (taglist presentation)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-05.001 — Display the Browser Interface in Named Browsers
