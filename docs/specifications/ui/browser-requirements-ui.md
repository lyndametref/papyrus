# UI Description — Papyrus Local Browser (Requirement Content)

**Ticket:** TASK-6
**Specification:** [`../browser-requirements.md`](../browser-requirements.md) — sections 4 (UI content scope), 5 (HTTP endpoints), 6 (assets and rendering), 7 (editing contract), 10 (browser support and accessibility), and the UI design handoff
**Mockup package:** [`mockups/MOCK-0001-browser-requirements-review.html`](mockups/MOCK-0001-browser-requirements-review.html) — one fully self-contained review package containing the review landing, the design deck, and the web prototype (styles inlined in the file, no external stylesheet or other external reference); registry in [`mockups/README.md`](mockups/README.md)
**Status:** `approved` — explicitly approved by the user on 2026-11-03 (UTC); approved for implementation

## 1. Purpose and scope

This document describes the layout, states, elements, and interactions of the first browser increment: listing, viewing, and editing business-requirement and non-functional-requirement groups and requirements, plus the error pages and the responsive/accessibility behaviour.

Layout only. It names no library, framework, or component set. It defines no colour palette, logo, or visual identity beyond the neutral wireframe convention.

Out of scope by specification: creating, deprecating, restoring, deleting, or moving records; browsing other record types; search inside the browser; Markdown rendering; rich-text editing; attachments; authentication, sessions, or TLS.

## 2. Sources

| Source | Use |
| --- | --- |
| `docs/specifications/browser-requirements.md` §4 | Record forms, list structure, view fields, editable fields |
| `docs/specifications/browser-requirements.md` §5 | Routes, form fields, status codes, error responses |
| `docs/specifications/browser-requirements.md` §7 | Field rules, section preservation, version token, 409 |
| `docs/specifications/browser-requirements.md` §10 | Browser baseline, landmarks, labels, errors, contrast, 320 px |
| `docs/specifications/search-requirements.md` | Record model context (ID from file stem, title from heading) |
| `src/requirement.rs` | Record format: `# <ID> <title>`, `## User Story`, `## Group`, `## Description` |
| REQ-05.001, REQ-05.002, REQ-06.002, NFR-01.002, NFR-01.003, NFR-02.002, NFR-04.002, NFR-05.001 | Business and non-functional requirements |
| BR-004, BR-016 | Browser command behaviour; configuration keys |
| TASK-6 | Backlog ticket and acceptance criteria |

## 3. Conventions

- Wireframes are grayscale. Text in the mockups uses neutral system type; identifiers are shown in a monospace face only to make IDs distinguishable.
- The prototype draws every view on a 1440 × 900 CSS-px desktop canvas, scaled to fit the reviewer's window. The 320-px views place a 1:1 narrow frame beside their annotation legend on the same canvas.
- `responsive-list-320` and `responsive-edit-320` show a 320 CSS-px frame next to a numbered annotation legend.
- Sample records are illustrative. They reuse real Papyrus titles so reviewers recognise the content; counts and titles are not fixtures.
- Status `pending` means "awaiting explicit user approval". A mockup becomes `approved` only after the user approves it; no implementation should treat a pending mockup as final.
- One file is one MOCK id. MOCK-0001 is the only review artifact for this ticket: its landing page exposes the design deck and the web prototype, and the prototype navigation band exposes every view.

## 4. Screen map

| Route | Screen | Prototype views (MOCK-0001) |
| --- | --- | --- |
| `GET /` (`?type=req\|nfr`, default `req`) | Requirement list | `list-business`, `list-nonfunctional`, `list-empty-project`, `list-empty-type`, `list-group-empty`, `list-long` |
| `GET /doc/{id}` | Record view | `view-req-business`, `view-req-nfr`, `view-group-business`, `view-group-nfr`, `view-unresolved` |
| `GET /doc/{id}/edit` | Edit form | `edit-business`, `edit-nonfunctional`, `edit-dirty`, `edit-errors`, `edit-conflict`, `view-after-save` |
| any error status | Error page | `error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500` |
| 320 CSS px and assistive-technology review | Responsive list and edit form with annotations | `responsive-list-320`, `responsive-edit-320` |

All pages share one shell:

