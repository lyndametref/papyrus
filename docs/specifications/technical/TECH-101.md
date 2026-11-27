---
id: TECH-101
title: "Browser Server, Asset Routing, and Caching"
created_date: '2026-11-25T20:53:42Z'
updated_date: '2026-11-27T19:13:49Z'
labels: []
references:
  - REQ-006.002
  - NFR-001.003
status: verified
---
# TECH-101 Browser Server, Asset Routing, and Caching

## Description
Covers the embedded HTTP server's transport rules, static asset routing, response cache classes, the serving snapshot, and the security header set. Build and packaging mechanics, the per-file asset inventory, rendering-library behavior, and record write concurrency are out of scope.

## Transport

- The server is blocking HTTP/1.1 over `std::net::TcpListener`, `std::net::TcpStream`, and `std::io`: no web framework, async runtime, or additional runtime dependency.
- One request per connection. Every response carries `Connection: close`; there is no keep-alive and no pipelining.
- Each connection runs on its own `std::thread`. At most 16 connections are handled concurrently; the 17th is rejected immediately with `503`.
- Socket read and write timeouts are 5 seconds each. Request-line, request-target, header, and body sizes are capped.
- Supported methods are `GET`, `HEAD`, and `POST`; any other method is rejected. `Transfer-Encoding`, keep-alive, pipelining, TLS, HTTP/2, WebSocket, and compression are rejected, as are duplicate or conflicting `Content-Length` values.
- The bind address is `127.0.0.1` by default and the host is configurable (REQ-006.002); a non-loopback host is accepted and produces a warning.
- With no selected port, the server binds a free port above 1024 and below 9999, retrying when one is taken, and reports the chosen address.
- The server makes no outbound network access, keeps no authentication, session, cookie, or TLS state, and never includes an absolute filesystem path in a response.

## Request validation

- The allowed authority is fixed when the listener binds: `127.0.0.1:<port>` and `localhost:<port>` when bound to loopback, or `<configured-host>:<port>` otherwise, where `<port>` is the actual bound port, including an automatically chosen one.
- Every request's `Host` header must equal an allowed authority. A mismatch returns `403` with the shared error page and writes nothing.
- Every `POST` must carry an `Origin` equal to `http://<allowed-authority>` for one of the allowed authorities. A missing or different `Origin` returns `403`, and the body is not parsed.
- Every `POST` must carry `Content-Type: application/x-www-form-urlencoded`; any other media type returns `415` and the body is not parsed.
- `GET` and `HEAD` require no `Origin`.
- Validation runs before routing, body parsing, and any store access.
- A configuration reload never changes the allowed authority; the listener and its authority are fixed at startup.
- These checks are the CSRF and DNS-rebinding mitigation.

## Static asset routing

- One static table is defined in `src/browser/assets.rs`:

  ```rust
  pub struct Asset { pub path: &'static str, pub mime: &'static str, pub bytes: &'static [u8] }
  pub static ASSETS: &[Asset];
  pub fn lookup(path: &str) -> Option<&'static Asset>;
  ```
- `lookup` is a linear scan over exact path strings. There is no prefix matching, no directory expansion, no path join, and no percent-decoded filesystem access.
- A miss returns `404` for every method. A hit with a method other than `GET` or `HEAD` returns `405` with `Allow: GET, HEAD`. `HEAD` sends `Content-Length` and no body.
- Binary bodies are carried by `Response::bytes(status, content_type, &'static [u8])`. No module serves a file from disk for the browser; every served byte comes from `ASSETS`.
- Vendored files are served from `/assets/<package>/<asset-version>/<file>`, mirroring `src/browser/vendor/<package>/<asset-version>/`. The version segment is a literal part of the table entry, never a runtime value, and is not the Papyrus package version.
- Unversioned vendor paths return `404`; they are neither redirected nor aliased. First-party assets keep unversioned paths: `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js`.
- Pages reference only table entries, only from `/assets/`. No CDN, web font, icon font, image, remote `@import`, or remote `url(...)` is referenced.

## Cache classes

Every response belongs to exactly one class, decided by what the response is and never by the route that produced it.

| Class | Responses | `Cache-Control` | Validator | `304` |
|---|---|---|---|---|
| Immutable vendor | every file under `/assets/<package>/<asset-version>/` | `public, max-age=31536000, immutable` | none | never |
| Revalidated first-party | `/assets/app.css`, `/assets/app.js`, `/assets/nav-state.js` | `no-cache` | strong `ETag` | yes |
| No-store | every HTML response, every JSON response, every error and not-found response, and any path that is not a table entry | `no-store` | none | never |

