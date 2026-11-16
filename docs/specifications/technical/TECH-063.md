---
id: TECH-063
title: "Browser Configuration View, Single Record View Layout, and Lifecycle Configuration (Design Review)"
created_date: '2026-11-16T17:46:52Z'
updated_date: '2026-11-16T17:47:28Z'
labels: []
references:
  - ADR-0010
  - ADR-0012
  - ADR-0013
  - MOCK-0001
  - MOCK-0013
  - MOCK-0014
  - REQ-01.002
  - REQ-01.003
  - REQ-06.002
  - REQ-06.003
  - REQ-06.006
  - REQ-06.007
  - REQ-06.008
  - REQ-06.009
  - REQ-06.010
  - REQ-06.011
  - REQ-06.012
  - TASK-86
  - TASK-87
  - TASK-93
  - TASK-94
  - TASK-95
status: verified
---
# TECH-063 Browser Configuration View, Single Record View Layout, and Lifecycle Configuration (Design Review)

## Description
**Ticket:** TASK-87
**Milestone:** m-16 (Release 1.0)
**Status:** Refined (execution-ready; UI mockup approved 2026-11-12)
**UI mockups:** [`MOCK-0014`](ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html) — `approved` (explicitly approved by the user on 2026-11-12 (UTC), TASK-87); UI description [`ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md`](ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md)

## Purpose

Produce the approved clickable UI review package that the browser implementation tasks (TASK-93, TASK-94, TASK-95) must follow for the revised browser contract introduced by REQ-06.003 and REQ-06.006 through REQ-06.012: an editable configuration view, a consistent single record view layout, curated per-artifact descriptions with read-only configuration on the lifecycle page, non-functional requirement grouping under owning groups, shared header branding, record references as links, and a concise glossary landing page. The package is created under TASK-87 and approved by the user before any implementation starts; no browser task may invent layout or skip the approval gate.

This ticket is UI design work only. It does not change requirements, business rules, specifications, or implementation code. The revised browser contract is already specified in `docs/specifications/browser-requirements.md` (§4a–§4d, §10a), `docs/specifications/browser-managed-artifacts-and-lifecycles.md` (§4a), and `docs/specifications/browser-glossary-terms.md` (§3a), produced by TASK-86.

## Scope

### In scope