1. `header` — product name "Papyrus", project name (the project directory name), and a right-aligned "Local browser / offline · 127.0.0.1:6420" note. No top navigation menu: the list is the only top-level page.
2. `aside`/`nav` (persistent left sidebar) — document types as vertical choices (Business requirements, Non-functional requirements), extensible for future types. The selected type is bold, underlined, and marked `aria-current="page"`. On narrow viewports the sidebar collapses behind an accessible toggle button.
3. `main` — page-specific content.
4. `footer` — one line stating that every page is served from this project with no network access.
5. Exactly one `h1` per page.

## 5. Requirement list (`/`)

### 5.1 Layout

- `h1` "Requirements" and one muted sentence describing the page.
- Type navigation: a persistent left sidebar `nav` (labelled "Document types") listing the document types as vertical links — Business requirements, Non-functional requirements — extensible for future types. The selected type is bold, underlined, and marked `aria-current="page"`; colour is never the only cue. Selecting a type shows only that category in the main panel; business and non-functional requirements are separate document categories, not a filter on one set.
- The selected category renders its section (`h2`) with its groups and "Other requirements" subsection; the other category is not rendered on that page.
- Each section header shows the group count and requirement count on the right.
- Each group is a bordered row containing the group ID and title (one link), a member count, and the member requirements as a nested list of links. Requirement link text is "ID + title".
- "Other requirements" is an `h3` subsection at the end of its type section. Each row shows the requirement link and, right-aligned, the stored group reference as plain text: `Group: REQ-99 (not found)`.
- Empty states are dashed boxes with a sentence; the empty project and empty type states also state that records are created from the command line and that this browser does not create records.

### 5.2 States

| State | Prototype view | Notes |
| --- | --- | --- |
| Default, Business requirements selected | `list-business` | Only the business section is rendered; the sidebar marks Business requirements current |
| Non-functional requirements selected | `list-nonfunctional` | Only the non-functional section is rendered; the sidebar marks Non-functional requirements current |
| Empty project | `list-empty-project` | The selected section renders its empty state and the count reads "No groups"; the copy states that no requirement records were found in the configured directories |
| One document type empty | `list-empty-type` | Only the selected category renders; the empty state points at the other category in the sidebar |
| Group with no members | `list-group-empty` | Group row shows "No requirements" and the in-row note "No requirements in this group" |
| Many groups, long titles | `list-long` | Group titles wrap; an unbreakable requirement title wraps inside the row; no horizontal scroll |

### 5.3 Interactions

- Sidebar type links are ordinary links (`/`, `/?type=req`, `/?type=nfr`), so selecting a category works without JavaScript and is bookmarkable. There is no `type=all`: the default (`/`) selects Business requirements.
- Group ID/title opens the group record; a requirement link opens the requirement.
- Nothing on this page creates, edits, deletes, deprecates, restores, moves, or searches records.
- Empty and error states keep the filter available so the user can recover.

## 6. Record view (`/doc/{id}`)

### 6.1 Layout

- Breadcrumb `nav`: Requirements › (owning group, when resolvable) › record ID (current, not a link).
- Header block: record ID in small monospace text, record title as `h1`.
- Actions, right-aligned: primary "Edit" only. The "Back to requirements" link is redundant with the breadcrumb and is omitted.
- Read-only metadata box as a definition list: Record ID, Record form (Group / Requirement), Document type (business / non-functional, group or requirement), Owning group.
  - Owning group is a link when the group is active.
  - For a group record the value is "None (group record)".
  - For an unresolved reference the stored text is shown with "(not an active group)" and is not a link.
- "User story" section (`h2`) only for business-requirement records (groups and requirements) that have a story.
- "Description" section (`h2`); paragraphs are separated by blank lines, line breaks are preserved (`white-space: pre-wrap`), no Markdown is rendered, and requirement references stay literal text.
- Unknown and editor-added `## ` sections (for example `## Acceptance Criteria`, `## Traceability`) are preserved by the editor but are **not displayed** in this increment — see §11.
- For a group record, after its own fields the view lists the requirements belonging to that group (ID + title, in ID order, as links to their record views).

### 6.2 States

| State | Prototype view | Notes |
| --- | --- | --- |
| Business requirement with story and multi-paragraph description | `view-req-business` | Full metadata, story, description |
| Non-functional requirement without story | `view-req-nfr` | No story heading |
| Business requirement group | `view-group-business` | Owning group "None (group record)"; member list |
| Non-functional requirement group | `view-group-nfr` | No story heading; member list |
| Unresolved group reference | `view-unresolved` | Reference is plain text with a note; no group link, no group in the breadcrumb |

