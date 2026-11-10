# Feature Specification — Browser Page for Agent Skills and Workflows

**Ticket:** TASK-77
**Milestone:** m-12 (Final Cross-Type Integration)
**Status:** Refined (execution-ready; UI mockup approved)
**UI mockups:** [`MOCK-0011`](ui/mockups/MOCK-0011-browser-agent-skills-review.html) (approved 2026-11-10, TASK-76) — UI description [`ui/browser-agent-skills-ui.md`](ui/browser-agent-skills-ui.md); reuses the approved baseline shell [`MOCK-0001`](ui/mockups/MOCK-0001-browser-requirements-review.html). No new mockup is required for this ticket.

## Purpose

Add a read-only page to the offline, project-local web interface that lists the agent skills and workflows used to configure and operate Papyrus, per REQ-06.005. The page is agent-facing (not end-user) reference material: it presents instructions whose values are parameterized by the effective `papyrus.conf.yaml` (prefixes, `id_padding`, statuses, enabled document and decision types, directories, paths, date format) so generated instructions match the configured project. It advances NFR-01.002 (display all Papyrus content) and NFR-01.003 (offline use).

## Scope

### In scope

- A new page route that serves the skills page, reusing the shared shell, sidebar, header, footer, CSP, escaping, and responsive rules of the baseline (TASK-6, ADR-0005/ADR-0006).
- A persistent sidebar entry labelled `Agent skills & workflows`, placed after `Managed artifacts & lifecycles` (the last entry), that links to the page and carries `aria-current="page"` when active.
- A page body with:
  - one `h1` titled `Agent skills and workflows`;
  - a subtitle stating the page is agent-facing, read-only, and that values reflect the effective configuration;
  - a `Configuration in effect` block (`dl`) summarizing every effective key listed in section 3;
  - eight grouped skill sections (`h2`) in the order listed in section 4, each containing skill articles (`h3` title, short description, copyable monospace snippet block, Copy affordance).
- Parameterization of prose and example values by the effective configuration (section 5).
- Unit/integration tests extending `tests/browser_cli.rs` (or a sibling file) for the new page.

### Out of scope

- Listing, viewing, editing, creating, deprecating, restoring, archiving, deleting, or searching any record from this page; the page is reference only.
- Any API endpoint for the page; the page is HTML only.
- Changes to configured `document_types`, `decision_types`, prefixes, or statuses; the page only reflects them.
- Changes to MOCK-0001..MOCK-0010, to MOCK-0011, or to any UI description document.
- New dependencies, a web framework, an async runtime, a templating engine, a second crate, or a new ADR. ADR-0005/ADR-0006 remain the baseline.
- Packaging and release verification — TASK-20.

## Behavior

### 1. Route and navigation

1. `GET /skills` serves the page. `GET /?type=skills` serves the same page (alias) so the sidebar/filter model stays consistent. Any other method on a known path is `405` with an `Allow: GET, HEAD` header; an unknown path is `404`.
2. The sidebar entry is labelled exactly `Agent skills & workflows` and is placed after `Managed artifacts & lifecycles` (the last document-type entry). It is always present; it is not gated by any `document_types.*.enabled` flag.
3. The sidebar entry links to `/skills`. When the page is active, the entry carries `aria-current="page"` and the non-colour cue (bold + underline) used by every other entry.
4. The page renders the shared shell: header (product name, project name, static offline/bind note), persistent left sidebar, `main`, footer, and exactly one `h1` with the text `Agent skills and workflows`.
5. A subtitle paragraph states: `Agent-facing instructions for configuring and operating Papyrus. Values reflect the effective project configuration. Read-only reference.`
6. The breadcrumb is not required for this page; the sidebar is the navigation. (The page is not a record view.)

### 2. Page shell and sidebar

