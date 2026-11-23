---
id: TASK-090
title: >-
  Add per-artifact descriptions and read-only configuration to the browser
  lifecycle page
status: Done
assignee: []
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-13T09:07:00Z'
labels:
  - browser
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-082
  - TASK-083
  - TASK-085
  - TASK-089
references:
  - docs/requirements/business/REQ-006.003.md
  - docs/requirements/business/REQ-006.008.md
  - docs/specifications/browser-managed-artifacts-and-lifecycles.md
  - >-
    docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html
    docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md
    docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md
priority: high
type: feature
ordinal: 39120
---


## Description
REQ-006.003 requires each lifecycle section to describe its artifact and present the configuration attached to it, read-only, with edits confined to the configuration view (REQ-006.008). TASK-95 extends the existing lifecycle page (TASK-72) so every one of the twelve fixed sections carries curated description text and its effective enabled state, ID prefix, and storage folder; the task section also shows configured statuses. Values are display-only and the page links to /config for changes.

Scope:
- In scope: curated descriptions for all twelve fixed types; read-only effective configuration per section; task statuses reuse existing derivation; link to configuration view; tests/fixtures for disabled type, renamed prefix/folder, and custom task statuses.
- Out of scope: changing the covered type set; editing configuration from this page; new record types; changing lifecycle wording; packaging/release verification.

Behavior:
- Each section renders: curated description, existing lifecycle sentence, and a read-only configuration line: "enabled: <bool>; ID prefix: <prefix>; storage folder: <folder>" (plus "Statuses shown above." for Tasks).
- Description text is static content compiled into the binary (offline; not derived from requirements, records, or configuration).
- Enabled/prefix/folder are derived from effective configuration: tasks/milestones/releases/decisions use their dedicated keys; document types use document_types.<key>; decisions aggregate enabled decision types.
- Storage folders join the configured base directory with the type subpath and strip trailing slashes.
- The page provides a link to /config and no editing controls.
- Exactly twelve <section> elements remain, in existing order.

Assumptions:
- Curated descriptions are the static strings established by MOCK-0014/TASK-87.
- Tasks, milestones, and releases are always enabled; decisions are enabled when at least one decision type is enabled.
- A missing document type entry reports enabled=false and an empty prefix/folder rather than failing.
- REQ-006.009 branding and REQ-006.008 config view are already delivered (TASK-93, TASK-94).

References:
- docs/requirements/business/REQ-006.003.md
- docs/requirements/business/REQ-006.008.md
- docs/specifications/browser-managed-artifacts-and-lifecycles.md (section 4a)
- docs/specifications/browser-spec-revisions-task-86.md (section 8)
- docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md
- docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md
- docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html

## Acceptance Criteria
- [x] Each of the twelve fixed artifact sections renders curated description text compiled into the browser binary, available offline, and not derived from requirement files, record content, or configuration.
- [x] Each section renders read-only effective configuration: enabled state, ID prefix, and storage folder; the Tasks section also renders the configured statuses.
- [x] Configuration values on the page are display-only; the page links to the configuration view (/config) and offers no editing controls.
- [x] The fixed covered set of twelve types and the task-status derivation and fallback behavior remain unchanged.
- [x] The page retains exactly twelve <section> elements in the existing order and remains offline-capable.
- [x] Tests or fixtures cover a disabled type, a renamed prefix or folder, and custom task statuses.

## Definition of Done
- [x] #1 Unit/integration tests pass (cargo test --test browser_cli).
- [x] #2 No regressions in existing lifecycle tests (t72a, t72b).
- [x] #3 Documentation and specification links are up to date.
- [x] #4 Acceptance criteria reviewed and signed off.


## Implementation Notes
1. Update lifecycle_page in src/browser/render.rs to emit curated description, lifecycle sentence, and read-only config line per section.
2. Add a helper mapping the twelve fixed types to effective enabled/prefix/folder (plus statuses for Tasks) sourced from Config.
3. Escape all dynamic values; keep exactly twelve <section> elements and existing order.
4. Add link to /config; ensure no editing controls.
5. Update tests in tests/browser_cli.rs for disabled type, renamed prefix/folder, and custom statuses.
6. Run cargo test --test browser_cli and fix regressions.


## Notes
Implemented lifecycle_page per-artifact descriptions and read-only config in src/browser/render.rs. 12 sections unchanged order. Config values derived from Config; storage folders join base+subpath and strip trailing slashes. Disabled/missing doc type reports false/empty. Link to /config included; no editing controls. Verified cargo build/test/clippy/fmt and browser_cli 148 passed.

AC #6: added t95a_lifecycle_disabled_document_type (disabled tech-spec renders enabled: false, 200, twelve sections, no form/input) and t95b_lifecycle_renamed_prefix_and_folder (BRQ prefix and documentation/requirements/business-rules folder in the read-only config line, curated description unchanged). Updated spec §7 with items 5-7. cargo test --test browser_cli: 150 passed; cargo build, clippy -D warnings, fmt --check clean.

## Final Summary

<!-- SECTION:FINAL_SUMMARY:BEGIN -->
Updated src/browser/render.rs lifecycle_page to emit curated description, existing lifecycle sentence, and read-only effective configuration (enabled, ID prefix, storage folder) for all 12 fixed artifact sections; task section retains configured statuses. Values derive from Config; storage folders join the base directory with the type subpath and strip trailing slashes. Page links to /config and has no editing controls. AC #6 tests: t95a (disabled tech-spec renders enabled: false, 200, twelve sections) and t95b (renamed BRQ prefix and documentation/requirements/business-rules folder reflected with curated description intact) in tests/browser_cli.rs; spec §7 items 5-7 added. Verification: cargo test --test browser_cli 150 passed; cargo build, clippy -D warnings, fmt --check clean.
