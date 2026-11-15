---
id: TASK-146
title: Render Mermaid diagrams with strict security and source-text fallback
status: To Do
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:13:00Z'
labels:
  - browser
  - rendering
  - security
  - blocked
milestone: m-21
dependencies:
  - TASK-143
  - TASK-145
references:
  - docs/requirements/business/REQ-06.017.md
  - docs/architecture/adr/ADR-0026-mermaid-only-offline-diagram-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
  - src/browser/render.rs
  - src/browser/http.rs
priority: high
ordinal: 41590
---

## Description

<!-- SECTION:DESCRIPTION:BEGIN -->
**Gate.** ADR-0026 is **Proposed**, not Accepted. This task is `blocked`: no implementation, vendored bundle, notice entry, CSP relaxation, or test fixture may land before formal acceptance. Acceptance must also resolve the ADR open questions on the Mermaid minor version and the supported diagram types; if acceptance changes a decision (the `mmd` alias, `style-src 'unsafe-inline'`, the input caps, or a sandboxed-iframe fallback), the task is re-planned before work starts.

**Why.** REQ-06.017 requires Mermaid fenced blocks to render as diagrams, offline, without arbitrary script execution, with a source-text fallback, and full-width diagram content; PlantUML is excluded and shown as source with a note. ADR-0026 fixes the mechanics once accepted.

**What.** A Mermaid-only diagram path on the TASK-145 pre-pass, with a vendored bundle, strict security settings, input caps, and a per-figure fallback.

**In scope.**
- A fenced block with info string `mermaid` becomes `<figure class="diagram" data-diagram="mermaid"><pre>escaped source</pre></figure>`; the `<pre>` is both fallback and client input. The `mmd` alias only if acceptance keeps it.
- `plantuml`, `puml`, `uml`, `dot`, and `graphviz` fences display as source with a visible note that they are not rendered; every other fence stays ordinary code text and never becomes an error.
- Per-fence caps before any client hand-off: 100,000 bytes, 1,000 lines, and 500 Mermaid edges; over-limit input is displayed as source with a note.
- Vendored Mermaid bundle under the TASK-143 layout with `MANIFEST.md` entry, `LICENSE`, `UPSTREAM.md`, and an MIT section in the package copyright file; loaded only on pages that contain a Mermaid fence.
- `mermaid.initialize` exactly as accepted, with `securityLevel: 'strict'`, `htmlLabels: false`, no click bindings, deterministic render ids, and per-figure `try`/`catch` so one failure cannot abort the page or other figures.
- On error the `<pre>` stays, the figure gains `data-render-error="diagram"`, and a one-line visible message is shown.
- CSP: `style-src 'self' 'unsafe-inline'` only if acceptance grants it; `script-src 'self'` and `default-src 'none'` unchanged; the exact CSP asserted per response class and `unsafe-inline` absent from `script-src`.
- The figure is a full-width `.bleed` region with an `overflow-x: auto` container, and the page keeps no horizontal scrollbar.
- Tests: hostile fixtures (`<img onerror>`, a `classDef` payload, a `javascript:` link) leave no script node and no event-handler attribute; each non-Mermaid fence stays literal; over-limit input; per-figure failure isolation; CLI and search output unchanged; a grep assertion that no JVM or PlantUML invocation exists.

**Out of scope.** PlantUML rendering, a second runtime, server-side diagram parsing, a record-body view, and diagram rendering outside curated prose fields.

**Assumptions.** REQ-06.017 names only the `mermaid` info string; the `mmd` alias is an ADR extension that requires the acceptance decision. The diagram type promise is limited to the types accepted against the pinned version.
<!-- SECTION:DESCRIPTION:END -->

## Acceptance Criteria
<!-- AC:BEGIN -->
- [ ] #1 ADR-0026 is formally Accepted before any implementation, asset, notice, or CSP change in this task, and any acceptance change to a decision is re-planned first.
- [ ] #2 A mermaid fence renders as a diagram on pages that contain one, and the bundle is loaded only on those pages.
- [ ] #3 PlantUML and the other unsupported fence languages display as source text with a note and never as an error.
- [ ] #4 The per-fence byte, line, and edge caps are enforced before client hand-off, and over-limit input is shown as source with a note.
- [ ] #5 Mermaid initializes with strict security level, HTML labels disabled, and no click bindings, and those settings are asserted by a test.
- [ ] #6 A render failure restores the source, marks the figure, shows a one-line message, and does not abort other figures or the page.
- [ ] #7 The figure is a full-width bleed region with its own horizontal scroll container, and the page gains no horizontal scrollbar.
- [ ] #8 Hostile fixture input leaves no script node and no event-handler attribute, and unsafe-inline never appears in script-src.
- [ ] #9 The vendored bundle has a manifest digest, a licence file, and a package notice section, and no JVM or PlantUML invocation exists.
<!-- AC:END -->