1. The sidebar lists entries in this order: Business requirements, Business rules, Audit records, Technical specifications, Non-functional requirements, Backlog, Decisions, Glossary, UI specifications, UI mockups, Managed artifacts & lifecycles, Agent skills & workflows. Entries gated by `document_types` keep their existing gating; `Managed artifacts & lifecycles` and `Agent skills & workflows` are always present.
2. At 320 CSS pixels the sidebar collapses behind the existing accessible toggle, matching the baseline.
3. The page works with JavaScript disabled; the Copy affordance (section 6) is progressive enhancement only.

### 3. Configuration in effect block

1. The page renders a `Configuration in effect` block as a `dl` with one `dt`/`dd` pair per key, in this order:
   - Task prefix
   - Milestone prefix
   - Release prefix
   - ID padding (rendered as `{n} (e.g. {task_prefix}-{example})`, where `{example}` is `1` zero-padded to `id_padding` width, e.g. `001`)
   - Date format
   - Statuses (configured task statuses joined by ` → `)
   - Enabled document types (comma-space list of enabled `document_types` keys in configured order; `(none)` when none are enabled)
   - Enabled decision types (comma-space list of enabled `decision_types` keys in configured order; `(none)` when none are enabled)
   - Backlog directory
   - Decision directory
   - Doc directory
2. When the configured status list is empty or invalid, the server falls back to its configured/default status handling (as in the lifecycle page) rather than rendering an empty status chain; the status chain is never empty.
3. When `id_padding` is outside the supported range (1–6), the server falls back to the default padding (3) for both the displayed value and the example.
4. All values in the block are HTML-escaped and rendered as text, never as markup.

### 4. Skill groups and entries

The page renders exactly eight skill groups, in this order, each as a `section` containing an `h2` with the group title from this list:

1. `Project setup and init`
2. `Backlog, tasks, milestones, releases`
3. `Requirements and documentation management`
4. `Search`
5. `Browser usage`
6. `Release and audit workflows`
7. `Configuration management`
8. `Decisions`

Each group contains the skill articles listed below. Each article is an element containing an `h3` title, a short description paragraph, and a monospace snippet block (`pre`) followed by a Copy affordance.

| Group | Skill (h3) | Description (short) | Snippet (template) |
| --- | --- | --- | --- |
| Project setup and init | Initialize a new project | Create the project skeleton and write the initial configuration. | `papyrus init --dir ./my-project` |
| Project setup and init | Inspect effective configuration | Prefixes, padding, statuses, enabled types, and directories are read from `papyrus.config.yml` with built-in defaults. | `papyrus config list` |
| Backlog, tasks, milestones, releases | Task IDs and statuses | New tasks use prefix `{task_prefix}` with padding `{id_padding}`: `{task_prefix}-{example}`. Statuses flow `{status_chain}`. | `papyrus task create "Title" --status {default_status}` |
| Backlog, tasks, milestones, releases | Milestones and releases | Milestone IDs use prefix `{milestone_prefix}` (`{milestone_prefix}-{example}`); release IDs use prefix `{release_prefix}` (`{release_prefix}-{example}`). Release lifecycle is separate from task workflow status. | `papyrus milestone create "Title"` newline `papyrus release create "Title"` |
| Requirements and documentation management | Document types and directories | Enabled types (`{enabled_doc_types}`) are stored under `{doc_directory}` in their configured directories. Disabled types are hidden. | `papyrus req create "Title"` newline `papyrus nfreq create "Title"` |
| Requirements and documentation management | Business rules, glossary, specifications | Rules, glossary terms, UI specs, technical specs, and mockups follow the same record lifecycle: active → deprecated (restored via restore). | `papyrus rule create "Title"` newline `papyrus glossary create "Term"` |
| Search | Search across records | Search spans enabled document and decision types; disabled types are excluded. | `papyrus search "query"` |
| Browser usage | Serve the local browser | Offline, loopback by default. No internet connection required. | `papyrus browser --port 6420` |
| Release and audit workflows | Releases and audits | Releases move active → released; audit records are created for release audits. Decision types enabled: `{enabled_decision_types}`. | `papyrus release status {release_prefix}-{example} released` |
| Configuration management | Set and unset keys | Prefixes, padding, statuses, enabled types, and directories are configuration keys (BR-016). | `papyrus config set task_prefix {task_prefix}` newline `papyrus config unset statuses` |
| Decisions | Decision records | Enabled decision types: `{enabled_decision_types}`. Lifecycle: proposed → accepted \| rejected; accepted → deprecated \| superseded. | `papyrus decision create --type adr "Title"` |

