# Embedded Dependency-Free HTTP Server and Server-Rendered Browser UI

## Status

Proposed

## Date

2026-11-03

## Context

TASK-6 delivers the first increment of `papyrus browser`: an offline, project-local web interface over business-requirement and non-functional-requirement records. The behavior contract is `docs/specifications/browser-requirements.md`, traced to REQ-06.002, BR-004, NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.002, and NFR-05.001.

ADR-0001 selects Rust and ADR-0004 fixes the repository-root single crate. The TASK-6 specification additionally forbids a web framework, an async runtime, a templating engine, a Markdown renderer, a second crate/application, and any new runtime dependency. Those constraints leave an unresolved architectural question: how the HTTP interface is built, how UI assets are delivered, and what security posture applies. The answer is cross-cutting because later browser increments (TASK-11, TASK-15, TASK-19, TASK-23 … TASK-27) and packaging (TASK-7, binary-only `.deb`) inherit it.

The alternatives considered were a Rust web framework (for example axum or actix-web), an async runtime (tokio), a template engine (askama or tera), externally served assets, an external web server or CGI, and a WebView/Tauri shell. Each adds dependencies, runtime files, or a second execution model that conflicts with the offline, dependency-free, single-binary packaging requirement.

## Decision

- Implement the browser as a **blocking HTTP/1.1 server** using only `std::net::TcpListener`, `std::net::TcpStream`, and `std::io`. No web framework, async runtime, or new dependency.
- Serve **exactly one request per connection** with `Connection: close`; handle each accepted connection on its own `std::thread`; cap concurrent connections at 16; reject the 17th immediately with `503`; use 5-second read and write timeouts; support only `GET`, `HEAD`, and `POST`; reject `Transfer-Encoding`, keep-alive, pipelining, TLS, HTTP/2, WebSocket, and compression.
- Route against a **fixed route table** after single-pass percent-decoding. Request paths are never joined to filesystem paths. Record IDs are validated and resolved by exact match against active directory listings.
- Render **server-side HTML** from templates embedded with `include_str!` (`src/browser/templates/*.html`) using a fixed `{{name}}` placeholder syntax. Escaped values are escaped at render time; internal fragments (sidebar, breadcrumb, section, metadata, form) are pre-escaped raw fragments. There is no template engine and no expression evaluation.
- Embed CSS and JavaScript with `include_str!` (`src/browser/app.css`, `src/browser/app.js`). No CDN, web font, icon font, image, remote `@import`, or remote `url(...)`. JavaScript is optional progressive enhancement (dirty-form guard and narrow-viewport sidebar toggle); every page works without it.
- Provide a minimal read/write **JSON API** (`GET /api/requirements`, `GET /api/doc/{id}`, `POST /api/doc/{id}`) using `serde_json`, backed by the same record access as the HTML routes.
- Adopt the security posture fixed by the specification: no authentication, authorization, accounts, sessions, cookies, or TLS; loopback bind by default; non-loopback bind allowed but warned about; `Host` and `Origin` validation plus strict `Content-Type` checks as the CSRF and DNS-rebinding mitigation; HTML escaping, CSP, `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Cache-Control: no-store`; no outbound network access; no absolute filesystem paths in responses.
- Keep the module boundaries inside the single crate: `browser::http` (protocol parsing/writing), `browser::records` (record indexing, parsing, validation, atomic persistence), `browser::render` (templates and escaping), and `browser::mod` (option resolution, server lifecycle, routing, handlers, JSON shaping). Configuration keys live in `config`, and project-directory enforcement stays in `context`.
- Compile all assets into the binary so the `.deb` contains only `/usr/bin/papyrus` and the copyright file; no runtime asset directory is required.

## Consequences

**Positive:**

- The browser works fully offline and needs no runtime dependency, external server, asset directory, or network fetch (NFR-01.003).
- The package stays a single binary plus copyright, matching ADR-0002/ADR-0003 packaging and TASK-7 verification.
- No new supply-chain surface or license audit is introduced beyond the existing dependencies.
- The fixed route table and record-ID resolution avoid path traversal and keep the security model auditable.
- The single-crate layout and module split remain consistent with ADR-0004.

**Negative:**

- HTTP parsing, header handling, and response writing are maintained in-project rather than delegated to a framework; protocol defects are our responsibility.
- Concurrency is limited to 16 blocking threads with no keep-alive; this is adequate for a single-user loopback interface but not for multi-user or high-throughput use.
- Rendering is hand-rolled: the `{{name}}` substitution and raw-fragment allowlist must be maintained carefully, and there is no template engine safety net.
- No authentication or TLS means a non-loopback bind exposes an unauthenticated read/write interface to the network; the warning is the only guard.
- Browser-version support (NFR-05.001) is verified at markup/asset level, not by driving real GUI browsers.

**Risks and mitigations:**

- *Protocol and request-smuggling defects*: covered by the contract tests in `tests/browser_cli.rs` (request limits, duplicate `Content-Length`, `Transfer-Encoding`, method/status matrix).
- *XSS and injection*: render-time escaping, `serde_json` for JSON, CSP without `'unsafe-inline'`, and no inline scripts or event handlers.
- *DNS rebinding and CSRF*: strict `Host`/`Origin` validation and `Content-Type` enforcement; no session or credential exists to protect.
- *Resource exhaustion*: request-line, target, header, body, connection, and timeout caps.
- *Future scale or feature pressure*: introducing a framework, async runtime, template engine, or external asset pipeline requires a new ADR; this ADR is the baseline those changes must supersede.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Blocking `std::net` server, embedded templates/assets (this ADR) | Selected for TASK-6 | Meets offline, dependency-free, single-binary, and single-crate constraints; smallest operational surface. |
| Web framework (axum, actix-web, warp) | Rejected | Adds dependencies and an async runtime; conflicts with the TASK-6 no-new-dependency constraint. |
| Template engine (askama, tera, handlebars) | Rejected | Adds a dependency and build complexity; the placeholder set is small and fixed. |
| External web server or CGI | Rejected | Requires a second runtime/process and runtime files; not offline/self-contained. |
| WebView/Tauri desktop shell | Rejected | Adds a platform UI stack and packaging surface far beyond the local-browser requirement. |
| Markdown rendering in the browser | Rejected | Out of scope for this increment; displaying stored text literally is the specified behavior. |

## References

- `docs/specifications/browser-requirements.md` — TASK-6 behavior contract, HTTP routes, security, assets, browser support.
- `docs/specifications/search-browser-milestone-packaging-and-verification.md` — TASK-7 packaging and embedded-asset verification.
- `docs/requirements/non-functional/NFR-01.002.md`, `NFR-01.003.md`, `NFR-02.001.md`, `NFR-02.002.md`, `NFR-04.001.md`, `NFR-04.002.md`, `NFR-05.001.md`.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust and Cargo.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single-crate layout.
- Implementation: `src/browser/mod.rs`, `src/browser/http.rs`, `src/browser/records.rs`, `src/browser/render.rs`, `src/browser/templates/`, `src/browser/app.css`, `src/browser/app.js`.
- Tests: `tests/browser_cli.rs`, `tests/search_browser_smoke.rs`.
- Status note: implemented and verified in TASK-6/TASK-7; this ADR is `Proposed` pending explicit user acceptance.
