# UI Description — Browser Shell and Shared List Layout

**Ticket:** TASK-141
**Specification:** [`../browser-shell-shared-list-layout-and-page-width.md`](../browser-shell-shared-list-layout-and-page-width.md) §3–§5 (feature specification; superseded by TASK-148 when its clauses move into the browser specification corpus)
**ADR:** [`../../architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md`](../../architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md) — Accepted 2026-11-15, amended 2026-11-15 (decisions 1, 2, 3, 6, 8; constraints 10–15)
**Requirements:** REQ-06.016 (one shared list layout), REQ-06.015 (content width; width clause only), REQ-06.004 (browser/CLI front-matter parity), REQ-06.009, REQ-05.007
**Mockup package:** [`mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html`](mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html) — one self-contained review file (review landing, web prototype); registry [`mockups/README.md`](mockups/README.md)
**Status:** `draft` — awaiting explicit user approval of MOCK-0016.

## 1. Purpose and scope

Layout of three structural pieces, shown on the reference family (the milestone list at `/?type=milestone`):

1. **The page shell** — header, navigation, main region, footer, rendered once per page.
2. **The page-width class** — decided by the server per route; `Capped` only in this ticket.
3. **The shared list layout** — one table for every artifact family, with family variation limited to declared slot columns.

Layout only: no library, framework, component set, or technology is named or implied. Grayscale wireframe; the shared wireframe tokens stand.

**Not in this document:** the navigation toggle and its persisted state (TASK-144), the `.app--full` CSS rule and the board route that uses it (TASK-144), the remaining family migrations (TASK-142), row grouping visuals beyond the declared hierarchical variant (first exercised in TASK-142), and the record view/edit field sets (unchanged).

## 2. Shell regions

One shell per response; each region appears exactly once.

| Region | Contents | Notes |
| --- | --- | --- |
| Header (`site-header`) | Product mark, product name, project name, static top-meta line (local browser, offline, bind address) | Copy and order unchanged from today; the navigation-toggle slot is empty in this ticket and nothing is drawn in its place |
| Navigation (`site-nav`) | "Document types" sidebar: grouped entries (Backlog, Decisions, Documents, Project), selected entry marked and `aria-current="page"` | Same entries, order, and grouping as the approved MOCK-0015 shell |
| Main (`main`) | One `h1` per page, page sub-line, then page content | The list section is a child of main; no chrome inside main |
| Footer (`site-footer`) | Long offline note; short offline note at narrow widths | Copy unchanged |

Rules:

- The shell is the only place chrome exists; list, view, edit, and error bodies are fragments inside it.
- The shell does not render list actions, filters, or headings other than the project chrome; it only provides the four regions and the width class.
- No shell region carries its own width rule; the width class is applied once, on the page wrapper (section 3).
- The header carries no toggle in this ticket, so no button, icon, or placeholder occupies that space.

## 3. Page width classes

| Route property | Rendered class | Layout effect |
| --- | --- | --- |
| `Capped` (default; every route in this ticket) | `class="app"` | Unchanged: content column capped at 1040 px, centred, with the existing horizontal and bottom padding |
| `Full` (reserved: backlog board, mockup preview) | `class="app app--full"` | Named only; **not drawn** and not styled in this ticket |

- The class string is produced by the server. No script reads or writes it; the layout is identical with JavaScript disabled.
- Nothing in the list layout breaks out of the capped column in this ticket. Full-width inner regions (`.bleed`) belong to later tickets.

## 4. Shared list layout

### 4.1 Section structure

1. Section heading: `h2` with the family heading, followed by the count note on the same line (`4 milestones`, `0 milestones`).
2. One table with a header row and one body row per record.
3. No filter controls, create controls, or page-level record actions are added by this layout; they stay with the page that owns them.

The table carries `data-list-layout="shared"`; every header cell carries `scope="col"` and `data-column="<key>"`; every body row carries `data-row-id="<id>"`. Grouped families add `data-child-of="<parent id>"` on child rows; the milestone family is flat and uses none.

