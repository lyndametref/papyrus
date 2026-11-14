---
id: TASK-124
title: Guard init defaults and registry drift and finish archive terminology
status: To Do
assignee:
  - rust-dev
created_date: '2026-11-14T20:31:00Z'
updated_date: '2026-11-14T20:31:00Z'
labels:
  - rust
  - tests
milestone: m-18
dependencies:
  - TASK-120
references:
  - >-
    docs/architecture/adr/ADR-0022-document-lifecycle-state-and-archive-location.md
documentation:
  - src/init.rs
  - src/config.rs
priority: low
ordinal: 41260
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Why.** Two small drift risks survived m-17. First, init writes eleven document types from a literal DOCUMENT_ARTIFACTS array while Config::default() seeds document_types independently; nothing fails if one changes alone, so a new default type would be creatable through config inference but missing from init (or the reverse). Second, seven typed modules still name a helper active_or_deprecated and a requirement error still says group is deprecated, although the delivered lifecycle is active to archived and deprecated folders are inert; the stale vocabulary invites a future reader to reintroduce deprecated behavior.

**What.** Add one equality test that ties init defaults to the default configuration, and rename the stale identifiers and messages to archived terminology without changing behavior.

**In scope.**

- A test asserting init::DOCUMENT_ARTIFACTS and Config::default().document_types describe the same slugs in the same order, with no deploy entry and no extras either way.
- Renaming active_or_deprecated to active_or_archived in the typed modules that still carry it (audit, ui_spec, test_report, rule, test_case, tech_spec, use_case) and the deprecated local path bindings those helpers take.
- Renaming the requirement group error and helper wording from deprecated to archived.
- Updating any test that asserts the old message.

**Out of scope.**

- Changing lifecycle behavior, statuses, folders, validation, or migration of existing files; deprecated folders stay inert.
- Adding types, changing init output, or touching TASK-112 defaults.
- Reopening m-17 tasks; this is a drift guard on their outcome.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 A test asserts that init::DOCUMENT_ARTIFACTS and Config::default().document_types have the same slugs in the same order, and fails if either side gains, loses, or reorders an entry
- [ ] #2 The same test asserts there is no deploy entry and no duplicate slug on either side
- [ ] #3 No identifier or user-visible message in the typed modules says deprecated for the document lifecycle; active_or_deprecated and the group is deprecated message are renamed to archived wording
- [ ] #4 Behavior is unchanged: no status, folder, validation, or migration outcome differs, and existing tests pass after message updates
- [ ] #5 A source check confirms the remaining deprecated occurrences in src/ belong only to legacy migration or inert-folder handling and are documented in the final summary
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [ ] #1 cargo build, cargo test, and cargo clippy are clean.
- [ ] #2 No new dependency, type, or configuration key is introduced.
- [ ] #3 The final summary lists each renamed identifier and message and the drift test location.
<!-- DOD:END -->
