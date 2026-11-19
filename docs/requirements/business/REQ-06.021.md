---
id: REQ-06.021
title: "Mark Open and Closed Milestones and Releases in the Browser Backlog"
created_date: '2026-11-19T10:55:12Z'
updated_date: '2026-11-19T11:14:49Z'
labels: []
references:
  - RE-008
  - UI-002
  - MOCK-0016
  - REQ-06.020
  - REQ-06.022
---
# REQ-06.021 Mark Open and Closed Milestones and Releases in the Browser Backlog

## User Story
As a project contributor, I want milestone and release lifecycle state marked before their titles so that open and closed groups are recognizable at a glance.

## Group
REQ-06 Search and Local Web Interface

## Description
The browser Backlog view marks lifecycle state immediately before each milestone and release title: 🔘 for an open milestone or a planned release, and ✅ for a closed milestone or a released release. The marker is placed immediately before the title text in the milestone summary and in the release heading, in addition to the existing record ID and title. The marker does not replace status information provided elsewhere in the view. Archived milestones are not shown in the Backlog view and therefore carry no marker. The marker reflects the stored lifecycle state and changes after that state changes and the view is reloaded.

## Criticality
Should have

## Acceptance Criteria
- Every displayed open milestone shows 🔘 immediately before its title.
- Every displayed closed milestone shows ✅ immediately before its title.
- Every displayed planned release shows 🔘 immediately before its title.
- Every displayed released release shows ✅ immediately before its title.
- Archived milestones are not displayed and show no marker.
- The marker reflects the stored lifecycle state; after closing or releasing a record and reloading, the marker changes accordingly.
- The marker appears in addition to the record ID, not instead of it.
- The marker does not change milestone or release grouping or ordering.

## Traceability
- RE-008 Item 1 - markers before milestone and release titles.
- Requester decision 2026-11-19: ✅ marks closed milestones and released releases; 🔘 marks open milestones and planned releases; archived records stay hidden.
- REQ-03.001 - Manage Milestones (open, closed, archived lifecycle).
- REQ-03.002 - Manage Releases (planned to released lifecycle).
- REQ-06.013 - Present the Backlog as a Per-Milestone Kanban Board (milestone sections and release headings).
- BR-004 - Papyrus browser Command Behavior.
- REQ-06.020 - Order Milestones and Releases by Lowest Task Ordinal in the Browser Backlog (ordering in the same view).
- REQ-06.022 - Clean Up Closed Milestones and Released Releases in the Browser Backlog (cleanup controls in the same view).
- UI-002 - Papyrus Local Browser (Backlog Records): the current board layout does not define lifecycle markers; it requires revision for the marker placement.
- MOCK-0016 - Browser shell and shared list layout: shell and list baseline used by the UI revision; the flat milestone list keeps its Status column and is not marked by this requirement.
