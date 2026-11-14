---
id: TASK-115
title: >-
  Update business rules for artifact categories and config-defined document
  types
status: To Do
assignee:
  - business-analyst
created_date: '2026-11-13T21:26:00Z'
updated_date: '2026-11-13T21:59:00Z'
labels:
  - docs
  - requirements
milestone: m-17
dependencies:
  - TASK-108
  - TASK-109
  - TASK-110
  - TASK-111
  - TASK-112
documentation:
  - docs/requirements/business/REQ-07.001.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/requirements/business/REQ-04.002.md
priority: high
ordinal: 41090
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
The business-rule pages still describe a fixed per-type command set, the `nfreq` alias, `deprecate`/`restore` for documents, and a configuration key model without document slugs, statuses, or fail-closed validation. After the CLI, configuration, and generic handler land, those rules are wrong and will mislead both users and future tasks. This task aligns the affected rules with the validated requirements and the accepted ADRs; it is documentation only.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 BR-001 describes the dynamic papyrus <document-slug> command family, the baseline verbs, the status verb, and the removal of static document commands and nfreq.
- [ ] #2 BR-009 (req) and BR-011 to BR-014 and BR-017 to BR-020 describe each default type as an ordinary document_types entry with the baseline family and its declared extension verbs, its prefix/folder, and its document status model.
- [ ] #3 BR-010 is retired: no nfreq command exists, and non-functional requirements are documented only under the nfr slug.
- [ ] #4 BR-016 documents document_types keyed by slug, prefix/directory/enabled/statuses/default_status, whole-map semantics, list-valued statuses, case-insensitive built-in collision rejection, prefix and directory uniqueness, fail-closed validation, and the two repair surfaces.
- [ ] #5 Every updated rule distinguishes the document main statuses active/archived from custom front-matter statuses, states that archive is the only way to archive and there is no restore, keeps decision statuses separate, and keeps milestone/release lifecycles fixed.
- [ ] #6 No rule retains deprecate, restore-for-documents, deploy as a default, or a static per-type command table; rule indexes and cross-references resolve.
- [ ] #7 BR-008 states that decisions have no archive command and no archive state; the archive command line and description are removed, matching REQ-04.002.
<!-- AC:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-13T21:59:00Z

---

Scope addition from TASK-105 refinement Q2: TASK-108 removes the `papyrus decision archive` CLI surface; this task removes the matching BR-008 documentation so the rule and the command set agree with REQ-04.002.

---
<!-- COMMENTS:END -->