### 6.3 Interactions

- "Edit" opens the edit form for the same record.
- The breadcrumb "Requirements" returns to the list.
- The group link in the metadata opens the group record.
- Deprecated and unknown IDs never reach this page; they render the 404 error page.

## 7. Edit form (`/doc/{id}/edit`)

### 7.1 Layout

- Breadcrumb `nav`: Requirements › record ID › Edit (current). Exactly three crumbs, as drawn in every edit view; the owning group is not a crumb, it is shown in the metadata block.
- `h1` "Edit <ID>" and the current record title as a muted line. The right side of the page head holds the "Unsaved changes" status (§7.2), so the heading block and the status sit on one line group.
- Read-only metadata box: Record ID, Record form, Document type, Owning group (ID as a link). These values are never inputs and are never submitted.
- Fields:
  - Title — text input, `required`, labelled "Title (required)", help "Required · 1 to 200 characters · shown in lists and headings."
  - User story — textarea, business-requirement forms only, help "Optional · plain text · line breaks are preserved."
  - Description — textarea, help "Optional · a line starting with `## ` is not allowed."
- The version field is shown as a read-only, visible dashed note below the description field, drawn in MOCK-0001 (`edit-business`, `edit-dirty`, `edit-errors`) with the label "hidden field: version (current record token)"; the specification fixes the shipped label as "Hidden field: version (optimistic-concurrency token)" (§4.23). The note is static text: not an input, not focusable, not submitted, and it never displays the token value.
- A hidden `version` input carries the optimistic-concurrency token and is always submitted.
- Actions: primary "Save changes" (submit) and "Cancel" (link back to the view, no write).

### 7.2 States

| State | Prototype view | Notes |
| --- | --- | --- |
| Pristine, business requirement | `edit-business` | Title, user story, description |
| Pristine, non-functional requirement | `edit-nonfunctional` | Title and description only; no story field |
| Dirty | `edit-dirty` | "Unsaved changes" status in the page head; Save stays enabled; the leave guard is JavaScript-only |
| Validation errors (422) | `edit-errors` | Summary alert with links, field-level messages, submitted values kept, empty title and a `## ` line |
| Conflict (409) | `edit-conflict` | Nothing written; "Reload the form", "View current record", "Back to requirements"; attempted values shown read-only |
| Success | `view-after-save` | 303 redirect back to the view; the view reads the file from disk; no success banner page |

**Dirty state (as approved in `edit-dirty`).** The page head carries a right-aligned status element reading "Unsaved changes". It is hidden while the form is pristine and is revealed by the first `input` event on any field; it is static text, not a control, and it does not replace the Save button. Save stays enabled in every state. The form-actions note reads "Returns to the record view without saving." while pristine and "Leaving with unsaved changes warns only when JavaScript is available." once dirty; the `beforeunload` leave guard is progressive enhancement, so without JavaScript the form still posts and leaving is unguarded.

### 7.3 Error presentation

- Summary box at the top of the form: `role="alert"`, heading "There is a problem", one link per error, and the sentence "Nothing was saved. Your typed values are kept below."
- Field errors sit directly below the field, prefixed with "!" so the cue is not colour alone, and are referenced by `aria-describedby`; the field carries `aria-invalid="true"` and a thicker border.
- Title error text: "Title is required. Enter a title of 1 to 200 characters."
- `## ` line error text: "Description cannot contain a line starting with `## `. Remove that line or change it."
- Recovery keeps every typed value; resubmitting is the only action needed.

### 7.4 Conflict presentation

- `h1` "This record changed on disk", one sentence explaining that nothing was saved.
- Bordered notice repeating the contract message "409 Conflict — record changed on disk; reload the form."
- Primary action "Reload the form" (returns to the edit form with fresh content and a new version token), secondary "View current record" and "Back to requirements".
- "Your unsaved changes (read-only, not saved)" block repeats the attempted title, story, and description so the user can copy them before reloading.

## 8. Error pages

One generic layout, used for every HTML error status:

- Large monospace status code.
- `h1` short message, one explanatory sentence, one muted recovery hint.
- A single link "Back to requirements".