- Immutable vendor `200`: `Server: papyrus`, `Content-Type`, `Content-Length`, `Connection: close`, `Cache-Control: public, max-age=31536000, immutable`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `X-Frame-Options: DENY`; no `ETag`, `Last-Modified`, `Expires`, `Vary`, `Content-Encoding`, or CSP.
- Revalidated first-party `200`: the same set, with `Cache-Control: no-cache` and one strong `ETag`.
- Revalidated first-party `304`: `Server: papyrus`, `Connection: close`, `Cache-Control: no-cache`, the same `ETag`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `X-Frame-Options: DENY`; no `Content-Type`, no `Content-Length`, no body, no CSP.
- No-store: `Content-Type`, `Content-Length`, `Connection: close`, `Cache-Control: no-store`, the three security headers, and, on HTML only, the CSP.
- `max-age` is 31536000 seconds and `immutable` is the RFC 8246 directive. No other `Cache-Control` directive or parameter is emitted, and no handler supplies its own `Cache-Control`; the default class for a response that names none is no-store.
- The `ETag` is strong and formatted `"<64 lowercase hex characters>"`: the SHA-256 of the exact served bytes, computed once per process from the embedded bytes.
- `If-None-Match` is the only conditional header with an effect, and only for the revalidated class on `GET` and `HEAD`. All header lines are combined into one comma-separated list and each element is trimmed of spaces and tabs. `*` matches; a strong tag matches only byte-identically, quotes included; a weak tag (`W/"…"`), an unparseable element, or an unknown tag never matches and yields the normal `200`. `If-Modified-Since`, `If-Match`, `If-Unmodified-Since`, and `If-Range` are ignored.
- Conditional evaluation happens after the method check and after the exact lookup, and only for a revalidated hit. Immutable responses never answer `304`. The `304` is bodyless and terminal; the connection still closes and no `Content-Length: 0` is emitted.
- No compression is introduced: no `Content-Encoding`, no `Accept-Encoding` negotiation, no pre-compressed body, no `Vary`, and no range request.

## Serving snapshot

- One immutable `ServingSnapshot` owns exactly the effective validated `Config`, the `ArtifactRegistry` built from it, the `BrowserRoutes` table derived from that registry, and the input fingerprint that produced all three.
- `ServerState` holds the current snapshot behind one swap point — a `RwLock<Arc<ServingSnapshot>>` or an equivalent standard-library single-writer, multi-reader cell — and holds no second copy of any part.
- A request takes one snapshot handle when it starts and passes it to routing, handlers, and rendering, so one request cannot mix configuration generations.
- The snapshot is server state only: created after the listener binds, alive until process exit, never read or written by a CLI command, never persisted.
- A fingerprint is the file modification time (seconds and nanoseconds) plus the byte length, read with `std::fs::metadata`, with no content read. The only input file is the configuration file selected at startup; the registry and routes are derived in-process and share its fingerprint, so the three parts invalidate together.
- The fingerprint check runs once per request, before routing. When every fingerprint matches, the request performs only the `stat` calls and no rebuild. When any differs — including a missing or unreadable file — the request attempts one reload.
- A reload is the startup construction path in startup order: load the configuration, validate it fully, build the registry, derive the route table. Publication is one atomic swap of the `Arc`; a request already holding the previous handle finishes on it, and the previous snapshot is dropped with its last handle.
- Reloads are serialized under one reload lock and re-check the fingerprint inside it, so a rebuild is never repeated. A request that cannot take the lock does not wait and serves the current snapshot, so one request may observe the previous configuration once.
- Reload is fail-closed: on any load, validation, registry, or route error the new snapshot is discarded, the previous snapshot stays current, and the request is served from it with its normal status and page. The server does not exit, does not serve an error page for a configuration problem, and does not fall back to defaults.
- A reload failure writes one `warning: ...` line to standard error naming the failing input and the error. No status code, header, page, endpoint, JSON member, or exit code is added, and no configuration error content reaches the client. The failing fingerprint is remembered, so a broken state is reported once per distinct input state and retried when the file changes again.
- Startup has no last-known-good snapshot: a missing configuration exits 1 and an invalid one exits 4.
- Settings consumed before the listener binds — host, port, open — are read from the startup configuration only. A reload changes pages, navigation, lists, and routes, not the bound socket, the connection cap, or the `Host`/`Origin` allowlist.
- A request may build one request-scoped record snapshot: a map from record ID to the record it resolves to, over the enabled descriptors' active directories. It is built on first use, lists each enabled directory once by file name, parses each resolved record once and memoizes it, and lives only for the request. It is display-only: it supplies no write bytes and no version token, and every write re-reads the live record from disk.

## Security headers

- Every response carries `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, and `Referrer-Policy: no-referrer`.
- HTML responses are the only responses carrying a CSP. Two policies exist: a strict policy for every HTML response without a rendered diagram, and a diagram policy for an HTML response that renders one.
- The strict policy is:

```
default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

- The diagram policy is:

```
default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
```

- `script-src` never contains `'unsafe-inline'`, `'unsafe-eval'`, a nonce, or a hash.
