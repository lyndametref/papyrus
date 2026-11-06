---
id: TASK-32
title: 'Design review: glossary browser UI mockup'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-04T19:20:00Z'
updated_date: '2026-11-06T13:43:00Z'
labels:
  - design-review
  - ui-design
milestone: m-5
dependencies:
  - TASK-30
references:
  - docs/requirements/business/REQ-05.003.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-011-glossary-command-behavior.md
  - docs/specifications/glossary-terms-management.md
  - docs/specifications/search-glossary-terms.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-decision-records.md
  - docs/specifications/ui/browser-decision-records-ui.md
  - docs/specifications/ui/mockups/README.md
documentation:
  - docs/specifications/ui/browser-glossary-terms-ui.md
  - docs/specifications/ui/mockups/MOCK-0004-browser-glossary-terms-review.html
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 23200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
UI designer produces and obtains explicit user approval for the glossary browser mockup package before TASK-23 (Expose glossary terms in the local browser) is implemented.

Deliverable: one new self-contained HTML review package under docs/specifications/ui/mockups/ (next available MOCK id, expected MOCK-0004), a UI description document under docs/specifications/ui/, and a registry update, following the approved pattern of MOCK-0001 (requirements), MOCK-0002 (backlog records), and MOCK-0003 (decision records). Layout only: no branding, colour palette, logo, or technology/library references.

Scope:
- Reuse the approved shell (header, persistent document-type sidebar, main, footer, one h1) and add a Glossary sidebar entry plus a `?type=glossary` category.
- Glossary list: rows with ID + term; empty states; long-title wrapping.
- Glossary view: ID, term, definition; breadcrumb `Glossary › ID`; single Edit action; deprecated terms unreachable.
- Edit form: term (required) and definition (optional); pristine, dirty, validation-error (422), conflict (409), and 320 px states; hidden version input with dashed annotation.
- Error pages reused from MOCK-0001.
- Responsive 320 px list and edit form with sidebar toggle and annotation legend.
- Deprecated glossary terms excluded from lists and navigation (REQ-05.007, BR-011).

Out of scope:
- Browser implementation (TASK-23), CLI behaviour (TASK-30), search (TASK-31), packaging (TASK-33).
- Creating, deprecating, restoring, or deleting glossary terms from the browser (CLI-only).
- Modifying approved MOCK-0001/0002/0003 in place; allocate a new MOCK id.
- New dependencies, crates, or ADRs.

Assumptions:
- Assumption: The glossary browser surface reuses the transport, asset, rendering, security, and configuration baseline of TASK-6/ADR-0005/ADR-0006 and the decision-record extension (TASK-15); no new ADR.
- Assumption: Sidebar entry is labelled "Glossary" and placed after "Decisions" (or at the end of the document-type list); ui-designer confirms final ordering.
- Assumption: Category selector value is `glossary` (default remains `req`); any other value is 400, consistent with the baseline.
- Assumption: The extension package lists all document-type sidebar entries, with only glossary views generated and prior entries non-clickable, consistent with MOCK-0003.
- Assumption: Glossary term file format follows glossary-terms-management.md: `# <ID> <term>` and `## Definition`.
- Assumption: Mockup status starts `pending` and becomes `approved` only after explicit user approval; TASK-23 must not implement until approved.

Open questions:
- OQ-1: Exact sidebar ordering and whether "Glossary" should precede "Decisions" (ui-designer to confirm with user).
- OQ-2: Whether the glossary edit form needs a multi-line definition textarea (recommended yes) and maximum length display; follow baseline field rules.
- OQ-3: Whether definition is rendered with line-break preservation (`white-space: pre-wrap`) as in requirements (recommended yes).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A new self-contained HTML review package is created under docs/specifications/ui/mockups/ with the next available MOCK id (expected MOCK-0004), containing a review landing, design deck, and web prototype; styles are inlined and no external file, script, font, image, or network reference is used.
- [ ] #2 The package reuses the approved shell and adds a Glossary sidebar entry plus a ?type=glossary category; glossary list rows show ID + term, with empty states and long-title wrapping.
- [ ] #3 The package includes glossary view (ID, term, definition; breadcrumb Glossary › ID; single Edit action) and edit form (term required, definition optional; pristine, dirty, validation-error, conflict, and 320 px states) views.
- [ ] #4 Deprecated glossary terms are excluded from list/navigation views and are not reachable, consistent with REQ-05.007 and BR-011.
- [ ] #5 Error pages are reused from MOCK-0001 (404 unknown/deprecated, 400, 405, 415, 500) and 320 px list/edit views include sidebar toggle and annotation legend.
- [ ] #6 A UI description document is created/updated under docs/specifications/ui/ (expected browser-glossary-terms-ui.md) describing screens, states, elements, interactions, active elements, MOCK register, assumptions, and approval status.
- [ ] #7 The mockup registry docs/specifications/ui/mockups/README.md is updated with the new MOCK row (id, type, name, slug, artifact, platform, views, status, source) and notes.
- [ ] #8 The mockup is explicitly approved by the user before TASK-23 implementation; registry and UI description status reflect approved (not pending).
- [ ] #9 No implementation code, new dependencies, crates, or ADRs are introduced; approved MOCK-0001/0002/0003 are not modified in place.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 Mockup package created and approved per ui-clickable-mockup.
- [ ] #2 Registry docs/specifications/ui/mockups/README.md updated with the new MOCK entry.
- [ ] #3 UI description document created/updated under docs/specifications/ui/.
- [ ] #4 No code, dependency, or ADR changes.
- [ ] #5 User approval explicitly recorded (date and ticket) in registry and UI description.
<!-- DOD:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
UI mockup MOCK-0004 created as draft: docs/specifications/ui/mockups/MOCK-0004-browser-glossary-terms-review.html (17 views). UI description: docs/specifications/ui/browser-glossary-terms-ui.md. Registry updated. Awaiting explicit user approval before TASK-23 implementation. Open questions resolved with defaults: OQ-1 Glossary after Decisions; OQ-2 multi-line definition textarea (max 20000); OQ-3 white-space: pre-wrap.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: ui-designer
created: 2026-11-06T13:24:00Z

---

MOCK-0004 (browser glossary terms review package) approved by user on 2026-11-06; registry status updated to approved.

---
<!-- COMMENTS:END -->
