---
id: ADR-034
title: "Remove Configuration, Managed Artifacts, and Agent Skills Sections from the Browser"
created_date: '2026-11-18T20:14:42Z'
updated_date: '2026-11-18T20:22:24Z'
labels: []
references:
  - REQ-06.018
  - RE-005
  - REQ-06.003
  - REQ-06.005
  - REQ-06.008
  - ADR-0005
  - ADR-0020
  - TECH-050
  - TECH-051
  - TECH-068
  - TECH-074
superseded_by: null
---
# ADR-034 Remove Configuration, Managed Artifacts, and Agent Skills Sections from the Browser

## Status
accepted
## Decision
1. Remove navigation entries and routes for Configuration, Managed artifacts and lifecycles, and Agent skills and workflows from the browser. 2. Remove the browser configuration view; configuration editing remains available only through the CLI using papyrus config. 3. Remove the managed-artifacts-and-lifecycles and agent-skills pages from the browser; their documentation remains available outside the browser. 4. Underlying records and configuration files are not changed by the removal. 5. Other browser pages, navigation, and routes continue to work. 6. Affected building blocks TECH-050, TECH-051, and the config-view portions of TECH-063, TECH-068, TECH-074, and TECH-076 are retired or revised to no longer provide these surfaces.
## Context
REQ-06.018 and RE-005 require removing the Configuration, Managed artifacts and lifecycles, and Agent skills and workflows sections from the browser navigation and routes. The browser currently hosts these as utility pages: TECH-050 managed artifacts and lifecycles, TECH-051 agent skills, and the browser configuration view of TECH-063 and TECH-068. Configuration editing is also available through the CLI and managed-artifact and skill documentation lives outside the browser. Removing the browser-side surfaces keeps the browser focused and keeps configuration CLI-authoritative. Underlying records and configuration files are unchanged.
## Consequences
Positive: simpler, focused browser interface; single configuration authority in the CLI; no browser-side configuration writes. Negative: browser can no longer edit configuration or display managed-artifact and skill documentation; users must use the CLI and external docs. Risks: stale references to removed routes in TECH specs, mockups, and tests must be updated; implementation must preserve other pages.
