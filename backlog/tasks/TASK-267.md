---
id: TASK-267
title: "Collapse the nine conflict pages and five not-found pages into one generic pair"
created_date: '2026-11-20T17:42:43Z'
updated_date: '2026-11-20T17:42:43Z'
labels: []
references:
  - ADR-039
  - REQ-06.014
  - TECH-068
status: ToDo
assignee:
  - rust-dev
milestone: MS-035
dependencies:
  - TASK-266
ordinal: 60
documentation: []
type: implementation
---

## Description
Context: the MS-032 audit lists 9 conflict-page renderers and 5 not-found pages. They differ only in labels and type names, which now come from the seed.

Scope: implement one conflict page and one not-found page, routed from the generic handler, with type labels from the descriptor. Delete every per-type variant listed in the audit. Confirm no route can reach a per-type 404 and that unknown document slugs return the generic 404 with a 404 status.
