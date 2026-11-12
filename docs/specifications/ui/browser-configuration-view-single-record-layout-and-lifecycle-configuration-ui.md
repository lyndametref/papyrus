# UI Description — Papyrus Local Browser (Configuration View, Single Record View Layout, and Lifecycle Configuration)

**Ticket:** TASK-87
**Specification:** [`../browser-requirements.md`](../browser-requirements.md) (§4a, §4b, §4c, §4d, §10a); [`../browser-managed-artifacts-and-lifecycles.md`](../browser-managed-artifacts-and-lifecycles.md) (§4a); [`../browser-glossary-terms.md`](../browser-glossary-terms.md) (§3a); [`../browser-spec-revisions-task-86.md`](../browser-spec-revisions-task-86.md) (§9)
**Mockup package:** [`mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html`](mockups/MOCK-0014-browser-configuration-view-single-record-layout-and-lifecycle-configuration-review.html) — one self-contained review package (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-12 (UTC) (TASK-87).

## 1. Purpose and scope

Layout, states, elements, and interactions for (A) the configuration view, (B) the shared single record view layout, (C) shared header branding, (D) record references as links, (E) NFR grouping under owning groups, (F) the glossary landing page revisions, and (G) the lifecycle page per-artifact descriptions and read-only configuration. Reuses the approved MOCK-0001 shell and prior per-type mockups (MOCK-0002..MOCK-0013) as context only.

## 2. Shared shell and branding

Every page shares one shell: header (product name, project name, static offline/bind note), persistent left sidebar `nav` "Document types", `main`, footer, one `h1`. Shared header branding (§4a): project name renders large and bold; "Papyrus" renders in smaller letters; the "P" mark keeps its current size and styling; project name is the configured project name. The rule applies to every page and is not restated per type.

Sidebar entries include document types plus, in order: `Configuration` (after document types), `Managed artifacts &amp; lifecycles`, `Agent skills &amp; workflows`. Selected entry carries `aria-current="page"` plus bold+underline (non-colour cue). At 320 px the sidebar collapses behind an accessible toggle labelled "Document types" with `aria-expanded`/`aria-controls`.

## 3. Screen map

| Route | Screen | Prototype view |
| --- | --- | --- |
| `/config` (alias `/?type=config`) | Configuration view | `config` |
| `/doc/{id}` | Single record view (business req) | `view-req` |
| `/doc/{id}` | Single record view (sub-requirement) | `view-req-nfr` |
| `/doc/{id}` | Record with dependency/reference links | `view-task` |
| `/?type=nfr` | NFR grouping list | `list-nfr` |
| `/?type=glossary` | Glossary landing | `list-glossary` |
| `/doc/{id}` | Glossary detail (single record layout) | `view-glossary` |
| `/lifecycle` (alias `/?type=lifecycle`) | Lifecycle page | `lifecycle` |
| 320 CSS px | Responsive config | `responsive-config-320` |
| 320 CSS px | Responsive single record view | `responsive-view-320` |
| 320 CSS px | Responsive lifecycle | `responsive-lifecycle-320` |
| any error status | Error pages | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |

## 4. Configuration view

- Route `/config`, alias `/?type=config`; sidebar entry labelled `Configuration` placed after the document types.
- Shows effective configuration values, including built-in defaults, and marks which keys are explicitly set in `papyrus.config.yml` (badge `explicit` vs `default`).
- Editable keys are exactly those `papyrus config set` accepts; same validation and precedence; no key treated as secret; environment values never shown as editable config.
- Folder-key changes relocate through the single relocation routine (REQ-01.002); the browser never moves a folder; every path passes the project-directory boundary check.
- Links to the lifecycle page for per-artifact read-only configuration; does not duplicate editing there.
- Table columns: Key, Effective value, Source (explicit/default), Edit action.

## 5. Single record view layout

- Summary card at the beginning shows only ID, created date, updated date. A sub-requirement additionally shows its owning group in the card; nothing else is added.
- Summary card does not show status, milestone, assignee, labels, or references.
- Labels shown as a taglist under the title.
- Dependencies and references appear at the end as native HTML `<details>` disclosure sections, each shown only when the record has at least one such value; work with JavaScript disabled.
- Applies to every managed Markdown record type with a single record view; UI mockup HTML views excluded.

## 6. Record references as links

- Task dependencies, a task's milestone, `references` values, and a sub-requirement's owning group render as hyperlinks when they resolve to a Papyrus record; activating navigates to the referenced record's view.
- Unresolved references render as plain text with no error and no 404.

## 7. NFR grouping

- Non-functional atomic requirements listed under their owning group entry; no trailing "Other requirements" subsection and no catch-all section.
- Owning group derived from the requirement ID prefix (`NFR-XX.YYY` → `NFR-XX`); no `Group` field added to files.
- Group heading shows the group record's title; missing group record falls back to the group number (`NFR-04`).
- A group with no atomic requirements renders as an entry with no children and member count "No requirements".
- Every atomic requirement appears under exactly one group.

## 8. Glossary landing page

- Omits created and updated dates.
- Labels shown as tags (same taglist as single record view layout).
- No references shown.
- Detail views follow the single record view layout.

## 9. Lifecycle page

- One section per fixed record type (Tasks, Milestones, Releases, Decisions, Business requirements, Non-functional requirements, Glossary terms, UI specifications, Technical specifications, UI mockups, Business rules, Audit records).
- Each section: curated description, lifecycle sentence, read-only effective configuration (enabled, ID prefix, storage folder; task section also statuses).
- Configuration values are display-only; changes only via the configuration view; page points to it.
- Covered set fixed; not derived from configured document types.
- Subtitle states Backlog.md docs, drafts, and project configuration files are not covered.

## 10. Error pages and responsive/a11y

Reused from MOCK-0001: 404 unknown, 404 deprecated, 400, 405, 415, 500, each with a recovery link. 320 px variants collapse the sidebar behind an accessible toggle and stack content. Focus order follows document order; every control has a visible label; works without JavaScript; landmarks and `aria-current` as in approved packages.

## 11. Active elements

- Configuration view: sidebar entry, Edit buttons per key, link to lifecycle page.
- Single record view: breadcrumb links, Edit button, disclosure summaries for dependencies/references.
- Reference links: task dependencies, milestone, references, owning group.
- NFR grouping: group/requirement links.
- Glossary landing: term links.
- Lifecycle page: sidebar entry, link to configuration view.
- 320 px: sidebar toggle.
- Error pages: recovery link.

## 12. Open questions (recommended defaults)

- OQ-1 (resolved default): sidebar entry labelled `Configuration`, placed after the document types, linking to `/config`. Subject to TASK-87 approval.
- OQ-2: whether the configuration view is reached from the sidebar or a header link. Recommended: sidebar, consistent with lifecycle and skills pages.
- OQ-3: whether each per-type specification restates the shared single record view layout or only references it. Recommended: reference by section number to avoid drift.
- OQ-4: disclosure sections default open/closed. Recommended: collapsed by default.
- OQ-5: responsive breakpoints and exact 320 px layout details. Recommended: reuse approved responsive patterns.
path: /home/lyly/Documents/Dev/papyrus/docs/specifications/ui/browser-configuration-view-single-record-layout-and-lifecycle-configuration-ui.md