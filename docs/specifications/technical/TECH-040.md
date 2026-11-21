---
id: TECH-040
title: "Shared Record Envelope and Front-Matter Patcher"
created_date: '2026-11-16T17:46:48Z'
updated_date: '2026-11-21T08:11:33Z'
labels: []
references:
  - ADR-0004
  - ADR-0005
  - ADR-0006
  - ADR-0013
  - ADR-0023
  - ADR-0036
  - ADR-0039
  - NFR-04.001
  - NFR-04.002
  - NFR-04.004
  - REQ-05.010
  - REQ-05.011
  - REQ-06.004
  - REQ-07.004
status: verified
---
# TECH-040 Shared Record Envelope and Front-Matter Patcher

## Description

One loadable envelope, one model-neutral field map, one patch API, and one atomic
writer serve every managed Markdown record. No module formats a front-matter
block, renders a record body, or writes a record file on its own.

## Purpose

A record has to be readable and writable without any module knowing a document
type's fields. The envelope carries the general record data, the field map
carries the type-declared content, and every change is expressed as a list of
patch operations that the single writer applies. Type knowledge therefore lives
in the descriptor and its field sequence, not in per-type code.

## Scope

In scope:

- `record::frontmatter` - detection, parsing, patching, and serialization of the
  leading YAML block, with typed accessors for the general keys.
- `record::document` - the lossless envelope: optional front matter, optional
  heading line, preamble lines, and ordered level-2 sections with raw bodies.
- `record::patch` - the only mutation API, a list of `PatchOp` values.
- `record::store` - record location, containment, read, atomic write, and the
  optimistic-concurrency version token.
- The model-neutral field map and its PatchOp mapping.
- Descriptors for every managed Markdown record type; HTML mockups are excluded
  from front-matter handling and keep their metadata in the HTML comment.

Out of scope:

- Lazy adoption of an existing record that lacks front matter, and the
  descriptor source used for it.
- The option surface generated from a field sequence.
- Record layout, ID shapes, status vocabularies, archive and delete behaviour.
- Browser rendering and search.

## Behavior

### 1. Module ownership

1. All record parsing, metadata formatting, and record writing live under
   `src/record/`. No other module may hand-format a `---` block, re-serialize an
   untouched block, or split front matter with ad-hoc string scanning.
2. `record::frontmatter`, `record::document`, `record::patch`, and
   `record::store` are the only entry points.
3. Descriptors carry Papyrus IDs only. No second identity, no foreign-ID field,
   and no translation at a storage boundary.

### 2. Record envelope

1. A loaded Markdown record exposes: `id`, `title`, `status`, `labels`,
   `references`, `created_date`, `updated_date`, the body units, the version
   token, the file path, and one field map.
2. `labels`, `references`, `created_date`, and `updated_date` are general record
   metadata, never fields. `title` is fixed and is not a field. `status` is
   reserved and read-only outside the verbs that own it.
3. An undeclared front-matter key found on disk is preserved as stored and is
   surfaced read-only as extra metadata. It is never repaired, merged, renamed,
   or removed.
4. A body and front-matter value that already disagree on disk are left as
   stored. A read displays the body value; a write sets both locations from the
   submitted value.

### 3. Field map

1. The field map is built once per loaded record from the descriptor and the
   record's role. It is the only content projection: no typed content field and
   no second field table is consulted.
2. The map carries the role, the resolved fields in descriptor sequence order,
   the fixed metadata block (HTML mockup only), the undeclared stored keys, and
   the resolved reference visibility.
3. A resolved field carries its key, label, help, kind (`text`, `multiline`,
   `list`), required flag, editability, and value. The value is text, a list of
   items, or absent.
4. Role resolution is per model:

   | Model | Field sequence | Role |
   |---|---|---|
   | `SingleMarkdown` | top-level `fields` | single |
   | `GroupedMarkdown` | `group.fields` for a group record, `member.fields` for a member record | group or member |
   | `SingleHtmlMockup` | none; `fields` is empty | single, with the fixed metadata block |

5. Value resolution is deterministic:

   - a field with only `heading` reads the body section with that heading;
   - a field with only `frontmatter_key` reads that front-matter key;
   - a field with both reads the body section as the display value and does not
     merge the two locations.

6. An absent optional value is absent. An absent required value is absent and is
   flagged on an edit surface; the record is not repaired.

### 4. `record::frontmatter`

1. Detection: a leading YAML block is recognized only when the first line is
   exactly `---` and a later line is exactly `---`. Nothing may precede the
   opening delimiter.
2. Parsing preserves key order, raw scalar text, and unknown keys.
3. Typed accessors cover the general keys and the keys named by declared fields.
4. Serialization emits the canonical order: the general keys first, then
   `status` for a model that stores status in front matter, then the
   declared-field keys in field sequence order, then any preserved undeclared
   keys in their stored order.
5. Patching rewrites only the changed keys and preserves unknown keys, key
   order, comments, and quoting. A full re-serialization that reorders or
   requotes foreign keys is prohibited.
6. Timestamps are quoted ISO-8601 UTC strings of the form
   `'YYYY-MM-DDTHH:MM:SSZ'`.
