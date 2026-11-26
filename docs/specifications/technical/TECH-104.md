---
id: TECH-104
title: "Man-Page Generation and Installation"
created_date: '2026-11-25T20:53:43Z'
updated_date: '2026-11-25T20:53:43Z'
labels: []
references:
  - REQ-008.001
  - REQ-008.002
  - NFR-001.003
  - NFR-003.003
status: draft
---
# TECH-104 Man-Page Generation and Installation

## Description
Covers the packaged manual: the page set, the Markdown sources, the build-time
conversion to roff, compression, Debian staging, and the build and packaging
checks. Covers the single `<slug>` documentation of config-defined document and
decision types. Does not cover per-command `--help` rendering, the configuration
schema itself, or any runtime man-page generation.

## Page set

The package ships exactly two manual pages:

- `papyrus(1)` — name and synopsis, description, global options, environment
  variables, the files Papyrus reads and writes, exit statuses, and every
  built-in command with its arguments and options.
- `papyrus-config(5)` — the configuration file location, the precedence between
  command-line options, environment variables, and configuration values, every
  configuration key with its default and accepted values, and the model-level
  defaults that apply to a document type.

Record-type commands are not documented per type. Document types and decision
types are declared in configuration and cannot be enumerated when the package is
built, so the package contains no page for an individual configured type, no
page per default slug, and no page generated at install or run time.

## `<slug>` family section

`papyrus(1)` states the command family of recorded types once, in a single
section written against the `<slug>` placeholder, covering both configured
families: document types and decision types. The section:

- gives the command forms with the placeholder, for example
  `papyrus <slug> create`;
- gives the model-derived verbs, the field options in field-sequence order, and
  the title contract for a document type, and the verbs of a decision type;
- states that the slug is both the configuration key and the command name;
- names the configuration map that declares the type, `document_types` for
  document types and `decision_types` for decision types;
- names the entry keys that apply to a configured type;
- states that the type's own `--help` lists the options generated from its
  configuration entry.

The options of a specific configured type are not enumerated in the manual. The
type's `--help` output is the only surface that enumerates them.

## Sources

- Man-page sources are Markdown files under `docs/man/`:
  `docs/man/papyrus.1.md` and `docs/man/papyrus-config.5.md`.
- `docs/man/` is the only man-page source. No other file is converted into a
  packaged page.
- A command added to the CLI is documented in the manual before the release that
  ships it.
- `README.md` carries a pointer to the installed pages and does not repeat their
  text as a second maintained copy.

## Converter

`scripts/md2man.py` is an in-repository converter that uses the Python 3 standard
library only. It runs at build time, requires no third-party module, and makes no
network access. Python is not a runtime dependency of the installed package.

The converter supports exactly these sections:

`NAME`, `SYNOPSIS`, `DESCRIPTION`, `COMMANDS`, `OPTIONS`, `EXIT STATUS`,
`FILES`, `ENVIRONMENT`, `EXAMPLES`, `SEE ALSO`.

It emits roff using `.TH`, `.SH`, `.SS`, `.TP`, `.PP`, `.IP`, `.B`, and `.I`.

## Build output and compression

- Roff output is written to `target/man/man1/papyrus.1` and
  `target/man/man5/papyrus-config.5`.
- Each page is compressed with `gzip -9n`; the `-n` flag omits the original file
  name and the modification timestamp, so repeated builds from the same sources
  produce identical bytes.
- The build produces no page outside `target/man/man1` and `target/man/man5`, and
  the build output is not committed.

## Debian staging

- `scripts/build-deb.sh` stages `usr/share/man/man1/*.1.gz` and
  `usr/share/man/man5/*.5.gz` in the package tree.
- `packaging/deb/` mirrors the same `usr/share/man` layout.
- No man page is installed outside the standard man paths, and no runtime asset
  directory is added for manual content.

## Verification

- A build-time `groff -man -Tutf8` pass renders every generated page; any warning
  fails the build.
- A packaging test unpacks the built `.deb` and asserts that every committed page
  is installed at its standard path: `usr/share/man/man1/papyrus.1.gz` and
  `usr/share/man/man5/papyrus-config.5.gz`.
- Verification confirms that every built-in command appears in `papyrus(1)` and
  that the document-type and decision-type sections use the `<slug>` placeholder
  together with the configuration map, the entry keys, and the generated
  `--help`.

## Build and runtime constraints

The whole generation path is build time only and works offline. It adds no
runtime dependency, no runtime file lookup, and no network access; the installed
binary reads no man-page source at run time.
