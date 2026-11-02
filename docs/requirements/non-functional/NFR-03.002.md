# NFR-03.002 Distribute Papyrus as a Debian Package

## Description
Papyrus shall be distributed as `.deb` packages for the amd64 architecture on Debian Forky only.

## Rationale
This records the requested Debian package format, architecture, and supported release. Supported-release scope is narrowed to Debian Forky only per ADR-0003 (accepted), which supersedes ADR-0002 (superseded); Debian Trixie and any other Debian release are not claimed or verified as supported.

## Verification Method
Inspect the distribution artifacts to confirm that an amd64 `.deb` package is provided, and install and smoke-test it on Debian Forky.

## Open Questions
No future Debian release is claimed as supported without explicit verification. Adding a release to the supported set requires an explicit verification pass and a follow-up ADR/requirement update (ADR-0003).

## Traceability
- ADR-0002 — Distribution Format — amd64 `.deb` Packages for Debian Trixie, Forky, and Later Releases (superseded by ADR-0003).
- ADR-0003 — Supported Debian Release Target — Forky Only (amd64 `.deb`) (accepted).
