---
id: TECH-007
title: "Offline Browser Access for Requirement Content"
created_date: '2026-11-16T17:46:43Z'
updated_date: '2026-11-21T07:44:32Z'
labels: []
references:
  - ADR-0001
  - ADR-0003
  - ADR-0004
  - ADR-0005
  - ADR-0006
  - ADR-0007
  - ADR-0012
  - ADR-0013
  - ADR-0016
  - ADR-0017
  - ADR-0019
  - ADR-0020
  - ADR-0022
  - ADR-0025
  - ADR-0026
  - ADR-0027
  - ADR-0028
  - ADR-0033
  - ADR-0034
  - ADR-0035
  - ADR-0036
  - ADR-0037
  - ADR-0038
  - ADR-0039
  - BR-004
  - BR-016
  - MOCK-0001
  - NFR-001.001
  - NFR-01.002
  - NFR-01.003
  - NFR-02.002
  - NFR-04.001
  - NFR-04.002
  - NFR-05.001
  - REQ-001.001
  - REQ-01.002
  - REQ-01.003
  - REQ-01.005
  - REQ-05.001
  - REQ-05.002
  - REQ-05.007
  - REQ-06.002
  - REQ-06.006
  - REQ-06.007
  - REQ-06.009
  - REQ-06.010
  - REQ-06.011
  - REQ-06.012
  - REQ-06.018
  - REQ-06.019
  - REQ-07.004
status: verified
---
# TECH-007 Offline Browser Access for Requirement Content

## Purpose

Deliver `papyrus browser`: a blocking, dependency-free HTTP server inside the
single Rust crate that serves a project-local, offline, read-and-edit web
interface. Business-requirement and non-functional-requirement content is
browsed through the same unified document surface as every other document type,
not through per-type handlers. The record's field map drives the view, edit
form, conflict page, and list row.

The browser serves from the immutable `ServingSnapshot` of ADR-037: one
validated configuration, one registry, and one route table. Transport, security,
and cache behavior follow ADR-0005 and ADR-0038.

## Scope

### In scope

- `papyrus browser [--host <host>] [--port <port>] [--no-open] [--debug]`
  exactly as documented in the browser command contract.
- Option resolution with precedence command line > environment > configuration
  file > built-in defaults, defaults `127.0.0.1` / `6420` / open enabled, and a
  warning for a non-loopback bind address.
- A blocking HTTP/1.1 server over `std::net::TcpListener` in the existing
  crate, with request limits, timeouts, deterministic error responses, and no
  outbound network access.
- Listing, viewing, and editing requirement groups and members through the
  document surface, with save-then-reread persistence.
- Deprecation visibility: archived records are not listed and are not
  reachable.
- A minimal read/write JSON API over the same records.
- Embedded HTML/CSS/JS assets compiled into the binary: first-party files,
  versioned vendored KaTeX files and fonts, and the versioned vendored Mermaid
  bundle, all same-origin.
- The browser configuration keys `browser_host`, `browser_port`, and
  `browser_open` with defaults, validation, and configuration-command
  integration.
- HTML escaping, JSON encoding, ID resolution that never joins user input into
  a path, project-directory boundary checks, symlink-escape rejection,
  request-body caps, and Host/Origin validation.
- Firefox 115+ support using widely supported HTML/CSS/JS, with basic
  accessibility.
- Unit and integration tests, including spawning the binary and speaking HTTP
  over `TcpStream`.

### Out of scope

- Creating, deprecating, restoring, or deleting records from the browser;
  moving records between groups; changing record IDs.
- Browsing or editing backlog, decision, or non-document records.
- Browsing the backlog store, the decisions store, or the
  `docs/specifications/` tree as unmanaged prose.
- Search inside the browser UI, Markdown rendering, syntax highlighting,
  rich-text editing, attachments, or file uploads.
- Authentication, authorization, user accounts, sessions, cookies, or TLS.
- Automatic opening of the browser when the platform opener is absent beyond a
  warning.
