---
id: TASK-79
title: 'Design review: use cases, test cases, and test reports browser UI mockup'
status: Done
assignee:
  - '@ui-designer'
created_date: '2026-11-10T20:16:00Z'
updated_date: '2026-11-11T08:15:00Z'
labels:
  - design-review
  - ui-design
milestone: m-15
dependencies:
  - TASK-78
references:
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-05.010.md
  - docs/requirements/business/REQ-05.012.md
  - docs/requirements/business/REQ-05.013.md
  - docs/requirements/business/REQ-05.014.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-06.003.md
  - docs/requirements/business/REQ-06.004.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/business-rules/BR-018-use-case-document-type-behavior.md
  - docs/business-rules/BR-019-test-case-document-type-behavior.md
  - docs/business-rules/BR-020-test-report-document-type-behavior.md
  - docs/specifications/use-cases-test-cases-and-test-reports-management.md
  - docs/specifications/ui/mockups/README.md
  - >-
    docs/architecture/adr/ADR-0007-yaml-front-matter-as-single-record-metadata-source.md
  - >-
    docs/architecture/adr/ADR-0009-shared-record-envelope-and-unified-write-path.md
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
documentation:
  - docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html
  - docs/specifications/ui/mockups/README.md
priority: medium
type: task
ordinal: 35200
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
UI designer produces and obtains explicit user approval for the use-case, test-case, and test-report browser mockup package before TASK-81 (Expose use cases, test cases, and test reports in the local browser) is implemented.

Deliverable: one new self-contained HTML review package under docs/specifications/ui/mockups/ (next available MOCK id, expected MOCK-0013), a UI description document under docs/specifications/ui/ (expected browser-use-cases-test-cases-and-test-reports-ui.md), and a registry update, following the approved pattern of MOCK-0001..MOCK-0012. Layout only: no branding, colour palette, logo, or technology/library references.

Scope:
- Reuse the approved shell (header, persistent document-type sidebar, main, footer, one h1) and add sidebar entries for Use Cases, Test Cases, and Test Reports plus category selector values (recommended `use-case`, `test-case`, `test-report`).
- For each of the three types: list rows with ID + title and the first paragraph of the description as a preview; empty states; long-title wrapping; deprecated records excluded.
- View: ID, title, description, applicability (when present); breadcrumb `<Type> › ID`; single Edit action; deprecated records unreachable.
- Edit form: title (required), description (required, multi-line), applicability (optional, multi-line); pristine, dirty, validation-error (422), conflict (409), and 320 px states; hidden version input with dashed annotation.
- Front-matter metadata projection for all three types: list/detail/edit views show id, title, created_date, updated_date, labels, references (plus type-specific keys if any); created_date and updated_date are read-only in edit; metadata edits target front matter, not body sections (per MOCK-0010, REQ-05.010, REQ-06.004).
- Error pages reused from MOCK-0001.
- Responsive 320 px list and edit form with sidebar toggle and annotation legend.
- Deprecated records of all three types excluded from lists and navigation (REQ-05.007, BR-018/BR-019/BR-020).

Out of scope:
- Browser implementation (TASK-81), CLI behaviour (TASK-78), search (TASK-80), packaging (TASK-82).
- Creating, deprecating, or restoring records from the browser (CLI-only); the browser only lists/views/edits active records and renders the deprecated 404.
- Modifying approved MOCK-0001..MOCK-0012 in place; allocate a new MOCK id.
- New dependencies, crates, or ADRs.
- Inventing field-length or validation rules not present in use-cases-test-cases-and-test-reports-management.md or BR-018/BR-019/BR-020; follow those sources.

Assumptions:
- Assumption: The browser surface for these three types reuses the transport, asset, rendering, security, and configuration baseline of TASK-6/ADR-0005/ADR-0006 and the decision-record extension (TASK-15); no new ADR.
- Assumption: Sidebar entries are labelled "Use Cases", "Test Cases", and "Test Reports" and are placed after "Technical Specifications" (or at the end of the document-type list); ui-designer confirms final ordering.
- Assumption: Category selector values are `use-case`, `test-case`, and `test-report` (default remains `req`); any other value is 400, consistent with the baseline.
- Assumption: The extension package lists all document-type sidebar entries, with only the three new types' views generated and prior entries non-clickable, consistent with MOCK-0003/MOCK-0009.
- Assumption: Record file format follows use-cases-test-cases-and-test-reports-management.md: `# <ID> <title>` then `## Description`, then `## Applicability` when present.
- Assumption: Front-matter metadata follows REQ-05.010: general fields only (id, title, created_date, updated_date, labels, references); description and applicability are body fields.
- Assumption: Mockup status starts draft/pending and becomes approved only after explicit user approval; TASK-81 must not implement until approved.
- Assumption: Applicability is multi-line and rendered with white-space: pre-wrap (recommended); title/description length follow baseline field rules.
- Assumption: List rows show the first paragraph of the description as a preview (recommended), consistent with audit/technical specifications.
- Assumption: No validate_mockups.sh script exists in this repository; registry/HTML checks are manual.