7. `assignee` is a YAML list on task records.

### 5. `record::document`

1. The envelope splits a file into an optional front-matter block, an optional
   heading line, preamble lines, and ordered `## ` sections that each retain
   their heading and raw body lines.
2. Section text and unknown content are kept as raw lines and are never
   re-rendered.
3. Every unmodified line is preserved byte-for-byte, including its line endings.
4. Task and milestone records have no `# <ID> <title>` heading. Every other
   Markdown record type keeps one, generated from the front-matter `id` and
   `title` so the two cannot diverge.
5. Frozen decision bodies keep their existing heading and sections; the envelope
   never alters them.

### 6. `record::patch`

1. `PatchOp` supports at least: `SetFrontMatter`, `SetFrontMatterValue`,
   `ClearFrontMatter`, `SetSection`, `ClearSection`, `SetHeadingTitle`, the
   heading/ID operations, and the model-specific HTML-metadata operations.
2. Applying a patch returns the new file text.
3. An empty patch, and a patch whose values equal the stored values, return
   byte-identical text.
4. A patch that changes no byte causes no write, so `updated_date` is not
   refreshed by a no-op.
5. `record::patch` is the only mutation API.

### 7. Field-to-PatchOp mapping

The engine builds operations from the field sequence and the submitted values.
The mapping is fixed by the field's storage shape and kind:

| Field shape | Kind | Create/edit operations |
|---|---|---|
| `heading` only | `text`, `multiline` | `SetSection(heading, value)`; an optional cleared value uses `ClearSection(heading)` |
| `frontmatter_key` only | `text`, `multiline` | `SetFrontMatter(key, value)`; an optional cleared value uses `ClearFrontMatter(key)` |
| `frontmatter_key` only | `list` | `SetFrontMatterValue(key, sequence)` with one scalar item per submitted value, in submission order |
| `heading` and `frontmatter_key` | any | both operations above, built from the same submitted value |

1. A field whose kind is `list` and whose only storage is a heading writes one
   list item per line under its section.
2. A field with both locations writes both; neither location is left stale.
3. A required field missing on create produces one validation outcome listing
   every missing key, and no patch is built.
4. A required field cleared on edit produces one validation outcome and no
   patch. An optional field may be cleared by the operation for its shape.
5. Fixed operations outside the field sequence:

   | Value | Operations |
   |---|---|
   | Title | `SetHeadingTitle(title)` plus the front-matter `title` |
   | `status` | `SetFrontMatter` from the status and archive verbs only; never from a field option |
   | `group` link | `SetFrontMatter("group", group-id)` at member create only; never on edit |
   | HTML mockup metadata | HTML-metadata operations for the fixed keys; no YAML key is written |
   | Supersession | HTML-metadata operations that set the state and the back-reference |

6. A field whose verbs exclude a verb contributes no operation for that verb.

### 8. Only patch operations write a record

1. The engine never writes a record body. It does not format a `---` block,
   re-render a body, call `fs::write`, or hold a per-type writer.
2. A create and an edit each produce exactly one patch, applied by
   `record::store::write_atomic`.
3. A patch that changes nothing performs no write and exits successfully.
4. A validation failure writes nothing and leaves the file byte-identical.
5. A write preserves every key, section, comment, and line ending that the
   operation does not change.
6. `updated_date` is refreshed once per write that changes at least one byte,
   including the archive and delete writes, and is never touched by a read or a
   no-op patch.

### 9. `record::store`

1. Resolves every path through the project-containment check before a read or a
   write.
2. Returns the document envelope and the version token on read.
3. Writes atomically: create a temporary file in the same directory, write,
   `sync_all`, set permissions, rename over the target, and clean up on failure.
4. Derives the version token from file metadata as `seconds.nanos.length`.
5. Accepts both a record without front matter and a record with front matter;
   front matter is used first, otherwise the reader falls back to section and
   heading parsing.
6. Reading does not modify the file. Adoption of a record that lacks front
   matter is a separate, idempotent path.

### 10. Error behavior

1. An unreadable file, or a path that escapes the project directory, is a
   runtime error (exit code 1).
2. Malformed or unparseable front matter is a validation error (exit code 4),
   never a panic.
3. A missing record is a not-found error (exit code 3).
4. An atomic-write failure leaves the original file byte-identical and removes
   any temporary file.
5. No partial write is reported as success.

## Inputs and outputs

- Input: a project context, a record path or a descriptor-claimed location, and
  a list of patch operations.
- Output: the envelope, the field map, and the version token on read; new file
  text on patch; success or a typed error on write.

## Assumptions

- Assumption: `yaml_serde` stays the only YAML dependency; no new crate is
  added.
- Assumption: the existing project-containment check is reused for path safety.
- Assumption: `record::store` may wrap or reimplement the existing atomic-write
  sequence without changing its contract.
- Assumption: HTML mockups remain HTML-only and are not claimed by a
  front-matter descriptor.

## Open questions

- Whether a descriptor is built per load or resolved once per command is not
  fixed here, provided the byte-identity and preservation contracts hold.