- Packaging and release verification.
- New dependencies, a web framework, an async runtime, a templating engine, a
  Markdown renderer, or a second crate/application.

## Behavior

### 1. Command shape and option resolution

1. The command is exactly `papyrus browser [--host <host>] [--port <port>]
   [--no-open] [--debug]`. It has no positional arguments and no subcommands.
   `--debug` is an option of the `browser` command only; it is not a global
   option and is not accepted by any other command.
2. `--host <host>` takes one value. `--port <port>` takes one value.
   `--no-open` is a boolean flag with no value.
3. Supplying an unknown option, an extra positional argument, or a flag without
   its required value is a usage error (exit 2).
4. `--port` must be an integer in `1..=65535`. Non-numeric, negative, zero, and
   out-of-range values are usage errors (exit 2).
5. `--host` must be a non-empty string that is either an IPv4 literal, an IPv6
   literal, or a DNS host name.
6. Effective values are resolved independently per setting, highest precedence
   first: command-line option, environment variable
   (`PAPYRUS_BROWSER_HOST`, `PAPYRUS_BROWSER_PORT`, `PAPYRUS_BROWSER_OPEN`),
   configuration file key (`browser_host`, `browser_port`, `browser_open`),
   then the built-in default (`127.0.0.1`, `6420`, open enabled).
7. Environment values are read only when the corresponding command-line option
   is absent. An invalid environment value is a validation error (exit 4), and
   an empty environment value is treated as unset.
8. Host validation applies to the resolved value from any source: at most 255
   bytes; no leading or trailing whitespace; no whitespace, NUL, other control
   characters, `/`, `\`, `@`, `?`, `#`, `,`, or `%`; it must parse as
   `std::net::IpAddr` or match a host-name pattern and not end with `.` or `-`.
9. A resolved host that is not a loopback address prints
   `warning: binding to non-loopback address <host>; the interface is reachable from the network`
   to standard error before binding. Loopback means `127.0.0.0/8`, `::1`, or
   `localhost` (case-insensitive).
10. Global options (`--json`, `--dir`, `--config`, `--no-color`) behave as for
    the existing commands. `--dir` and `--config` select the project directory
    and configuration file before the server starts. The command never writes
    the configuration file.

### 2. Startup, opening, and lifecycle

1. The server loads and validates the startup configuration first. An invalid
   configuration fails with exit 4 before any socket is opened. The first
   `ServingSnapshot` is built after the listener binds.
2. The listener is bound with `std::net::TcpListener::bind((host, port))`
   before any output or browser launch.
3. Bind failure exits 1 with a message of the form
   `cannot bind <host>:<port>: <os message>`; with `--json`, the standard error
   object is emitted instead.
4. On success the command prints exactly one startup line to standard output
   and flushes it before serving: `Serving http://<host>:<port>/`, or the JSON
   object with the same URL, host, port, and open flag.
5. If opening is enabled, the default browser is opened at that URL after the
   startup line is flushed. The opener is spawned without waiting and with its
   standard streams discarded. A missing or failed opener is a warning, never a
   fatal error.
6. After startup the process serves requests until interrupted or a fatal
   listener error. There is no graceful-shutdown protocol, daemon mode, PID
   file, or automatic exit after inactivity.
7. Standard output carries only the single startup line. Per-request
   diagnostics and warnings go to standard error. With `--debug`, each request
   is logged as one `<method> <path> <status> <duration>ms` line; headers and
   bodies are never logged.

### 3. HTTP server behavior

1. The server is a blocking HTTP/1.1 server implemented with
   `std::net::TcpListener` and `std::io::{BufReader, Read, Write}` only. No web
   framework, async runtime, or new dependency is used.
2. Each accepted connection is handled on its own thread. At most 16
   connections are handled concurrently. A connection accepted over the limit
   is rejected without reading the request: the server writes `503 Service
   Unavailable` with `Connection: close`, closes, and logs `- - 503`.
3. Requests may use HTTP/1.0 or HTTP/1.1. Any other version receives `505`.
4. Exactly one request is served per connection. Every response includes
   `Connection: close`; there is no keep-alive and no pipelining.