- Designing one new numbered mockup package (MOCK-0014) with a review landing page, design deck, and clickable web prototype covering every surface listed in [Surfaces](#surfaces).
- Authoring the matching UI description document under `docs/specifications/ui/`.
- Registering MOCK-0014 in `docs/specifications/ui/mockups/README.md` with status `approved` (explicitly approved by the user on 2026-11-12 (UTC), TASK-87).
- Submitting the package for explicit user approval and recording `approved` in the registry only after explicit approval.
- Reusing the approved wireframe styles (grayscale, layout conventions) of MOCK-0001..MOCK-0013; no branding, color palette, logo, or visual identity.

### Out of scope

- Implementing the revised browser behavior (TASK-93, TASK-94, TASK-95).
- Revising existing approved mockup packages in place (MOCK-0001..MOCK-0013 revisions are separate follow-ups; this package consolidates the new surfaces for review).
- Changing requirements (`docs/requirements/`), business rules (`docs/business-rules/`), or the revised browser specifications.
- Creating, deprecating, restoring, or renaming records.

## Surfaces covered (MOCK-0014)

1. **Configuration view** — effective configuration values with explicit/default markers; editable keys exactly those `papyrus config set` accepts; folder-move relocation performed via config set; no path outside the project; link to the lifecycle page for read-only per-artifact configuration; route `/config` with `/?type=config` alias; sidebar entry labelled `Configuration`.
2. **Single record view layout** — summary card showing only record ID, created date, updated date; sub-requirement additionally shows owning group; labels as a taglist under the title; dependencies and references at the end as native HTML `<details>` sections shown only when at least one value exists.
3. **Shared header branding** — project name large and bold, "Papyrus" mention smaller, "P" mark unchanged, on every page.
4. **NFR grouping** — non-functional atomic requirements listed under their owning group (derived from ID prefix `NFR-XX.YYY` → `NFR-XX`); group heading shows group title or number fallback; an empty group renders as a group row with no member rows and the shared count note is computed from the rows emitted; no catch-all section.
5. **Dependency and reference links** — task dependencies, milestone, `references`, owning group as hyperlinks; unresolved references rendered as plain text without error; works offline.
6. **Glossary landing page** — created/updated dates omitted; labels as tags; references omitted; detail views follow the single record view layout.
7. **Lifecycle page per-artifact configuration** — one section per fixed record type with curated description plus read-only effective configuration (enabled, ID prefix, storage folder; task section also shows configured statuses); link to the configuration view; no editing on this page.
8. **Responsive and accessibility** — 320 CSS px layouts with the sidebar toggle; landmarks, `aria-current="page"`, visible focus, no JavaScript required for core navigation; error pages (404 unknown/deprecated, 400, 405, 415, 500).

## Deliverables

- `docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html` (status `approved`).
- `docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md`.
- Registry entry in `docs/specifications/ui/mockups/README.md` (status `approved`).
- Updated TASK-87 references (MOCK-0014, UI description).

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06.003, REQ-06.006, REQ-06.007, REQ-06.008, REQ-06.009, REQ-06.010, REQ-06.011, REQ-06.012, REQ-06.002, REQ-01.002, REQ-01.003 |
| ADR | ADR-0010, ADR-0012, ADR-0013 |
| Ticket | TASK-86 (prerequisite), TASK-87, downstream TASK-93, TASK-94, TASK-95 |
| UI mockups | MOCK-0014 (approved) — `docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html` |
| UI description | `docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md` |
| Related specification | `docs/specifications/browser-requirements.md` (§4a–§4d, §10a), `docs/specifications/browser-managed-artifacts-and-lifecycles.md` (§4a), `docs/specifications/browser-glossary-terms.md` (§3a), `docs/specifications/browser-spec-revisions-task-86.md` (§9) |
| Baseline shell | MOCK-0001 (approved) |

## Assumptions

- Assumption: The revised browser contract is already specified by TASK-86; this ticket only designs the UI for it and does not re-specify behavior.
- Assumption: MOCK-0014 is the next sequential global MOCK id and is never reused or renumbered.
- Assumption: The mockup was explicitly approved by the user on 2026-11-12 (UTC) (TASK-87); registry status is `approved` and implementation may begin.
- Assumption: Existing MOCK ids are not renumbered; revisions to MOCK-0001..MOCK-0013 (where required) are separate follow-ups and do not block this package.
- Assumption: The configuration view route/sidebar placement defaults recorded in `browser-requirements.md` §4d (OQ-1) are subject to approval during this review; any change is recorded in the UI description and specification without changing other behavior.

## Open questions

- OQ-1 (recommended default: sidebar entry labelled `Configuration`, placed after the document types, linking to `/config`) — whether the configuration view is reached from the sidebar or a header link. Subject to user approval during review.
- OQ-2 (recommended default: per-type specifications reference the shared single record view layout by section number) — whether per-type specs restate or only reference the shared layout. Subject to reviewer preference.
- OQ-3 (recommended default: disclosure sections collapsed by default) — initial open/closed state of dependency/reference `<details>`.
- OQ-4 (recommended default: reuse approved responsive patterns for 320 px) — whether any new responsive pattern is needed for the configuration table.

## Verification

- `docs/specifications/ui/mockups/validate_mockups.sh` reports no errors for MOCK-0014.
- Registry entry status is `approved` (explicitly approved by the user on 2026-11-12 (UTC), TASK-87).
- Every REQ-06.003 / REQ-06.006–REQ-06.012 surface listed in [Surfaces](#surfaces) is represented in the prototype and UI description.
