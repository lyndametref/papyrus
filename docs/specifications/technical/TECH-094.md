---
id: TECH-094
title: "Browser Server, Security, Offline Operation, and Performance"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-006
  - REQ-006.002
  - NFR-001
  - NFR-001.003
  - NFR-002
  - NFR-002.002
  - NFR-005
  - NFR-005.001
  - NFR-007
  - NFR-007.001
status: draft
---
# TECH-094 Browser Server, Security, Offline Operation, and Performance

## Description
Covers the serving, security, offline, and performance part of the local web
interface (REQ-006): bind defaults and selection, non-loopback handling,
automatic browser opening, offline operation, input sanitization on the browser
input paths, the Firefox support floor, and inventory-page latency at 1,000
records. It does not cover list, view, edit, board, or rendering behaviour.
Content-security policy, cache headers, startup behaviour, and internal server
architecture are outside this specification.

## Command Surface and Bind Defaults

- `papyrus browser` starts the web UI over the backlog store and the
  documentation store (REQ-006.002).
- The default bind address is `127.0.0.1` and the default port is `6420`
  (REQ-006.002).
- The host and the port are selectable; a selected value replaces the
  corresponding default for that run (REQ-006.002).
- A user-specified host that is not a loopback address is accepted and causes a
  warning (REQ-006.002).
- The default browser opens automatically on start; automatic opening is
  suppressed by `--no-open` or by configuration (REQ-006.002).
- The warning, when raised, does not block startup; the command serves with the
  requested host (REQ-006.002).

## Offline Operation

- The `browser` command and the local browser interface operate without an
  internet connection (NFR-001, NFR-001.003).
- With external networking disabled and loopback available, the command starts
  and the interface stays usable for list, view, and edit across artifact
  families (NFR-001.003).
- The interface attempts no outbound network request while that operation is
  exercised (NFR-001.003).
- Offline operation is a property of the served interface as a whole, not of a
  subset of pages (NFR-001.003).

## Input Sanitization

- Every user-supplied value from CLI options, standard input, and browser forms
  is sanitized before use (NFR-002.002).
- A value that cannot be normalized into a valid identifier, name, title, or
  path is rejected with an error; no invalid value is silently cleaned,
  truncated, or written (NFR-002.002).
- Browser form values take the same contract as command input; a rejected value
  fails the save and no record content is written (NFR-002.002).
- Control characters, path-separator values, and traversal values are rejected
  rather than cleaned, both on the command line and in a browser form
  (NFR-002.002).
- The host and the port are user-supplied option values and follow the same
  sanitization contract (NFR-002.002, REQ-006.002).

## Browser Compatibility

- The supported browser family is Firefox, version 115 and later
  (NFR-005, NFR-005.001).
- Supported means the interface renders and is usable with no layout breakage in
  Firefox 115 and later (NFR-005.001).
- Compatibility is verified by exercising the interface in version 115 and later
  and confirming rendering and usability with no layout breakage in each tested
  version (NFR-005.001).
- Visual layout is not fixed here; the layout is owned by the UI specification
  (NFR-005).

## Inventory Latency at 1,000 Records

- The document-type inventory page, at route `/`, is served with a p95 latency
  below 250 ms when 1,000 active records exist across the enabled document types
  (NFR-007.001).
- The measurement corpus is 1,000 active records across the enabled document
  types, including a `GroupedMarkdown` group with 200 members and a
  `SingleHtmlMockup` type with 50 records (NFR-007.001).
- Reference conditions are 4 vCPU, 8 GiB RAM, an SSD, Linux, one server process,
  loopback HTTP, and a release build with LTO and strip (NFR-007.001).
- The run uses 10 warm-up requests followed by 100 measured sequential requests
  against one running server; the reported figures are the server-observed p50
  and p95 (NFR-007.001).
- The first request after a cold start and any request that triggers a
  configuration reload are excluded from the measurement (NFR-007.001).
- The run and the measured p95 are recorded with the release (NFR-007.001).
- The target is stated for server-observed latency only; the requirement sets no
  target for client rendering time (NFR-007.001).