5. Allowed methods are `GET`, `HEAD`, and `POST`. `HEAD` returns the same
   status and headers as `GET` with no body. Any other method on a known path
   receives `405` with `Allow`; on an unknown path it receives `404`.
6. `POST` is accepted only on `/doc/{id}` and `/api/doc/{id}`. A `POST` to a
   known path that does not accept it receives `405`; to an unknown path it
   receives `404`.
7. Request-line and header limits: request line at most 8192 bytes (`414`);
   target at most 2048 bytes (`414`); header block at most 16384 bytes and at
   most 64 fields (`431`). Header names and values are read as ISO-8859-1
   bytes and converted lossily to UTF-8 for matching.
8. Body limits: at most 1 MiB (`413`); `Transfer-Encoding` receives `501`;
   `POST` without `Content-Length` receives `411`; duplicate or non-numeric
   `Content-Length`, or both `Transfer-Encoding` and `Content-Length`, receive
   `400`. The body is read exactly `Content-Length` bytes.
9. Timeouts: the complete request must arrive within 5 seconds of connection
   acceptance (`408`); writing the response must complete within 5 seconds.
10. Only origin-form targets (`/path` or `/path?query`) are accepted. The path
    is percent-decoded exactly once; an encoded NUL, encoded `/`, encoded `\`,
    or any decoded control character receives `400`.
11. Path traversal is rejected after decoding: `.` or `..` segments, an empty
    segment other than a single trailing slash, or a path that does not start
    with `/` receives `400`. Paths are matched against a fixed route table; no
    request path is ever joined to a filesystem path.
12. Routing and status codes:
    - known route with a valid request: the route's success response;
    - well-formed request for an unknown path: `404` for every method;
    - known path with a disallowed method: `405`;
    - malformed request line or headers: `400`;
    - unsupported media type on `POST`: `415`;
    - body over the cap: `413`;
    - invalid or missing Host/Origin validation: `400` or `403`;
    - unexpected internal failure: `500` with a generic message.
13. Every response carries `Content-Type` with an explicit charset where
    textual, plus `Content-Length`, `Connection: close`, and the security
    headers. HTML responses carry the CSP. The cache class decides the
    `Cache-Control` line and whether an `ETag` is emitted (section 6).
14. There is no TLS, HTTP/2, WebSocket, compression, or outbound connection.
    The default bind address is loopback-only; binding elsewhere is warned
    about.
15. A missing or unreadable configured document directory is not a server
    error: the corresponding list renders its empty state and the API returns
    an empty array.
16. Concurrent requests are independent. The server keeps no session state and
    no in-memory copy of record content across requests. Each request takes one
    `ServingSnapshot` handle for configuration, registry, and routes; record
    content is read from disk for that request.

### 4. UI content scope

1. The interface covers every enabled document type through the unified document
   surface. For the requirement families, `req` and `nfr` are
   `GroupedMarkdown` types with group and member records.
2. Record sources are the active directory of `doc_directory` plus the
   descriptor's configured directory, resolved from the snapshot. Directories
   and prefixes are never hardcoded.
3. Only files directly inside those directories whose extension follows the
   model and whose stem matches the descriptor's prefix and ID shape are
   records. Subdirectories (including `archived/` and a legacy `deprecated/`),
   `index.html`, and non-matching files are ignored.
4. A document type absent from the configuration or disabled contributes no
   records and no error. Its sidebar entry, list, detail, edit, and API routes
   are absent; a direct request is `404`.
5. Archived records are not listed, not counted, not reachable by URL, and not
   editable. A request for an archived record returns the same `404` page as an
   unknown ID.
6. **Navigation model.** Every page shares one shell: a header with the project
   name and a static "Local browser / offline" note, a persistent left sidebar
   grouped by `Backlog`, `Decisions`, and `Documents`, `main`, and a footer. The
   sidebar has one entry per enabled descriptor. The header carries the
   navigation toggle; the expanded state is the default and the choice
   persists in `localStorage` under `papyrus.nav`.
7. The sidebar selects exactly one category page at a time. A list page
   renders only the selected type; there is no combined "all documents" view.
8. The default page is the inventory. `GET /` without a `type` renders the
   inventory; `GET /?type=<slug>` renders one type's list.
9. A list renders through the shared list contract: ID, Title, Status, family
   slots, Created, Updated, Actions. Group and member rows use the shared
   hierarchical row variant in merged group-then-member order.
10. A requirement whose group link does not resolve to an active group of the
    same type, and a requirement with no group link, renders as a top-level row
    in ID order. The group slot shows the stored reference verbatim; an absent
    reference shows `(none)`. There is no trailing catch-all section.
11. A group with no members still renders its group row with no children; the
    shared empty state and count rules apply.
12. **View.** The document view builder renders the ADR-033 order: the shared
    head and chrome, breadcrumb, a small record ID above the title, the title,
    read-only status beside the ID, the labels taglist, and the created and
    updated dates under the title; then one section per non-empty field in
    descriptor order, where an absent optional field is omitted and an absent
    required field renders as an empty section; then read-only extra metadata;
    then references, dependencies, and dependents as three separate native
    `<details>` disclosures, each rendered only when non-empty; then an `Edit`
    action. A `GroupedMarkdown` group renders the group fields then a read-only
    member list; a member renders the member fields then a read-only group
    link.
13. **Edit.** The document edit builder renders the fixed title control, the
    editable declared fields in sequence order, the general labels and
    references controls, read-only status and dates, and the hidden version
    token. `status` is never a control, and a field whose verbs exclude `edit`
    is read-only. A `SingleHtmlMockup` form offers `title` plus the list-valued
    `specs` and `refs` editable; `labels`, `references`, `status`, `task`, and
    `superseded_by` are read-only.
14. Field content is displayed with the existing prose pipeline; raw Markdown
    is not rendered as formatted text outside that pipeline. References are
    displayed as links when they resolve and as plain text otherwise.
15. No create, delete, move, or rename operation is offered. There is no
    "new record" control.
16. The interface is fully usable without internet access: every asset is
    served by the same process, and no page references an external origin.
17. The edit form carries the optimistic-concurrency token in a hidden
    `version` input. The approved mockup draws this as a visible,
    non-interactive dashed annotation; the shipped wording is
    `Hidden field: version (optimistic-concurrency token)`. The annotation is
    static status text, not a form control.

### 4a. Shared header branding

1. On every browser page, the project name renders large and bold.
2. The "Papyrus" mention renders in smaller letters.
3. The "P" mark keeps its current size and styling.
4. The project name shown is the configured project name.
5. The rule applies to the shared header of every page.
6. Every per-type browser specification references this rule rather than
   restating it.

### 4b. Record references as links

1. Every rendered record reference that resolves to a Papyrus record is a
   hyperlink: task dependencies, a task's milestone, `references` values, a
   member's owning group, and the linked records of a field.
2. Activating a link navigates to the referenced record's view; the browser
   back button returns to the originating page.
3. A reference that does not resolve is rendered as plain text and raises no
   error.
4. Links are generated in the page and work offline.

### 4c. Single record view layout

1. The view follows the ADR-033 metadata order: a small record ID above the
   title, the title, read-only status beside the ID, the labels taglist, and
   the created and updated dates under the title.
2. A member requirement additionally shows its owning group as a read-only link
   after the metadata block; nothing else is added.
3. The artifact content follows the metadata block with one section per
   non-empty field in descriptor order. An absent optional field is omitted; an
   absent required field renders as an empty section.
4. References, dependencies, and dependents appear as three separate native
   `<details>` disclosures, each shown only when the record has at least one
   such value.
5. Collapsible sections use native HTML disclosure elements and work with
   JavaScript disabled.
6. The layout applies to every managed Markdown record type that has a single
   record view. UI mockup HTML views use the same head and chrome with a fixed
   metadata block.
7. The view does not repeat the document type. Created and updated dates sit
   directly under the title, and there is no summary card.

### 4d. Configuration view (retired)

1. Per the retired-surface decision, the browser serves no configuration view.
   `GET /config`, `GET /config/edit`, `POST /config`, and `GET /?type=config`
   return `404` and relocate nothing.
2. There is no `Configuration` sidebar entry and no link from any page to a
   lifecycle view.
3. Configuration editing is CLI-only (`papyrus config get/set/unset`).
4. Clauses 1-8 of the earlier configuration-view contract remain historical
   only and are not served.

### 5. HTTP endpoints

Routes are exact. `{id}` is a record ID validated per section 7.1.

| Method | Path | Request | Success response |
|---|---|---|---|
| `GET` | `/` | optional `?type=<slug>` | `200 text/html` inventory, or one type's list |
| `GET` | `/doc/{id}` | — | `200 text/html` document view |
| `GET` | `/doc/{id}/edit` | — | `200 text/html` edit form |
| `POST` | `/doc/{id}` | `application/x-www-form-urlencoded`: `version`, `title`, declared field options, `labels`, `references` | `303 See Other` with `Location: /doc/{id}` |
| `GET` | `/api/docs/{slug}` | — | `200 application/json` list for one type |
| `GET` | `/api/doc/{id}` | — | `200 application/json` record |
| `POST` | `/api/doc/{id}` | `application/json`: `version`, `title`, field values | `200 application/json` updated record |
| `GET` | `/assets/app.css`, `/assets/app.js`, `/assets/nav-state.js` | — | `200` embedded first-party asset |
| `GET` | `/assets/<package>/<asset-version>/<file>` | — | `200` embedded vendor asset |
| `HEAD` | any `GET` route | — | same status and headers, no body |
| any | any other route | — | `404` for every method on an unknown path; `405` for a known path with a disallowed method |

1. The canonical `/api/docs/{slug}` response is the same shape as the type's
   seed-declared legacy alias. An alias is indistinguishable from the
   canonical form. Response ordering is by ID ascending, with grouped types
   merged group-then-member.
2. A document view and edit form are built from the field map:
   `SingleMarkdown` uses the top-level `fields` with role `Single`;
   `GroupedMarkdown` uses `group.fields` with role `Group` or `member.fields`
   with role `Member`; `SingleHtmlMockup` uses fixed metadata. A field with
   only `heading` reads the body section; a field with only `frontmatter_key`
   reads that key; a field with both reads the body section and does not merge
   the two locations.
3. Error responses:
   - HTML routes return an HTML error page with the status, a short message,
     and a link back to `/`.
   - JSON routes return
     `{"error":{"code":<http-status>,"message":"<message>"}}` with the matching
     HTTP status. The `code` field is the HTTP status, not the CLI exit code.
   - A form `POST` that fails validation returns `422` with the edit form
     re-rendered, the submitted values, a form-level summary, and field-level
     messages.
4. `POST /doc/{id}` requires
   `Content-Type: application/x-www-form-urlencoded` and rejects anything else
   with `415`. `POST /api/doc/{id}` requires `Content-Type: application/json`
   and rejects anything else with `415`.
5. Form fields are `version` (required), `title`, the declared field keys, and
   the general `labels` and `references` lists. A `status` field is rejected
   with `422`. Unknown fields are rejected. A repeated list-valued field
   accumulates into one ordered list as ADR-036 decision 17 and the ADR-039
   field map require; the last value does not win. For `SingleHtmlMockup` the
   form offers `title` plus the list-valued `specs` and `refs`; `labels`,
   `references`, `status`, `task`, and `superseded_by` are read-only. Values
   are percent-decoded with `+` as space; malformed percent-encoding receives
   `400`.
6. A successful form `POST` returns `303` and no body; the client re-fetches
   the view, which reads the file from disk again.
7. The JSON `POST` replaces the fields it is given and leaves omitted fields
   unchanged, except that an explicit `null` clears an optional section.
   `title` and `version` are required. Unknown members are rejected; a body
   that is not a JSON object, or a member of the wrong type, receives `400`.
8. Assets are served with the cache classes of section 6 and are identical for
   every request; they contain no per-request data.

### 6. Assets and rendering

1. HTML templates are text files `src/browser/templates/shell.html`,
   `list.html`, `view.html`, `edit.html`, and `error.html`; all are embedded
   with `include_str!` and rendered server-side. `view.html` and `edit.html`
   are single descriptor-driven templates for every document model.
2. Substitution uses one placeholder syntax `{{name}}` with a fixed per-template
   allowlist. There is no template engine dependency, no expression evaluation,
   and no placeholder expansion inside substituted values.
3. Every substituted value is HTML-escaped in text and attribute context:
   `&` to `&amp;`, `<` to `&lt;`, `>` to `&gt;`, `"` to `&quot;`, `'` to
   `&#39;`. Attribute values are always double-quoted. Escaping happens at
   render time only; stored content is never pre-escaped.
