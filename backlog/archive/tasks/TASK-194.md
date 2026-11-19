---
id: TASK-194
title: "Update release smoke tests to removed-surface contract"
created_date: '2026-11-18T21:35:14Z'
updated_date: '2026-11-19T10:54:24Z'
labels: []
references:
  - AUD-107
  - AUD-108
  - REQ-06.018
  - RE-005
  - ADR-034
  - TC-033
  - TC-034
  - TC-035
status: Done
assignee:
  - test-manager
milestone: MS-027
dependencies:
  - TASK-174
ordinal: 51
documentation:
  - tests/release_document_types_smoke.rs
  - tests/release_native_layout_smoke.rs
type: implementation
---

## Description
Correction for AUD-107 finding 1 and AUD-108 finding 1 (high): both release smoke tests still assert removed browser surfaces, so the full suite is red. Replace the deleted Configuration sidebar anchor used as the sidebar end marker at line 1067 of the document-types smoke test with the nav close tag and rewrite its lifecycle assertions at lines 1204-1210 and 1265-1267 to the removed-surface contract. In the native-layout smoke test, replace the lines 1686-1695 expectation that POST to the configuration route returns 303 with a 404 assertion and keep relocation coverage through the CLI assertions already present above the removed browser block (papyrus config set backlog_directory). Assert that the configuration, lifecycle, and skills routes return 404 and that 404 bodies contain no removed-section markers. Done when cargo test --test release_document_types_smoke --test release_native_layout_smoke passes and a full-suite run leaves only the pre-existing AUD-039 no_drift and characterization failures.
## Comments
- **test-manager**: Correction implemented for AUD-107 finding 1 and AUD-108 finding 1. Earlier short probe entries on this task were a CLI formatting test (papyrus rejects comment text containing path separators) and carry no evidence. release_document_types_smoke t117_08_browser_matrix_over_http: the B-02 sidebar end marker is now the nav close tag searched after the Documents group instead of the removed Configuration anchor; added assertions that the served list sidebar carries no Configuration, Managed artifacts, or Agent skills entry; B-15 rewritten to the removed-surface contract (GET lifecycle and its type alias return 404 with a Record not found body, no removed-section marker, no enabled rows); B-14 restart tail asserts lifecycle and its alias return 404 and keeps the disabled-state proof through the CLI (papyrus config get document_types.note.enabled prints false). release_native_layout_smoke t98_08_relocation_atomicity_and_browser_path: replaced the POST config 303 expectation with 404 assertions for the config, config edit, lifecycle, and skills routes and their type aliases (bodies carry Record not found with no removed markers; POST config returns 404 with no Location header) and added proof that a browser POST relocates nothing (backlog extra.txt intact, moved target absent, papyrus.config.yml unchanged). CLI relocation coverage kept unchanged (papyrus config set backlog_directory native store, unmanaged file and subfolder moved, empty source removed). Checks: cargo fmt --check pass; cargo clippy --all-targets -D warnings pass (HttpResponse header stays used by the no-Location assertion); cargo test --test release_document_types_smoke --test release_native_layout_smoke = 8 passed 0 failed and 7 passed 2 ignored 0 failed; cargo test --no-fail-fast leaves only the pre-existing AUD-039 failures (list_characterization_replay empty fixture, no_drift id formatting) with both release smoke targets green. Files: tests release_document_types_smoke.rs and tests release_native_layout_smoke.rs.
