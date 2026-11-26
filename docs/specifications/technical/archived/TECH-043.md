---
id: TECH-043
title: "Migrate Document Record Types to Front Matter Lazily on First Read and First Write"
created_date: '2026-11-16T17:46:48Z'
updated_date: '2026-11-21T08:01:22Z'
labels: []
references:
  - ADR-0005
  - ADR-0006
  - ADR-0013
  - ADR-0016
  - ADR-0023
  - ADR-0036
  - ADR-0037
  - NFR-004.001
  - NFR-004.002
  - NFR-004.004
  - REQ-005.007
  - REQ-005.010
  - REQ-005.011
  - REQ-006.004
  - REQ-007.002
status: archived
---
# TECH-043 Migrate Document Record Types to Front Matter Lazily on First Read and First Write

## Description

A managed record that lacks a front-matter block is adopted per record, on its
first read and on its first write. Adoption resolves the descriptor from the
descriptor set already in hand, never by building a registry per record, and one
directory scan serves every record resolved in the same pass.

## Behavior

### 1. Managed record set and descriptor claim

1. A file is an adoption candidate only when a record descriptor claims it.
2. A file is claimed when its path is under the descriptor's active directory
   and its file stem matches the descriptor's prefix and ID shape.
3. Claimed Markdown families are every `SingleMarkdown` and `GroupedMarkdown`
   document type, plus the bespoke backlog and decision records.
4. The HTML mockup model carries no front matter and is never a candidate.
5. Prose Markdown that no descriptor claims is never read for adoption and never
   modified.
6. An inactive legacy folder left by an earlier version is not read, scanned,
   migrated, or inventoried.

### 2. Descriptor resolution

1. Adoption resolves descriptors from the descriptor set already built for the
   current operation: the serving snapshot in the browser, and the registry
   built once per process for a command invocation. It does not construct a
   registry, a descriptor, or a configuration per record.
2. The snapshot a request holds is immutable and is used for the whole request;
   a record's adoption therefore uses the same descriptor generation as the rest
   of that request's page.
3. A descriptor is only used to decide claim, ID shape, directory, and the keys
   to seed. Record bytes always come from the live file read, and a write always
   computes its version token from that live read.
4. A disabled type contributes no candidate: its directory is not scanned and
   its records are not adopted.

### 3. One-scan guarantee

1. One traversal lists each claimed directory once. The listing is reused for
   every candidate file in that directory for the duration of the pass, so the
   cost of resolving N records is one listing per directory plus one read per
   changed record, not N listings.
2. The listing is pass-local. It is not held across requests, not cached in
   process state, and not persisted.
3. A record whose path is not in the listing is not opened. A record that
   already carries front matter is detected from the bytes already read and is
   not read again for the same decision.
4. Duplicate detection and conflict reporting use the same listing, so a
   duplicate is reported without a second directory walk.
5. No corpus-wide recursive walk runs on a read path: only the claimed
   directories of the descriptors in hand are listed.

### 4. Trigger: first read and first write

1. The reader checks whether the file begins with a front-matter block. If it
   does, nothing is migrated and the record is returned byte-identical.
2. If the block is absent and the record is claimed, the reader builds an
   adoption patch, applies it, and writes the result atomically before returning
   the record.
3. The writer performs the same check before a normal write: an adoption patch
   is applied first, then the requested operations, in one atomic write.
4. After either path the record carries front matter and later reads and writes
   are no-ops.

### 5. Adoption patch contents

1. The patch sets the general keys in canonical order:

   - `id` - from the existing heading, otherwise from the file stem;
   - `title` - from the existing heading, otherwise empty;
   - `created_date` - from the provenance rule below;
   - `updated_date` - the adoption time;
   - `labels` and `references` - empty lists when the body carries none;
   - `status` - the descriptor's seeded value when the model stores status in
     front matter.