4. JSON is produced only with `serde_json`, never by string concatenation.
5. Assets are same-origin entries of one static table. First-party assets are
   `/assets/app.css`, `/assets/app.js`, and `/assets/nav-state.js`; they are
   served with `Cache-Control: no-cache` and a strong SHA-256 `ETag`, and a
   matching `If-None-Match` on `GET` or `HEAD` returns a bodyless `304`.
6. Vendored assets are served from `/assets/<package>/<asset-version>/<file>`
   and carry `Cache-Control: public, max-age=31536000, immutable` with no
   `ETag`; a conditional request returns the full `200`. The unversioned vendor
   paths return `404`.
7. The first-party stylesheet uses the system font stack and contains no
   `url()` reference. KaTeX's relative `url(fonts/...)` values resolve under
   the versioned KaTeX directory. No served CSS contains `@import`; no
   reference is remote; there is no `<img>`, `srcset`, icon font, or CDN.
8. The scripts are optional progressive enhancement. `nav-state.js` applies
   the stored navigation state and reveals the toggle; `app.js` owns the
   toggle, the dirty-form guard, tag editing, and the math and diagram render
   passes. With JavaScript disabled, every page, link, form, and error state
   still works.
9. No server-rendered page emits an inline `<script>`, an inline event-handler
   attribute, or an inline `style` attribute. The only `'unsafe-inline'` is
   `style-src` on a diagram response, scoped as the browser requirements
   record.