Open questions:
- OQ-1: Exact sidebar labels and ordering for the three entries (recommended "Use Cases", "Test Cases", "Test Reports" after "Technical Specifications").
- OQ-2: Category query values (`use-case`, `test-case`, `test-report` recommended) for the ?type= selector.
- OQ-3: Whether the three types share one consolidated sidebar entry or three separate entries (recommended three separate, consistent with other types).
- OQ-4: Whether applicability is editable in the browser or read-only (recommended editable multi-line textarea, consistent with description).
- OQ-5: Whether front-matter metadata is demonstrated on all three types or a representative subset (recommended all three, since this ticket is specifically about them).
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 A new self-contained HTML review package is created under docs/specifications/ui/mockups/ with the next available MOCK id (expected MOCK-0013), containing a review landing and web prototype; styles are inlined and no external file, script, font, image, or network reference is used.
- [ ] #2 The package reuses the approved shell and adds sidebar entries and category selectors for Use Cases, Test Cases, and Test Reports; list rows show ID + title (with first-paragraph preview), empty states, long-title wrapping; deprecated records are excluded.
- [ ] #3 The package includes, for each of the three types, a view (ID, title, description, applicability when present; breadcrumb <Type> › ID; single Edit action) and an edit form (title required, description required multi-line, applicability optional multi-line; pristine, dirty, validation-error, conflict, and 320 px states).
- [ ] #4 Front-matter metadata projection for all three types is shown in list/detail/edit views (id, title, created_date, updated_date, labels, references); created_date and updated_date are read-only in edit; metadata edits target front matter, not body sections.
- [ ] #5 Deprecated records of all three types are excluded from lists and navigation and are not reachable, consistent with REQ-05.007 and BR-018/BR-019/BR-020.
- [ ] #6 Error pages are reused from MOCK-0001 (404 unknown/deprecated, 400, 405, 415, 500) and 320 px list/edit views include sidebar toggle and annotation legend.
- [ ] #7 A UI description document is created/updated under docs/specifications/ui/ (expected browser-use-cases-test-cases-and-test-reports-ui.md) describing screens, states, elements, interactions, active elements, MOCK register, assumptions, and approval status.
- [ ] #8 The mockup registry docs/specifications/ui/mockups/README.md is updated with the new MOCK row (id, type, name, slug, artifact, platform, views, status, source) and notes.
- [ ] #9 The mockup is explicitly approved by the user before TASK-81 implementation; registry and UI description status reflect approved (not draft/pending).
- [ ] #10 No implementation code, new dependencies, crates, or ADRs are introduced; approved MOCK-0001..MOCK-0012 are not modified in place.
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
MOCK-0013 created as draft by @ui-designer on 2026-11-11 (UTC); UI description docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md and registry updated. Awaiting explicit user approval before TASK-81 implementation. validate_mockups.sh reports no errors for MOCK-0013; pre-existing failures on MOCK-0001/MOCK-0002 (data-review=deck) are unrelated and were not modified.
<!-- SECTION:NOTES:END -->

## Comments

<!-- COMMENTS:BEGIN -->
author: @ui-designer
created: 2026-11-11T07:26:00Z

---

UI mockup created: MOCK-0013 (docs/specifications/ui/mockups/MOCK-0013-browser-use-cases-test-cases-and-test-reports-review.html); UI description: docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md. Status draft, pending explicit user approval.

---

author: @ui-designer
created: 2026-11-11T07:27:00Z

---

MOCK-0013 (draft) and docs/specifications/ui/browser-use-cases-test-cases-and-test-reports-ui.md created by @ui-designer on 2026-11-11 (UTC); awaiting explicit user approval before TASK-81 implementation. No status change to approved.

---
<!-- COMMENTS:END -->
