---
id: TASK-28
title: >-
  Review proposed ADR-0005 embedded dependency-free HTTP server and
  server-rendered UI
status: Done
assignee:
  - '@architect'
created_date: '2026-11-03T21:11:00Z'
updated_date: '2026-11-04T18:36:00Z'
labels:
  - architecture
  - adr
  - review
milestone: m-2
dependencies:
  - TASK-5
  - TASK-6
  - TASK-7
references:
  - >-
    docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md
  - docs/specifications/browser-requirements.md
  - docs/architecture/adr/ADR-0001-implementation-language-rust.md
  - docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md
  - TASK-5
  - TASK-6
  - TASK-7
  - docs/specifications/search-browser-milestone-packaging-and-verification.md
  - docs/business-rules/BR-004-browser-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
  - docs/requirements/business/REQ-06.md
  - docs/requirements/business/REQ-06.002.md
  - docs/requirements/business/REQ-05.001.md
  - docs/requirements/business/REQ-05.002.md
  - docs/requirements/business/REQ-05.007.md
  - docs/requirements/business/REQ-01.003.md
  - docs/requirements/non-functional/NFR-01.002.md
  - docs/requirements/non-functional/NFR-01.003.md
  - docs/requirements/non-functional/NFR-02.001.md
  - docs/requirements/non-functional/NFR-02.002.md
  - docs/requirements/non-functional/NFR-04.001.md
  - docs/requirements/non-functional/NFR-04.002.md
  - docs/requirements/non-functional/NFR-05.001.md
  - docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - >-
    docs/architecture/adr/ADR-0006-optimistic-concurrency-and-atomic-record-writes.md
  - docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html
  - docs/specifications/ui/browser-requirements-ui.md
  - src/browser/mod.rs
  - src/browser/http.rs
  - src/browser/records.rs
  - src/browser/render.rs
  - src/browser/templates/list.html
  - src/browser/templates/view.html
  - src/browser/templates/edit.html
  - src/browser/templates/error.html
  - src/browser/app.css
  - src/browser/app.js
  - src/config.rs
  - src/context.rs
  - Cargo.toml
  - tests/browser_cli.rs
  - tests/search_browser_smoke.rs
  - docs/packaging.md
type: task
ordinal: 7800
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
ADR-0005 was written during the TASK-6 architecture checkpoint with status **Proposed**. It fixes the browser transport, asset delivery, rendering, module boundaries, and security posture that every later browser increment (TASK-11, TASK-15, TASK-19, TASK-23 … TASK-27) inherits, so it must be decided before m-2 implementation starts.

Scope of this review: verify the ADR against the code actually delivered in m-1 (TASK-5, TASK-6, TASK-7) and against the specifications and requirements it cites. Decide accepted, rejected, or amend. If accepted or amended, update the ADR and regenerate the ADR index.

Review must check, at minimum:
- Decision vs. delivered code: blocking `std::net` HTTP/1.1 server, one request per connection, connection/thread cap, timeouts, method and header rejection rules, fixed route table, ID resolution, embedded assets, escaping, JSON API.
- Context accuracy: the constraints attributed to ADR-0001, ADR-0004 and the TASK-6 specification.
- Consequences and residual risks: stated risks still match reality; any risk not recorded in the ADR.
- Module boundaries claimed in the ADR (`browser::http`, `browser::records`, `browser::render`, `browser::mod`, `config`, `context`) versus the actual layout.
- Checkpoint follow-ups the review must either fold into the ADR or explicitly defer: config loading out of `commands`, per-record config re-reads, duplicate record parsing, template placeholder hardening, test temp-directory hygiene, manpage over-promising, NFR-01.002 scope, and NFR-05.001 real-browser verification.

Follow-ups: this task may spawn separate follow-up tasks for any accepted gap. Do not create implementation tasks as part of this review unless the decision makes one necessary; capture them as follow-up items instead.