10. All pages declare `<!doctype html>`, `<html lang="en">`, a unique
    `<title>`, `<meta charset="utf-8">`, and the viewport meta tag.
11. Pages contain no absolute URLs to other origins. The only URLs are
    root-relative paths on the serving origin.

### 7. Editing contract

1. Record ID validation: an ID is accepted only if it is 1-128 bytes, contains
   only ASCII letters, digits, `.`, `_`, and `-`, does not start with `.` or
   `-`, does not contain `..`, and is not `.` or `..`. The ID is resolved by
   exact, case-sensitive match against the enabled descriptors' active
   directories; it is never concatenated into a path. An ID that matches no
   descriptor returns `404`. An ID that matches more than one enabled prefix is
   a `500` with a generic message, because a shared prefix is an ambiguous
   configuration. The descriptor is taken from the match, never from the
   request.
2. The record role (group or member) is derived from the matched ID shape and
   the descriptor, never from the request.
3. The model-neutral field map is built once per loaded record. Editable fields
   and their rules:
   - `title` is required, single-line, non-empty, and 1-200 Unicode scalar
     values; it rejects NUL, CR, LF, and other control characters.
   - a `text` field accepts a single-line scalar and rejects control
     characters.
   - a `multiline` field accepts LF, normalizes CR and CRLF to LF, allows TAB,
     rejects other control characters, and rejects a line whose first two
     characters are `## `.
   - a `list` field is a repeatable list of scalar values; each entry follows
     the `text` rules.
   - a required field may be changed but not cleared; an optional field may be
     cleared.
