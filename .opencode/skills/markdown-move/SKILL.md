---
name: markdown-move
description: 'Move a markdown file to a new location and update all relative links within it to reflect the new path.'
argument-hint: 'Provide the source path and destination path for the markdown file to move. Example: "move docs/old/file.md to docs/new/file.md"'
user-invocable: true
disable-model-invocation: false
---

# Markdown File Move with Relative Link Rewriting

## Outcome

Move a markdown file from `source` to `destination` while updating every relative link (image, anchor, file reference) inside it so that all references remain valid after the move.

## When To Use

- You need to reorganize documentation and move a `.md` file to a different directory.
- You need to preserve the validity of intra-documentation links after a move.
- You are moving a spec, requirement, ADR, or any other markdown file that contains relative links.

Do not use this skill for:

- Moving non-markdown files (code, images, binaries, etc.) — the link-rewriting logic only applies to `.md` files.
- Copying files (use `cp` directly instead).
- Files without relative links — use a plain `mv` command instead.

## Procedure

this needs to be done for each markdown file which needs to be moved.

### 1. Resolve paths

1. Normalize the source and destination to absolute paths.
2. Verify the source file exists and is a markdown file (`.md` extension).
3. Verify the destination directory exists (create it if necessary, with user approval).
4. Verify the destination file does not already exist (or confirm overwrite with user).

### 2. Identify all relative links in the file

Scan the markdown content for the following link patterns:

- **Markdown links**: `[text](uri)` and `[text](uri "title")`
- **Markdown images**: `![alt](uri)` and `![alt](uri "title")`
- **Reference-style links**: `[text][ref]` where `[ref]: uri` is defined elsewhere
- **Auto-linked URLs**: `<uri>` — skip these (they are absolute or protocol-relative)

For each link target `uri`, classify it as:

| Type                           | Example                         | Action         |
| ------------------------------ | ------------------------------- | -------------- |
| External URL                   | `https://example.com`           | Keep unchanged |
| Protocol-relative              | `//example.com/resource`        | Keep unchanged |
| Anchor-only                    | `#section`                      | Keep unchanged |
| Absolute path                  | `/docs/architecture/ADR-001.md` | Keep unchanged |
| Relative path (same directory) | `./other.md` or `other.md`      | Rewrite        |
| Relative path (parent)         | `../dir/file.md`                | Rewrite        |

### 3. Compute new relative paths

For each relative link target `uri` found in the file:

1. **Resolve the target** relative to the source file's directory:
   
   - `resolved = normalize(source_dir / uri)`

2. **Compute the new relative path** from the destination file's directory to the resolved target:
   
   - `new_uri = relative_path(destination_dir, resolved)`

3. If `new_uri` differs from `uri`, record the replacement.

### 4. Apply replacements

1. For each recorded replacement, substitute the old URI with the new URI in the file content.
2. Be precise — only replace the URI portion of a link, not surrounding text.
3. Handle both inline links (`[text](uri)`) and reference definitions (`[ref]: uri`).

### 5. Move the file

1. Write the updated content to the destination path.
2. Remove the source file.
3. Confirm the file was moved.

## URI Parsing Details

### Inline links

Pattern: `[text](uri)` or `[text](uri "title")`

- Extract `uri` (between first `(` and first `)`/` "`)
- Replace only the `uri` portion

### Reference definitions

Pattern: `[ref]: uri` or `[ref]: uri "title"`

- Extract `uri` after `]: `
- Replace only the `uri` portion

### Reference-style links

Pattern: `[text][ref]` — the URI is in the definition, not here. No direct edit needed.

### Images

Pattern: `![alt](uri)` — handled identically to inline links.

### What NOT to match

- URLs in code blocks or inline code (backtick-delimited) — skip these entirely
- URLs in HTML comments (`<!-- ... -->`) — skip these
- URLs in fenced code blocks (triple backtick) — skip these entirely

## Example

### Before move

File: `docs/sheep/feeding.md`

```markdown
# Sheep Feeding

See [breed types](../breeds/overview.md) for more info.

![feeding chart](images/feeding-chart.png)

References:
- [health guide](../health/checkups.md)
- [external resource](https://vet.org/sheep)
```

### After move to `docs/animals/sheep/feeding.md`

Updated links:

| Original                   | New                           |
| -------------------------- | ----------------------------- |
| `../breeds/overview.md`    | `../../breeds/overview.md`    |
| `images/feeding-chart.png` | `../images/feeding-chart.png` |
| `../health/checkups.md`    | `../../health/checkups.md`    |
| `https://vet.org/sheep`    | (unchanged)                   |

## Completion Checks

- [ ] Source markdown file no longer exists at the old path.
- [ ] Destination file exists at the specified path.
- [ ] All relative links in the file have been checked and updated as needed.
- [ ] No relative link points to a non-existent target (warn if a target does not exist — may be intentional).
- [ ] External URLs, protocol-relative URLs, absolute paths, and anchor-only links are left untouched.
- [ ] Links inside code blocks and fenced code blocks are left untouched.

## Decision Rules

- If a relative link's resolved target does not exist after the move, **warn** the user but do not block the move — the target may be created later or may be a generated file.
- If the source file is not a `.md` file, abort and suggest using a plain `mv` instead.
- If the destination path is the same as the source, abort (no-op).
- If the destination file already exists, ask the user before overwriting.
- If the source file contains only non-relative links (or no links at all), skip the rewriting step and just move the file — but still note this in the summary.

## Output Template

After the operation, provide:

- **From**: `<source-path>`
- **To**: `<destination-path>`
- **Links rewritten**: `<N>`
- **Links skipped (external/anchor/absolute)**: `<N>`
- **Warnings**: `<list of any unresolved issues>`
