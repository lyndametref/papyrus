---
id: TASK-277
title: "Load Mermaid lazily and stop sending it no-store"
created_date: '2026-11-20T17:43:20Z'
updated_date: '2026-11-20T17:43:20Z'
labels: []
references:
  - ADR-038
  - ADR-0026
  - NFR-007.003
status: ToDo
assignee:
  - frontend-dev
milestone: MS-036
dependencies:
  - TASK-276
ordinal: 70
documentation: []
type: implementation
---

## Description
Context: the Mermaid bundle is the largest transferred asset and is loaded on pages that contain no diagram.

Scope: load Mermaid only when the document contains a diagram block, using the versioned immutable URL. Keep offline behavior and the existing render output unchanged. KaTeX remains available where math exists and follows the same caching rules. Confirm no page requests Mermaid when it has no diagram.
