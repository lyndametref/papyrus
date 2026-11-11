# REQ-06.008 Provide a Browser Configuration View

## User Story
As a project contributor, I want to modify the current project configuration from the browser so that settings can be changed without the CLI.

## Group
REQ-06 Search and Local Web Interface

## Criticality
Should have

## Description
The browser shall provide a configuration view to modify the current project configuration. It uses `papyrus config get/set` to update configuration so that eventual folder moves are properly performed (REQ-01.002). No folder outside the project may be defined (NFR-04.002). The view shows effective values, including values that come from built-in defaults, and marks which keys are explicitly set in `papyrus.config.yml`. Editable keys are exactly the keys `papyrus config set` accepts: the view applies the same keys, validation, and precedence as the CLI and cannot store a value the CLI would reject. No configuration key is treated as secret in the current configuration, and the view does not read or display environment variables (`PAPYRUS_DIR`, `PAPYRUS_CONFIG`, `NO_COLOR`) as editable values.

## Acceptance Criteria
- The local web interface serves a configuration view for the current project.
- Editing configuration uses `papyrus config get/set` so folder moves are performed.
- No path outside the project directory can be defined.
- The view shows effective configuration values, including defaults, and indicates which keys are explicitly set in `papyrus.config.yml`.
- Editable keys are exactly those `papyrus config set` accepts; the view cannot persist a key or value the CLI rejects, and precedence matches the CLI (REQ-01.003).
- The view treats no configuration key as secret in the current configuration and never reads or displays environment-variable values as editable configuration.
- Changing an artifact folder from this view performs the full folder move defined in REQ-01.002.

## Traceability
- Source: customer-feedback handoff — maintainer, Item 8.
- Maintainer decision 2026-11-11: editable keys are exactly those `papyrus config set` accepts (no separate allowlist); effective values are shown read-only as computed, with explicitly set keys marked; no secret handling is required while the configuration has no secret keys; environment-variable values are not shown as editable.
- REQ-01.002 — Read and Edit Project Configuration (folder moves)
- REQ-01.003 — Select Project and Configuration Context (precedence)
- REQ-06.002 — Serve the Local Web Interface
- REQ-06.003 — Describe Managed Artifacts and Their Lifecycles in the Browser (per-artifact configuration is read-only there)
- NFR-01.002 — Display, View, and Edit All Papyrus Content in a Local Browser
- NFR-02.002 — Sanitize Input
- NFR-04.002 — Restrict File Access to the Project Directory
