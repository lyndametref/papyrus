---
id: ADR-033
title: "Uniform single-document browser view layout for managed Markdown records"
created_date: '2026-11-18T08:27:12Z'
updated_date: '2026-11-18T14:41:30Z'
labels: []
references:
  - REQ-06.012
  - NFR-05.001
  - NFR-01.002
  - RE-004
  - ADR-0005
  - ADR-0020
  - ADR-0028
  - TECH-063
superseded_by: null
---
# ADR-033 Uniform single-document browser view layout for managed Markdown records

## Status
accepted
## Decision
1. A single shared rendering path produces the detail view for every managed Markdown record type; no per-type summary card or type-specific metadata block may be introduced. 2. Metadata block order is fixed: small record ID above title; title big and bold; status beside ID in a label element (color deferred, no color now); created and updated dates small directly under title before references; then references, dependencies, and dependents as separate collapsible native details sections under title. 3. Each collapsible section appears only when the record has at least one value; disclosures collapsed by default and use native HTML so they work with JavaScript disabled. 4. Document type is not repeated anywhere on the view. 5. All metadata elements placed under artifact content. 6. Rendering consumes shared RecordView metadata projection (REQ-06.004, ADR-0013 and ADR-0020); no browser-only metadata projection. 7. UI mockup HTML views excluded. 8. No new dependency, asset, or template engine; disclosures use existing embedded templates and single-pass renderer.
## Context
REQ-06.012 requires one uniform layout for every single record view across all managed Markdown record types: small record ID above title; title big and bold; status beside ID in a label format colorable later; created and updated dates as small text directly under title before references; references, dependencies, and dependents as separate collapsible sections under title, each shown only when the record has at least one value, using native HTML disclosure that works without JavaScript; no repetition of document type; all metadata under artifact content. UI mockup HTML views excluded. NFR-05.001 requires Firefox 115+ support without layout breakage; NFR-01.002 requires all Papyrus content viewable and editable in local browser. RE-004 records the 2026-11-16 product decision to replace the previous summary-card layout with this uniform layout. ADR-0005 fixes embedded dependency-free server-rendered UI; ADR-0020 fixes registry-derived views and RecordView metadata projection; ADR-0028 fixes shared list layout. No ADR currently governs single-record view layout.
## Consequences
Positive: consistent single-record view across all managed Markdown types; works without JavaScript and in Firefox 115+. Negative: replaces summary-card layout, requiring UI and TECH spec revisions; status color deferred. Risks: refactor of render.rs with regression risk; characterization tests needed.
