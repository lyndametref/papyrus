---
name: html-template-migration
description: 'Migrate Markdown-driven HTML index generators to separate HTML templates and the shared placeholder renderer. Use when refactoring or adding a generated HTML index.'
argument-hint: 'Describe the index generator to migrate and its template/data inputs.'
user-invocable: true
disable-model-invocation: false
---

# HTML Template Migration

## Outcome

Generated HTML structure, styles, and browser behavior live in a standalone template. A source-specific generator parses and validates its Markdown corpus, then supplies JSON values to the shared renderer.

## Renderer

The shared renderer is `scripts/render_html_template.sh`. It uses POSIX sh and awk and accepts:

```
~/.config/opencode/skills/html-template-migration/scripts/render_html_template.sh TEMPLATE OUTPUT DATA_JSON PAGE_TITLE PAGE_HEADING
```

`DATA_JSON` is a compact, single-line JSON object containing the visualization-specific data. The page title and heading are separate arguments so the renderer can escape them for HTML text and attribute contexts.

Templates use these typed slots:

- `{{HTML:PAGE_TITLE}}` inserts the escaped page title.
- `{{HTML:PAGE_HEADING}}` inserts the escaped page heading.
- `{{JSON:DATA}}` inserts the JSON payload with `<`, `>`, and `&` escaped for safe embedding in a script element.

The renderer rejects missing, unrecognized, or unresolved placeholders, requires a single-line JSON object, and writes output atomically beside the destination.

## Migration Rules

- Keep each visualization's template beside its skill under `templates/`.
- Keep source parsing, corpus validation, data schema, and success summary in the visualization skill's generator.
- Have generators write their compact JSON payload to a temporary file in the project-local `./.tmp/` directory (create it if missing), pass it and the page labels to the shared renderer, and remove the temporary file on all exit paths.
- Keep Markdown as the source of truth; generated HTML remains an output artifact.
- Escape values according to their template context; never interpolate source text directly into HTML or JavaScript.
- Update the visualization skill instructions when changing its template or renderer usage.

## Verification

- Run the visualization skill's documented generator command.
- Confirm the output contains no unresolved placeholders and all expected source IDs.
- Check the embedded JSON parses and the generated page retains its list, filter, and detail behavior.
- Check that the renderer rejects a deliberately missing slot and preserves the existing destination.
