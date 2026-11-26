---
id: ADR-033
title: "Uniform single-document browser view layout for managed Markdown records"
created_date: '2026-11-18T08:27:12Z'
updated_date: '2026-11-26T18:24:52Z'
labels: []
references:
  - REQ-006.012
  - NFR-005.001
  - NFR-001.002
  - RE-004
  - ADR-0005
  - ADR-0020
  - ADR-0028
  - TECH-063
superseded_by: null
---
# ADR-033 Uniform single-document browser view layout for managed Markdown records

## Status
deprecated
## Decision
1. A single shared rendering path produces the detail view for every document record type — `SingleMarkdown`, `GroupedMarkdown`, and `SingleHtmlMockup`; the artifact content comes from the descriptor field map of ADR-039, and no per-type summary card or type-specific metadata block may be introduced.

2. Metadata block order is fixed: small record ID above title; title big and bold; status beside ID in a label element (color deferred, no color now); created and updated dates small directly under title before references; then references, dependencies, and dependents as separate collapsible native details sections under title. The field sections of ADR-039 decision 19 are the artifact content and sit between the fixed head and those disclosures.

3. Each collapsible section appears only when the record has at least one value; disclosures collapsed by default and use native HTML so they work with JavaScript disabled.

4. Document type is not repeated anywhere on the view.

5. All metadata elements placed under artifact content.

6. Rendering consumes the shared `RecordView` metadata projection (REQ-006.004, ADR-0013 and ADR-0020) and the descriptor field map; no browser-only metadata projection and no second content projection.

7. Every document model uses this layout. `SingleMarkdown` renders its single field sequence, `GroupedMarkdown` renders the group or member field set resolved for the record's role, and `SingleHtmlMockup` renders its fixed HTML metadata (status, task, specs, references, superseded_by) instead of a field sequence. There is no HTML-view exclusion.

8. No new dependency, asset, or template engine; disclosures use existing embedded templates and single-pass renderer.

### Amendment (2026-11-20) — ADR-039

- **Amended:** decision 1 (every document record type; artifact content from the descriptor field map), decision 2 (field sections between the fixed head and the reference disclosures), decision 6 (the field map is part of the projection), and decision 7 (the HTML mockup is included with fixed metadata instead of excluded).
- **Not changed:** decisions 3, 4, 5, and 8, the metadata order, the no-JavaScript disclosures, and this record's accepted status.
## Context
REQ-006.012 requires one uniform layout for every single record view across all managed Markdown record types: small record ID above title; title big and bold; status beside ID in a label format colorable later; created and updated dates as small text directly under title before references; references, dependencies, and dependents as separate collapsible sections under title, each shown only when the record has at least one value, using native HTML disclosure that works without JavaScript; no repetition of document type; all metadata under artifact content. UI mockup HTML views excluded. NFR-005.001 requires Firefox 115+ support without layout breakage; NFR-001.002 requires all Papyrus content viewable and editable in local browser. RE-004 records the 2026-11-16 product decision to replace the previous summary-card layout with this uniform layout. ADR-0005 fixes embedded dependency-free server-rendered UI; ADR-0020 fixes registry-derived views and RecordView metadata projection; ADR-0028 fixes shared list layout. No ADR currently governs single-record view layout.
## Consequences
Positive: consistent single-record view across all managed Markdown types; works without JavaScript and in Firefox 115+. Negative: replaces summary-card layout, requiring UI and TECH spec revisions; status color deferred. Risks: refactor of render.rs with regression risk; characterization tests needed.
