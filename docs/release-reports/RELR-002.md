---
id: RELR-002
title: "Release 1.0.0 approval"
created_date: '2026-11-23T15:24:06Z'
updated_date: '2026-11-23T15:24:06Z'
labels: []
references:
  - REL-001
  - RELR-001
status: issued
---
# RELR-002 Release 1.0.0 approval

## Description
# Release 1.0.0 approval record

- Release: REL-001, now titled Release 1.0.0 and status released.
- Scope and audit findings: accepted by the owner on 2026-11-23, including the measured-only NFR-007.001 and NFR-007.002 status, the ADR-044 non-loopback unauthenticated risk, and the recorded low follow-ups.
- Release approval: granted by the owner on 2026-11-23.
- Artifact: builds/1.0.0/.build/papyrus/papyrus_1.0.0_amd64.deb, 2,429,284 bytes, sha256 b2d747ac017a65659b5147300775d423b95961c3b611bfbc733df09c57ddf43c, built from commit 38736ae.
- Verification build: builds/1.0.0-rc.1, discarded and marked superseded.
- Gates at approval: code AUD-155 closed with artifact correspondence confirmed; security AUD-157 pass with findings, 0 critical/high/medium; quality AUD-156 pass with MS-037 AC-1 to AC-7 met; 1446 tests passed, 0 failed; clippy and fmt clean.
