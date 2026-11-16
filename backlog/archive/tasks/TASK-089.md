---
id: TASK-089
title: Add the browser configuration view that edits configuration through config set
status: Done
assignee:
  - @rust-dev
created_date: '2026-11-11T20:43:00Z'
updated_date: '2026-11-13T07:03:00Z'
labels:
  - browser
  - configuration
  - release-1.0
milestone: MS-017
dependencies:
  - TASK-082
  - TASK-085
references:
  - docs/requirements/business/REQ-06.008.md
  - docs/requirements/business/REQ-01.002.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/specifications/browser-requirements.md
  - >-
    docs/specifications/browser-configuration-view-single-record-layout-and-lifecycle-configuration.md
  - >-
    docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md
  - >-
    docs/specifications/ui/mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html
  - docs/specifications/project-initialization-and-configuration.md
  - docs/specifications/browser-spec-revisions-task-86.md
priority: high
type: feature
ordinal: 39110
---


## Description
REQ-06.008 asks for configuration editing in the browser, while REQ-01.002 requires a folder change to move folders atomically and NFR-04.002 restricts access to the project directory. A browser writer that touched papyrus.config.yml directly would bypass relocation and containment, so the view must go through the same command path as the CLI. This task delivers that view on top of the relocation routine.

Scope (in scope):
- Serve a configuration view for the current project at /config (alias /?type=config), with a sidebar entry labelled Configuration placed after the document types, following MOCK-0014.
- List every effective configuration key (including built-in defaults) with its effective value and a source badge: explicit when the key is set in papyrus.config.yml, otherwise default.
- Offer edit controls only for keys papyrus config set accepts (scalar leaves of Config; array-valued keys such as statuses are shown read-only). The view applies the same keys, validation, and precedence as the CLI and cannot persist a key or value the CLI rejects.
- Delegate every save to commands::config_set (and commands::config_unset for clearing), so folder-key changes perform the full atomic relocation of REQ-01.002 and conflicts are reported without persisting a value.
- Reject any path that does not pass the project-directory boundary check before a write.
- Never read or display environment-variable values (PAPYRUS_DIR, PAPYRUS_CONFIG, PAPYRUS_BROWSER_*, NO_COLOR) as editable configuration, and treat no configuration key as secret (no masking).
- Link to /lifecycle for per-artifact read-only configuration; do not duplicate editing there.

Out of scope (handled by other tasks):
- Per-artifact descriptions and read-only configuration on the lifecycle page (TASK-95).
- Console verbosity / debug request logging (TASK-96).
- Removal of legacy serializers and the browser write path (TASK-97).
- Single record view layout, reference links, NFR grouping, glossary landing, and header branding (TASK-93).

Behavior and contracts:
- The view re-reads effective configuration on every request; it keeps no session state or cache.
- Effective values come from Config::flattened (file plus built-in defaults). Explicitness is determined by reading the raw papyrus.config.yml and checking whether the dotted key is present.
- Folder keys (backlog_directory, decision_directory, doc_directory, document_types.<type>.directory) are sanitized and joined to the project directory through the existing folder_paths/ensure_within checks inside config_set; a containment violation is reported as a 422 validation error and no write occurs.
- A failed relocation (destination conflict or write failure) surfaces the config_set validation error and leaves both folders and the configuration unchanged.
- Edit interaction follows the existing record-edit pattern: GET /config renders the table; GET /config/edit?key=<key> renders a form for one key; POST /config applies key + value (or clears when an unset flag is set) and returns 303 to /config on success, or 422 with the form re-rendered on validation failure.
- Error responses use the baseline HTML error pages (400/404/405/415/500) and JSON error object conventions from browser-requirements.md.

Assumptions:
- Assumption: MOCK-0014 (approved 2026-11-12 UTC) is the sole visual authority for the configuration surface; no UI is invented outside it.
- Assumption: all scalar leaves produced by Config::flattened except statuses are editable, because papyrus config set accepts a YAML scalar for each of them; statuses remains read-only because it is an array.
- Assumption: the edit form reuses the existing template and validation styling; no new MOCK id is allocated.
- Assumption: environment overrides are not part of the displayed effective value for configuration keys; the view reflects file plus built-in defaults, matching config list.
- Assumption: no optimistic-concurrency token is required for configuration saves, because configuration is single-file and re-read on every request; the last write wins, matching the CLI.

Open questions: none. Recommended defaults from browser-requirements.md section 4d and MOCK-0014 are used for route, sidebar placement, and form encoding.