### 4.2 Column contract

Fixed order for every family; only the family slot column(s) vary in label and presence.

| Position | Column key | Header label | Content | Width behaviour |
| --- | --- | --- | --- | --- |
| 1 | `id` | `ID` | Record ID as text | No wrap |
| 2 | `title` | `Title` | Record title as a link to the record view, plus the tag line below it (4.3) | Takes the remaining width |
| 3 | `status` | `Status` | Status as a status tag; unset values render muted `(none)` | No wrap |
| 4 | family slot(s) | Declared label, e.g. `Release` for milestones | Slot value as text; unset values render muted `(none)` | No wrap |
| 5 | `created` | `Created` | Created date as stored, shown read-only | No wrap |
| 6 | `updated` | `Updated` | Updated date as stored, shown read-only | No wrap |
| 7 | `actions` | `Actions` | View then Edit (4.4) | No wrap |

- The column set is structural: no family adds, removes, or reorders a structural column, and no slot may reuse a structural key.
- The milestone family declares exactly one slot, `Release`; the release list declares none (Status is the structural column for every family).
- Header labels are fixed strings, not per-family prose. Capitalisation matches the table above.
- Dates show the stored value; the wireframe shortens them to the calendar date for legibility, which is not a change to the rendered text.
- Row order is the existing index order (ascending ID). The layout never reorders rows.

### 4.3 Title cell and tag line

- First line: the record title, a link to `/doc/{id}`. The ID column is not a second link.
- Below the link, the general front-matter metadata, as two separate stacked lines:
  - **labels line** — one tag per label, on a single line that wraps if needed;
  - **references line** — the reference identifiers as plain text, separated by commas, one size step smaller and muted.
- Rules: neither line carries a "Labels:"/"References:" prefix (the tag shape and the muted text distinguish them, and no extra label text is added to the markup model). A line is omitted when it has no values, so a record with no metadata has no tag area at all and no blank space.
- Labels and references are identical for every family (structural front-matter metadata, never a family slot). Values are escaped text; the list does not link references — reference links stay on the record view.
- The tag line is visually subordinate to the title and is not a second heading.

### 4.4 Action cell

- Last column, always present, one cell per row.
- Order: `View`, then `Edit`. A family action, if the family already has one, follows `Edit`; the milestone family has none.
- `View` opens the record view; `Edit` opens the edit form.
- Each link carries an accessible name that includes the record ID — `View MS-021`, `Edit MS-021` — so repeated "View"/"Edit" labels are unambiguous for screen-reader and voice users.
- Links are inline, separated by a space, and have no wrapping icon-only form.
- Archive/delete/create controls are not part of this cell; they remain page-level and unchanged.

### 4.5 Grouped families (declared variant, not shown in this mockup)

Requirements and NFRs keep their hierarchy: the group row appears first, its member rows follow in the same body, each member row carries `data-child-of` pointing at the group row and an indent that does not rely on colour alone. The milestone family is flat and proves only the flat variant; the grouped variant is first exercised by TASK-142.

## 5. Empty state

- When the family has no records, the table is not rendered. The section keeps its heading and count note (`0 milestones`).
- The empty block shows one message line (`No milestones in this project.`) and one help line (`Records are created from the command line; this browser does not create records.`).
- No create control, no zero-value table header, and no disabled filter are shown.
- Copy comes from the list's empty state, not from per-family list markup.

## 6. Narrow treatment (320 CSS px)

- The page height grows instead of the page width: no page-level horizontal scrolling at any width.
- The navigation stays expanded above the main region. There is no automatic narrow-viewport collapse in this ticket; the collapse control is TASK-144.
- The seven-column table keeps its header row and column semantics. At 320 px it scrolls horizontally inside its own container (the table region), so `Created`, `Updated`, and `Actions` are reached by scrolling that region, not the page.
- Table text drops one size step; cell content keeps its no-wrap columns so a scrolled view stays readable.
- The header keeps the mark and product name and hides the top-meta line, as today. The footer switches to the short copy, as today.
- No column is hidden, merged, or relabelled at narrow widths; the stacked-row alternative was rejected because it would require per-cell label markup the row model does not carry.