4. `status` is never accepted from the form or JSON. `labels` and `references`
   are general metadata and are saved as the resulting lists.
5. Validation failures return `422` (HTML form re-rendered with messages, JSON
   error object) and never modify the file.
6. Record ID, group link, and unknown sections are never editable. Unknown and
   editor-added sections are preserved. On save, the field's declared
   `heading` section and `frontmatter_key` are set from the same submitted
   value; a field with both writes both locations.
7. Writes are atomic and project-local: a temporary file is created in the same
   directory with `create_new(true)`, written as UTF-8, synced, given the
   original permissions, and renamed over the target. The directory is synced
   on a best-effort basis. On failure the temporary file is removed and the
   original is left untouched.
8. Every read and write path passes through `Context::ensure_within` before
   use. A symlink that resolves outside the project directory is rejected; the
   request receives `404` and the outside content is neither read nor written.
9. Concurrent edits use optimistic concurrency without locking. The version
   token is `"<mtime-seconds>.<mtime-nanos>.<length>"` of the live file at
   render time, sent as the `version` form field or JSON member. On `POST`:
   missing or malformed token is `400`; a current token mismatch is
   `409 Conflict` with no write; a matching token proceeds.
10. A record file that is not valid UTF-8 is skipped by the list and API index
    and returns `500` with a generic message when requested directly.
