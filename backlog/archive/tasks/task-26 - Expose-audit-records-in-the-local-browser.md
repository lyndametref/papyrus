---
id: TASK-26
title: Expose audit records in the local browser
status: Done
assignee: []
created_date: '2026-11-02T20:19:00Z'
updated_date: '2026-11-07T10:13:00Z'
labels:
  - browser
  - documentation
milestone: m-9
dependencies:
  - TASK-47
  - TASK-49
references:
  - README.md
  - docs/requirements/business/REQ-05.006.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-014-audit-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/specifications/audit-records-management.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/browser-business-rules.md
  - docs/specifications/browser-audit-records.md
  - docs/specifications/ui/browser-audit-records-ui.md
  - docs/specifications/ui/mockups/MOCK-0008-audit-records-review.html
type: feature
ordinal: 27300
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
Expose audit records in the local browser: list, view, and edit active audit records (ID, title, audit type, content), reusing the document views built for earlier document types. The transport, asset delivery, rendering, security, and configuration baseline of TASK-6 and ADR-0005/ADR-0006 is reused unchanged; this ticket adds one documentation category (`audit-record`), its record model, screens, API endpoints, and tests.

Scope (in):
- Sidebar entry "Audit Records" immediately after "Business Rules"; `?type=audit-record` category selector.
- List active audit records (ID + title, first paragraph of content as preview, count, empty state); deprecated records excluded.
- View: title as h1, audit type shown at the beginning (badge next to title/ID plus "Audit type" row in metadata), content as body text with pre-wrap and no "Content" heading, breadcrumb, single Edit action.
- Edit: title (required, 1-200), audit_type (select security/licence/architecture, default security), content (optional, up to 20000); hidden version + annotation; pristine/dirty/422/409/320 px; no JS required.
- Field-preserving writes: only the title heading and the `## Audit Type` and `## Content` sections change; preamble and unknown sections preserved byte-for-byte.
- Read/write JSON API (`/api/audit-records`, alias `/api/audit-record`, `/api/doc/{id}`).
- Deprecated audit records hidden and unreachable (404 on view/edit/POST/API).
- Unit and integration tests extending `tests/browser_cli.rs`.

Scope (out):
- Create, deprecate, restore, delete from the browser (CLI-only).
- Cross-type navigation beyond the sidebar and final completeness check (TASK-19).
- Search in browser UI, Markdown rendering, rich-text editing, attachments, uploads.
- Auth, sessions, cookies, TLS.
- New dependencies, crates, or ADRs.
- Packaging/verification (TASK-50).
- Custom (non-built-in) audit types in the edit select (browser limited to security/licence/architecture per MOCK-0008).

Record sources and IDs:
- Active directory `<doc_directory>/<document_types.audit.directory>/` (default `audit-records/`), prefix default `AUD`, resolved from config; never hardcoded.
- Only regular `*.md` files directly in the active folder whose stem matches `<prefix>-<digits>` are records; `deprecated/`, non-matching files, and non-`.md` files ignored; no recursion.
- ID is the file stem, exact case-sensitive match; never derived from content or accepted from the request.

Record model:
- title from first line `# <ID> <title>` (ID alone when no title).
- audit_type from `## Audit Type` body; absent/empty defaults to `security`.
- content from `## Content` body; absent/empty is null/empty.
- Unknown sections and preamble preserved on save.

Validation:
- title: trimmed, 1-200 Unicode scalar values, no control characters.
- audit_type: one of security, licence, architecture (default security).
- content: trimmed, at most 20000 Unicode scalar values, LF allowed, CR/CRLF normalized to LF, TAB allowed, other control chars rejected, empty clears, lines starting with `## ` rejected.
- Field messages: "Title is required.", "Title must be at most 200 characters.", "Title cannot contain control characters.", "Audit type is required.", "Audit type must be one of security, licence, or architecture.", "Content must be at most 20000 characters.", "Content cannot contain control characters.", "Content contains a line starting with ## ."
- 422 on validation failure with no write; 400 on missing/malformed version; 409 stale version with "record changed on disk; reload the form".

Persistence:
- Field-preserving atomic write (baseline section 7.10): replace heading, Audit Type and Content sections (insert/remove per rules), preserve all other sections and preamble, LF endings, single trailing newline.
- ID, directory, deprecation state never change on save.
- Symlink escapes rejected (404); no temp file left behind; save-then-reread.

Deprecated visibility decision (resolved): hide deprecated audit records from browser lists and make them unreachable, consistent with the other document types (REQ-05.007 leaves audit visibility unstated; this ticket adopts hide).

