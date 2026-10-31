# REQ-03.002 Manage Releases

## User Story
As a project maintainer, I want to create and maintain releases so that milestones can be grouped for planned delivery.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, and edit releases. A release has a title and may have a description. A release may contain multiple milestones, while each milestone may be assigned to zero or one release. Release IDs use the configured prefix and padding (default example `REL-001`). The man page lists an active-to-archived release lifecycle but does not document a release archive or restore command.

## Acceptance Criteria
- The documented release commands create, list, view, and edit release title and description.
- A release can be associated with multiple milestones, and each milestone is associated with at most one release.
- With default settings, a generated release ID follows the `REL-001` format.
- The release lifecycle is documented as active to archived; the man page does not specify how users perform this transition.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → release (lines 279–305), STATUSES → Releases (lines 765–769)
- README.md, `papyrus-config(5)` → General keys (lines 949–962)
