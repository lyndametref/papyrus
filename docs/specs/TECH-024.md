---
id: TECH-024
title: "Man-Page Generation and Installation"
created_date: '2026-11-27T20:35:42Z'
updated_date: '2026-11-28T08:09:29Z'
labels: []
references:
  - REQ-008.001
  - REQ-008.002
  - NFR-001.003
  - NFR-003.003
status: verified
---
# TECH-024 Man-Page Generation and Installation

## Description
Covers man-page generation and installation: the page set, the Markdown sources, the build-time converter, compression, Debian staging, and the build and packaging checks. Page content, the `<slug>` documentation, per-command `--help`, and the configuration schema are separate specifications.

## Page set

- The `papyrus` package ships `papyrus(1)` and `papyrus-config(5)`; the `papyrus-browser` package ships `papyrus-browser(1)`. The `papyrus-core` package ships no manual page.
- No per-type page is shipped: document types and decision types are declared in configuration and cannot be enumerated when the package is built.
- No page is generated per configured type at install or run time.

## Sources

- Man-page sources are Markdown files under `docs/man/`: `docs/man/papyrus.1.md`, `docs/man/papyrus-config.5.md`, and `docs/man/papyrus-browser.1.md`.
- `docs/man/` is the only man-page source. No other file is converted into a packaged page.
- `README.md` carries a pointer to the installed pages and does not repeat their text as a second maintained copy.

## Converter

`apps/scripts/md2man.py` is an in-repository converter that uses the Python 3 standard library only. It runs at build time, requires no third-party module, and makes no network access. Python is not a runtime dependency of the installed package.

The converter supports exactly these sections:

`NAME`, `SYNOPSIS`, `DESCRIPTION`, `COMMANDS`, `OPTIONS`, `EXIT STATUS`, `FILES`, `ENVIRONMENT`, `EXAMPLES`, `SEE ALSO`.

It emits roff using `.TH`, `.SH`, `.SS`, `.TP`, `.PP`, `.IP`, `.B`, and `.I`.

## Build output and compression

- Roff output is written under `apps/target/man/`: `apps/target/man/man1/papyrus.1`, `apps/target/man/man5/papyrus-config.5`, and `apps/target/man/man1/papyrus-browser.1`.
- Each page is compressed with `gzip -9n`; the `-n` flag omits the original file name and the modification timestamp, so repeated builds from the same sources produce identical bytes.
- The build produces no page outside `target/man/man1` and `target/man/man5`, and the build output is not committed.

## Debian staging

- `apps/scripts/build-deb.sh` stages the pages into the appropriate package tree: `papyrus(1)` and `papyrus-config(5)` into the `papyrus` package, and `papyrus-browser(1)` into the `papyrus-browser` package.
- `packaging/deb/` mirrors the same `usr/share/man` layout.
- No man page is installed outside the standard man paths, and no runtime asset directory is added for manual content.

## Verification

- A build-time `groff -man -Tutf8` pass renders every generated page; any warning fails the build.
- A packaging test unpacks the built `.deb` files and asserts that every committed page is installed at its standard path in the owning package: `usr/share/man/man1/papyrus.1.gz` and `usr/share/man/man5/papyrus-config.5.gz` in the `papyrus` package, and `usr/share/man/man1/papyrus-browser.1.gz` in the `papyrus-browser` package.
- Verification confirms that every built-in command appears in `papyrus(1)` and that configured-type commands are documented once against the `<slug>` placeholder.

## Build and runtime constraints

- The whole generation path is build time only and works offline.
- It adds no runtime dependency, no runtime file lookup, and no network access.
- The installed binary reads no man-page source at run time.
