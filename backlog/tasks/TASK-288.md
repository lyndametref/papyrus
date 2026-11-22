---
id: TASK-288
title: "Produce the single release package and verify its contents"
created_date: '2026-11-20T17:44:23Z'
updated_date: '2026-11-22T15:27:22Z'
labels: []
references:
  - TECH-069
  - TECH-027
  - ADR-0027
  - ADR-038
  - NFR-007.003
status: ToDo
assignee:
  - rust-dev
milestone: MS-037
dependencies:
  - TASK-287
ordinal: 90
documentation:
  - scripts/build-deb.sh
  - packaging/deb
  - scripts/build-package.sh
type: packaging
---

## Description
Context: owner decision 2 allows one release only; earlier builds exist for verification and must not be shipped.

Scope: produce the single final release package with scripts/build-package.sh --version <final> --out builds/<version>/.build/<component>, the only packaging entry point per ADR-043, which delegates Debian assembly to scripts/build-deb.sh. The same entry point serves the verification builds and the release boundary. Include the man pages generated from the current help surface, the versioned vendor assets, the notices required for bundled assets, and the deb metadata. Confirm the package contains no document-type modules, no debug symbols in the stripped binary, and the documented asset version. Verify installation, papyrus --version, papyrus config list, and a create and edit round trip from the installed package. Record checksums and a manifest.
