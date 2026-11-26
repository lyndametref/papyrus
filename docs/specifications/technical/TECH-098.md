---
id: TECH-098
title: "Browser Rendering, Landing Pages, and Console"
created_date: '2026-11-25T20:42:11Z'
updated_date: '2026-11-25T20:42:11Z'
labels: []
references:
  - REQ-006.007
  - REQ-006.011
  - REQ-006.017
status: draft
---
# TECH-098 Browser Rendering, Landing Pages, and Console

## Description
Covers console output of the `browser` command, the reduced row of the glossary
landing page, and the rendering of math formulas and diagrams in record views.
It does not cover list and view layout, the edit form, the Backlog board, or the
internals of math and diagram rendering.

## Console Verbosity

- By default the `browser` command outputs only errors to the console
  (REQ-006.007).
- Errors are output whether or not `--debug` is set (REQ-006.007).
- `--debug` is an option of the `browser` command only: it is not a global
  option and is not accepted by any other command (REQ-006.007).
- With `--debug`, each received request is logged as one line containing the
  method, the path, the response status, and the duration (REQ-006.007).
- Request headers and request bodies are not logged, so record content does not
  reach the console (REQ-006.007).

## Glossary Landing Page

- The glossary landing page is the one document list with a deliberately reduced
  row (REQ-006.011).
- It shows no created date and no updated date (REQ-006.011).
- It shows no reference list (REQ-006.011).
- It presents labels as a tag list, using the label presentation defined for
  glossary terms by the browser UI specification (REQ-006.011).
- The reduction is limited to the glossary landing page: glossary
  single-record views keep the shared layout, and the landing pages of other
  document types are unchanged (REQ-006.011).

## Rendered Fields and Surfaces

- Math formulas and Mermaid-style diagrams embedded in the curated prose fields
  that record views display are rendered (REQ-006.017).
- Rendering applies to those prose fields only and introduces no separate
  record-body view (REQ-006.017).
- Rendering is browser-only: command-line output and every other surface show
  the source text unchanged (REQ-006.017).

## Recognised Input Forms

- Display math is recognised between `$$` delimiters (REQ-006.017).
- Inline math is recognised between single `$` delimiters (REQ-006.017).
- A fenced code block whose info string is `math`, `tex`, or `latex` renders as
  a math formula (REQ-006.017).
- A literal dollar sign is written as `\$` and is not a delimiter
  (REQ-006.017).
- Diagrams are recognised only as fenced code blocks whose info string is
  `mermaid` (REQ-006.017).
- A fenced `plantuml` block is shown as source text with a note that it is not
  rendered (REQ-006.017).
- Any other fence info string or delimiter is left as ordinary Markdown
  (REQ-006.017).
- The input forms and the constraints below are contractual; the rendering
  library itself is outside this specification (REQ-006.017).

## Rendering Constraints

- Rendering works fully offline: all rendering assets are bundled, and no
  network request, external service, or server-side rendering is required
  (REQ-006.017).
- Rendered output conforms to the supported browsers and must not result in
  arbitrary script execution (REQ-006.017).
- Unrenderable input — an unclosed delimiter, an unparseable formula, or an
  invalid diagram — is shown as source text with a note, instead of breaking the
  page (REQ-006.017).
- Any rendering dependency is open source and permits linking in a
  GPLv3-or-later application (REQ-006.017).
