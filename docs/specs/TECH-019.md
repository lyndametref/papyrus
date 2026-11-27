---
id: TECH-019
title: "User Documentation and Help"
created_date: '2026-11-27T20:35:40Z'
updated_date: '2026-11-27T20:35:40Z'
labels: []
references:
  - REQ-008
  - REQ-008.001
  - REQ-008.002
  - REQ-008.003
  - NFR-003.003
status: verified
---
# TECH-019 User Documentation and Help

## Description
Covers the user documentation surfaces: the packaged manual pages and their content, the single packaged source the pages are generated from, per-command help, the `help` command, and man-page installation in the Debian package. Configuration keys and record management are outside this specification.

## Manual pages

- Papyrus ships the manual pages `papyrus(1)` and `papyrus-config(5)`.
- The `papyrus` package installs `papyrus(1)` and `papyrus-config(5)` to the standard manual path; the `papyrus-browser` package installs `papyrus-browser(1)`. `man papyrus`, `man papyrus-config`, and `man papyrus-browser` display them after installation.
- The pages need no network access.
- A command added to the CLI is documented in the manual before the release that ships it.

## Content of `papyrus(1)`

- The name and synopsis, the description, the global options, the environment variables, the files Papyrus reads and writes, the exit statuses, and every built-in command with its arguments and options.
- One section, written against the `<slug>` placeholder, states the command family that applies to every configured document type and every configured decision type: the model-derived verbs and field options and the title contract for documents, and the fixed verbs and options of a decision type.
- That section explains how a slug is configured: which configuration map declares it (`document_types` or `decision_types`), which entry keys apply, and that the slug is both the configuration key and the command name.
- It states that a type's own `--help` lists the options generated from its configuration entry.
- A specific type's options are not enumerated in the manual; they appear only in that type's command help.
- No per-type manual page is shipped, because types are declared in the configuration and cannot be enumerated when the package is built.

## Content of `papyrus-config(5)`

- The configuration file location.
- The precedence between command-line options, environment variables, and configuration values.
- Every configuration key with its default and accepted values.
- The model-level defaults that apply to a document type.

## Manual source and generation

- The packaged source is the single source of the manual text.
- Man page sources are Markdown files under `docs/man/`; the build generates roff from them and installs the roff pages to the standard man paths in the `.deb`.
- The same source is compiled into the `papyrus` binary at build time, so `papyrus help` prints it without reading a file at runtime.
- The installed pages and the `papyrus help` output therefore carry the same text.

## Per-command help

- Every built-in command and every subcommand accepts `-h` and `--help`.
- `papyrus --help` prints the top-level help, which lists the built-in commands and every configured record-type command, for document types and decision types alike.
- A command's help prints its name, description, positional arguments, options, and default values.
- A document-type command's help shows the verbs derived from its model, the options generated from its field sequence in sequence order, and the title contract; this is the only surface that enumerates a specific type's options.
- A decision-type command's help shows its fixed verbs and fixed options; it has no field-derived options.
- Help output may use ANSI styling; `--no-color` and `NO_COLOR` disable it and the content stays readable.
- Help output is produced without reading records and without writing anything, and the command exits successfully.
- Help output for one configuration is identical across runs, machines, working directories, terminal widths, and locales, and contains no absolute path, host name, timestamp, process identifier, or memory address.

## `help` command

- `papyrus help` prints the Papyrus manual, with the same content as `man papyrus`.
- `papyrus help <COMMAND>` prints that command's help, with the same content as `papyrus <COMMAND> --help`.
- `papyrus help <COMMAND>` for a command that does not exist is a usage error and names the unknown command.
- The output is human-readable text and may use ANSI styling; the global `--no-color` option and the `NO_COLOR` environment variable disable the styling, and the content stays readable.
- Both forms read no record, write nothing, and exit successfully when they print help.