1. The set of groups and skill titles above is normative for this page and matches the approved MOCK-0011. No group or skill may be added, removed, or reordered by configuration.
2. Group order, headings, and count are fixed and do not depend on configuration, project contents, or enabled document/decision types.
3. Descriptions are static prose except where a `{placeholder}` is shown; placeholders are replaced per section 5.
4. Snippets are monospace blocks (`pre` or equivalent) and are copyable. The Copy affordance is a button (or equivalent control) associated with each snippet.

### 5. Parameterization (derived vs static)

1. Prose and example values use placeholders replaced from the effective configuration:
   - `{task_prefix}`, `{milestone_prefix}`, `{release_prefix}` — configured prefixes (defaults `TASK`, `MS`, `REL`).
   - `{id_padding}` — configured padding width (default 3).
   - `{example}` — `1` zero-padded to `{id_padding}` width (default `001`).
   - `{status_chain}` — configured task statuses joined by ` → ` (default `ToDo → Initiated → Refined → Implemented → Validated → Done`), with the same empty/invalid fallback as section 3.2.
   - `{default_status}` — configured default status (default `ToDo`).
   - `{enabled_doc_types}` — comma-space list of enabled `document_types` keys.
   - `{enabled_decision_types}` — comma-space list of enabled `decision_types` keys.
   - `{doc_directory}`, `{decision_directory}`, `{backlog_directory?}` — configured directories (only those listed in section 3 are required; backlog directory is not referenced by a skill snippet but is shown in the config block).
   - `{date_format}` — configured date format (shown in the config block; not required in snippets).
2. Static prose and static snippets (those without placeholders) are not derived from configuration and are rendered verbatim.
3. All substituted values are HTML-escaped before rendering. Status names, type keys, prefixes, directories, and date formats are rendered as text, never as markup.
4. The page makes no outbound network requests and embeds no external resources; all styles come from `/assets/app.css` and scripts from `/assets/app.js`.

### 6. Copy affordance

1. Each snippet block has a Copy affordance (button or equivalent) that copies the rendered snippet text to the clipboard.
2. The Copy affordance is progressive enhancement: the page is fully readable and usable with JavaScript disabled; copying then requires manual selection. No core content depends on JavaScript.
3. The Copy affordance has an accessible name (e.g. `Copy`).

### 7. Rendering, security, and accessibility

1. All dynamic values (project name, bind note, configured prefixes, padding, statuses, type keys, directories, date format) are HTML-escaped before rendering.
2. The page sets the same CSP and security headers as every other page (`default-src 'none'; style-src 'self'; script-src 'self'; ...`).
3. The page loads no external stylesheet, script, font, image, or network resource; styles come from `/assets/app.css`.
4. The page works with JavaScript disabled.
5. Focus order follows document order; every interactive control (sidebar toggle at 320 px, sidebar links, Copy buttons) has a visible label or accessible name.
6. The 320 CSS-pixel responsive layout collapses the sidebar behind the existing accessible toggle, matching the baseline.

### 8. Tests

