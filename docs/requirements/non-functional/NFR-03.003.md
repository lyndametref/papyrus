# NFR-03.003 Provide Command Man Pages in the Debian Package

## Description
Papyrus shall provide a man page for every command, and these man pages shall be included in the Debian package. Man page sources shall be Markdown files under `docs/man/`, generated to roff at build time, and installed to the standard man paths in the `.deb`.

## Rationale
This records the requested packaging and documentation requirement that the man page is not optional or deferred, and that each command is documented as a man page. The man pages also serve as development input and are intended to be reusable with minimal improvements.

## Verification Method
Inspect the Debian package contents to confirm that a man page is present for every documented Papyrus command, that roff man pages are installed under standard man paths and generated from Markdown sources under `docs/man/`, and verify that each command's documented behavior matches its man page.
## Traceability
- README.md, `papyrus(1)` and `papyrus-config(5)` — man page content
- `docs/specifications/rust-cli-and-debian-package-baseline.md` (TASK-1; currently assumes man page optional/deferred — to be revised)
- `packaging/deb` — Debian package build