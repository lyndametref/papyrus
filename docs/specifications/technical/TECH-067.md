---
id: TECH-067
title: "Search Config-Defined Document Types"
created_date: '2026-11-16T17:46:53Z'
updated_date: '2026-11-21T08:01:22Z'
labels: []
references:
  - ADR-0013
  - ADR-0017
  - ADR-0022
  - ADR-0023
  - ADR-0024
  - ADR-0036
  - ADR-0037
  - BR-015
  - REQ-005.007
  - REQ-006.001
  - REQ-006.004
  - REQ-006.014
  - REQ-007.002
  - REQ-007.003
  - REQ-007.004
status: verified
---
# TECH-067 Search Config-Defined Document Types

## Description

Search reads the searched document-type set and each type's fields from the
descriptor registry. A type declared in configuration is searched with no code
change, and the fields it contributes come from its declared field sequence.

## Behavior

### 1. Document set

1. The searched document set is exactly the enabled document descriptors of the
   registry, in registry order. No module-local list of document slugs exists.
2. The registry is the instance built from the validated effective
   configuration; search does not build a second type model.
3. A disabled type contributes no candidate and no error. A slug absent from
   configuration does not exist in the registry and contributes nothing.
4. Every model is handled by the same descriptor-driven path, including a
   user-defined type that reuses a seed prefix, directory, or field name.
5. Task, milestone, release, and decision collection is unchanged.

### 2. Declared-field projection

1. The fields a document type contributes are the descriptors' declared fields
   for the record's role. There is no static field table and no `DOC_KEYS`-style
   enumeration in the search module.
2. A field with a `frontmatter_key` contributes that front-matter key; a field
   with a `heading` contributes that body section. A field with both contributes
   both locations, read from the record as stored.
3. A `kind: list` field is a sequence of scalars. Each item is indexed as text,
   and its stored order is kept.
4. A field that declares neither location cannot exist, so no undeclared key is
   projected. An undeclared key already present on disk is still part of the
   persisted record text and stays matchable.
5. Projection reads the stored record; it never re-derives, merges, or repairs a
   value, and it never reformats the file.
6. Projection does not narrow matching: which fields exist affects which keys
   and sections belong to the record, not how a query is evaluated.

### 3. Configuration collections that are not list-valued keys

1. The declared field collections (`fields`, `group.fields`, `member.fields`) are
   sequences of mappings, not list-valued configuration keys. They are read only
   from the descriptor.
2. A comma-separated scalar list cannot express a mapping, so these collections
   are never classified as list-valued keys and are never written through the
   configuration value surface.
3. The list-valued classification stays limited to the declared scalar status
   lists; a status list is a sequence of scalars and is read as one.
4. Search therefore distinguishes a field sequence (structure) from a list-valued
   field value (data) by shape: a sequence of mappings is configuration
   structure, a sequence of scalars is record data.

### 4. Directory, files, and identity

1. A document descriptor's active directory is `<project>/<doc_directory>/<descriptor relative directory>`,
   contained by the project-boundary check. Neither `docs/` nor a prefix is
   hardcoded.
2. Only files directly inside the active directory are candidates. The scan is
   not recursive; the `archived/` subdirectory, a legacy inert folder, and every
   other subdirectory are never scanned.
3. The candidate extension follows the model: the HTML mockup model uses
   `.html`, every other model uses `.md`.
4. A candidate is a record only when its file stem matches the descriptor's
   prefix and ID shape: a grouped type tries the dotted member form and then the
   plain group form, a dotted-ID type accepts the dotted form only, and a
   plain-ID type accepts the plain form only. No slug literal selects a parse
   rule.
5. A missing active directory yields no result for that type and no error. An
   unreadable directory is an I/O error (exit code 1). A non-UTF-8 file is
   skipped.
6. Record identity and title come from the shared projection used by list and
   view: the front-matter `id` and `title` when present, otherwise the heading.
   The HTML mockup title comes from the HTML title element. A record with no
   usable title is displayed with its ID as the title.

### 5. Matching, ordering, and output

1. The searchable text is the whole persisted record text, including front
   matter, so title, labels, references, status, declared fields, and body text
   are all matchable. There is no field-restricted query syntax.
2. Matching is case-insensitive Unicode substring matching with the query
   trimmed; a quoted phrase is one contiguous substring. No tokenization,
   stemming, fuzzy matching, ranking, or regular expression applies.
3. Results are ordered by ID ascending byte-wise, then by documented type value,
   then by path. A limit keeps the first N results after ordering.
4. Text output is one line per result with the ID, the documented type value, and
   the title. Every document result uses the document type value.
5. JSON output is an ordered array of objects carrying the ID, the documented
   type value, the title, and the canonical absolute path. No match produces
   empty standard output, or an empty array in JSON, with exit code 0.
6. The document type filter returns document results from every enabled type,
   including config-defined ones; a backlog or decision type filter never
   returns a document. With no type filter, every store is searched.
7. A config-defined record's type value is the document type value; the slug is
   not a type value and does not appear in output.

### 6. Visibility

1. A record in the active directory whose stored status is `archived` is
   relocated to the type's `archived/` subdirectory with the archive sequence
   before a result could be returned, and is not returned. A successful
   relocation is reported on standard error.
2. If relocation fails, search fails with a validation outcome (exit code 4)
   naming the source record; the source is left intact and no result is emitted.
3. A record carrying a custom status stays in the active directory and is
   searchable; a custom status is not a visibility switch.
4. An HTML mockup whose metadata state is `archived` is never returned. The HTML
   metadata is not part of the front-matter relocation contract, so search does
   not relocate it.
5. Files inside `archived/` are never scanned, so an archived record cannot
   reappear after relocation.
6. Relocation on read is the only write permitted on the search path.

### 7. Validation and error behavior

1. A missing directory yields an empty result set and exit code 0.
2. An unreadable directory, or a symlink that escapes the project directory, is
   a runtime error (exit code 1).
3. A relocation failure is a validation outcome (exit code 4) naming the source
   record, with the source left intact.
4. No match is exit code 0 with empty output.
5. An invalid configuration is a startup validation outcome; search does not
   duplicate configuration validation.

## Inputs and outputs

- Input: a query string, an optional list of documented type filters, an
  optional limit, the project context, and the registry built from the effective
  configuration.
- Output: an ordered result set on standard output or as JSON, with no match
  producing an empty result and exit code 0.

## Assumptions

- Assumption: the type filter vocabulary stays the documented values; a
  per-document-slug filter would require a documented command change and is not
  added here.
- Assumption: relocation on read applies to front-matter status only; an
  archived HTML mockup is excluded from results but is not relocated by search.
- Assumption: matching stays over the whole persisted text, so a field
  projection change cannot hide a match.
- Assumption: a registry build failure is already a startup validation outcome;
  search does not repeat it.

## Open questions

- Whether a documented type filter per document slug should be added; it is a
  documented command change, not a search implementation change.
