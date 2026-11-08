# REQ-03.002 Manage Releases

## User Story
As a project maintainer, I want to create and maintain releases so that milestones can be grouped for planned delivery.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, and edit releases. A release has a title and may have a description. A release may contain multiple milestones, while each milestone may be assigned to zero or one release. Release IDs use the configured prefix and padding (default example `REL-001`). The release lifecycle is `active -> released`: `released` represents delivery completion and is distinct from archiving (record management); releases are not archived. The transition from `active` to `released` is performed by an explicit status command, `papyrus release edit ... --status released`; `released` is a stored release attribute, not derived from milestone or task completion.

## Acceptance Criteria
- The documented release commands create, list, view, and edit release title and description.
- A release can be associated with multiple milestones, and each milestone is associated with at most one release.
- With default settings, a generated release ID follows the `REL-001` format.
- The release lifecycle is documented as `active -> released`; `released` is delivery completion, distinct from archiving, and releases are not archived.
- The active-to-released transition is performed explicitly via `papyrus release edit ... --status released`, and `released` is stored as a release attribute rather than derived from milestone or task completion.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → release (lines 279–305), STATUSES → Releases (lines 737–741)
- docs/business-rules/BR-007-release-command-behavior.md — release command behavior (verbatim manpage excerpt, lines 279–305)
- docs/domain-model/business-glossary/release.md — Release term lifecycle wording
- docs/domain-model/object-model/business-object-model.md — Release object description lifecycle wording
- docs/specifications/release-lifecycle.md — Release Lifecycle Semantics
- README.md, `papyrus-config(5)` → General keys (lines 949–962)
