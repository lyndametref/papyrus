---
id: TASK-58
title: 'Design review: technical specifications browser UI mockup'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-04T19:21:00Z'
updated_date: '2026-11-07T16:33:00Z'
labels:
  - design-review
  - ui-design
milestone: m-11
dependencies:
  - TASK-55
references:
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.009.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-017-tech-spec-command-behavior.md
  - docs/specifications/technical-specifications-management.md
  - docs/specifications/search-technical-specifications.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/ui/browser-audit-records-ui.md
  - docs/specifications/ui/mockups/README.md
documentation:
  - docs/specifications/ui/browser-technical-specifications-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 29300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
UI designer produces and obtains explicit user approval for the technical-specifications browser mockup package before TASK-57 (Expose technical specifications in the local browser) is implemented.

Deliverable: one new self-contained HTML review package under docs/specifications/ui/mockups/ (next available MOCK id, expected MOCK-0009), a UI description document under docs/specifications/ui/, and a registry update, following the approved pattern of MOCK-0001 (requirements), MOCK-0002 (backlog records), MOCK-0003 (decision records), MOCK-0004 (glossary), MOCK-0005 (UI specifications), MOCK-0006 (UI mockups), MOCK-0007 (business rules), and MOCK-0008 (audit records). Layout only: no branding, colour palette, logo, or technology/library references.

Scope:
- Reuse the approved shell (header, persistent document-type sidebar, main, footer, one h1) and add a Technical Specifications sidebar entry plus a category selector value (recommended `tech-spec`).
- Technical specifications list: rows with ID + title; empty states; long-title wrapping; list excludes deprecated specifications.
- Technical specification view: ID, title, description; breadcrumb `Technical Specifications › ID`; single Edit action; deprecated specifications unreachable.
- Edit form: title (required) and description (required, multi-line); pristine, dirty, validation-error (422), conflict (409), and 320 px states; hidden version input with dashed annotation.
- Error pages reused from MOCK-0001.
- Responsive 320 px list and edit form with sidebar toggle and annotation legend.
- Deprecated technical specifications excluded from lists and navigation (REQ-05.007, BR-017).

Out of scope:
- Browser implementation (TASK-57), CLI behaviour (TASK-55), search (TASK-56), packaging (TASK-59).
- Creating, deprecating, restoring technical specifications from the browser (CLI-only).
- Modifying approved MOCK-0001..MOCK-0008 in place; allocate a new MOCK id.
- New dependencies, crates, or ADRs.
- Inventing field-length or validation rules not present in technical-specifications-management.md or BR-017; follow those sources.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A new self-contained HTML review package is created under docs/specifications/ui/mockups/ with the next available MOCK id (expected MOCK-0009), containing a review landing and web prototype; styles are inlined and no external file, script, font, image, or network reference is used.
- [x] #2 The package reuses the approved shell and adds a Technical Specifications sidebar entry plus a category selector; technical specification list rows show ID + title, with empty states and long-title wrapping; deprecated specifications are excluded.
- [x] #3 The package includes technical specification view (ID, title, description; breadcrumb Technical Specifications › ID; single Edit action) and edit form (title required, description required multi-line; pristine, dirty, validation-error, conflict, and 320 px states) views.
- [x] #4 Deprecated technical specifications are excluded from list and navigation views and are not reachable, consistent with REQ-05.007 and BR-017.
- [x] #5 Error pages are reused from MOCK-0001 (404 unknown/deprecated, 400, 405, 415, 500) and 320 px list/edit views include sidebar toggle and annotation legend.
- [x] #6 A UI description document is created/updated under docs/specifications/ui/ (expected browser-technical-specifications-ui.md) describing screens, states, elements, interactions, active elements, MOCK register, assumptions, and approval status.
- [x] #7 The mockup registry docs/specifications/ui/mockups/README.md is updated with the new MOCK row (id, type, name, slug, artifact, platform, views, status, source) and notes.
- [x] #8 The mockup is explicitly approved by the user before TASK-57 implementation; registry and UI description status reflect approved (not draft/pending).
- [x] #9 No implementation code, new dependencies, crates, or ADRs are introduced; approved MOCK-0001..MOCK-0008 are not modified in place.
- [x] #10 validate_mockups.sh reports no errors for MOCK-0009.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Mockup package created and approved per ui-clickable-mockup.
- [x] #2 Registry docs/specifications/ui/mockups/README.md updated with the new MOCK entry.
- [x] #3 UI description document created/updated under docs/specifications/ui/.
- [x] #4 No code, dependency, or ADR changes.
- [x] #5 User approval explicitly recorded (date and ticket) in registry and UI description.
- [x] #6 validate_mockups.sh reports no errors for MOCK-0009.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Expected deliverables (ui-designer):
- MOCK id: MOCK-0009 (next available; confirm no new MOCK allocated before this ticket starts).
- Artifact: docs/specifications/ui/mockups/MOCK-0009-browser-technical-specifications-review.html
- UI description: docs/specifications/ui/browser-technical-specifications-ui.md
- Registry: docs/specifications/ui/mockups/README.md updated with MOCK-0009 row and notes.