Security/offline/a11y: baseline sections 6, 9, 10 of browser-requirements.md apply unchanged (embedded assets, escaping, CSP/headers, Host/Origin validation, request limits, timeouts, Firefox/Edge/Safari 100+, accessibility basics, no external resources, no inline scripts/styles).

Assumptions:
- Assumption: transport/asset/rendering/security/config baseline of TASK-6 and ADR-0005/ADR-0006 reused without change; one category and record type added only.
- Assumption: audit record file is Markdown with fixed section template; browser edits are field-preserving and never regenerate the file from a parsed model.
- Assumption: `## Audit Type` body is the source of audit type; absent/empty defaults to security on read.
- Assumption: browser edit restricts audit type to the three built-in values per approved MOCK-0008; records created via CLI with a custom type are viewable but cannot be edited without changing the type.
- Assumption: content may be cleared on edit, removing the `## Content` section.
- Assumption: native `<select>` is operable without JavaScript; no tag-field progressive enhancement needed.
- Assumption: JSON API exposes the three editable fields; no status/task/supersession metadata and no requirement references.
- Assumption: real-browser verification (NFR-05.001) is a deferred follow-up.
- Assumption: no new dependencies, crates, or ADRs.

Open questions:
- OQ-1 (non-blocking, product): whether the browser should support editing non-built-in (custom) audit types created via the CLI.
- OQ-2 (non-blocking, layout): visual treatment of the audit type badge (MOCK-0008 is authority).
- OQ-3 (non-blocking, follow-up): real-browser verification deferred.

References for context/rationale: docs/specifications/browser-audit-records.md (this feature spec), docs/specifications/audit-records-management.md (TASK-47), docs/specifications/browser-requirements.md (TASK-6 baseline), docs/specifications/browser-business-rules.md (TASK-25 sibling), docs/specifications/ui/browser-audit-records-ui.md, MOCK-0008.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 AC-1: ?type=audit-record renders the category, the default stays req, other values are 400, and the sidebar entry is ordered after Business Rules with aria-current=page and omitted when the type is disabled.
- [x] #2 AC-2: The list shows active records only, ascending by ID, with ID + title links, content preview, count, empty state, and no non-record or deprecated files.
- [x] #3 AC-3: The view shows ID, form, category, title, audit type (badge and metadata row), content (no Content heading), breadcrumb, and one Edit action.
- [x] #4 AC-4: The edit form matches MOCK-0008: title, audit type select, content textarea, hidden version plus annotation, Save/Cancel, pristine/dirty/422/409/320 px, and operates without JavaScript.
- [x] #5 AC-5: An edit round-trip persists all three fields and is visible in the browser and through papyrus audit view.
- [x] #6 AC-6: Editing a record preserves every section and line not being edited; no whole-file regeneration occurs.
- [x] #7 AC-7: Title extraction and update follow the spec (heading); unknown sections preserved.
- [x] #8 AC-8: Validation returns 422 with the documented messages and writes nothing (title, audit type, content).
- [x] #9 AC-9: A missing/malformed version is 400; a stale version is 409 with the documented message and no write.
- [x] #10 AC-10: Deprecated audit records are hidden and return 404 on view, edit, POST, and API.
- [x] #11 AC-11: The JSON API returns the documented summary/full shapes, supports the documented POST semantics, and rejects wrong types.
- [x] #12 AC-12: Writes are atomic, project-local, leave no temporary file, and symlink escapes return 404.
- [x] #13 AC-13: The interface remains offline-capable with no external resources, escaped content, unchanged security headers, and keyboard-operable controls.
- [x] #14 AC-14: Build, tests, clippy, and formatting pass with existing dependencies only.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Implementation matches docs/specifications/browser-audit-records.md
- [x] #2 Unit and integration tests added/updated in tests/browser_cli.rs
- [x] #3 cargo test, cargo clippy, cargo fmt pass
- [x] #4 No new dependencies, crates, or ADRs introduced
- [x] #5 MOCK-0008 (approved) recorded as layout source of truth
<!-- DOD:END -->

## Comments

<!-- COMMENTS:BEGIN -->
created: 2026-11-07T10:13:00Z

---

Validated 2026-11-07 (UTC): quality-gatekeeper Pass with findings (14/14 AC, 5/5 DoD); cargo test 101 browser_cli passed; auto-closed under no-validation mode. Non-blocking: unused pub(crate) widenings in audit.rs/rule.rs (carried from m-8), NFR-05.001 deferred.

---
<!-- COMMENTS:END -->
