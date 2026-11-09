# REQ-06.005 Browser Skills Page for Agent Workflow Configuration

## User Story
As an agent or developer configuring Papyrus for a new project via AI workflow, I want a browser page that lists available skills/tools for using Papyrus so that I can configure the workflow correctly.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
Browser mode should include a dedicated, navigable page listing the skills and workflows agents use to configure and operate Papyrus on a new project. The page presents agent-facing instructions that account for the effective `papyrus.conf.yaml` (prefixes, `id_padding`, statuses, enabled document/decision types, directories, and paths) so that generated instructions are precise for the configured project. The page is served by the local web interface and is reachable from its navigation. Its route, navigation entry, and format are design decisions left to implementation, consistent with REQ-06.003.

## Acceptance Criteria
- The local web interface serves a page listing all available Papyrus agent skills/workflows.
- Each skill entry includes instructions on how to use Papyrus for agents (not end users).
- Skill content is derived from or parameterized by the effective project configuration (`papyrus.conf.yaml`), including configured prefixes, `id_padding`, statuses, enabled document/decision types, and directories, so instructions match the project.
- The page is accessible via browser mode and navigable within the local web interface.
- Skills cover, at minimum: new project setup/init, backlog/task/milestone/release management, requirements and documentation management, search, browser usage, and release/audit workflows.
- The page does not require an internet connection (NFR-01.003).
- The set of skills/workflows covered and the page route/navigation entry are documented; content format (list vs. copyable snippets) is a design decision.

## Open Questions
- Skill content/format: list only vs copyable snippets.
- Derived vs static: whether skill content is derived from configuration or stored as static snippets.
- Page location/navigation: route and navigation entry within the local web interface.
- Which agent workflows to cover (new project setup, backlog, requirements, docs, release, and others).

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 7 (medium), raw quote:
  - "browser mode a page with a list of skill on how to use papyrus for agents. this will help to configure ai workflow on new project. the skills presented there are considering the configuration in papyrus.conf.yaml to give precise instructions to agents"
- Affected area: Browser / agent integration.
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.003 — Describe Managed Artifacts and Their Lifecycles in the Browser (route/format left to design)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- BR-016 — Configuration Keys and Precedence (effective `papyrus.conf.yaml` inputs)
