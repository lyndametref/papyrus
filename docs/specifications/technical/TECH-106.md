---
id: TECH-106
title: "Distribution Artifact"
created_date: '2026-11-25T21:03:10Z'
updated_date: '2026-11-27T18:50:24Z'
labels: []
references:
  - NFR-003
  - NFR-003.002
status: verified
---
# TECH-106 Distribution Artifact

## Description
Covers the three `.deb` artifacts, their metadata, payloads, versioning, and supported Debian target. Build invocation, man-page generation, and licensing are separate specifications.

## Artifact

- Format: three Debian binary packages (`.deb`), assembled with `dpkg-deb` in one packaging run, all at one version.
- Architecture: `amd64` only.
- File names:
  - `papyrus-core_<version>_amd64.deb` — library artifact for reuse; installs no executable and declares no dependency on another Papyrus package.
  - `papyrus_<version>_amd64.deb` — installs `/usr/bin/papyrus` and declares `Depends: papyrus-core (= <version>)`.
  - `papyrus-browser_<version>_amd64.deb` — installs `/usr/bin/papyrus-browser` and declares `Depends: papyrus-core (= <version>)`.
- No front-end package depends on the other front end: `papyrus` does not depend on `papyrus-browser` and vice versa. Neither executable invokes the other. Each binary statically links `papyrus-core`; the package dependency records the shared library version even though no file from the core package is loaded at runtime.
- Supported target: Debian Forky only. Debian Trixie and every other release are not claimed or verified.
- The binaries are built and smoke-tested on Forky; glibc and toolchain compatibility are verified there.

## Metadata

- `DEBIAN/control` carries the package metadata; `Architecture` is `amd64` and `Version` is the artifact version.
- The default version is derived from the crate manifests under `apps/`; an explicitly supplied build version overrides it and appears in both the `Version:` field and the file name.
- `papyrus` and `papyrus-browser` declare `Depends: papyrus-core (= <version>)`; `papyrus-core` declares no dependency on another Papyrus package. `stty` comes from Essential `coreutils`, so no additional `Depends` entry is required. No package starts a service, daemon, or network operation.
- Installation and removal are non-interactive.

## Payload

- `papyrus-core` package: no executable; carries the library artifact for reuse and its copyright file.
- `papyrus` package: `/usr/bin/papyrus`; its copyright file; manual pages under `/usr/share/man` (`papyrus(1)`, `papyrus-config(5)`).
- `papyrus-browser` package: `/usr/bin/papyrus-browser`; its copyright file; manual page `papyrus-browser(1)`.
- No runtime asset directory, database, index, cache, or sidecar is shipped by any package; browser assets are embedded in the binary.
- The staging trees under `packaging/deb/` mirror the installed layout per package.

## Asset Version Binding

- Vendored browser assets carry their own version directories and manifest rows.
- The package build fails when a vendor version directory, its manifest version, and its copyright section disagree.
- The package version is not the asset version and appears in no asset URL.

## Verification

- Install all three `.deb` files on Forky and smoke-test both installed binaries and their commands.
- Assert that each package's payload files and manual pages are installed at the standard paths.
- Record the artifact paths, sizes, and SHA-256 values of all three packages with the release.
