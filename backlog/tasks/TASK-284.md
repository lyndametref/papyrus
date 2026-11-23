---
id: TASK-284
title: "Update README, man pages, upgrade notes, and packaging documentation"
created_date: '2026-11-20T17:44:01Z'
updated_date: '2026-11-23T06:19:55Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-07.002
  - REQ-07.004
  - TECH-069
  - ADR-042
  - TECH-008
  - ADR-0021
status: ToDo
assignee:
  - tech-writer
milestone: MS-037
dependencies:
  - TASK-282
ordinal: 50
documentation:
  - README.md
  - docs/release-1.0-upgrade-notes.md
  - docs/packaging.md
  - packaging/deb
  - docs/specifications/technical/TECH-008.md
type: docs
---

## Description
## Context
Release 1.0 changes the document surface: three category handlers and one descriptor-driven engine over three `DocumentModel` values (`SingleMarkdown`, `GroupedMarkdown`, `SingleHtmlMockup`); a config-declared field schema with nested `group`/`member` blocks, a nine-key `FieldSpec`, selector paths, scalar fill and collection replace; the uniform title contract; `--term` removal; and changed cache headers. The README currently still says user-defined types use a generic baseline and built-in types keep their typed fields (document-family section of `README.md`), which is stale after ADR-036.

## Scope
- `README.md`:
  - papyrus(1) section: describe the three category handlers and the three document models; the uniform title contract (first positional `TITLE` on `create`/`create-group`, `--title` on create/create-group/edit, title is not a field, is required on create, and cannot be cleared); the model-derived verbs and options (`status`, `create-group`/`--group`, `supersede`); and the field vocabulary, pointing to papyrus-config(5).
  - papyrus-config(5) section: add the nested `group`/`member` blocks; the nine-key `FieldSpec` (`key`, `heading`, `frontmatter_key`, `option`, `label`, `kind`, `required`, `verbs`, `help`); selector paths and the fail-closed diagnostic form `document_types.<slug>.<path>`; seed scalar fill and collection replace; the reserved field keys; and the file-edit-only rule for adding, removing, or reordering fields (`config set`/`unset` reject field selectors; `config get`/`list` resolve them), per ADR-036 and ADR-042 rulings 1-6.
  - Keep the AI-assistance notice as the last non-empty line of the file (the root README rule).
- `docs/release-1.0-upgrade-notes.md`: add or correct the title migration (positional title, `--title`), the removal of glossary `--term` (the term is the title), the new `document_types` shape with a minimal before/after example including `fields` and `group.fields`/`member.fields`, and the changed cache headers (`no-store` HTML and JSON, `no-cache` plus a strong ETag for app assets, immutable versioned vendor assets) per ADR-038.
- `docs/packaging.md`: add a current, dated release-1.0 section: asset version source (vendor directory, `src/browser/vendor/MANIFEST.md`, and the copyright file), the single release boundary and exactly one retained artifact, the RC-as-discardable verification build under the git-ignored `builds/<version>/`, and the retained artifact layout (`/usr/bin/papyrus` 0755, `/usr/share/doc/papyrus/copyright` 0644, no man page per TECH-008 section 1.7 and ADR-0021).
- `packaging/deb/`: keep `DEBIAN/control` and `usr/share/doc/papyrus/copyright` consistent with the documented layout; add no packaged file.
- Do not document a man page the package does not ship (ADR-0021 remains proposed; TECH-008 section 1.7 says the inventory has no man page) and do not document behaviour that is not in the release.

## Out of scope
- No source, test, requirement, ADR, specification, mockup, or packaging-layout change.
- No new man page, man-page generator, or `docs/man/` tree; ADR-0021 is proposed and unselected.
- No version bump and no package build.

## Assumptions
- "papyrus(1)" and "papyrus-config(5)" name the README sections, not files shipped in the `.deb`.
- The current help surface (`target/release/papyrus <slug> --help`) is the source for the title contract and derived options.

## Definition of done
The four documents describe the released behaviour, the README AI notice remains the last non-empty line, `papyrus check` reports no new finding, and every documented command or option is proved against `target/release/papyrus --help`.
## Acceptance Criteria
- [ ] README papyrus(1) names the three category handlers and the three document model values and no longer states that user-defined types use a generic baseline or that built-in types keep typed fields.
- [ ] README papyrus(1) states the uniform title contract and it matches the current glossary create, req create, req create-group, and ui-mockup help output.
- [ ] README papyrus(1) states the model-derived verbs and options: status, create-group with --group, and supersede with no status.
- [ ] README papyrus-config(5) documents the fields, group.fields, and member.fields sequences; the nine FieldSpec keys; selector paths and the document_types.<slug>.<path> diagnostic; scalar fill versus collection replace; the reserved field keys; and the file-edit-only rule.
- [ ] README papyrus-config(5) states that config get and config list resolve field selector paths and config set and config unset reject every selector inside fields, group.fields, or member.fields.
- [ ] Upgrade notes add the title migration and the glossary --term removal and state that the glossary term is the title.
- [ ] Upgrade notes include a minimal before-and-after document_types example with a single-role fields sequence and a grouped group and member pair.
- [ ] Upgrade notes state the cache headers: HTML and JSON no-store; app CSS, app JS, and nav-state JS no-cache with a strong ETag and 304; versioned vendor assets public, max-age=31536000, immutable with no ETag.
- [ ] Packaging documentation has a dated release-1.0 section stating the asset version source, the single release boundary, the RC-under-builds rule, and the package inventory with a 0755 binary, a 0644 copyright file, and no man page.
- [ ] The documented package inventory matches dpkg-deb -c of the current package and TECH-008 section 1.7; no man page is claimed.
- [ ] The README last non-empty line is the AI-assistance notice.
- [ ] papyrus check reports no new finding from the edited files.
## Comments
- **task-refiner**: Packaging doc item is coupled to the TASK-288 output-path escalation: until the planner and architect decide between allowing the builds path and staging under the temp directory, describe the single release boundary and mark the RC staging path as pending.
