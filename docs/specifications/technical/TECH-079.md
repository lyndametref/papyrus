---
id: TECH-079
title: "Browser Single-Record View Layout"
created_date: '2026-11-18T14:42:48Z'
updated_date: '2026-11-19T21:19:39Z'
labels: []
references:
  - REQ-06.012
  - RE-004
  - NFR-05.001
  - NFR-01.002
  - ADR-033
status: draft
---
# TECH-079 Browser Single-Record View Layout

## Description
Building block: browser single-record view rendering.

Contract (ADR-033, REQ-06.012, RE-004):
- For every managed Markdown record type with a single record view:
  - render id small above title;
  - title big bold;
  - status beside id in a label element (color deferred);
  - created and updated dates as small text directly under title before references;
  - references, dependencies, dependents as separate collapsible native details sections under title (each only when at least one value, collapsed by default);
  - no document type repetition;
  - all metadata under artifact content.
- Works without JavaScript and in Firefox 115+ (NFR-05.001).
- All Papyrus content viewable and editable (NFR-01.002).
- UI mockup HTML views excluded.
- Rendering consumes shared RecordView metadata projection (REQ-06.004, ADR-0013, ADR-0020); no browser-only projection.
- No new dependency, asset, or template engine.
- Per-type browser TECH specs must conform to this shared layout.