Verification:
- cargo build succeeds with no warnings introduced by this change.
- cargo test passes, including new or updated browser tests for effective-versus-explicit display, rejection parity with the CLI, a folder move performed from the view, and containment refusal.
- cargo clippy --all-targets -- -D warnings and cargo fmt --check pass.
- Manual smoke comparison against MOCK-0014 for the configuration surface.

## Acceptance Criteria
- [x] The browser serves a configuration view at /config (alias /?type=config) for the current project, reachable from a sidebar entry labelled Configuration placed after the document types, following MOCK-0014.
- [x] The view lists every effective configuration key, including built-in defaults, with its effective value and a source badge of explicit when the key is set in papyrus.config.yml, otherwise default.
- [x] Edit controls are offered only for keys papyrus config set accepts (scalar leaves of Config; array-valued keys such as statuses are read-only); the view applies the same keys, validation, and precedence as the CLI and cannot persist a key or value the CLI rejects.
- [x] Editing a key uses the same validation as papyrus config set; a value the CLI would reject is rejected with a 422 response and papyrus.config.yml is not changed.
- [x] Saving a folder key (backlog_directory, decision_directory, doc_directory, or document_types.<type>.directory) invokes the shared relocation routine through commands::config_set; on conflict nothing is moved, the configuration is not persisted, and the conflict is reported to the user.
- [x] No submitted path can escape the project directory; containment violations are rejected before any write.
- [x] Environment-variable values (PAPYRUS_DIR, PAPYRUS_CONFIG, PAPYRUS_BROWSER_*, NO_COLOR) are never displayed as editable configuration, and no configuration key is masked or treated as secret.
- [x] The view links to /lifecycle for per-artifact read-only configuration and does not duplicate editing there.
- [x] Tests cover effective-versus-explicit display, rejection parity with the CLI, a folder move performed from the view, and containment refusal.

## Definition of Done
- [x] #1 Implementation follows approved MOCK-0014 for the configuration surface; no UI is invented outside the approved mockup.
- [x] #2 cargo build succeeds with no warnings introduced by this change.
- [x] #3 cargo test passes, including new or updated browser tests for effective-versus-explicit display, rejection parity with the CLI, a folder move performed from the view, and containment refusal.
- [x] #4 cargo clippy --all-targets -- -D warnings and cargo fmt --check pass.
- [x] #5 No requirement or business-rule files were modified.
- [x] #6 Configuration writes go only through commands::config_set/config_unset; no browser handler writes papyrus.config.yml directly or moves a folder.


## Implementation Notes
1. Review MOCK-0014 and the revised browser specifications (browser-requirements.md section 4d and 10a, browser-spec-revisions-task-86.md section 9, ui/browser-configuration-view-...-ui.md section 4) to fix the exact markup, table columns, badges, responsive 320 px layout, and error pages for the configuration surface.
2. Add a /config route (alias /?type=config) and a sidebar entry labelled Configuration after the document types, reusing the shared shell and header branding from TASK-93.
3. Render the configuration table from Config::flattened: Key, Effective value, Source (explicit/default), and an Edit action. Determine explicitness by reading the raw papyrus.config.yml with yaml_get.
4. Restrict Edit controls to scalar keys papyrus config set accepts (all flattened keys except statuses); render array/object keys as read-only values.
5. Add GET /config/edit?key=<key> and POST /config handlers. The GET renders a single-key form with the current effective value; the POST calls commands::config_set (or commands::config_unset when clearing) and returns 303 to /config on success or 422 with the form re-rendered on validation failure, preserving submitted values and showing field-level messages.
6. Map AppError::Validation from config_set/config_unset to 422 (including relocation conflicts and containment violations) and AppError::Runtime to 500, so a conflict is reported without persisting a value and no folder is moved.
7. Ensure folder keys flow through the existing relocation routine and ensure_within boundary checks; verify no browser code writes papyrus.config.yml directly or moves a folder.
8. Add tests covering effective-versus-explicit display, rejection parity with the CLI, a folder move performed from the view, and containment refusal, plus route/alias and sidebar placement checks.
9. Verification: cargo build, cargo test, cargo clippy --all-targets -- -D warnings, cargo fmt --check; manual smoke comparison against MOCK-0014.


## Notes
Implemented /config and /?type=config routes, sidebar Configuration entry, config table with source badges, /config/edit form, POST handling via commands::config_set/config_unset, containment mapping to 422. Verified build/test/clippy/fmt and manual folder move/containment/rejection.
