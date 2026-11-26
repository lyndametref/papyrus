---
id: TECH-106
title: "Distribution Artifact"
created_date: '2026-11-25T21:03:10Z'
updated_date: '2026-11-25T21:07:36Z'
labels: []
references:
  - NFR-003
  - NFR-003.002
status: draft
---
# TECH-106 Distribution Artifact

## Description
Covers the `.deb` artifact, its metadata, payload, versioning, and supported Debian target. Build invocation, man-page generation, and licensing are separate specifications.

## Artifact

- Format: Debian binary package (`.deb`), assembled with `dpkg-deb`.
- Architecture: `amd64` only.
- File name: `papyrus_<version>_amd64.deb`.
- Supported target: Debian Forky only. Debian Trixie and every other release are not claimed or verified.
- The binary is built and smoke-tested on Forky; glibc and toolchain compatibility are verified there.

## Metadata

- `DEBIAN/control` carries the package metadata; `Architecture` is `amd64` and `Version` is the artifact version.
- The default version is derived from `Cargo.toml`; an explicitly supplied build version overrides it and appears in both the `Version:` field and the file name.
- An RC and a final package of the same base version are distinguishable by the version string.
- The package adds no runtime dependency beyond the base system. `stty` comes from Essential `coreutils`, so no new `Depends` entry is required.
- Installation and removal are non-interactive; the package starts no service, daemon, or network operation.

## Payload

- `/usr/bin/papyrus` — the executable.
- `/usr/share/doc/papyrus/copyright` — Papyrus and third-party licence notices.
- Manual pages under `/usr/share/man`; their generation and content are a separate specification.
- No runtime asset directory, database, index, cache, or sidecar is shipped.
- The staging tree `packaging/deb/` mirrors the installed layout: `DEBIAN/`, `usr/bin/`, `usr/share/doc/`, and `usr/share/man/`.

## Asset Version Binding

- Vendored browser assets carry their own version directories and manifest rows.
- The package build fails when a vendor version directory, its manifest version, and its copyright section disagree.
- The package version is not the asset version and appears in no asset URL.

## Verification

- Install the `.deb` on Forky and smoke-test the installed commands.
- Assert that the payload files and manual pages are installed at the standard paths.
- Record the artifact path, size, and SHA-256 with the release.
