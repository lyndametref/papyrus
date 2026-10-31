# REQ-06.002 Serve the Local Web Interface

## User Story
As a project contributor, I want to open a local web interface over project stores so that I can access project information through a browser.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Must have

## Description
The browser command serves the local web UI over both backlog and documentation stores. Its default bind address is `127.0.0.1` and default port is `6420`; the host and port can be selected. A non-loopback host is permitted but causes a warning. The default browser opens automatically unless disabled by option or configuration.

## Acceptance Criteria
- `papyrus browser` starts the web UI over the backlog and documentation stores using the configured/default host and port.
- The default host is `127.0.0.1` and the default port is `6420`.
- A user-specified non-loopback host is accepted and causes a warning.
- `--no-open` prevents automatic opening of the default browser.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → browser (lines 94–107)
- README.md, `papyrus-config(5)` → Web UI keys (lines 1028–1037)