11. After a successful write the server keeps no cached copy; the redirect
    target re-reads the file, so a successful save is proven by the follow-up
    `GET`.
12. Editing a record never changes its ID, its directory, or its archive state.

### 8. Configuration changes

1. `Config` carries three browser keys with defaults:

| Key | Type | Default | Validation |
|---|---|---|---|
| `browser_host` | string | `127.0.0.1` | section 1.8 |
| `browser_port` | unsigned 16-bit integer | `6420` | `1..=65535`, therefore `0` is rejected |
| `browser_open` | boolean | `true` | must parse as a boolean |

2. `Config::validate()` enforces these rules and returns a validation error
   (exit 4) naming the offending key. Every command that validates the merged
   configuration fails on an invalid browser key.
3. The browser reads these values from the startup configuration only. After
   the listener binds, a fingerprint of the configuration file is checked once
   per request; a change rebuilds configuration, registry, and routes and
   publishes one new snapshot atomically. A reload does not rebind the socket,
   change the concurrency cap, or change the Host/Origin allowlist, so a change
   to `browser_host`, `browser_port`, or `browser_open` takes effect at the next
   start.
4. A failed reload keeps the previous snapshot serving and reports one
   `warning: ...` line on standard error; it does not exit, serve a
   configuration error page, or fall back to defaults.
5. `config list` prints the three effective keys in stable order after the
   `document_types.*` entries. `config get` prints one value. `config set`
   persists and re-validates the merged configuration; an invalid value is
   refused and the file is not changed. `config unset` removes the key and
   falls back to the default, re-validating the result. `config list --json`
   includes the three fields.

### 9. Security

1. **HTML escaping.** Every dynamic value is escaped as specified in section
   6.3, in text and attribute context. No user-controlled value reaches the
   response unescaped.
2. **JSON encoding.** All JSON is built with `serde_json`; no JSON is embedded
   in HTML.
3. **ID and path handling.** Record IDs are validated and then resolved by
   exact match against a directory listing; no request value is joined to a
   path. Request paths are matched against a fixed route table after
   single-pass percent-decoding, with traversal and control characters
   rejected.
4. **Boundary checks.** Every filesystem read and write passes
   `Context::ensure_within` against the canonicalized project directory.
   Symlink escapes are rejected before any content is read or written.
5. **Request-size caps.** Request line 8192 bytes, target 2048 bytes, header
   block 16384 bytes and 64 fields, body 1 MiB, 16 concurrent connections,
   5-second read and write timeouts.
