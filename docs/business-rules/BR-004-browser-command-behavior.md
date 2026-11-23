---
id: "BR-004:"
title: "Papyrus browser Command Behavior"
created_date: '2026-11-01T20:26:25Z'
updated_date: '2026-11-16T16:57:07Z'
labels: []
references: []
status: active
---
# BR-004: Papyrus browser Command Behavior

## Description

The papyrus browser command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → browser); behavior not stated there is unspecified.

## Rationale

Based on REQ-006.002 — Serve the Local Web Interface, this rule exists because the documented browser behavior is the contract for papyrus browser; README.md is the sole behavior source.

## Applicability

Applies to all invocations of `papyrus browser` and the project data they manage. The excerpt below reproduces relevant manpage behavior verbatim; heading identifies README section and exact source line range.

### papyrus(1): browser — `README.md` lines 94–108

```text
browser

       Serve the local web UI over both the backlog and documentation stores.
       
       papyrus browser [--host <host>] [--port <port>] [--no-open]
       
       --host <host>
       Bind address. Default: 127.0.0.1. A non-loopback address is allowed but prints a warning.

       --port <port>
       TCP port. Default: 6420.

       --no-open
       Do not open the default browser automatically.
```