| Status | Title | Message |
| --- | --- | --- |
| 404 unknown | Record not found | That record does not exist or is not available. |
| 404 deprecated | Record not available | That record is not available. |
| 400 | Bad request | The request could not be understood. |
| 405 | Method not allowed | That action is not available on this address. |
| 415 | Unsupported media type | That content type is not accepted. |
| 500 | Something went wrong | The request could not be completed. |

The two 404 variants share the layout. The message may be identical so a deprecated ID is not revealed; the mockup shows a distinct but non-revealing message. Error pages never show absolute paths, request bodies, or internal detail. The prototype draws all six statuses as separate views (`error-404-unknown`, `error-404-deprecated`, `error-400`, `error-405`, `error-415`, `error-500`); the type navigation stays visible so recovery never depends on browser history.

## 9. Responsive and accessibility

### 9.1 Narrow viewport (320 CSS px)

- The header stacks: product and project name on one line group; the "Local browser" note is omitted at this width.
- The document-type sidebar collapses to a single accessible toggle button (labelled "Document types", `aria-expanded` reflecting state, `aria-controls` pointing at the type list). When expanded the type choices stack vertically; when collapsed they are hidden. Type selection still works without JavaScript via the links.
- Group rows stack: group ID, group title (wrapping), member count on its own line, then member links.
- Long titles and long unbreakable words wrap inside the row; there is no horizontal scrolling of text.
- Edit form fields are full width; Save and Cancel stack and each spans the column.
- At 200% zoom the same rules apply.

### 9.2 Annotations (`responsive-list-320`, `responsive-edit-320`)

| # | Annotation |
| --- | --- |
| 1 | Landmarks: `header`, `nav` (document types or breadcrumb), `main`, `footer`; the project name is text, not a link |
| 2 | One `h1` per page; type sections are `h2`; groups are `h3`; no level is skipped |
| 3 | Document-type sidebar is one `nav`/`aside` labelled "Document types" with vertical links; the active link uses `aria-current="page"` plus bold and underline; on narrow viewports it collapses behind an accessible toggle |
| 4 | A group row is one link (ID + title) with the member count as adjacent text; deprecated members are not counted |
| 5 | Member links read as ID + title, so they are descriptive out of context |
| 6 | Focus order follows document order; focus ring 2 px with 2 px offset, visible on every control |
| 7 | Every form control has a visible `label` associated by `for`/`id`; required fields say "required" in text and set the `required` attribute |
| 8 | Read-only metadata is a definition list, not disabled inputs; it is neither focusable nor submitted |
| 9 | Error summary has `role="alert"` and links to fields; field messages use `aria-describedby`; fields set `aria-invalid="true"`; error text names the field |
| 10 | Contrast at least 4.5:1; status and errors are never conveyed by colour alone |
| 11 | No JavaScript is required: filter links, record links, form posts, and server-rendered errors all work; only the dirty-form warning is progressive enhancement |
| 12 | The `version` token travels in a hidden input (never focusable, always submitted) and is shown on the edit form as a read-only dashed note below the description: visible static text, not an input, not focusable, never edited |

### 9.3 Keyboard-only operation

- Sidebar: Tab to the toggle (narrow viewports) or the type links; Enter/Space activates; the current type is announced.
- List: Tab moves group link → its member links → next group; Enter opens the record.
- View: Tab order is breadcrumb links, owning-group link, Edit, Back to requirements.
- Edit: Tab order is breadcrumb, read-only link, title, story (business only), description, Save, Cancel; Enter submits, Space activates buttons.
- Error summary links move focus to the field they name (anchor links); no keyboard trap.

## 10. Active elements

| Screen | Elements |
| --- | --- |
| List | 2 document-type sidebar links; 1 group link + 1 group-title link per group (same target); 1 link per member requirement; 1 link per "Other requirements" row |
| View | Breadcrumb links (Requirements, owning group); Edit; owning-group link in metadata |
| Edit | Title input; user story textarea (business only); description textarea; read-only version note (static text); hidden version input; Save changes; Cancel |
| Error | Back to requirements |
| All | Header text (not interactive), footer text |

No other control exists in this increment: no create, deprecate, restore, delete, move, search, Markdown toggle, or export control appears anywhere.

## 11. Interaction flows

