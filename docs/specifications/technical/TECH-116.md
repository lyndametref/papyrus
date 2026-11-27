---
id: TECH-116
title: "Browser Routes and Landing Page"
created_date: '2026-11-27T17:36:57Z'
updated_date: '2026-11-27T17:51:48Z'
labels: []
references:
  - REQ-006.014
  - REQ-007.002
  - REQ-007.003
status: verified
---
# TECH-116 Browser Routes and Landing Page

## Description
Covers the browser route table, route resolution, and the landing page. Server-side rendering and rich-content rendering are separate specifications; record list and view content, list layout, and edit behavior are also separate.

## Route table

- Routing uses one fixed route table; no route is derived from user input.
- The table holds these routes:

| Route | Page |
|---|---|
| `GET /` | Landing page: the document-type inventory |
| `GET /types/<type>/records` | List of one configured type |
| `GET /backlog` | The backlog |
| `GET /records/{id}` | Single-record view |
| `GET /records/{id}/edit` | Edit form |
| `POST /records/{id}` | Form submission |
| JSON endpoints | The machine-readable form of the list and view routes |
| Asset routes | The bundled CSS, script, and rendering assets |

- A type in `GET /types/<type>/records` is a configured document-type slug or decision-type slug, or one of the built-in backlog types `task`, `milestone`, and `release`.
- The request target is percent-decoded in a single pass before lookup; the decoded path is matched against the table and is never joined to a filesystem path.
- Record identifiers are validated and resolved by exact match against the active directory listing of the enabled type; a request value is never concatenated into a path.
- An unknown route, an unknown identifier, and a record of an unconfigured type produce the shared not-found page.

## Landing page

- `GET /` is the landing page and the document-type inventory.
- It lists every configured document type in configuration order.
- Each entry shows the type's `record_name`, `plural`, and `summary` from its configuration entry.
- Each entry links to that type's list at `GET /?type=<slug>`.
- A type added by configuration appears on the landing page without a code change.
- The Backlog surface is a separate entry that links to the board; the board's content is a separate specification.