2. The patch sets the keys of declared fields when their values already exist in
   the body:

   - a field with both `heading` and `frontmatter_key` has its section value
     copied to the key and the section is cleared, so no value is stored twice;
   - a field with only `heading` is left in the body and seeded with no key;
   - a field with only `frontmatter_key` is seeded empty when absent;
   - a list-valued field is seeded as a YAML sequence of the values found.

3. The engine-owned group link is seeded from the member's existing group
   reference, and only for a member record.
4. A retired metadata section whose value has moved to front matter is cleared;
   no other body section is added, removed, or rewritten.
5. A frozen decision body keeps its heading and all sections; only the
   front-matter block is inserted, and decision status and type stay in the
   document core.
6. Heading, preamble, comments, and line endings are preserved byte-for-byte.

### 6. `created_date` provenance

1. For a decision, parse its date section. When it carries a date, convert it to
   a quoted ISO-8601 UTC timestamp, using midnight UTC when only a date is
   given. Otherwise fall back to the file modification time.
2. For every other record, use the file modification time.
3. `updated_date` is the adoption time.
4. Provenance is best-effort: a seeded `created_date` approximates the real
   creation date and is never recomputed later.

### 7. Timestamp format

1. Both timestamps are quoted ISO-8601 UTC strings at second precision:
   `'YYYY-MM-DDTHH:MM:SSZ'`.
2. A small in-project UTC formatter is used; no time-formatting dependency is
   added.

### 8. Idempotency

1. A record that already begins with a front-matter block is a no-op on read and
   on write: no patch, no write, no timestamp refresh, no report entry.
2. Re-running adoption on an adopted record produces byte-identical content.
3. The file is the source of truth: adoption never runs twice for one record
   within a process once the bytes show front matter.

### 9. Per-file report

1. One report entry is emitted per file actually changed: relative path, action,
   and the resulting key list.
2. Action values are `added-front-matter` and, when a retired metadata section
   is cleared as well, `added-front-matter;cleared-retired-section`.
3. A live run prints one line per changed file on standard output. An unchanged
   file prints nothing.
4. A dry run lists each candidate with `would-migrate` or `already-migrated` and
   writes nothing.
5. The report is the convergence artifact: a dry run with no `would-migrate`
   entry means the claimed corpus is adopted.

### 10. Failure handling and concurrency

1. If adoption cannot complete its atomic write (read-only media, missing
   permission, I/O error), the original file stays byte-identical, any temporary
   file is removed, and the error is returned. The record is neither reported
   nor returned as adopted.
2. Adoption never partially writes: atomicity follows the shared store
   sequence.
3. Two processes adopting the same record serialize on the atomic write; the
   second observes front matter and becomes a no-op.
4. A reader that loaded pre-adoption bytes is not silently upgraded.

### 11. Validation and error behavior

1. A path that escapes the project directory is a runtime error (exit code 1).
2. Malformed front matter is a validation error (exit code 4), never a panic.
3. A missing record is a not-found error (exit code 3).
4. A file that no descriptor claims is never adopted, even when it is Markdown.
5. An adoption failure on a read path is reported as an error, not a warning.

## Inputs and outputs

- Input: a project context, the descriptor set in hand, and a record path or a
  claimed directory.
- Output: an adopted envelope and version token on read; new file text on
  patch; per-file report lines for changed files; success or a typed error.

## Assumptions

- Assumption: adoption inserts front matter and, when a declared field already
  had a body value and a front-matter key, moves that value and clears its
  section. No other body change is made.
- Assumption: `labels` and `references` default to empty lists; no value is
  inferred from prose.
- Assumption: `id` and `title` come from the existing heading when present;
  otherwise `id` comes from the file stem and `title` is empty.
- Assumption: a dry run is reachable from tests and internal callers; no
  user-facing migration subcommand is added.
- Assumption: the corpus converges lazily; no bulk migration is run by the tool.

## Open questions

- Whether the optional `superseded_by` value of a decision should be inferred
  from its status section during adoption, or left unset until a command writes
  it.
- Whether `labels` and `references` should be backfilled from body sections;
  adoption currently seeds empty values.