1. An integration test starts `papyrus browser --no-open` on a free port and asserts:
   - `GET /skills` returns `200` with `Content-Type: text/html; charset=utf-8`;
   - the response body contains the `h1` text `Agent skills and workflows`;
   - the response body contains all eight group headings in order;
   - the response body contains the configured default status chain `ToDo → Initiated → Refined → Implemented → Validated → Done`;
   - the response body contains the config-block values for the default fixture (task prefix `TASK`, milestone prefix `MS`, release prefix `REL`, padding `3 (e.g. TASK-001)`, enabled document types `req, nfr, glossary, rule, ui-spec, ui-mockup, audit, deploy, tech-spec`, enabled decision types `adr, bdr`);
   - the response body contains the sidebar entry `Agent skills &amp; workflows`;
   - the response body contains `<a href="/skills" aria-current="page">Agent skills &amp; workflows</a>`;
   - the response contains no `http://` or `https://` reference and no external resource reference.
2. The same test asserts `GET /?type=skills` returns the same page (alias).
3. The same test asserts `POST /skills` returns `405` with an `Allow` header.
4. A second test (or parameterised case) initialises a project with a custom configuration (for example custom prefixes, a custom status list such as `["Backlog", "Doing", "Done"]`, custom padding, and a disabled document type) and asserts:
   - the config block renders the custom prefixes, padding, status chain, and enabled-type list;
   - the custom status chain appears in the `Task IDs and statuses` article;
   - the default chain does not appear.
5. A test (or the same test) asserts the eight group headings appear in the order listed in section 4.

## Assumptions

- Assumption: The page is read-only and renders no project records; it does not call into the record index or any store.
- Assumption: The route is `/skills` with `/?type=skills` as an alias, matching the UI description's recommended default (OQ-4) and keeping the sidebar/filter model consistent.
- Assumption: The sidebar entry is always present and is not affected by `document_types.*.enabled`, because the page is a fixed reference page independent of configuration.
- Assumption: Content format is structured sections with per-skill headings, short descriptions, and copyable command/snippet blocks (OQ-1 resolved to recommended default on 2026-11-10).
- Assumption: Snippets are templates parameterized by effective config values, with static prose for workflow narrative (OQ-2 resolved to recommended default on 2026-11-10).
- Assumption: Sidebar label is `Agent skills & workflows`, placed after `Managed artifacts & lifecycles`, always present (OQ-3 resolved on 2026-11-10).
- Assumption: Config summary covers all effective keys (OQ-5 resolved on 2026-11-10).
- Assumption: Custom task statuses have no per-state descriptions; only their names and order are rendered.
- Assumption: Example record IDs in snippets use the configured prefix and `id_padding` width.
- Assumption: Snippet command verbs are illustrative and follow the documented command behavior in BR-001..BR-016; the page does not validate or execute them.
- Assumption: Copy buttons are progressive enhancement; core content works without JavaScript.

## Open questions

- None. REQ-06.005's open questions (content format, derived vs static, page location/navigation, covered workflows) are resolved by the approved MOCK-0011 and the UI description's section 9 (user confirmation 2026-11-10). OQ-1..OQ-5 in the UI description are resolved to their recommended defaults.

## Related artifacts

- Requirement: `docs/requirements/business/REQ-06.005.md` (primary); `docs/requirements/business/REQ-06.002.md`; `docs/requirements/business/REQ-06.003.md`; `docs/requirements/non-functional/NFR-01.002.md`; `docs/requirements/non-functional/NFR-01.003.md`
- Business rule: `docs/business-rules/BR-016-configuration-keys-and-precedence.md`
- UI description: `docs/specifications/ui/browser-agent-skills-ui.md`
- UI mockup: `docs/specifications/ui/mockups/MOCK-0011-browser-agent-skills-review.html` (approved)
- Baseline browser spec: `docs/specifications/browser-requirements.md`
- Sibling lifecycle spec: `docs/specifications/browser-managed-artifacts-and-lifecycles.md`
- Milestone: `m-12`
- Dependency: TASK-76 (Done)
- Related: TASK-19, TASK-20 (depend on this ticket)
path: /home/lyly/Documents/Dev/papyrus/docs/specifications/browser-agent-skills-and-workflows.md