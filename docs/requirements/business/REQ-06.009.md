---
id: REQ-06.009
title: "Set Browser Page Title Branding"
created_date: '2026-11-11T19:57:56Z'
updated_date: '2026-11-15T15:47:29Z'
labels: []
references: []
---
# REQ-06.009 Set Browser Page Title Branding

## User Story
As a project contributor, I want the browser page title to emphasize the project name so that the project context is clear.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Could have

## Description
The browser page title shall show the project name in big bold letters and the "Papyrus" mention in smaller letters (currently the opposite). The rule applies to the visible title in the shared page header on every page of the local web interface. The "P" mark keeps its current size and styling; only the typography of the project name and the "Papyrus" mention changes.

## Acceptance Criteria
- The browser page title renders the project name in big bold letters.
- The "Papyrus" mention is rendered in smaller letters.
- The branding applies to the shared header of every browser page.
- The "P" mark keeps its current size.
- The project name shown is the configured project name (REQ-01.002).

## Traceability
- Source: customer-feedback handoff — maintainer, Item 9.
- Maintainer decision 2026-11-11: the rule covers the shared header on all browser pages; the "P" mark is not resized.
- REQ-06.002 — Serve the Local Web Interface
- REQ-01.002 — Read and Edit Project Configuration (`project_name`)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
