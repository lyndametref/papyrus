# REQ-07 Browser Skills Page for Agent Workflow Configuration

## User Story
As an agent or developer configuring Papyrus for a new project via AI workflow, I want a browser page that lists available skills/tools for using Papyrus so I can configure the workflow correctly.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Medium

## Description
Browser mode should include a dedicated page listing skills on how to use Papyrus for agents. This page helps configure an AI workflow on a new project by presenting skills that consider configuration in `papyrus.conf.yaml` to give precise instructions to agents. The page should be navigable within browser mode.

## Acceptance Criteria
- The page lists all available skills for Papyrus agents.
- Each skill includes instructions on how to use Papyrus for agents.
- The page references or pulls skill content from `papyrus.conf.yaml` or static snippets.
- The page is accessible via browser mode and navigable within the local web interface.
- Skills cover new project setup, backlog, requirements, docs, release, and other agent workflows.

## Traceability
- Source: customer-feedback handoff — Internal team discussion, Item 7 (medium), raw quote:
  - "browser mode a page with a list of skill on how to use papyrus for agents. this will help to configure ai workflow on new project. the skills presented there are considering the configuration in papyrus.conf.yaml to give precise instructions to agents"
  - Open questions: skill content/format (list only vs copyable snippets), derived vs static from config, page location/navigation, which agent workflows to cover (new project setup, backlog, requirements, docs, release).