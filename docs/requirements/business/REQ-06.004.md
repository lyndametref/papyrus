# REQ-06.004 Use Front-Matter Metadata in CLI and Browser Views

## User Story
As a project contributor, I want the command line and the browser to show record information from the front matter so that both views present the same values and I do not have to look inside the file.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
CLI list and view output, in plain text and in JSON, and the browser list and detail views read record metadata from the front matter defined in REQ-05.010. Both views show the same metadata values for the same record. They show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, plus the type-specific keys that apply to the record type. The browser keeps its current editable field set; `created_date` and `updated_date` are read-only there. When the browser saves an edit to a metadata-backed field, it writes the new value into the front matter and preserves the remaining front-matter fields and the rest of the file.

## Acceptance Criteria
- CLI list and view output show front-matter metadata values for the viewed or listed record, in plain text and in JSON.
- Browser list and detail views show the same front-matter metadata values as the CLI for the same record.
- Both views show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references`, and the type-specific front-matter keys of the record type.
- The browser's editable field set stays as it is today, with `created_date` and `updated_date` read-only.
- A browser save of a metadata-backed field writes the new value to the front matter.
- A browser save preserves the other front-matter fields and the remainder of the file.
- A browser save does not copy metadata back into the document core.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 6 (high), raw quote and clarifications (verbatim in REQ-05.010): "update the browser view and the cli view to use those inftormation."
- User decision 2026-11-08 (display scope): CLI and browser show at least `id`, `title`, `created_date`, `updated_date`, `labels`, `references` plus type-specific keys; browser keeps its current editable field set with read-only timestamps.
- REQ-05.010 — Record Metadata in YAML Front Matter
- REQ-05.011 — Maintain Record Metadata Through All Commands
- REQ-02.002 — View and Edit Tasks
- ADR-0009 — Shared Record Envelope and Unified Write Path for CLI and Browser (one projection for both views)
- REQ-06.002 — Serve the Local Web Interface
- docs/specifications/browser-*.md — existing browser persistence contract (metadata preserved on save)