## 7. Accessibility and interaction baseline

- Semantic table: `scope="col"` on every header cell; header row visually distinct; no layout tables.
- Heading order: one `h1` per page, the list section heading as `h2`.
- Every interactive element is drawn: title links, action links, navigation links, breadcrumb links, form controls on the context views.
- Focus order follows document order; controls have visible labels; the page works with JavaScript disabled (the mockup's script is review navigation only).
- Status is not conveyed by colour alone (tag with text).
- Counts and empty copy are plain text, not images.

## 8. Views in the review package (MOCK-0016, web)

| View id | Screen | Role |
| --- | --- | --- |
| `list-milestones` | Milestone list in the shell | The feature: shell regions, seven-column table, Title-cell tag line, action cell |
| `view-milestone` | Milestone record view in the shell | Context: unchanged content inside the same shell |
| `edit-milestone` | Milestone edit form in the shell | Context: unchanged fields inside the same shell |
| `list-milestones-empty` | Milestone list, no records | Empty state |
| `narrow-320` | Milestone list at 320 CSS px | Narrow treatment |

## 9. MOCK register

| MOCK | File | Platform | Views | Status |
| --- | --- | --- | --- | --- |
| MOCK-0016 | [`mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html`](mockups/MOCK-0016-browser-shell-and-shared-list-layout-review.html) | web | 5 (list, record, edit, empty, 320 px) | `draft` — pending user approval |

MOCK-0016 reuses the approved MOCK-0015 shell (navigation grouping, header copy, footer copy) and supersedes the list-row treatment of MOCK-0001..MOCK-0015 for every family that migrates to the shared layout; it does not revise or renumber them. `browser-final-cross-type-ui.md` §5 is reconciled in this ticket; per-family UI documents are reconciled in TASK-142.

## 10. Assumptions

- Example records are illustrative; widths assume a project name and IDs of realistic length.
- The 1040 px cap, the sidebar column width, header copy, and footer copy are unchanged by this ticket.
- Dates are shown as stored; the wireframe's date-only form is a legibility choice, not a formatting rule.
- The status tag uses the configured status text as stored.
- A record without a slot value or date shows muted `(none)`, matching how dates are already shown today.
- References are plain text in the list and become links on the record view only.

## 11. Open questions

- **OQ-1 (reference family)** — resolved by the ADR-0028 amendment of 2026-11-15: milestone list. The feature specification §8 still lists this as open; the ADR governs (see the discrepancy note below).
- **OQ-2 (`created_date`)** — resolved by the same amendment: `Created` is a structural column between the family slot(s) and `Updated`.
- **OQ-3 (release slot)** — resolved by the same amendment: the release family declares no slot.
- **OQ-4 (narrow treatment)** — scroll container chosen (section 6). Reviewer confirmation requested.
- **OQ-5 (empty copy)** — the family-neutral message plus the existing help line (section 5). Reviewer confirmation requested.

## 12. Approval status

| Item | Status |
| --- | --- |
| MOCK-0016 | `draft` — awaiting explicit user approval |
| This UI description | `draft` |
| Registry row | `draft` |

## 13. Specification alignment

`docs/specifications/browser-shell-shared-list-layout-and-page-width.md` was patched by a tech-writer while this package was produced. It now states the amended contract (seven columns including `Created`, release → no slot, per-record accessible names, OQ-1..OQ-3 resolved) and matches this document and MOCK-0016. Earlier in that concurrent patch the page still carried the pre-amendment order and the open questions; no discrepancy remains at handoff.

One detail the mockup follows from the specification's markup contract (§5.2): the Title cell holds the title link, a labels list, and a references list as separate elements, not one prefixed sentence.
