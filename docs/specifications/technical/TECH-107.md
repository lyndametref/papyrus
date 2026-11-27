---
id: TECH-107
title: "Licensing"
created_date: '2026-11-25T21:03:10Z'
updated_date: '2026-11-27T18:54:32Z'
labels: []
references:
  - NFR-006
  - NFR-006.001
  - NFR-006.002
status: verified
---
# TECH-107 Licensing

## Description
Covers where and how the licence is recorded, the acceptable licence families, vendored asset provenance, packaged notices, and the per-release licence review list. The licensing policy itself is specified by NFR-006.

## Licence records

- The repository licence declaration, the `license` field in `Cargo.toml`, and the distributed copyright notice all state GPL-3.0-or-later.
- The three records are checked together; a disagreement fails the release check.

## Acceptable licences

Non-exhaustive examples of licences accepted for crates and bundled assets:

- MIT, ISC, BSD-2-Clause, BSD-3-Clause, Zlib, Unlicense, CC0-1.0
- Apache-2.0
- MPL-2.0
- LGPL-2.1-only, LGPL-2.1-or-later, LGPL-3.0-only, LGPL-3.0-or-later

The list is illustrative, not a whitelist. The audit decides compatibility for any licence not listed; an incompatible licence blocks shipment.

## Vendored asset provenance

- Assets live under `src/browser/vendor/<package>/<version>/` and are byte-identical to upstream unless a documented `.patch` accompanies them.
- Each version directory carries the upstream files, `LICENSE`, and `UPSTREAM.md` with package, version, upstream URL, and retrieval date.
- `src/browser/vendor/MANIFEST.md` records, per asset, the package, version, licence, upstream URL, byte size, and SHA-256.
- A digest test recomputes every embedded asset's SHA-256 and fails on a mismatch. Updating an asset updates its version directory, manifest row, notices, and tests.
- No automatic updater, lockfile fetch, or network access is part of the build.

## Packaged notices

- `packaging/deb/usr/share/doc/papyrus/copyright` carries the Papyrus statement and one section per compiled crate and per bundled asset: name, version, licence identifier, and the full licence text or a Debian-style reference to the on-disk text.
- The package keeps one documentation file; no separate third-party notice file is shipped.

## Release review list

- Every release is accompanied by a generated list of every compiled crate and every bundled asset, each with name, version, licence identifier, and origin.
- The list is derived from Cargo metadata and the vendor manifest at build time and is stored with the release, not embedded in the binary.
- Review checks each entry against the licence policy and the notice file, and records the outcome with the release.
- A missing entry, a version mismatch, or an incompatible licence blocks the release.
