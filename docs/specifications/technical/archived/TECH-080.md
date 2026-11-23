---
id: TECH-080
title: "Browser Removal of Configuration, Managed Artifacts, and Agent Skills Sections"
created_date: '2026-11-18T20:15:27Z'
updated_date: '2026-11-19T18:12:59Z'
labels: []
references:
  - REQ-006.018
  - RE-005
  - REQ-006.003
  - REQ-006.005
  - REQ-006.008
  - ADR-034
  - TECH-050
  - TECH-051
  - TECH-068
  - TECH-074
  - TECH-076
status: archived
---
# TECH-080 Browser Removal of Configuration, Managed Artifacts, and Agent Skills Sections

## Description
Building block: browser removal of Configuration, Managed artifacts and lifecycles, and Agent skills and workflows sections (ADR-034, REQ-006.018, RE-005). Removed: navigation entries and routes for config, config edit, skills, lifecycle and aliases; browser configuration view; managed-artifacts page; agent-skills page. Unchanged: underlying records and configuration files; other browser pages, navigation, routes, list, detail, edit behavior. Configuration editing remains CLI-only via papyrus config. Managed-artifact and skill documentation remains outside the browser. Affected specs: TECH-050 and TECH-051 retired; TECH-063 config-view portions removed; TECH-068 fixed routes and utility entries removed; TECH-074 and TECH-076 utility navigation entries removed. Traceability: REQ-006.018, RE-005, REQ-006.003, REQ-006.005, REQ-006.008, ADR-034, NFR-001.002, NFR-001.003.
