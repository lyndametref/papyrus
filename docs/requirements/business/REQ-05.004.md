# REQ-05.004 Manage UI Specifications

## User Story
As a project contributor, I want to maintain UI specifications associated with views so that UI documentation can be found and updated.

## Group
REQ-05 Project Documentation Management

## Criticality
Must have

## Description
The user can create a UI specification with a title, associated view, and description; list, view, and edit it; and deprecate or restore it. Deprecated specifications are hidden from lists, search, and the web UI, and restored records retain their IDs.

## Acceptance Criteria
- A UI specification can be created with the required title, view, and description.
- UI specifications can be listed, viewed, and edited for view and description.
- Deprecation hides a specification from lists, search, and the web UI.
- Restoration returns a specification to its original folder with its ID preserved.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → ui-spec and Documents lifecycle (lines 566–610, 778–784)
