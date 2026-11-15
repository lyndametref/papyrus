---
id: REQ-06.004
title: "Use Front-Matter Metadata in CLI and Browser Views"
created_date: '2026-11-13T20:56:43Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-06.004 Use Front-Matter Metadata in CLI and Browser Views

## User Story
As a project contributor, I want the command line and the browser to show record information from the front matter so that both views present the same values and I do not have to look inside the file.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
CLI list and view output, in plain text and in JSON, and the browser list and detail views read record metadata from the front matter defined in REQ-05.010. Both views show the same metadata values for the same record. They show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, plus the type-specific keys that apply to the record type. For a document record, they show its front-matter `status` value as stored, including a custom status declared for its type (REQ-07.001, REQ-07.002); a configured status value is displayed verbatim and is never renamed or replaced by a display label. The browser keeps its current editable field set; `created_date` and `updated_date` are read-only there, and `status` is not an editable field because it is lifecycle- and configuration-owned (REQ-05.011). When the browser saves an edit to a metadata-backed field, it writes the new value into the front matter and preserves the remaining front-matter fields and the rest of the file.

## Acceptance Criteria
- CLI list and view output show front-matter metadata values for the viewed or listed record, in plain text and in JSON.
- Browser list and detail views show the same front-matter metadata values as the CLI for the same record.
- Both views show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and the type-specific front-matter keys of the record type.
- Both views show a document record's `status` value as stored, including a custom status declared for its type, and the value is identical in the CLI and in the browser.
- A configured custom status value is displayed verbatim; the view does not rename or map it to another label.
- The browser's editable field set stays as it is today, with `created_date` and `updated_date` read-only and `status` not editable.
- A browser save of a metadata-backed field writes the new value to the front matter.
- A browser save preserves the other front-matter fields and the remainder of the file.
- A browser save does not copy metadata back into the document core.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote and clarifications (verbatim in REQ-05.010): "update the browser view and the cli view to use those inftormation."
- User decision 2026-11-08 (display scope): CLI and browser show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references` plus type-specific keys; browser keeps its current editable field set with read-only timestamps.
- Project-owner decision 2026-11-13 (D5): both views display a document record's `status`, including custom statuses, verbatim; `status` is not editable in the browser because it is lifecycle- and configuration-owned (REQ-05.011).
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
- REQ-07.001 — Organize Artifacts into Three Categories (document status values)
- REQ-07.002 — Configure Document Types and Slugs (`statuses`, `default_status`)
- REQ-02.002 — View and Edit Tasks
- ADR-0009 — Shared Record Envelope and Unified Write Path for CLI and Browser (one projection for both views)
- REQ-06.002 — Serve the Local Web Interface
- docs/specifications/browser-*.md — existing browser persistence contract (metadata preserved on save)
