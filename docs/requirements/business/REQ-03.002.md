# REQ-03.002 Manage Releases

## User Story
As a project maintainer, I want to create and maintain releases so that milestones can be grouped for planned delivery.

## Group
REQ-03 Milestone and Release Planning

## Criticality
Must have

## Description
The user can create, list, view, edit, and close releases. A release has a title and may have a description. A release may contain multiple milestones, while each milestone may be assigned to zero or one release. Release IDs use the configured prefix and padding (default example `REL-001`).

The release lifecycle is fixed by the backlog category (REQ-07.001): `planned -> released`. A release starts `planned`, is stored with its lifecycle state in the front-matter `status` field (REQ-05.010), and becomes `released` through `papyrus release close <id>`. `released` represents delivery completion and is distinct from archiving; releases are not archived. There is no configurable release status list, no `--status` edit option, and no release restore command.

## Acceptance Criteria
- The documented release commands create, list, view, and edit release title and description.
- A release can be associated with multiple milestones, and each milestone is associated with at most one release.
- With default settings, a generated release ID follows the `REL-001` format.
- A release starts `planned` and its lifecycle state is stored in the front-matter `status` field.
- `papyrus release close <id>` changes the release from `planned` to `released`.
- The release lifecycle is documented as `planned -> released`; `released` is delivery completion, distinct from archiving, and releases are not archived.
- The lifecycle is not configurable, no `--status` option performs the transition, and no restore command exists.

## Traceability
- README.md, `papyrus(1)` → COMMANDS → release (lines 279–305), STATUSES → Releases (lines 737–741)
- docs/business-rules/BR-007-release-command-behavior.md — release command behavior (verbatim manpage excerpt, lines 279–305)
- docs/domain-model/business-glossary/release.md — Release term lifecycle wording
- docs/domain-model/object-model/business-object-model.md — Release object description lifecycle wording
- docs/specifications/release-lifecycle.md — Release Lifecycle Semantics
- README.md, `papyrus-config(5)` → General keys (lines 949–962)
- Project-owner decision 2026-11-13: release lifecycle is `planned -> released`, transitioned by `papyrus release close <id>`; the state is stored in front matter; the `--status released` option is removed.
- REQ-05.010 — Record Metadata in YAML Front Matter (release `status`)
- REQ-07.001 — Organize Artifacts into Three Categories
- BR-007 — release Command Behavior (needs revision: close replaces `--status released`)
