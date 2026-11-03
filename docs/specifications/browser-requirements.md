# Feature Specification — Offline Browser Access for Requirement Content

**Ticket:** TASK-6
**Milestone:** m-1 (Search and Browser for Requirements)
**Status:** Refined (execution-ready; UI approved)
**UI mockups:** MOCK-0001 — approved 2026-11-03 (UTC); see [UI design handoff](#ui-design-handoff)

## Purpose

Deliver the first increment of `papyrus browser`: a blocking, dependency-free HTTP server inside the existing single Rust crate that serves a project-local, offline, read-and-edit web interface over the requirement documents implemented so far — business-requirement and non-functional-requirement groups and their member requirements — plus the configuration keys, security controls, and tests that make that interface safe and verifiable. The command contract is REQ-06.002 and BR-004 (`papyrus(1)` → COMMANDS → browser, `papyrus-config(5)` → Web UI). Where the man page is silent, this specification fixes explicit, testable defaults recorded in [Open questions](#open-questions).

## Scope

### In scope

- `papyrus browser [--host <host>] [--port <port>] [--no-open]` exactly as documented in `papyrus(1)` (BR-004, REQ-06.002).
- Option resolution with precedence command line > environment > configuration file > built-in defaults, defaults `127.0.0.1` / `6420` / open enabled, and a warning for a non-loopback bind address (REQ-06.002, REQ-01.003).
- A blocking HTTP/1.1 server over `std::net::TcpListener` in the existing crate, with request limits, timeouts, deterministic error responses, and no outbound network access (NFR-01.003, NFR-02.002).
- Listing, viewing, and editing active business-requirement and non-functional-requirement groups and requirements in the browser, with save-then-reread persistence (REQ-05.001, REQ-05.002, NFR-01.002).
- Deprecation visibility in the browser: deprecated records are not listed and are not reachable (REQ-05.001, REQ-05.002, REQ-05.007).
- A minimal read/write JSON API over the same records, used by the tests and by later increments.
- Embedded HTML/CSS/JS assets and templates compiled into the binary; no CDN, no external fonts, no network fetch, no new dependencies.
- New configuration keys `browser_host`, `browser_port`, `browser_open` with defaults, validation, `config list/get/set/unset` integration, and serialization (BR-016, REQ-01.002).
- HTML escaping, JSON encoding, ID/record resolution that never joins user input into a path, project-directory boundary checks on every read and write, symlink-escape rejection, request-body caps, and Host/Origin validation as the CSRF and DNS-rebinding mitigation (NFR-02.002, NFR-04.002).
- Support in Firefox 100+, Edge 100+, Safari 100+ using only widely supported HTML/CSS/JS, with basic accessibility (NFR-05.001).
- Unit and integration tests, including spawning the binary and speaking HTTP over `TcpStream`.

### Out of scope

- Creating, deprecating, restoring, or deleting records from the browser; moving records between groups; changing record IDs.
- Browsing or editing tasks, milestones, releases, decisions, glossary terms, business rules, UI specifications, audit records, deployment records, or specification documents — later tickets (TASK-11, TASK-15, TASK-19, TASK-23 … TASK-27).
- Browsing the backlog store, the decisions store, or the `docs/specifications/` tree.
- Search inside the browser UI, Markdown rendering, syntax highlighting, rich-text editing, attachments, or file uploads.
- Authentication, authorization, user accounts, sessions, cookies, or TLS.
- Automatic opening of the browser when the platform opener is absent beyond a warning.
- Packaging and release verification — TASK-7.
- New dependencies, a web framework, an async runtime, a templating engine, a Markdown renderer, or a second crate/application (ADR-0001, ADR-0004).

## Behavior

### 1. Command shape and option resolution

1. The command is exactly `papyrus browser [--host <host>] [--port <port>] [--no-open]`. It has no positional arguments and no subcommands.
2. `--host <host>` takes one value. `--port <port>` takes one value. `--no-open` is a boolean flag with no value.
3. Supplying an unknown option, an extra positional argument, or a flag without its required value is a usage error (exit 2).
4. `--port` must be an integer in `1..=65535`. Non-numeric, negative, zero, and out-of-range values are usage errors (exit 2).
5. `--host` must be a non-empty string that is either an IPv4 literal, an IPv6 literal, or a DNS host name. Validation rules are in section 1.10; a value that fails them is a validation error (exit 4).
6. Effective values are resolved independently per setting, highest precedence first:
   1. command-line option (`--host`, `--port`, `--no-open`);
   2. environment variable (`PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, `PAPYRUS_BROWSER_OPEN`);
   3. configuration file key (`browser_host`, `browser_port`, `browser_open`);
   4. built-in default (`127.0.0.1`, `6420`, open enabled).
7. The built-in defaults are `127.0.0.1`, `6420`, and `browser_open: true` (REQ-06.002, BR-016).
8. `--no-open` can only disable opening; there is no `--open` flag. An environment or configuration value can therefore be overridden in the "off" direction only, which matches the documented option set.
9. Environment values are read only when the corresponding command-line option is absent. `PAPYRUS_BROWSER_PORT` is parsed with the same rules as `--port`; an invalid environment value is a validation error (exit 4). `PAPYRUS_BROWSER_OPEN` accepts exactly `true` or `false` (case-insensitive); any other value is a validation error (exit 4). An empty environment value is treated as unset.
10. Host validation (applies to the resolved value from any source): at most 255 bytes; no leading or trailing whitespace; no whitespace, NUL, other control characters, `/`, `\`, `@`, `?`, `#`, `,`, or `%`; must parse as `std::net::IpAddr` or match `^[A-Za-z0-9]([A-Za-z0-9.-]{0,253})$` and not end with `.` or `-`.
11. A resolved host that is not a loopback address prints a warning to standard error before binding: `warning: binding to non-loopback address <host>; the interface is reachable from the network`. Loopback means an IPv4 address in `127.0.0.0/8`, the IPv6 address `::1`, or the literal host name `localhost` (case-insensitive). Host names other than `localhost` are never treated as loopback, and are never resolved for the purpose of the warning.
12. A non-loopback host is allowed, not rejected (REQ-06.002, BR-004).
13. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for the existing commands. `--dir` and `--config` select the project directory and configuration file before the server starts; the project directory is canonicalized and every served path is checked against it (NFR-04.002).
14. The command never writes to the configuration file.

### 2. Startup, opening, and lifecycle

1. The server resolves the effective configuration first. An invalid configuration (including an invalid `browser_host`, `browser_port`, or `browser_open`) fails with exit 4 before any socket is opened.
2. The listener is bound with `std::net::TcpListener::bind((host, port))` before any output or browser launch.
3. Bind failure — port already in use, permission denied, unresolvable host name, unsupported address — exits 1 with a message of the form `cannot bind <host>:<port>: <os message>` on standard error. With `--json`, the standard error object `{"error":{"code":1,"message":"..."}}` is emitted instead.
4. On success the command prints exactly one startup line to standard output and flushes it before serving:
   - text mode: `Serving http://<host>:<port>/`
   - `--json` mode: `{"status":"serving","url":"http://<host>:<port>/","host":"<host>","port":<port>,"open":<bool>}`
5. `<host>` in the printed URL is the effective bind host, except that a wildcard bind address (`0.0.0.0` or `::`) is printed as `127.0.0.1`, because a wildcard address is not a usable navigation target.
6. If opening is enabled, the default browser is opened at that URL after the startup line is flushed, using the platform opener: `xdg-open <url>` on Linux and other Unix systems, `open <url>` on macOS, `cmd /C start "" <url>` on Windows. The opener process is spawned without waiting for it and with its standard streams discarded.
7. Opening is suppressed when `--no-open` is present, or when the resolved open setting is `false`. No other condition suppresses it.
8. A missing, non-executable, or failing opener is a warning, never a fatal error: `warning: could not open the default browser: <reason>` on standard error. The server keeps serving.
9. After startup the process serves requests until it is interrupted (SIGINT/SIGTERM) or a fatal listener error occurs. There is no graceful-shutdown protocol, no `--daemon`, no PID file, and no automatic exit after inactivity.
10. A fatal accept-loop error (for example the listener becomes unusable) exits 1 with a runtime message.
11. Standard output carries only the single startup line; per-request diagnostics go to standard error. Warnings are plain `warning: ...` lines on standard error in both text and `--json` modes, because they are diagnostics and not command output.
12. Per-request logging is one line per request on standard error: `<METHOD> <path-without-query> <status>`. Logged paths are the decoded request path, truncated to 200 characters; request bodies, form values, and query strings are never logged.

### 3. HTTP server behavior

1. The server is a blocking HTTP/1.1 server implemented with `std::net::TcpListener` and `std::io::{BufReader, Read, Write}` only. No web framework, async runtime, or new dependency is used (ADR-0001, ADR-0004).
2. Each accepted connection is handled on its own `std::thread`. At most 16 connections are handled concurrently; a connection accepted beyond that limit receives `503 Service Unavailable` with `Connection: close` and is closed immediately.
3. Requests may use HTTP/1.0 or HTTP/1.1. Any other version receives `505 HTTP Version Not Supported`.
4. Exactly one request is served per connection. Every response includes `Connection: close`, and the connection is closed after the response body is written (no keep-alive, no pipelining).
5. Allowed methods are `GET`, `HEAD`, and `POST`. `HEAD` returns the same status and headers as the corresponding `GET` with no body. Any other method receives `405 Method Not Allowed` with `Allow: GET, HEAD, POST`.
6. `POST` is accepted only on `/doc/{id}` and `/api/doc/{id}`; `POST` to any other path receives `405`.
7. Request-line and header limits:
   - request line (method, target, version) at most 8192 bytes, otherwise `414 URI Too Long`;
   - request target at most 2048 bytes, otherwise `414 URI Too Long`;
   - header block at most 16384 bytes and at most 64 header fields, otherwise `431 Request Header Fields Too Large`;
   - header field names and values are read as ISO-8859-1 bytes and converted lossily to UTF-8 for matching; only the headers named in this specification are interpreted.
8. Body limits and framing:
   - request body at most 1 MiB (1048576 bytes), otherwise `413 Content Too Large`;
   - `Transfer-Encoding` is not supported; its presence receives `501 Not Implemented`;
   - `POST` without `Content-Length` receives `411 Length Required`;
   - duplicate `Content-Length` headers, a non-numeric `Content-Length`, or both `Transfer-Encoding` and `Content-Length` receive `400 Bad Request`;
   - the body is read exactly `Content-Length` bytes; extra bytes after the body are ignored and the connection is closed.
9. Timeouts: the complete request (request line, headers, and body) must arrive within 5 seconds of connection acceptance, otherwise `408 Request Timeout`; writing the response must complete within 5 seconds, otherwise the connection is closed. Both are implemented with socket read/write timeouts.
10. Request-target parsing: only origin-form targets (`/path` or `/path?query`) are accepted. Absolute-form (`http://...`), authority-form, and asterisk-form targets receive `400`. The path is percent-decoded exactly once; an encoded NUL (`%00`), encoded `/` (`%2F`), encoded `\` (`%5C`), or any decoded control character receives `400`.
11. Path traversal is rejected: after decoding, any `.` or `..` segment, any empty segment other than a single trailing slash, or a path that does not start with `/` receives `400`. Paths are matched against a fixed route table; no request path is ever joined to a filesystem path.
12. Routing and status codes:
    - known route with a valid request: the route's success response;
    - well-formed request for an unknown route: `404 Not Found`;
    - known path with a disallowed method: `405 Method Not Allowed`;
    - malformed request line or headers: `400 Bad Request`;
    - unsupported media type on `POST`: `415 Unsupported Media Type`;
    - request body over the cap: `413`;
    - invalid or missing origin/host validation: `400` or `403` as specified in section 9;
    - unexpected internal failure: `500 Internal Server Error` with a generic message and no internal detail.
13. Every response carries `Content-Type` with an explicit charset where textual (`text/html; charset=utf-8`, `application/json; charset=utf-8`, `text/css; charset=utf-8`, `text/javascript; charset=utf-8`, `text/plain; charset=utf-8`), plus `Content-Length`, `Connection: close`, `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `X-Frame-Options: DENY`. HTML responses additionally carry the Content-Security-Policy in section 9.7. A `Date` header is optional; `Server: papyrus` is sent without a version number.
14. There is no TLS, no HTTP/2, no WebSocket, and no compression. The default bind address is loopback-only; binding elsewhere is the operator's explicit choice and is warned about (section 1.11).
15. The server makes no outbound network connections. It does not fetch, proxy, resolve remote resources, or contact the opener's target over the network itself.
16. Binding a host name uses the platform resolver; on a disconnected host an IP literal or `localhost` therefore binds reliably, while an arbitrary host name may fail with exit 1. This is the only name-resolution step in the command.
17. A missing or unreadable configured document directory is not a server error: the corresponding document type category renders its empty state and the API returns an empty array (consistent with `req list`).
18. Concurrent requests are independent; the server keeps no session state, no cache, and no in-memory copy of record content between requests. Every request re-reads the configuration and the record file from disk.

### 4. UI content scope for this increment

1. The interface covers exactly four record forms:
   - business-requirement group (`document_types.req` prefix plus `-<number>`),
   - business requirement (`document_types.req` prefix plus `-<number>.<number>`),
   - non-functional-requirement group (`document_types.nfr` prefix plus `-<number>`),
   - non-functional requirement (`document_types.nfr` prefix plus `-<number>.<number>`).
2. Record sources are the active directories `doc_directory` + `document_types.req.directory` and `doc_directory` + `document_types.nfr.directory`, resolved from the effective configuration. Directories and prefixes are never hardcoded; the default prefixes are `REQ` and `NFR`.
3. Only files directly inside those directories whose `.md` file stem matches the form in section 4.1 are records. Subdirectories (including `deprecated/`), `index.html`, and files whose stem does not match are ignored. There is no recursive traversal.
4. A document type that is absent from the configuration or has `enabled: false` contributes no records and no error. Its sidebar entry is omitted, and requesting it directly with `?type=` renders that category's empty state rather than an error (OQ-13).
5. Deprecated records are not listed, not counted, not reachable by URL, and not editable through the interface (REQ-05.001, REQ-05.002, REQ-05.007). A request for a deprecated record's ID returns `404`, exactly as for an unknown ID.
6. **Navigation model — persistent document-type sidebar.** Every page shares one shell: a `header` (product name, project name, and a static "Local browser / offline · `<host>:<port>`" note), a persistent left sidebar `nav` labelled "Document types", `main`, and a `footer`. The sidebar lists the document types as vertical links, business requirements first, then non-functional requirements, and is extensible: a later increment adds a document type by adding one sidebar entry and one category route value, without changing the list page structure.
7. The sidebar selects exactly one document category at a time. The list page renders only the selected category; the other category is not rendered at all (not as an empty section, and not as a second section). The two document types are separate categories, not a filter over one combined list.
8. The default category is business requirements: `GET /` and `GET /?type=req` render the same page.
9. The selected category renders one section (`h2`) titled with the category name, showing the category's group count and requirement count. Each group is a row containing the group ID and title as one link to the group record, the group's member count, and its member requirements as nested links in ascending ID order; the group row links and the member links use link text "ID + title".
10. A requirement whose `## Group` reference does not match an active group of the same type is listed in a trailing "Other requirements" subsection (`h3`) of its category, with the stored group reference shown as text (for example `Group: REQ-99 (not found)`).
11. A group with no member requirements shows an explicit empty state in its row (for example "No requirements in this group") and a member count of "No requirements".
12. The category selection is a query parameter on the list page: `type=req` (default) or `type=nfr`. `GET /` and `?type=req` select business requirements, `?type=nfr` selects non-functional requirements. There is no combined "all" view: any other value, including `all`, is `400`.
13. Viewing a requirement shows: ID (read-only), record form (group or requirement), document type (business or non-functional), title, owning group ID and title (as a link to the group record when the group is active), user story for business-requirement records only, and description.
14. Viewing a group shows the group's own fields — ID, record form, document type, title, user story for business-requirement groups only, and description — followed by a "Requirements in this group" section (`h2`) listing the group's member requirements in ascending ID order as links whose text is "ID + title". A group with no members shows the empty state from section 4.11 in that section.
15. Breadcrumbs are the primary navigation on record pages: `Requirements` (a link to the record's category, `/?type=req` or `/?type=nfr`), then the owning group when it resolves, then the record ID as the current, non-linked crumb. On the edit page the last crumb is `Edit`.
16. The record view offers exactly one action, `Edit`. There is no "Back to requirements" action on the record view: the breadcrumb and the sidebar are the navigation. Error pages keep their single recovery link to `/`.
17. Editing is limited to the documented content fields:
    - business-requirement group: title, user story, description;
    - business requirement: title, user story, description;
    - non-functional-requirement group: title, description;
    - non-functional requirement: title, description.
18. Record ID, record form, document type, and group membership are not editable. They are displayed as read-only text and are never accepted from the request.
19. Requirement references inside any field (for example `REQ-05.001`, `NFR-04.002`) are displayed as literal text, never auto-linked or interpreted.
20. Field content is displayed as text with preserved line breaks and collapsed runs of whitespace only where CSS chooses; no Markdown is rendered, so `*`, `#`, backticks, and links appear literally.
21. No create, deprecate, restore, delete, move, or rename operation is offered. There is no "new record" control, and no control performs any of those actions.
22. The interface is fully usable without internet access: every asset is served by the same process, and no page references an external origin (NFR-01.003).

### 5. HTTP endpoints

Routes are exact. `{id}` is a record ID as defined in section 7.1.

| Method | Path | Request | Success response |
|---|---|---|---|
| `GET` | `/` | optional `?type=req\|nfr` (default `req`) | `200 text/html` list page for one document category |
| `GET` | `/doc/{id}` | — | `200 text/html` record view |
| `GET` | `/doc/{id}/edit` | — | `200 text/html` edit form |
| `POST` | `/doc/{id}` | `application/x-www-form-urlencoded`: `version`, `title`, optional `story`, optional `description` | `303 See Other` with `Location: /doc/{id}` |
| `GET` | `/api/requirements` | optional `?type=req\|nfr` (default `req`) | `200 application/json` record index for one document category |
| `GET` | `/api/doc/{id}` | — | `200 application/json` record |
| `POST` | `/api/doc/{id}` | `application/json`: `{"version":"...","title":"...","story":"..."\|null,"description":"..."\|null}` | `200 application/json` updated record |
| `GET` | `/assets/app.css` | — | `200 text/css` embedded stylesheet |
| `GET` | `/assets/app.js` | — | `200 text/javascript` embedded script |
| `HEAD` | any `GET` route | — | same status and headers, no body |
| any | any other route | — | `404` (unknown route) or `405` (known path, wrong method) |

1. `/api/requirements` response:

```json
{
  "groups": [
    {"id": "REQ-001", "kind": "req", "form": "group", "title": "Payments Group"}
  ],
  "requirements": [
    {"id": "REQ-001.001", "kind": "req", "form": "requirement", "title": "Capture payment", "group": "REQ-001"}
  ]
}
```

`kind` is `req` or `nfr`; `form` is `group` or `requirement`; `group` is the stored group reference or `null` when absent. The response covers exactly one document category, matching the list page: `type=req` (default) returns business-requirement records, `type=nfr` returns non-functional records; any other value, including `all`, is `400`. Both arrays are ordered by ID ascending and every record carries the `kind` of the selected category.

2. `/api/doc/{id}` and the `POST` success response:

```json
{
  "id": "REQ-001.001",
  "kind": "req",
  "form": "requirement",
  "title": "Capture payment",
  "group": {"id": "REQ-001", "title": "Payments Group"},
  "story": "As a cashier, I want ...",
  "description": "The quick brown fox ...",
  "version": "1762000000.123456789.412",
  "path": "docs/requirements/business/REQ-001.001.md"
}
```

`story` is `null` for non-functional records and when absent; `description` is `null` when absent; `group` is `null` for group records and for requirements whose group cannot be resolved. `path` is relative to the project directory, uses `/` separators, and never exposes an absolute path.

3. Error responses:
   - HTML routes return an HTML error page with the status, a short message, and a link back to `/`.
   - JSON routes return `{"error":{"code":<http-status>,"message":"<message>"}}` with the matching HTTP status. The `code` field is the HTTP status, not the CLI exit code.
   - A `POST` to a form endpoint that fails validation returns `422` with the edit form re-rendered, the submitted values preserved, a summary message, and field-level messages.
4. `POST /doc/{id}` requires `Content-Type: application/x-www-form-urlencoded` (parameters ignored) and rejects anything else with `415`. `POST /api/doc/{id}` requires `Content-Type: application/json` and rejects anything else with `415`.
5. Form fields are `version` (required), `title` (required), `story` (business-requirement forms only), and `description`. Unknown form fields are ignored. Repeated fields: the last value wins. Field values are percent-decoded with `+` as space; malformed percent-encoding receives `400`.
6. A successful form `POST` returns `303` and no body; the client re-fetches the view page, which reads the file from disk again (save-then-reread).
7. The JSON `POST` replaces the fields it is given and leaves omitted fields unchanged, except that an explicit `null` for `story` or `description` clears that section. `title` is required and must be a string. `version` is required. Unknown JSON members are ignored; a body that is not a JSON object, or a member of the wrong type, receives `400`.
8. `story` sent to a non-functional record is ignored (not an error), matching the CLI, which has no story option for `nfreq`.
9. Assets are served with an `ETag`-free, cache-free policy (`Cache-Control: no-store`) and are identical for every request; they contain no per-request data.

### 6. Assets and rendering

1. HTML templates are text files `src/browser/templates/list.html`, `view.html`, `edit.html`, and `error.html`, embedded at compile time with `include_str!`. CSS and JS are `src/browser/app.css` and `src/browser/app.js`, embedded with `include_str!`.
2. Substitution uses a single documented placeholder syntax `{{name}}` with a fixed, per-template allowlist of names. There is no template engine dependency, no expression evaluation, and no placeholder expansion inside substituted values (a value containing `{{...}}` is emitted literally).
3. Every substituted value is HTML-escaped in text and attribute context: `&` → `&amp;`, `<` → `&lt;`, `>` → `&gt;`, `"` → `&quot;`, `'` → `&#39;`. Attribute values are always double-quoted. Escaping happens at render time only; stored content is never pre-escaped.
4. JSON is produced only with `serde_json`, never by string concatenation or format strings.
5. The stylesheet uses a system font stack (`system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`); no web fonts, icon fonts, images, or external URLs. There is no `<img>` element, no `@import`, no `url(...)` reference to a remote resource, and no `srcset`.
6. The script is optional progressive enhancement: a dirty-form guard that warns before navigating away from a modified edit form, and the narrow-viewport document-type sidebar toggle. With JavaScript disabled, every page, link, form, and error state still works; the type links stay visible because the collapse is script-driven.
7. There is no inline `<script>`, no inline event handler attribute, and no inline `style` attribute; therefore the CSP needs no `'unsafe-inline'`.
8. All pages declare `<!doctype html>`, `<html lang="en">`, a unique `<title>`, and `<meta charset="utf-8">`; the `<meta name="viewport" content="width=device-width, initial-scale=1">` tag is present.
9. Pages contain no absolute URLs to other origins. The only URLs are root-relative paths on the serving origin.

### 7. Editing contract

1. Record ID validation: an ID is accepted only if it is 1–128 bytes, contains only ASCII letters, digits, `.`, `_`, and `-`, does not start with `.` or `-`, does not contain `..`, and is not `.` or `..`. After validation the ID is resolved by exact, case-sensitive match against the file stems found by listing the active `req` and `nfr` directories; the ID is never concatenated into a path. An ID that matches a stem in exactly one directory resolves to that record; an ID that matches in neither is `404`; an ID that matches in both is `500` with a generic message, because a shared prefix across two document types is an ambiguous configuration. The document type is taken from the directory that matched, never from the request.
2. The record form (group versus requirement) is derived from the matched file stem, never from the request.
3. Editable fields and their rules:

| Field | Applies to | Required | Rules |
|---|---|---|---|
| `title` | all forms | yes | trimmed; 1–200 Unicode scalar values; no NUL, CR, LF, or other control characters; may contain `#`, `*`, and other punctuation |
| `story` | `req` group and `req` requirement | no | trimmed; at most 20000 Unicode scalar values; LF allowed; CR and CRLF normalized to LF; TAB allowed; other control characters rejected; empty clears the section |
| `description` | all forms | no | same rules as `story` |

4. Additional content rules for `story` and `description`:
   - a line whose first two characters are `## ` is rejected with a validation error, because the record format uses `## ` to delimit sections and such a line would change the parsed record;
   - a line consisting only of whitespace is preserved as an empty line;
   - values are stored unescaped and unmodified apart from the trimming and newline normalization above.
5. Validation failures return `422` (HTML form re-rendered with messages, JSON error object) and never modify the file.
6. Record ID, group reference, `## Group` section content, and unknown sections are never editable.
7. Unknown and editor-added sections are preserved. On save:
   1. the file is read as UTF-8 text;
   2. the first line is replaced with `# <ID> <title>`;
   3. each of the sections `## User Story`, `## Group`, and `## Description` keeps its original position; a non-empty value replaces that section's body, an empty value removes the section, and a newly non-empty value whose section is absent is inserted after the last preceding known section when one exists, otherwise before the first following known section, otherwise appended at the end of the file;
   4. every other `## ` section is written back content-identically, in its original order and position;
   5. a single trailing newline terminates the file and line endings are LF.
8. A record with no `## Group` section is displayed and edited as a requirement without a resolved group; saving never adds a `## Group` section.
9. If the `## Group` reference does not resolve to an active group of the same type, the stored line is preserved unchanged and the group is displayed as unresolved text.
10. Writes are atomic and project-local:
    1. a temporary file `<id>.md.tmp` is created in the same directory with `create_new(true)`, retrying with a numeric suffix if it already exists;
    2. the content is written as UTF-8 bytes and `sync_all` is called;
    3. the original file's permissions are copied to the temporary file;
    4. the temporary file is renamed over the target with `std::fs::rename`;
    5. the directory is synced on a best-effort basis;
    6. on any failure the temporary file is removed and the original file is left untouched.
11. Every read and write path — record directory, record file, temporary file — passes through `Context::ensure_within` before use. A symbolic link that resolves outside the project directory is rejected; the request receives `404` and the outside content is neither read nor written. No temporary file is left behind after a failed or rejected request.
12. Concurrent edits use optimistic concurrency without locking. The version token is `"<mtime-seconds>.<mtime-nanos>.<length>"` of the record file at render time, sent as the `version` form field or JSON member. On `POST`:
    - missing or malformed `version` → `400`;
    - current on-disk token differs → `409 Conflict` with the message "record changed on disk; reload the form", and no write;
    - token matches → the write proceeds.
    The check and the rename are not one atomic operation; this residual race is accepted for a single-user loopback interface and documented here.
13. A record file that is not valid UTF-8 is skipped by the list and API index (consistent with `req list`) and returns `500` with a generic message when requested directly; it is never partially decoded or rewritten.
14. After a successful write the server does not keep a cached copy; the redirect target re-reads the file, so a successful save is proven by the follow-up `GET`.
15. Editing a record never changes its ID, its directory, or its deprecation state.

### 8. Configuration changes

1. `Config` gains three fields, serialized and deserialized with the existing `serde(default)` / `deny_unknown_fields` behavior:

| Key | Type | Default | Validation |
|---|---|---|---|
| `browser_host` | string | `127.0.0.1` | rules in section 1.10 |
| `browser_port` | unsigned 16-bit integer | `6420` | `1..=65535`, therefore `0` is rejected |
| `browser_open` | boolean | `true` | must parse as a boolean |

2. `Config::validate()` enforces the rules above and returns a validation error (exit 4) naming the offending key. Because the existing commands validate the merged configuration on load, an invalid browser key also fails those commands; this is the existing behavior for every configuration key.
3. `config list` prints the three effective keys in stable order, appended after the `document_types.*` entries in the order `browser_host`, `browser_port`, `browser_open`.
4. `config get browser_host|browser_port|browser_open` prints the effective value.
5. `config set` parses the value as a YAML scalar, persists it, and re-validates the merged configuration; an out-of-range port, a non-boolean open value, or an invalid host is a validation error (exit 4) and the file is not changed.
6. `config unset` removes the key and falls back to the default, re-validating the merged configuration.
7. `config list --json` includes the three fields with their effective values.
8. The man page already documents these keys (`papyrus-config(5)` → Web UI, `README.md` lines 1000–1009 and the example at 1040–1042); no requirement change is needed. The environment variables in section 1.6 are not documented in the man page (see OQ-1).

### 9. Security

1. **HTML escaping.** Every dynamic value in HTML is escaped as specified in section 6.3, in both text and attribute context. No user-controlled value reaches the response unescaped.
2. **JSON encoding.** All JSON is built with `serde_json`. User-controlled strings are encoded, not concatenated, and `</script>`-style breakouts are impossible because no JSON is embedded in HTML.
3. **ID and path handling.** Record IDs are validated and then resolved by exact match against a directory listing; no request value is joined to a path. Request paths are matched against a fixed route table after single-pass percent-decoding, with `.`/`..` segments, encoded separators, encoded NUL, and control characters rejected.
4. **Boundary checks.** Every filesystem read and write — active directory, record file, temporary file, configuration file — passes `Context::ensure_within` against the canonicalized project directory (NFR-04.002). Symlink escapes are rejected before any content is read or written.
5. **No traversal or symlink escape.** A symlinked record pointing outside the project is not listed, returns `404` when requested, and cannot be written through.
6. **Request-size caps.** Request line 8192 bytes, target 2048 bytes, header block 16384 bytes and 64 fields, body 1 MiB, 16 concurrent connections, 5-second read and write timeouts.
7. **Headers and content policy.** HTML responses send `Content-Security-Policy: default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'`, plus `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Cache-Control: no-store`. All responses send `Connection: close`.
8. **DNS rebinding and CSRF mitigation (chosen mitigation).** There is no authentication, no cookie, and no session, so the chosen mitigation is strict origin validation:
   1. the `Host` header must be present and must equal one of `<effective-host>:<port>`, `localhost:<port>`, `127.0.0.1:<port>`, or `[::1]:<port>` (case-insensitive host, exact port); when the effective port is `80`, the `:<port>` suffix may be omitted, as browsers do for the default HTTP port; a mismatch receives `400`;
   2. on `POST`, if `Origin` is present it must equal `http://<matching-host>:<port>` with the same host value that passed the `Host` check (again allowing an omitted `:80`), otherwise `403`;
   3. on `POST`, `Content-Type` must be `application/x-www-form-urlencoded` or `application/json` as appropriate, otherwise `415`, which prevents cross-origin `text/plain` and `multipart/form-data` simple requests from being useful;
   4. no CSRF token is used, because there is no session or credential to protect and the origin checks above reject cross-origin form submissions in all supported browsers.
9. **No secret leakage.** Error responses and pages never include absolute filesystem paths, environment variable values, OS error chains beyond a short reason, stack traces, or request bodies. JSON `path` values are relative to the project directory. Log lines contain only method, truncated decoded path, and status.
10. **No external access.** The server performs no outbound connections and serves no external resource. There is no telemetry, no update check, no remote font, and no CDN.
11. **Transport.** There is no TLS. The default bind address is loopback. A non-loopback bind prints the warning in section 1.11 and exposes an unauthenticated read/write interface to the network; this is documented behavior, not a defect (REQ-06.002, BR-004).
12. **Response integrity.** Request-derived data is never written into response headers; `Location` values are fixed, root-relative paths derived from a validated ID.
13. **Input sanitization.** All request input is normalized and validated before use (NFR-02.002): IDs per section 7.1, paths per section 3.10–3.11, form values per section 7.3–7.4, JSON per section 5.7.

### 10. Browser support and accessibility

1. The interface uses only features available in Firefox 100, Edge 100, and Safari 100: HTML5 forms, `<!doctype html>`, UTF-8, CSS flexbox and grid, CSS custom properties, `white-space: pre-wrap`, `system-ui` font stack, and ES2019 JavaScript (no optional chaining required, no top-level await, no private class fields).
2. Forbidden: `:has()`, container queries, `color-mix()`, `@layer`, subgrid, `dialog`, popover, `inert`, and any feature that would require a newer engine than the stated baselines.
3. No JavaScript module loading, no `fetch`, no dynamic import, and no framework. The single script is a classic script loaded with `<script src="/assets/app.js" defer></script>`.
4. Accessibility basics:
   - semantic landmarks `header`, `nav`, `main`, `footer`; exactly one `h1` per page and no skipped heading levels;
   - the document-type sidebar is one `nav` (or `aside` containing a `nav`) labelled "Document types"; the entry for the selected category carries `aria-current="page"` and a non-colour cue (bold weight plus underline), so selection is never conveyed by colour alone; on record view and edit pages the entry for the record's document type carries `aria-current="page"`;
   - at narrow widths (320 CSS px) the sidebar collapses behind a toggle button labelled "Document types" with `aria-expanded` reflecting state and `aria-controls` referencing the type list; the collapse is progressive enhancement, so with JavaScript disabled the type links remain visible and usable;
   - every form control has a `<label for>`; required fields are marked in text and with the `required` attribute;
   - error messages are associated with their fields via `aria-describedby`, and the form error summary uses `role="alert"`;
   - visible focus indicators; all controls reachable and operable by keyboard; no keyboard trap;
   - text contrast at least 4.5:1 against its background; status is never conveyed by color alone;
   - the page remains usable at 200% zoom and at a 320 CSS-pixel viewport width without horizontal scrolling of text; long group titles and unbreakable words wrap inside their row;
   - link text is descriptive out of context (record ID plus title, not "click here").
5. The interface works with JavaScript disabled and with the browser's built-in font, zoom, and high-contrast settings.

## Inputs and outputs

| Input / output | Description |
|---|---|
| `--host <host>` | Bind address; IP literal or DNS name; default `127.0.0.1`. |
| `--port <port>` | TCP port `1..=65535`; default `6420`. |
| `--no-open` | Suppress opening the default browser. |
| `PAPYRUS_BROWSER_HOST` / `PAPYRUS_BROWSER_PORT` / `PAPYRUS_BROWSER_OPEN` | Environment tier of the precedence chain (OQ-1). |
| `browser_host` / `browser_port` / `browser_open` | Configuration keys, defaults `127.0.0.1` / `6420` / `true`. |
| Startup output | One line on stdout: `Serving http://<host>:<port>/` or the JSON object in section 2.4. |
| Diagnostics | `warning: ...` lines and one request log line per request on stderr. |
| HTTP responses | HTML pages, JSON documents, CSS, JS, and error pages per sections 3 and 5. |
| Files written | Only record `.md` files inside the configured active type directories, written atomically. |
| Exit status | `0` only if the server ends normally (not expected), `1` runtime/bind failure, `2` usage, `4` validation. `3` is never used. |

## Validation and error behavior

| Condition | Exit / status | Output |
|---|---|---|
| Unknown option, extra positional, missing option value | exit 2 | Usage message on stderr. |
| `--port` non-numeric, `0`, negative, or greater than 65535 | exit 2 | Usage message on stderr. |
| Invalid `PAPYRUS_BROWSER_PORT` or `PAPYRUS_BROWSER_OPEN` value | exit 4 | Validation message on stderr. |
| Invalid `--host` value | exit 4 | Validation message on stderr. |
| Invalid configuration key value | exit 4 | Validation message on stderr. |
| Bind failure (port in use, permission, unresolvable host) | exit 1 | `cannot bind <host>:<port>: <reason>` on stderr. |
| Non-loopback host | serves | `warning: binding to non-loopback address ...` on stderr. |
| Opener missing or failing | serves | `warning: could not open the default browser: ...` on stderr. |
| Malformed request line, headers, or target | 400 | HTML or JSON error page. |
| Unknown route | 404 | HTML or JSON error page. |
| Known path, wrong method | 405 | Error page with `Allow: GET, HEAD, POST`. |
| Unsupported media type on `POST` | 415 | Error page. |
| Request line or target too long | 414 | Error page. |
| Header block or field count too large | 431 | Error page. |
| Body too large | 413 | Error page. |
| Request read timeout | 408 | Error page, then close. |
| Bad `Host`, duplicate `Content-Length`, bad percent-encoding | 400 | Error page. |
| Cross-origin `Origin` on `POST` | 403 | Error page. |
| Unknown, deprecated, or non-matching record ID | 404 | Error page. |
| Invalid field value or `## ` line in a multi-line field | 422 | Edit form re-rendered with messages, or JSON error. |
| Stale `version` token | 409 | Conflict page or JSON error; no write. |
| Missing or malformed `version` | 400 | Error page or JSON error. |
| Symlink escaping the project directory | 404 | Error page; nothing read or written outside. |
| Non-UTF-8 record file | skipped in list; 500 when requested directly | Generic error page. |
| Internal failure | 500 | Generic error page; no internal detail. |

## Tests

### Unit tests (in-module `#[cfg(test)]`)

1. Configuration: defaults for the three browser keys, `0` and out-of-range port rejection, invalid host rejection, `browser_open` non-boolean rejection, flattened key order, and JSON serialization.
2. Host validation and loopback classification: IPv4, IPv6, `localhost`, DNS names, wildcard addresses, and each rejection rule in section 1.10.
3. Option resolution: a pure resolver over (CLI, environment, configuration) covering every tier, per-setting independence, and invalid environment values.
4. Request parsing: method, target, version, header count and size limits, duplicate `Content-Length`, `Transfer-Encoding` presence, missing `Content-Length` on `POST`, and body cap.
5. Target handling: single-pass percent-decoding, `%2F`, `%5C`, `%00`, control characters, `.`/`..` segments, empty segments, absolute-form, and over-long targets.
6. Route matching and status mapping for every row of the validation table.
7. HTML escaping in text and attribute contexts for all five characters, including a value that already contains entity-like text.
8. Placeholder rendering: known placeholders substituted, unknown placeholders rejected at build time or left untouched as specified, and a value containing `{{name}}` emitted literally.
9. Record block parser and serializer: replace, remove, and insert each known section; preserve unknown sections; a record without `## Group`; CRLF normalization; exactly one trailing newline.
10. Field validation: title length and control characters, story and description control characters, `## ` line rejection, trimming, and empty-value clearing.
11. Version token formatting and comparison, including the stale and malformed cases.
12. JSON response shapes for the record index, a record, and an error object.

### Integration tests (`tests/browser_cli.rs`, spawning the binary)

1. Harness: a helper starts the binary with `--dir` and an explicit `--port`, waits for the startup line, and returns the base URL; the child process is killed and its temporary project directory removed at test end. Ports are obtained by binding `127.0.0.1:0`, reading the assigned port, dropping the listener, and retrying on a bind failure, so tests do not collide.
2. A minimal HTTP client over `std::net::TcpStream` builds raw requests and parses the status line, headers, and body; helpers cover `GET`, `HEAD`, and `POST` with a body.
3. Scenarios: checks T-01 to T-30 in the mapping table below, including the edit round-trip, the JSON API, every error status, the request limits, XSS, symlink escape, port conflict, non-loopback warning, opener suppression and absence, deprecated visibility, UTF-8 handling, and the offline asset scan.
4. Tests never require internet access, never bind a non-loopback address except in the warning test (which binds `0.0.0.0` briefly and sends no traffic), and never leave a child process running.

## UI design handoff

UI mockups are required before any UI implementation work and must be approved by the user. The `@ui-designer` produces wireframe artifacts only — no branding, color palette, logo, or visual identity; layout, content, states, and interaction affordances only.

**Approved artifact:** MOCK-0001 — [`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`](ui/mockups/MOCK-0001-browser-requirements-review.html), approved 2026-11-03 (UTC). It is one fully self-contained review package: a review landing page (ticket, milestone, sources, approval status, one button per content), a 12-slide design deck (cover, grouped view slides with "What it shows" / "Why this way", cross-cutting decisions, open questions), and a clickable web prototype with 25 views (requirement list 6, record view 5, edit form 6, error page 6, 320 px 2). Styles are inlined in the file; it references no external stylesheet, script, font, image, or network resource. The layout description is [`docs/specifications/ui/browser-requirements-ui.md`](ui/browser-requirements-ui.md); the registry is [`docs/specifications/ui/mockups/README.md`](ui/mockups/README.md).

The approved package is the authoritative layout reference for this increment. Where this specification and MOCK-0001 differ on a layout detail, MOCK-0001 wins and the specification is corrected.

### Screens covered by the approved package

1. **Requirement list (home, `/`).**
   - Shared shell: header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types", `main`, `footer`.
   - Two category states: Business requirements (default, `/?type=req`) and Non-functional requirements (`/?type=nfr`), each rendering only its own `h2` section with group rows (ID + title link, member count, nested member links) and the "Other requirements" `h3` subsection.
   - Empty states: project has no records, the selected category is empty, a group has no members.
   - Long-content state: many groups, long titles that wrap, and a long unbreakable title.
2. **Record view (`/doc/{id}`).**
   - Business requirement with user story and multi-paragraph description; non-functional requirement without a story.
   - Group record with story and description (business) and group record without a story (non-functional), each followed by its "Requirements in this group" member list.
   - Metadata block: ID (read-only), record form, document type, owning group (link when resolvable); breadcrumb `Requirements › group › ID`; single `Edit` action and no "Back to requirements" action; unresolved-group variant.
3. **Edit form (`/doc/{id}/edit`).**
   - Pristine business-requirement variant (title, user story, description) and non-functional variant (title, description).
   - Read-only display of ID, record form, document type, and owning group.
   - Field help text (for example: "A line starting with `## ` is not allowed").
   - Dirty state (form modified, Save enabled).
   - Validation error state: form-level summary plus field-level messages, with submitted values preserved; includes an empty-title error and a `## `-line error.
   - Conflict state (409): the record changed on disk; reload affordance and read-only copy of the attempted values.
   - Success behavior: the save action returns to the record view; the resulting view state is shown rather than a success banner page.
4. **Error pages.**
   - 404 unknown record and 404 deprecated record (identical layout, message may differ).
   - 400 malformed request, 405 method not allowed, 415 unsupported media type, 500 internal error — one generic error layout with a status code, a short message, and a single recovery link back to the list.
5. **Responsive and accessibility annotations.**
   - 320 CSS px list and edit form, with the sidebar collapsed behind the labelled toggle.
   - Annotation layer on the list and edit form: focus order, label associations, error association, heading levels, contrast note, and the no-JavaScript note.

### States and interactions the approved package covers

- Navigation: list → group view → requirement view → edit → save → view → list.
- Switching category via the sidebar without losing the category section structure.
- Validation failure recovery without losing typed content.
- Conflict recovery: reload the form.
- Keyboard-only operation of sidebar links, record links, and the form.
- Empty and error states for every screen listed above.
- No create, deprecate, restore, delete, search, or Markdown rendering controls appear anywhere.

### Expected artifacts

- One self-contained HTML review package per ticket, named `MOCK-XXXX-<slug>.html` under `docs/specifications/ui/mockups/`, containing the review landing, the design deck, and the clickable per-platform prototype with every view; styles inlined and no external reference. For this ticket: `MOCK-0001-browser-requirements-review.html`.
- A UI description document `docs/specifications/ui/browser-requirements-ui.md` listing the MOCK id, the screens it covers, the states it covers, and the interaction notes.
- A registry entry in `docs/specifications/ui/mockups/README.md` with the MOCK id, type, artifact, views, and approval status.
- MOCK ids are global and assigned sequentially; they are never reused or renumbered. MOCK-0001 is recorded in this specification and in TASK-6, and its status is `approved` (2026-11-03, UTC).

## Acceptance criteria mapping

| Check | Verifies | How it is verified |
|---|---|---|
| T-01 | Command shape is exactly `papyrus browser [--host] [--port] [--no-open]` | `papyrus browser --help`; parse tests for valid and invalid invocations |
| T-02 | Defaults are `127.0.0.1`, `6420`, open enabled | Startup line and JSON startup object with no options and no config |
| T-03 | Precedence is CLI > environment > config > defaults per setting | Matrix test overriding each tier and asserting the effective URL and open flag |
| T-04 | `--no-open` and `browser_open: false` suppress opening; default opens | Fake opener script on `PATH` writing a marker file; marker present/absent |
| T-05 | Missing opener is a warning, not fatal, and the server keeps serving | Empty `PATH`, then a successful `GET /` |
| T-06 | Non-loopback host warns and still binds | `--host 0.0.0.0`, assert `warning:` on stderr and a successful request |
| T-07 | Port already in use exits 1 with a bind message | Occupy the port, run the binary, assert exit 1 and stderr |
| T-08 | Invalid port and invalid host exit 2 and 4 respectively | Exit-code assertions for `0`, `70000`, `abc`, `-1`, `bad/host`, empty host |
| T-09 | The list renders one document category at a time from the sidebar, groups nested with their requirements, ordered by ID | HTML assertions on a fixture with `REQ-001`, `REQ-001.001`, `NFR-001`, `NFR-001.001`: `/?type=req` and `/` contain the business group and its member links in ID order and no non-functional content; `/?type=nfr` the reverse; the selected sidebar link carries `aria-current="page"` |
| T-10 | Category parameter `req\|nfr` works, default is `req`, `all` and any other value are 400 | HTML and JSON assertions for `/`, `/?type=req`, `/?type=nfr`, `/?type=all`, `/?type=x`, and the same values on `/api/requirements` |
| T-11 | Deprecated records are absent from the list and return 404 by URL | `req deprecate`/`nfreq deprecate` via CLI, then list and direct GET |
| T-12 | View shows ID, form, type, group, story (business only), description, escaped; a group view lists its members; no "Back to requirements" action | HTML assertions for each of the four record forms, plus a group page whose member links appear in ID order and a scan confirming the view page has no "Back to requirements" link |
| T-13 | Edit round-trip persists and is visible through both the UI and the CLI | Form `POST` → 303 → `GET` shows new values; `req view` shows the same file |
| T-14 | Unknown and editor-added sections are preserved content-identically across a save | Fixture with `## Criticality`, `## Traceability`, `## Open Question`; compare before/after with line endings normalized |
| T-15 | Field validation returns 422 and does not write | Empty title, 201-character title, control character, `## ` line |
| T-16 | ID, form, and group membership are immutable through the UI | `POST` with `id`, `group`, and `form` fields; stored file and ID unchanged |
| T-17 | Stale version token returns 409 and does not write | Render form, modify the file out of band, then `POST` |
| T-18 | Writes are atomic and leave no temporary file | Directory snapshot before/after a save and after a rejected save |
| T-19 | Symlink escaping the project returns 404 and never exposes outside content | Symlink fixture to a file outside the project |
| T-20 | HTML escaping prevents script injection | Record title `"><script>alert(1)</script>`; response contains no literal `<script>` and the escaped form |
| T-21 | JSON API returns the documented shapes and rejects bad bodies | `GET /api/requirements`, `GET /api/doc/{id}`, `POST /api/doc/{id}`, malformed JSON, wrong types |
| T-22 | Request limits and status codes | 405, 400, 413, 414, 431, 415, 501, 411, 403, 408 fixtures over raw `TcpStream` |
| T-23 | One request per connection and `Connection: close` | Second request on the same socket sees EOF |
| T-24 | Offline operation | Served HTML/CSS/JS contain no external URL; no outbound socket code path; server works with an unroutable default route |
| T-25 | Non-UTF-8 record files are skipped and never rewritten | Fixture with invalid bytes |
| T-26 | `browser_host`/`browser_port`/`browser_open` are in `config list/get/set/unset` and validated | Config CLI assertions, including `0`, `70000`, `maybe`, invalid host |
| T-27 | Assets are embedded and served without external references | `GET /assets/app.css` and `/assets/app.js`; no `http://`, `https://`, `//`, `@import`, or `url(` remote reference |
| T-28 | Accessibility and browser-compatibility basics | Landmarks, labels, single `h1`, `aria-describedby`, the document-type sidebar `nav` with `aria-current="page"` plus a non-colour cue, the 320 px toggle's `aria-expanded`/`aria-controls`, no forbidden CSS/JS features in a source scan |
| T-29 | Build, tests, clippy, and formatting pass with existing dependencies only | `cargo build`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`; `Cargo.toml` dependency list unchanged |
| T-30 | No reads or writes outside the project directory | Boundary assertions on every handler path; configuration path checked as for existing commands |

## Assumptions

- Assumption: Rust 1.99.1, edition 2024, and the repository-root single-crate layout are retained (ADR-0001, ADR-0004). The browser is a module and subcommand inside the existing crate; no new crate, application, stack, or dependency is introduced, so no new ADR is required. The blocking `std::net::TcpListener` server is the intended implementation approach.
- Assumption: the man page (`papyrus(1)` → browser, `papyrus-config(5)` → Web UI) and BR-004 are the behavior contract; where they are silent, this specification fixes explicit defaults, recorded in Open questions.
- Assumption: "requirement content" in this increment means business-requirement and non-functional-requirement groups and requirements only; all other record types arrive in later tickets (REQ-06.002, NFR-01.002 scope is filled incrementally).
- Assumption: the browser exposes a read/write interface but no lifecycle operations; deprecation and restoration remain CLI-only in this increment (REQ-05.007).
- Assumption: group records are content too, so they are viewable and editable for their documented fields, not merely listed.
- Assumption: unknown and editor-added sections are preserved on save rather than dropped, because the record files in this repository already contain sections such as `## Criticality` and `## Traceability`.
- Assumption: a JSON API is part of this increment because the acceptance criteria require machine-checkable behavior and later browser increments reuse it.
- Assumption: one request per connection is sufficient for a local interface and keeps the server small and deterministic.
- Assumption: `Config` must gain the three documented browser keys for them to be settable at all, because `deny_unknown_fields` is in effect and `config set` validates the merged configuration.
- Assumption: no TLS and no authentication, matching the documented loopback default and the "non-loopback is allowed but warned" contract.
- Assumption: the environment tier uses `PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, and `PAPYRUS_BROWSER_OPEN`; the man page does not document them (OQ-1).
- Assumption: the approved UI mockup MOCK-0001 (2026-11-03, UTC) is the authoritative layout reference for this increment; it fixes the persistent document-type sidebar with one category at a time, the group member list on the record view, and the absence of a "Back to requirements" action on the record view.

## Open questions

Each item has a specified default in this document, so none blocks implementation. OQ-1, OQ-2, and OQ-6 are recommended for user confirmation before release.

- **OQ-1 (recommended default: use the three `PAPYRUS_BROWSER_*` variables)** — `papyrus-config(5)` PRECEDENCE lists only `PAPYRUS_DIR` and `PAPYRUS_CONFIG`, yet the required precedence is CLI > environment > configuration > defaults. This specification defines `PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, and `PAPYRUS_BROWSER_OPEN`; the man page should be updated to document them. If the user prefers not to extend the environment surface, the environment tier is dropped for browser settings and the precedence becomes CLI > configuration > defaults; nothing else in this specification changes.
- **OQ-2 (recommended default: one startup line, flushed)** — The man page does not define startup output. This specification fixes one text line `Serving http://<host>:<port>/` and one JSON object in `--json` mode, emitted before serving, with warnings and request logs on stderr.
- **OQ-3 (recommended default: preserve unknown sections)** — The CLI `edit` currently rewrites a record with only its known sections, dropping editor-added sections. The browser preserves them instead. If the user wants identical behavior to the CLI, the browser would limit edits to documented fields and drop unknown sections, which loses content.
- **OQ-4 (recommended default: 200-character title, 20000-character story/description)** — No length limits are documented. The chosen limits bound the request and the file while remaining far above normal content.
- **OQ-5 (recommended default: reject lines starting with `## `)** — Multi-line fields are new for the browser, and such a line would be parsed as a new record section. Rejection keeps the record model unambiguous; escaping or section-merging would be an alternative.
- **OQ-6 (recommended default: include group editing)** — Group records are listed and their content is editable in this increment. Excluding them would leave group content view-only while requirement content is editable.
- **OQ-7 (recommended default: 404 for deprecated records)** — The man page says deprecated documents are hidden from the web UI; returning `404` for a known deprecated ID matches "hidden" and avoids revealing that the ID exists.
- **OQ-8 (recommended default: mtime+size version token, 409 on mismatch)** — No concurrency contract is documented. The token is dependency-free and detects the realistic single-user conflict; the residual check-then-rename race is documented.
- **OQ-9 (recommended default: target the common feature subset)** — NFR-05.001 names "Safari version 100", whose meaning its own open question flags as unclear. This specification targets only features supported well before all three stated baselines, so the interpretation does not change the result.
- **OQ-10 (recommended default: include the JSON API)** — The man page describes a web UI only. The JSON endpoints are an internal contract used by tests and later increments; they add no user-visible navigation.
- **OQ-11 (recommended default: Origin+Host validation, no CSRF token)** — There is no session or credential to protect. A token would add state and a second source of truth without improving the loopback threat model.
- **OQ-12 (recommended default: wildcard hosts are allowed and the printed URL uses `127.0.0.1`)** — Binding `0.0.0.0` is a non-loopback bind and therefore allowed with a warning; a wildcard address cannot be navigated to, so the printed and opened URL substitutes `127.0.0.1`.
- **OQ-13 (recommended default: a disabled document type has no sidebar entry)** — The approved sidebar model lists document types, but no requirement covers a type that is absent from the configuration or `enabled: false`. The default is: no sidebar entry, and a direct `?type=` request renders that category's empty state rather than `400`, consistent with "contributes no records and no error". The approved mockup shows only the two enabled types and does not settle this case.

## Traceability

| Artifact | Reference |
|---|---|
| Requirement | REQ-06, REQ-06.002, REQ-05.001, REQ-05.002, REQ-05.007, REQ-01.003 |
| Non-functional | NFR-01.002, NFR-01.003, NFR-02.001, NFR-02.002, NFR-04.001, NFR-04.002, NFR-05.001 |
| Business rule | BR-004 (browser command behavior), BR-016 (configuration keys and precedence) |
| ADR | ADR-0001 (Rust), ADR-0002 (amd64 .deb), ADR-0003 (Debian Forky), ADR-0004 (repository-root single crate) |
| Ticket | TASK-6 |
| UI mockups | MOCK-0001 — approved 2026-11-03 (UTC): [`docs/specifications/ui/mockups/MOCK-0001-browser-requirements-review.html`](ui/mockups/MOCK-0001-browser-requirements-review.html) (single self-contained review package: landing, design deck, 25-view web prototype) |
| UI description | [`docs/specifications/ui/browser-requirements-ui.md`](ui/browser-requirements-ui.md) |
| Related specification | `docs/specifications/search-requirements.md` (TASK-5) |
| Related specification | `docs/specifications/business-and-non-functional-requirements-management.md` (TASK-3) |
| Related specification | `docs/specifications/project-initialization-and-configuration.md` (TASK-2) |
