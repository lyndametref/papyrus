---
id: TASK-284
title: "Update README, man pages, upgrade notes, and packaging documentation"
created_date: '2026-11-20T17:44:01Z'
updated_date: '2026-11-20T17:44:01Z'
labels: []
references:
  - ADR-036
  - ADR-037
  - ADR-038
  - ADR-039
  - REQ-07.002
  - REQ-07.004
  - TECH-069
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
type: docs
---

## Description
Context: users face a title migration, glossary --term removal, a new config shape, and new cache headers; none of it is documented yet.

Scope:
- README: the three category handlers, the three document models, the config entry shape, and how to add a field by editing the file. Keep the AI-assistance notice at the end of the file.
- papyrus(1): generated command surface with the uniform title contract, derived options, create-group, and supersede.
- papyrus-config(5): nested group and member blocks, field vocabulary, selector paths, defaults and collection replace, diagnostics, and the file-edit-only rule for adding fields.
- Upgrade notes: title migration, --term removal and glossary behavior, the new config shape with a minimal before and after example, and the changed cache headers.
- Packaging: the asset version source, the single release boundary, and the retained artifact layout.
Do not document features that are not present in the release.
