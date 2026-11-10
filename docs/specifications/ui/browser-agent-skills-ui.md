# UI Description — Papyrus Local Browser (Agent Skills and Workflows Page)

**Ticket:** TASK-76
**Specification:** [`../browser-requirements.md`](../browser-requirements.md) baseline; [`../browser-managed-artifacts-and-lifecycles.md`](../browser-managed-artifacts-and-lifecycles.md); requirements `REQ-06.005`, `REQ-06.003`; `NFR-01.002`; `BR-016`
**Mockup package:** [`mockups/MOCK-0011-browser-agent-skills-review.html`](mockups/MOCK-0011-browser-agent-skills-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-10.

## 1. Purpose and scope

Layout, states, elements, and interactions for a new read-only browser page listing agent skills and workflows for configuring and operating Papyrus. The page is agent-facing (not end-user) and presents instructions parameterized by the effective `papyrus.conf.yaml` (prefixes, `id_padding`, statuses, enabled document/decision types, directories, paths, date format). It reuses the approved MOCK-0001 shell and the sidebar pattern of MOCK-0010.

## 2. Shared shell

Header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types", `main`, footer, one `h1` per page. Selected entry bold + underline + `aria-current="page"`. The 320 CSS px sidebar collapses behind an accessible toggle. CSP/escaping/accessibility baseline as in MOCK-0001.

Sidebar entries (in order): Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts &amp; lifecycles, Agent skills &amp; workflows. The new entry is placed after "Managed artifacts &amp; lifecycles" (end of the list) and is always present (not gated by `document_types`).

## 3. Screen map

| Route | Screen | Prototype views |
| --- | --- | --- |
| `GET /skills` (alias `GET /?type=skills`) | Agent skills page | `skills` |
| any error status | Error page | `error-404`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px | Responsive skills page | `responsive-skills-320` |

## 4. Page content

- One `h1` "Agent skills and workflows"; subtitle states the page is agent-facing and read-only and that values reflect the effective configuration.
- A "Configuration in effect" block (`dl`) summarizing: task prefix, milestone prefix, release prefix, ID padding (with example), date format, statuses (joined by `&rarr;`), enabled document types, enabled decision types, backlog/decision/doc directories.
- Grouped skill sections (`h2`), each containing skill articles (`h3` title, short description, copyable monospace snippet block, Copy affordance). Groups:
  1. Project setup and init
  2. Backlog, tasks, milestones, releases
  3. Requirements and documentation management
  4. Search
  5. Browser usage
  6. Release and audit workflows
  7. Configuration management
  8. Decisions
- Additional likely groups (glossary, UI specs/mockups, technical specs, business rules) are covered within group 3.

## 5. Content format (OQ-1)

Recommended default: structured sections with per-skill headings, short description, and copyable command/snippet blocks (monospace) that interpolate configured values; each block has a "Copy" affordance as progressive enhancement (no JavaScript required for core content). List-only is the alternative.

## 6. Derived vs static (OQ-2)

Recommended default: template snippets parameterized by effective config values (prefixes, padding, statuses, enabled types, directories) so instructions match the project; static prose for workflow narrative. Fully static snippets are the alternative.

## 7. Active elements

- Sidebar entry "Agent skills &amp; workflows" (link, active state).
- Copy buttons (progressive enhancement).
- Sidebar toggle at 320 CSS px.
- Error-page recovery link.

## 8. Error pages and responsive/a11y

Reused from MOCK-0001: 404, 400, 405, 415, 500, each with a recovery link to the skills page. 320 px: sidebar toggle first, collapsed type list, stacked content. Focus order follows document order; every control has a visible label; works without JavaScript; no horizontal scrolling of text.

## 9. Open questions (resolved 2026-11-10)

All open questions resolved to their recommended defaults by explicit user confirmation on 2026-11-10.

- OQ-1 (content format): **resolved** — structured sections with per-skill headings, short descriptions, and copyable command/snippet blocks (monospace) with a Copy affordance; list-only rejected.
- OQ-2 (derived vs static): **resolved** — template snippets parameterized by effective config values (prefixes, `id_padding`, statuses, enabled document/decision types, directories, paths, date format) with static prose for workflow narrative; fully static snippets rejected.
- OQ-3 (sidebar label/placement): **resolved** — label "Agent skills &amp; workflows", placed after "Managed artifacts &amp; lifecycles" (end of document-type list), always present (not gated by `document_types`).
- OQ-4 (route): **resolved** — `GET /skills` with alias `GET /?type=skills`.
- OQ-5 (config summary scope): **resolved** — "Configuration in effect" block summarizes all effective keys (task/milestone/release prefixes, `id_padding` with example, date format, statuses, enabled document/decision types, directories).

## 10. Assumptions

- The page is read-only (no forms/edits); it is an agent-facing reference.
- Command snippets are illustrative; exact CLI verbs follow the documented command behavior in BR-001..BR-016.
- The page works offline; no external resources.
- Example records/values are illustrative.

## 11. Approval status

| Item | Status |
| --- | --- |
| MOCK-0011 | `approved` &mdash; explicitly approved by the user on 2026-11-10. |
| This UI description | `approved` |
| Registry row | `approved` |