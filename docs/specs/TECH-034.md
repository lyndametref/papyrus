---
id: TECH-034
title: "Input Sanitization"
created_date: '2026-11-27T20:35:45Z'
updated_date: '2026-11-27T20:40:02Z'
labels: []
references:
  - NFR-002
  - NFR-002.002
status: verified
---
# TECH-034 Input Sanitization

## Description
Covers the single sanitization contract applied to every user-supplied value before it is used, whichever surface supplies it. Sanitization distinguishes free-text values from identifier, name, and path-like values; a character that is invalid in a path is still valid prose. Structural validation of slugs, fields, statuses, and identifiers is separate; the project-directory boundary is also separate.

## Scope

- Every user-supplied value is sanitized before use: command-line options, standard input, browser form values, configuration values, the project name and paths supplied to `init`, and the host and port options.
- Sanitization applies before any structural validation, lookup, path resolution, or write.

## Contract

- A value that cannot be normalized into a valid identifier, name, title, or path is rejected; no invalid value is silently cleaned, truncated, or written.
- Control characters, path separators, and traversal segments are rejected in identifier, name, and path-like values and never reach the store.
- Free-text values (titles, descriptions) are never treated as identifiers, names, or paths; prose characters, including the path separator `/`, colons, quotes, and markup punctuation, are accepted and stored verbatim.
- Normalization of a free-text value (for example Unicode NFC) never fails on prose characters; it is applied only to genuine control characters, which are rejected with a field-specific validation outcome.
- A rejected value produces an error or a validation outcome that names the field and the reason; a bare internal error such as "input cannot be normalized" for a valid free-text value is a defect and writes nothing.

## Surfaces

- Command-line and standard-input values follow the contract; a rejected value writes no record.
- Browser form values take the same contract as command input; a rejected value fails the save and no record content is written.
- Configuration values are rejected with a validation outcome and nothing is written.
- `init` sanitizes the project name and every supplied path; a rejected value fails with a validation outcome and writes nothing.
- The host and the port are sanitized like every other option value.
- Free-text metadata options (`--title`, `--description`, `--label`) and their browser form counterparts accept the path separator and other prose punctuation; rejecting them there is a defect.