Record model (from technical-specifications-management.md, TASK-55): id (e.g. TECH-001), title (required), description (required, multi-line). ID, record form, and category are read-only; deprecation/restoration stay CLI-side. List rows show ID + title plus first paragraph of description (recommended). Example records are illustrative.

Assumptions:
- Assumption: The technical-specifications browser surface reuses the transport, asset, rendering, security, and configuration baseline of TASK-6/ADR-0005/ADR-0006 and the decision-record extension (TASK-15); no new ADR.
- Assumption: Sidebar entry is labelled "Technical Specifications" and placed after "Audit Records" (or at the end of the document-type list); ui-designer confirms final ordering.
- Assumption: Category selector value is `tech-spec` (default remains `req`); any other value is 400, consistent with the baseline.
- Assumption: The extension package lists all document-type sidebar entries, with only technical-specification views generated and prior entries non-clickable, consistent with MOCK-0003/MOCK-0008.
- Assumption: Technical specification file format follows technical-specifications-management.md: `# <ID> <title>` then `## Description`.
- Assumption: Mockup status starts `draft`/`pending` and becomes `approved` only after explicit user approval; TASK-57 must not implement until approved.
- Assumption: Description is multi-line and rendered with `white-space: pre-wrap` (recommended); title length follows baseline field rules.
- Assumption: Non-functional requirement/task references, when present, appear as literal text in title or description; no structured reference fields in the browser (OQ from technical-specifications-management.md).

Open questions:
- OQ-1: Exact sidebar label and ordering ("Technical Specifications" vs "Tech Specs") and position (after "Audit Records" recommended).
- OQ-2: Category query value (`tech-spec` recommended) for the `?type=` selector.
- OQ-3: Whether list rows show the first paragraph of the description as a preview (recommended yes, consistent with audit records).
- OQ-4: Title/description length limits and control-character rules follow the baseline field rules (recommended: title 1-200, description up to 20000); confirm against BR-017.
- OQ-5: Whether technical specifications may reference MOCK ids or other records is unspecified; browser shows literal text only (technical-specifications-management.md OQ-2).
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @ui-designer
created: 2026-11-07T12:27:00Z

---

MOCK-0009 (draft) and docs/specifications/ui/browser-technical-specifications-ui.md created by @ui-designer on 2026-11-07 (UTC); awaiting explicit user approval before TASK-57 implementation. No status change.

---

author: @ui-designer
created: 2026-11-07T16:28:00Z

---

MOCK-0009 and docs/specifications/ui/browser-technical-specifications-ui.md approved — explicit user approval on 2026-11-07 (UTC). Registry and UI description status set to approved; TASK-57 may implement against MOCK-0009. No status change.

---
<!-- COMMENTS:END -->