Acceptance of the ADR content is an explicit user decision: the ticket may only record `accepted` after the user has confirmed it through the orchestrator.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [x] #1 The ADR and the delivered m-1 implementation (TASK-5/6/7 code, tests, packaging) have been read and the review evidence is cited in the ticket.
- [x] #2 Decision, context, consequences, and residual risks are verified point by point against the code and the cited specifications/requirements, with discrepancies listed.
- [x] #3 A decision of accepted, rejected, or amend is recorded in the ticket with rationale and evidence.
- [x] #4 If accepted or amended: ADR-0005 Status is updated (and content amended where needed) and docs/architecture/adr/index.html is regenerated per the architecture-decision-record and adr-index-visualization skills.
- [x] #5 The user has explicitly confirmed the acceptance decision through the orchestrator, and that confirmation is recorded in the ticket.
- [x] #6 Follow-up items (including the checkpoint follow-ups named in the description) are captured in the ticket as follow-up notes or new tickets, without being implemented here.
<!-- AC:END -->

## Definition of Done
<!-- DOD:BEGIN -->
- [x] #1 Review evidence recorded in the ticket, each claim tied to a file path, test, or spec section.
- [x] #2 ADR-0005 status and content match the recorded decision; a rejection states the reason and what replaces the decision.
- [x] #3 docs/architecture/adr/index.html regenerated and consistent with ADR-0005 when the ADR changed.
- [x] #4 Follow-ups captured for rejected or amended decisions, with owners implied by the affected area.
- [x] #5 Explicit user acceptance recorded; no task closure without it.
- [x] #6 No changes outside the ADR files and the ADR index.
<!-- DOD:END -->

## Implementation Plan

<!-- SECTION:PLAN:BEGIN -->
1. Read ADR-0005 end to end; record its Status (Proposed), Date, Context claims, Decision bullets, Consequences, Risks/mitigations, Alternatives, and References.
2. Read the delivered m-1 implementation and tests: src/browser/{mod,http,records,render}.rs, src/browser/templates/{list,view,edit,error}.html, src/browser/{app.css,app.js}, src/config.rs, src/context.rs, Cargo.toml; tests/browser_cli.rs and tests/search_browser_smoke.rs; packaging spec and docs/packaging.md as needed.
3. Read the cited specifications, requirements, business rules, and ADRs: docs/specifications/browser-requirements.md, docs/specifications/search-browser-milestone-packaging-and-verification.md, REQ-06, REQ-06.002, REQ-05.001, REQ-05.002, REQ-05.007, REQ-01.003, BR-004, BR-016, NFR-01.002, NFR-01.003, NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001, ADR-0001, ADR-0002, ADR-0003, ADR-0004, and the approved UI package MOCK-0001 plus docs/specifications/ui/browser-requirements-ui.md.
4. Verify Decision vs delivered code point by point: blocking std::net HTTP/1.1 server; one request per connection with Connection: close; thread-per-connection; 16-connection cap with immediate 503 on the 17th; 5-second read/write timeouts; GET/HEAD/POST only; rejection of Transfer-Encoding, keep-alive, pipelining, TLS, HTTP/2, WebSocket, compression; fixed route table after single-pass percent-decoding; no filesystem path joining; ID validation and exact-match resolution against active directory listings; include_str! templates with {{name}} placeholders; include_str! CSS/JS with no remote references; optional JS; JSON API via serde_json; security posture (no auth/session/cookies/TLS, loopback default, non-loopback warning, Host/Origin validation, strict Content-Type, escaping, CSP and security headers, no outbound access, no absolute paths).
5. Verify Context accuracy: constraints attributed to ADR-0001, ADR-0004, and the TASK-6 specification; TASK-6 spec forbids framework/async runtime/template engine/Markdown renderer/second crate/new dependency.
6. Verify Consequences and residual risks against reality; list any risk not recorded in the ADR.
7. Verify module boundaries (browser::http, browser::records, browser::render, browser::mod, config, context) against the actual layout.
8. Review each checkpoint follow-up and decide fold into ADR or explicitly defer: config loading out of commands; per-record config re-reads; duplicate record parsing; template placeholder hardening; test temp-directory hygiene; manpage over-promising; NFR-01.002 scope; NFR-05.001 real-browser verification.
9. Record the decision (accepted/rejected/amend) in the ticket with rationale and evidence, using a claim-by-claim evidence table (ADR section | code/spec/test reference | match/discrepancy).
10. If accepted or amended: update ADR-0005 Status and content, then regenerate docs/architecture/adr/index.html per the architecture-decision-record and adr-index-visualization skills; record the changed files.
11. Capture follow-ups as ticket notes or new tickets (not implemented here), with owners implied by the affected area.
12. Obtain explicit user confirmation of the acceptance decision through the orchestrator and record it in the ticket; do not close the ticket without it.
13. Final verification: ADR status/content matches the recorded decision; index regenerated and consistent; no changes outside ADR files and the ADR index.
<!-- SECTION:PLAN:END -->

