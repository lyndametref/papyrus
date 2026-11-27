---
id: TECH-094
title: "Browser Server, Offline Operation, and Compatibility"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-27T17:45:38Z'
labels: []
references:
  - REQ-006
  - REQ-006.002
  - REQ-006.007
  - NFR-001
  - NFR-001.003
  - NFR-005
  - NFR-005.001
status: verified
---
# TECH-094 Browser Server, Offline Operation, and Compatibility

## Description
Covers the serving, offline, and compatibility part of the local web interface (REQ-006): bind address and automatic port selection, non-loopback handling, automatic browser opening, console output, offline operation, and the Firefox support floor. Input sanitization and latency are separate specifications; list, view, edit, board, and rendering behavior are separate; content-security policy, cache headers, startup behavior, and internal server architecture are also separate.

## Command surface and bind defaults

- `papyrus browser` starts the web UI over backlog, decision, and document records (REQ-006.002).
- The default bind address is `127.0.0.1`; there is no default port (REQ-006.002).
- With no selected port, the command chooses a free port above 1024 and below 9999, retrying with another port when one is taken (REQ-006.002).
- The chosen address is reported when the browser starts (REQ-006.002).
- The host and the port are selectable; a selected port is used as given, and an unavailable selected port fails with a conflict outcome (REQ-006.002).
- A user-specified host that is not a loopback address is accepted and causes a warning (REQ-006.002).
- The default browser opens automatically on start; automatic opening is suppressed by `--no-open` or by configuration (REQ-006.002).
- The warning, when raised, does not block startup; the command serves with the requested host (REQ-006.002).

## Console output

- By default the `browser` command outputs only errors to the console (REQ-006.007).
- Errors are output whether or not `--debug` is set (REQ-006.007).
- `--debug` is an option of the `browser` command only: it is not a global option and is not accepted by any other command (REQ-006.007).
- With `--debug`, each received request is logged as one line containing the method, the path, the response status, and the duration (REQ-006.007).
- Request headers and request bodies are not logged, so record content does not reach the console (REQ-006.007).

## Offline operation

- The `browser` command and the local browser interface operate without an internet connection (NFR-001, NFR-001.003).
- With external networking disabled and loopback available, the command starts and the interface stays usable for list, view, and edit across record types (NFR-001.003).
- The interface attempts no outbound network request while that operation is exercised (NFR-001.003).
- Offline operation is a property of the served interface as a whole, not of a subset of pages (NFR-001.003).

## Browser compatibility

- The supported browser family is Firefox, version 115 and later (NFR-005, NFR-005.001).
- The rendering engine is Gecko 115, the engine shipped with Firefox 115; later Firefox releases use later Gecko versions.
- The document format is the HTML Living Standard, served as `text/html`; XHTML is not used.
- The transport is HTTP/1.1 over loopback; TLS, HTTP/2, and HTTP/3 are not used.
- The JavaScript baseline is ECMAScript 2022; classic scripts and ES modules are supported, and no transpiler or polyfill is required.
- The CSS baseline is the set of features available in Gecko 115; no feature introduced after Firefox 115 is required, including `:has()`.
- Firefox 115 is an Extended Support Release; the interface must work in that release and in every later Firefox release.
- One implementation serves every supported version; there is no per-version code path.
- Supported means the interface renders and is usable with no layout breakage in Firefox 115 and later (NFR-005.001).
- Verification runs on Firefox 115 ESR, the floor of the supported range.
- Verification runs on the latest Firefox ESR release.
- Verification runs on the latest Firefox release.
- When the Firefox version installed on the verification machine differs from all three, verification also runs on that installed version.
- Each run exercises list, view, edit, board, and rendered content, and confirms rendering and usability with no layout breakage (NFR-005.001).
- Visual layout is not fixed here; the layout is owned by the UI specification (NFR-005).
