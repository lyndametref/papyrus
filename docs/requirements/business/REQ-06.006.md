# REQ-06.006 Display NFR Atomic Requirements Under Their Group in the Browser

## User Story
As a project contributor, I want non-functional atomic requirements displayed under their group in the browser like business requirements so that navigation is consistent.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
In browser mode, non-functional atomic requirements are currently shown under "Other requirements" at the bottom of the page. They shall be displayed under their owning group entry, matching the grouping used for business requirements. The owning group is derived by the browser from the requirement ID prefix (`REQ-XX.YYY` → `REQ-XX`, `NFR-XX.YYY` → `NFR-XX`) and the group record's title; the requirement file format is not changed and non-functional atomic files gain no `Group` section.

## Acceptance Criteria
- The browser lists non-functional atomic requirements under their group entry rather than under a catch-all "Other requirements" section.
- Grouping is consistent with the grouping used for business requirements.
- Grouping is derived from the requirement ID prefix in the browser for both requirement families; no `Group` field is added to non-functional atomic requirement files.
- A group heading shows the group record's title (for example `NFR-04 Persistence`); when the group record is missing, the group number is used.
- Empty groups are handled gracefully: a group with no atomic requirements still renders as an entry with no children.
- Every atomic requirement appears under exactly one group and none is dropped.

## Traceability
- Source: customer-feedback handoff — maintainer, Item 5.
- Maintainer decision 2026-11-11: grouping is fixed at the browser level by deriving the group from the ID prefix (the NFR atomic format has no `Group` section), so the data files are unchanged.
- REQ-06.002 — Serve the Local Web Interface
- REQ-05.010 — Record Metadata in YAML Front Matter (titles come from front matter)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