## Implementation Notes

<!-- SECTION:NOTES:BEGIN -->
Assumptions:
- This review is read-only except for ADR-0005 and docs/architecture/adr/index.html. No product code, specification, requirement, business rule, or packaging file is changed by this ticket.
- The ADR may be accepted, rejected, or amended, but it may only be recorded as accepted after explicit user confirmation through the orchestrator (AC #5); a rejection must state the reason and what replaces the decision.
- Follow-ups (including the checkpoint follow-ups named in the description) are captured as notes or new tickets and are not implemented here.
- Delivered m-1 code is that of TASK-5, TASK-6, and TASK-7; if any referenced path has moved, that is a blocker to be flagged, not silently adjusted.
- The review owner is @architect; final user confirmation is recorded by the orchestrator.

Evidence recording template (to be filled during review):
| # | ADR section / claim | Code, test, or spec reference | Result (match / discrepancy) | Notes |

Blockers check (2026-11-04): all referenced source, spec, requirement, NFR, BR, ADR, UI, and test paths verified present in the repository; no missing source paths identified. Specific paths confirmed: src/browser/{mod,http,records,render}.rs, src/browser/templates/{list,view,edit,error}.html, src/browser/{app.css,app.js}, src/config.rs, src/context.rs, Cargo.toml, tests/browser_cli.rs, tests/search_browser_smoke.rs, docs/specifications/browser-requirements.md, docs/specifications/search-browser-milestone-packaging-and-verification.md, docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html, docs/specifications/ui/browser-requirements-ui.md, docs/architecture/adr/ADR-000{1,2,3,4,5}.md.

Checkpoint follow-ups to resolve or explicitly defer:
- config loading out of commands
- per-record config re-reads
- duplicate record parsing
- template placeholder hardening
- test temp-directory hygiene
- manpage over-promising
- NFR-01.002 scope
- NFR-05.001 real-browser verification

Notes:
- ADR-0005 is currently Proposed; ADR-0006 (optimistic concurrency and atomic record writes) is also Proposed and references ADR-0005. This ticket reviews ADR-0005 only; ADR-0006 is out of scope unless the review finds a dependency that must be recorded as a follow-up.
- The ADR index at docs/architecture/adr/index.html must be regenerated only when ADR-0005 changes.

# TASK-28 review evidence - ADR-0005

Decision recorded: Accepted (explicit user confirmation obtained 2026-11-04 via orchestrator). ADR-0005 Status: Proposed -> Accepted. ADR amended in place (content additions only) then accepted; index regenerated (6 records).

Claim-by-claim evidence (match unless noted):
1. Blocking HTTP/1.1 server via std::net TcpListener/TcpStream/std::io; no framework/async/new dep -> src/browser/mod.rs serve/handle_connection; src/browser/http.rs; Cargo.toml (clap, serde, serde_json, yaml_serde only) -> match.
2. One request per connection, Connection: close; thread per connection; 16 cap; 17th -> 503; 5s read/write timeouts; GET/HEAD/POST only; reject TE/keep-alive/pipelining/TLS/HTTP2/WS/compression -> mod.rs serve/reject_over_capacity/handle_connection; http::write_response Connection: close; http::read_request (no request loop); IO_TIMEOUT 5s; Method::Other -> 405 -> match.
3. TE presence -> 501; TE+CL -> 400; duplicate CL -> 400; missing CL on POST -> 411; body >1MiB -> 413 -> http.rs read_request (~333-370); tests/browser_cli.rs T-22 -> match.
4. Fixed route table after single-pass percent-decoding; no path joining; ID exact match -> http::parse_target/percent_decode (once); mod.rs route(); records::resolve_id (exact stem match against directory listing); valid_id -> match.
5. Server-side HTML templates via include_str! (list/view/edit/error) with {{name}}; escaped at render time; internal fragments pre-escaped raw -> render.rs (templates, render_template, escape); tests -> match.
6. CSS/JS via include_str! (app.css, app.js); no remote refs; JS optional progressive enhancement -> mod.rs APP_CSS/APP_JS; grep found no http://, @import, url(, <img>, fetch/import in assets/templates -> match.
7. JSON API GET /api/requirements, GET /api/doc/{id}, POST /api/doc/{id} via serde_json -> mod.rs api_requirements/api_get/api_post; Response::json -> match.
8. Security posture: no auth/session/cookies/TLS; loopback default; non-loopback warning; Host/Origin validation; strict Content-Type; escaping; CSP + X-Frame-Options/X-Content-Type-Options/Referrer-Policy/Cache-Control; no outbound; no absolute paths -> mod.rs (warning, validate_host, validate_origin, media_type); http::write_response (headers, CSP); record_json (rel_path only) -> match.
9. Module boundaries browser::http/records/render/mod + config + context -> src/browser/{http,records,render,mod}.rs; src/config.rs; src/context.rs -> match (deferred follow-up: config loading still in commands).
10. Assets compiled into binary; .deb contains only binary + copyright -> packaging spec 1.7; tests -> match.
11. Context: ADR-0001 Rust, ADR-0004 single crate; TASK-6 spec forbids framework/async/template/Markdown/second crate/new dep -> ADR-0001, ADR-0004; browser-requirements.md Out of scope -> match.
12. Consequences: offline NFR-01.003; NFR-05.001 verified at markup/asset level only -> packaging spec OQ-1; ADR text -> match.
13. Risks: protocol smuggling covered by tests/browser_cli.rs; XSS escaping + CSP; DNS rebinding/CSRF; resource exhaustion; future scale needs new ADR -> tests/browser_cli.rs; code -> match.

Discrepancies / notes:
- None blocking. Minor: browser::mod and browser::records import crate::commands::load_effective_config rather than a config-module loader; tracked as deferred follow-up (config loading out of commands). ADR amended to state key ownership vs call site.
- ADR amended to state single-pass/no-re-scan substitution explicitly.
- ADR amended to note incremental delivery toward NFR-01.002 (Context).
- Deferred follow-ups captured below; not implemented in this ticket (DoD #6).

Follow-up items (captured, not implemented):
1. Config loading out of commands - move load_effective_config from crate::commands into config module; owner implied: backend-dev/config. (deferred)
2. Per-record config re-reads + duplicate record parsing - browser::records::build_index/resolve_id reload config and re-parse; optimize while preserving spec 3.18 re-read contract. (deferred)
3. Template placeholder hardening - add tests/compile-time checks for placeholder allowlist; substitution already single-pass and non-re-scanning. (deferred)
4. Test temp-directory hygiene - migrate older suites (cli.rs, req_cli.rs, search_cli.rs, milestone_smoke.rs) from std::env::temp_dir() to project-local ./.tmp/. (deferred)
5. Manpage/README over-promising - README/BR-004 says browser serves both backlog and documentation stores; current increment serves documentation store only; update wording. (deferred)
6. NFR-01.002 scope clarification - clarify all Papyrus content is delivered incrementally across TASK-11..TASK-27. (deferred; NFR edit by requirements-engineer)
7. NFR-05.001 real-browser verification - schedule manual GUI pass in Firefox/Edge/Safari 100+; current verification is markup/asset level. (deferred)
8. ADR-0006 (optimistic concurrency) remains Proposed and references ADR-0005; separate review ticket TASK-29 covers it.

ADR index: regenerated docs/architecture/adr/index.html (6 decision records). ADR-0005 Status now Accepted.

User explicitly confirmed acceptance of ADR-0005 through the orchestrator on 2026-11-04 (TASK-28 approval). AC #5 and DoD #5 satisfied. ADR-0005 Status remains Accepted.
<!-- SECTION:NOTES:END -->