1. **List → group → requirement → edit → save → view → list.** All → group `REQ-06` → requirement `REQ-06.002` → Edit → change title/description → Save changes → 303 → view shows the new content read from disk → Back to requirements.
2. **Category change keeps section structure.** Selecting "Business requirements" in the sidebar shows only the business `h2` with its groups and "Other requirements"; the non-functional section is absent, not empty. Selecting "Non-functional requirements" shows only that section. The two categories are independent document sets, not a filter on one set.
3. **Validation recovery without losing input.** Submit with an empty title and a `## ` line in the description → 422 re-renders the form with the summary, field messages, and all typed values → correct the two fields → Save succeeds.
4. **Conflict recovery.** Open the edit form, modify the file outside the browser, submit → 409 page → copy unsaved values if needed → Reload the form → new content and token → edit and save again.
5. **Error recovery.** Any error page offers exactly one way back to the list; no partial write or lost content is implied.
6. **Keyboard only.** All flows above are reachable with Tab/Shift+Tab, Enter, and Space; the visible focus ring is the only focus cue needed.

## 12. MOCK register

One review package covers the whole ticket. Its status is `approved`; it was explicitly approved by the user on 2026-11-03 (UTC) and is approved for implementation. The package is one self-contained HTML file: its styles are inlined in a `<style>` element in `<head>` and it references no external file, so it opens correctly from any location without the `mockups/` directory.

| MOCK | Type | Artifact | Contents |
| --- | --- | --- | --- |
| MOCK-0001 | scenario | [`mockups/MOCK-0001-browser-requirements-review.html`](mockups/MOCK-0001-browser-requirements-review.html) | Review landing (TASK-6, milestone m-1, sources, approval status, one button per content), design deck (12 slides: cover, 9 grouped view slides with "What it shows" / "Why this way", cross-cutting decisions, open questions), web prototype (25 views: requirement list 6, record view 5, edit form 6, error page 6, 320 px 2) |

## 13. Assumptions and open questions

These are design decisions or specification gaps found while mocking up; each needs confirmation or a specification note before implementation.

1. **Unknown sections are preserved but not displayed.** Section 7.7 preserves `## ` sections such as `## Criticality`, `## Acceptance Criteria`, and `## Traceability`, but §4.10 lists only story and description as view content. The mockups display only story and description. Confirm whether preserved sections should also be shown read-only (recommended: show them read-only, since users cannot see what they are preserving).
2. **Success feedback.** Per the handoff, a save returns to the plain record view with no success banner; `view-after-save` shows the resulting content. Confirm whether a brief inline "Saved" status is wanted instead of silence.
3. **Dirty-state button.** Disabling Save while pristine would require script and would break the no-JavaScript requirement, so Save is always enabled and the dirty state is shown as an "Unsaved changes" status element in the page head plus a JavaScript-only leave guard. Confirm.
4. **404 wording.** The mockups use distinct but non-revealing messages for unknown and deprecated records; the specification allows them to be identical. Confirm which wording ships.
5. **Project name in the header.** The specification says "project name"; no display name is configured, so the mockups use the project directory name. Confirm the source of that label.
6. **Group title in the edit metadata.** The edit form shows the owning group ID only (space is tight at 320 px); the view shows ID plus title. Confirm that the ID alone is acceptable on the edit form.
7. **Conflict value preservation.** The 409 page repeats the attempted values read-only. The specification requires only the message and a reload affordance; the block is an addition to avoid silent data loss. Confirm.
8. **Document-type navigation.** The sidebar lists document types as links (not buttons), so category selection works without script and is bookmarkable. The query parameter `?type=req|nfr` is the only selection mechanism; there is no "all" view. Confirm the default category (Business requirements) is acceptable.
9. **Counts.** Member counts exclude deprecated members (deprecated records are not counted). Confirm the wording "2 requirements" / "1 requirement" / "No requirements".
10. **Header note.** The mockups show the bind address in the header ("offline · 127.0.0.1:6420"). This is static text, not a link; confirm it is wanted, since a non-loopback bind would make the printed address misleading.

## 14. Approval status

| MOCK | Status |
| --- | --- |
| MOCK-0001 | `approved` — explicitly approved by the user on 2026-11-03 (UTC) |

No approval is pending. MOCK-0001 is approved for implementation; the registry status is `approved`.
