---
id: TASK-286
title: "Run the security audit on configuration, assets, and the browser surface"
created_date: '2026-11-20T17:44:13Z'
updated_date: '2026-11-23T08:51:08Z'
labels: []
references:
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-07.002
  - MS-037
  - REL-001
  - NFR-007.003
status: In-Progress
assignee:
  - security-officer
milestone: MS-037
dependencies:
  - TASK-283
  - TASK-284
  - TASK-315
ordinal: 70
documentation:
  - src/browser/render.rs
  - src/browser/shell.rs
  - src/config.rs
  - src/sanitize.rs
  - src/record/store.rs
  - src/record/relocate.rs
  - src/browser/mod.rs
  - src/browser/http.rs
  - src/browser/assets.rs
type: audit
---

## Description
## Context
Configuration now declares front-matter keys and presentation strings that reach HTML, and assets are cacheable (ADR-037, ADR-038, ADR-039). This is a release-wide security audit.

## Scope
Audit and record one audit record of type `security` with a severity per finding:
- HTML escaping of config-declared headings, labels, help text, field values, and record content: `src/browser/render.rs::escape` (~24) and every `render::` call site; the CSP literals in `src/browser/render.rs` and `src/browser/shell.rs`.
- Path handling for config-declared `directory`, `prefix`, `frontmatter_key`, `heading`, and file-edit fields: `src/config.rs` validation (~1299-1600), `src/sanitize.rs`, `src/record/store.rs`, `src/record/relocate.rs`.
- The generic `form_post`: `src/browser/mod.rs::form_post` (~2352), `parse_form` (~1921), the Host and Origin allowlist, the method and status surface (415, 400, 404, 409, 422, 303, 500), forged fields, mass assignment (`status`, `version`, `group`, undeclared keys), and the `SingleHtmlMockup` save path (~2023).
- Snapshot reload: `ServingSnapshot` and `reload_if_changed` fail-closed behaviour, stale or partial state, and no configuration error content sent to the client.
- Cache headers: `src/browser/http.rs::CacheClass`, `src/browser/assets.rs`, the ADR-038 classes; poisoning or leakage through a shared cache; `no-store` on every HTML, JSON, error, 404, and 405 response.

## Out of scope
- Do not fix findings; a blocker or high creates a fix task through the planner.
- Do not edit source, requirements, rules, ADRs, specifications, mockups, or test cases.

## Definition of done
One `AUD` record of type `security` with references, a verdict, severity counts, and per-finding severity, file and line, evidence, and disposition. A blocker or high finding is reported to the orchestrator.
## Acceptance Criteria
- [ ] Exactly one audit record of type security is created and references MS-037 and REL-001.
- [ ] The audit covers HTML escaping of every config-declared presentation string and record field that reaches HTML, naming the escape point and the call sites.
- [ ] The audit covers path handling for config-declared directories, prefixes, and field paths and reports traversal, absolute path, symlink, and reserved-key handling.
- [ ] The audit covers the generic form post: forged and undeclared fields, status, version, and group mass assignment, the method and status surface, and the mockup save path.
- [ ] The audit covers snapshot reload fail-closed behaviour and rejects serving stale or partial state or leaking configuration error content.
- [ ] The audit covers the cache classes and confirms no record-derived response is cacheable and no shared cache can leak record content.
- [ ] Every finding has a severity, a file and line, evidence, and a disposition; the record states severity counts and a pass or fail verdict.
- [ ] No blocker or high finding is left open; each is reported to the orchestrator with a proposed fix task.
- [ ] The audit record is referenced in a task comment and papyrus check reports no new finding.