6. **Headers and content policy.** Every `text/html` response carries exactly
   one of the two CSP strings below: the strict string on every HTML response
   whose pre-pass did not set `has_diagram`, and the diagram string only when
   the rendering pre-pass set `has_diagram`. Asset and non-HTML responses carry
   no CSP. `script-src 'self'` never gains `'unsafe-inline'`, `'unsafe-eval'`,
   a nonce, or a hash.

   Strict (`http::CSP`):

   ```
   default-src 'none'; style-src 'self'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
   ```

   Diagram (`http::CSP_DIAGRAM`):

   ```
   default-src 'none'; style-src 'self' 'unsafe-inline'; script-src 'self'; img-src 'self'; font-src 'self'; form-action 'self'; base-uri 'none'; frame-ancestors 'none'
   ```

   Every HTML response also sends `X-Frame-Options: DENY`,
   `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and
   `Cache-Control: no-store`. All responses send `Connection: close`.
7. **DNS rebinding and CSRF mitigation.** There is no authentication, cookie,
   or session, so the mitigation is strict origin validation:
   - the `Host` header must equal `<effective-host>:<port>`, `localhost:<port>`,
     `127.0.0.1:<port>`, or `[::1]:<port>` (case-insensitive host, exact port;
     an omitted `:80` is allowed); a mismatch receives `400`;
   - on `POST`, if `Origin` is present it must equal
     `http://<matching-host>:<port>`, otherwise `403`;
   - on `POST`, `Content-Type` must be `application/x-www-form-urlencoded` or
     `application/json` as appropriate, otherwise `415`;
   - no CSRF token is used.
8. **No secret leakage.** Error responses and pages never include absolute
   filesystem paths, environment values, OS error chains beyond a short
   reason, stack traces, or request bodies. JSON `path` values are relative to
   the project directory. Log lines contain only method, truncated decoded
   path, and status.
9. **No external access.** The server performs no outbound connections and
   serves no external resource. There is no telemetry, update check, remote
   font, or CDN.
10. **Transport.** There is no TLS. The default bind address is loopback; a
    non-loopback bind warns and exposes an unauthenticated read/write interface
    to the network.
11. **Response integrity.** Request-derived data is never written into response
    headers; `Location` values are fixed, root-relative paths derived from a
    validated ID.

### 10. Browser support and accessibility

1. The interface uses only features available in Firefox 115+: HTML5 forms,
   `<!doctype html>`, UTF-8, CSS flexbox and grid, CSS custom properties,
   `white-space: pre-wrap`, the system font stack plus the bundled KaTeX WOFF2
   fonts, and ES2019 JavaScript.
2. Forbidden: `:has()`, container queries, `color-mix()`, `@layer`, subgrid,
   `dialog`, popover, `inert`, and any feature that requires a newer engine
   than the stated baseline.
3. Every page, link, form, and error state works with JavaScript disabled.
4. The navigation toggle, tag editing, and math/diagram rendering are
   progressive enhancements, never required to read or edit a record.

## Conformance checks

| Check | How it is verified |
|---|---|
| Requirement content is served by the unified document surface | `req` and `nfr` list, detail, edit, conflict, and not-found go through the document handler; no per-type branch exists |
| Field map drives view and edit | A group uses `group.fields`, a member uses `member.fields`; story and description render as sections and form controls; body/front-matter precedence is asserted |
| Offline access | The page and every asset are same-origin; no external reference exists; the loopback run succeeds |
| Archive visibility | An archived requirement is absent from lists and API and returns the same `404` as an unknown ID |
| Status read-only | No form or JSON path writes `status`; a submitted `status` is `422` |
| Optimistic concurrency | A stale token returns `409` and writes nothing; a matching token writes once and the redirect target proves it |
| Configuration snapshot | A configuration edit changes pages and routes on the next request; host, port, and open changes take effect at the next start; a failed reload keeps the previous snapshot |
| Cache classes | App assets return `no-cache` plus `ETag`; vendor assets are immutable and versioned; HTML and JSON are `no-store` |
| Security | Escaping, boundary checks, request caps, Host/Origin validation, and CSP assertions pass |
| Browser support | Markup, CSS, and JavaScript use the stated baseline and work without JavaScript |
