---
id: TECH-107
title: "Licensing"
created_date: '2026-11-25T21:03:10Z'
updated_date: '2026-11-25T21:07:36Z'
labels: []
references:
  - NFR-006
  - NFR-006.001
  - NFR-006.002
status: draft
---
# TECH-107 Licensing

## Description
Covers the project licence, the licence policy for compiled crates and bundled assets, the provenance records, and the packaged notice file. Build tooling and the distribution format are separate specifications.

## Project Licence

- Papyrus is licensed under GPL-3.0-or-later.
- The repository licence declaration states GPL-3.0-or-later.
- The Cargo manifest licence field states GPL-3.0-or-later.
- The distributed copyright notice states GPL-3.0-or-later.

## Dependency and Asset Policy

- Every crate compiled into the binary carries an OSI-approved licence compatible with distribution under GPL-3.0-or-later.
- Every bundled third-party asset carries an OSI-approved licence compatible with distribution under GPL-3.0-or-later.
- The licence audit covers direct and transitive crates and every bundled asset; its result is recorded with the release.
- An asset or crate whose licence cannot be shown compatible does not ship, and the release is blocked.

## Vendored Asset Provenance

- Assets live under `src/browser/vendor/<package>/<version>/` and are byte-identical to upstream unless a documented `.patch` accompanies them.
- Each version directory carries the upstream files, `LICENSE`, and `UPSTREAM.md` with package, version, upstream URL, and retrieval date.
- `src/browser/vendor/MANIFEST.md` records, per asset, the package, version, licence, upstream URL, byte size, and SHA-256.
- A digest test recomputes every embedded asset's SHA-256 and fails on a mismatch. Updating an asset updates its version directory, manifest row, notices, and tests.
- No automatic updater, lockfile fetch, or network access is part of the build.

## Current Asset Inventory

- KaTeX 0.16.11: code MIT; bundled WOFF2 fonts SIL OFL-1.1.
- Mermaid 11.17.2: MIT.

## Packaged Notices

- `packaging/deb/usr/share/doc/papyrus/copyright` carries the Papyrus GPL-3.0-or-later statement and one section per bundled asset: name, version, licence identifier, and the full licence text or a Debian-style reference to the on-disk text.
- The package keeps one documentation file; no separate third-party notice file is shipped.
- The notice file covers the same crate and asset set as the audit.
