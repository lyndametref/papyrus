# AGENTS

## Purpose

This file list available agents and give general rules for the agents.

## Available Agents

- `android-dev`: Implement and improve Android/Kotlin code with emphasis on maintainability, testability, and security.
- `backend-dev`: Implement backend features with Spring Boot, PostgreSQL, and REST APIs, with emphasis on maintainability, testability, and security.
- `frontend-dev`: Implement frontend web features with Alpine.js and vanilla JavaScript, consuming REST APIs, with emphasis on simplicity, light payloads, and security.
- `rust-dev`: Implement Rust features and fixes with emphasis on long-term maintainability, testability, and security.
- `architect`: Define and evolve system architecture, boundaries, and technical design decisions.
- `music-engraver`: Write and edit LilyPond scores for any instrument, following the `music-engraving-piano` and `music-engraving-single-voice` formatting guidelines. Primary agent.
- `business-analyst`: Analyze business needs, clarify business rules, and align deliverables with stakeholder intent. Owns analysis, gap detection, and business rule management.
- `domain-modeler`: Create and maintain domain model artifacts: Mermaid state diagrams, structured attribute lists, and business process diagrams. Invoked at project start to produce at least a context map and a business object model.
- `task-refiner`: Refine backlog tickets into self-contained, implementation-ready work items: clarify scope, author feature specifications, delegate UI designs, and prepare testable acceptance criteria.
- `encyclopedist`: Primary research agent outside the software workflow: collect reliable knowledge from the internet with citations, neutral contradiction reporting, and bias disclosure.
- `key-account-manager`: Primary customer-feedback intake working in-session only (no feedback documents): capture raw feedback verbatim, triage it, group it into themes, and hand a structured package to `requirements-engineer` on explicit user order, then forget the handed-over items.
- `planner`: Backlog planning subagent for the software workflow, launched by `orchestrator`: maintain the feature roadmap, apply user feature priority, and derive milestone/task order from the roadmap and dependencies. Other primary agents (for example `music-engraver`) are outside this workflow.
- `orchestrator`: Primary-only execution owner for the software workflow: read-only with respect to project files (only backlog status transitions via CLI), route new features/requirements through `requirements-engineer` and `planner`, then drive tasks, milestones, and releases through refinement, implementation, quality review, release security audit, and user approval.
- `security-officer`: Independent read-only security auditor at the end of each release; reports severity-ranked findings and a Pass / Pass with findings / Block recommendation.
- `quality-gatekeeper`: Review completed work for correctness, completeness, risk, and acceptance-criteria compliance.
- `requirements-engineer`: Produce and maintain clear, traceable, and consistent requirements artifacts.
- `tech-writer`: Write and maintain technical documentation, specifications, and architecture guidance.
- `terminologist`: Maintain canonical business terminology and enforce wording consistency across docs and backlog.
- `ui-designer`: Design single-file clickable HTML review packages per ticket (review landing with ticket + one button per platform, clickable per-platform prototypes of the current feature's happy path) and UI description documents from requirements and specifications: numbered MOCK-xxxx artifacts under docs/specifications/ui/mockups/ and descriptions under docs/specifications/ui/, referenced by tickets and specs. Layout only, no technology or library details.

## Write Scopes

Agent frontmatter permissions are authoritative. Default write scopes:

- `requirements-engineer`: `docs/requirements/**`
- `business-analyst`: `docs/specifications/**`, `docs/business-rules/**`
- `domain-modeler`: `docs/domain-model/object-model/**`, `docs/business-processes/**`
- `task-refiner`: `docs/specifications/**`
- `architect`: `docs/architecture/**`, `docs/domain-model/**`, `docs/requirements/non-functional/**`
- `terminologist`: `docs/domain-model/business-glossary/**`
- `tech-writer`: `docs/specifications/**`
- `ui-designer`: `docs/specifications/ui/**`
- `android-dev`: `apps/android/**`
- `backend-dev`: `apps/backend/**`
- `frontend-dev`: `apps/webFrontend/**`
- `rust-dev`: `apps/rust/**`
- `planner`: backlog planning only (roadmap, priority, milestone/task order); no direct backlog file edits.
- `orchestrator`: execution orchestration only; no direct backlog file edits, no source or documentation edits.
- `security-officer`: read-only.
- `quality-gatekeeper`: read-only.

## Available Skills

- `architecture-decision-record`: Create and manage Architecture Decision Records (ADRs) in `docs/architecture/adr/`, regenerating the visual index after every change.
- `adr-index-visualization`: Regenerate `docs/architecture/adr/index.html` — a list + detail visualization of the ADR corpus — from the ADR markdown files, keeping it in sync after ADR changes.
- `attribute-list-definition`: Create a structured attribute list for a business object or event, documenting each attribute with name, type, optionality, constraint, and requirement traceability. Stored in docs/domain-model/object-model/<entity-name>-attributes.md.
- `backlog-md`: Manage project tasks, docs, and decisions using the Backlog.md CLI tool. Create, edit, search, and track tasks with full metadata support.
- `business-glossary-management`: Manage the business glossary: add and update business terms, merge duplicates, and keep wording consistent across docs and backlog. Use for business terminology only.
- `business-glossary-index-visualization`: Regenerate docs/domain-model/business-glossary/index.html — a searchable detail view of business glossary terms — from one-Markdown-file-per-term glossary files. Use after any glossary term file change.
- `business-process-diagramming`
- `business-requirements-management`: Create, update, reorganize and check consistency of business requirements under docs/requirements/.
- `business-rules-management`: Create, update, and validate business rules in the canonical format. Each business rule is a self-contained file under docs/business-rules/BR-XXX-short-name.md with unique BR-XXX ID, description, rationale grounded in requirements, and applicability statement.
- `business-rules-index-visualization`: Regenerate docs/business-rules/index.html — a searchable list and detail view of business rules — from BR-XXX Markdown files after rule changes.
- `business-terminology-consistency-check`: Check docs and backlog for non-canonical business terms, conflicting term variants, and missing glossary alignment. Use when validating wording consistency before or after glossary changes.
- `key-account-management`: Capture raw customer feedback in-session only (no files), triage it (open, already done, duplicate, in progress, and more), group it by theme, and hand a structured package to `requirements-engineer` on explicit user order; handed-over items are forgotten.
- `task-orchestration`: Orchestrate end-to-end implementation of a single backlog task under the `orchestrator`: refine via @task-refiner, delegate UI mockups to @ui-designer when needed, move through `Initiated`/`Refined`/`Implemented`/`Validated`, delegate implementation to @android-dev, @backend-dev, @frontend-dev, or @rust-dev per sub-task goals, review via @quality-gatekeeper, and request user approval before `Done`.
- `milestone-orchestration`: Drive every eligible task in one milestone to `Done` by invoking `task-orchestration` per task, respecting the milestone boundary, dependencies, priority, and approval gates; stop on blockers.
- `release-orchestration`: Close a release: verify all release tasks are `Done`, run release-level build and test checks, delegate the mandatory end-of-release audit to `security-officer`, compile the release report, and record completion only after explicit user approval.
- `security-audit`: Read-only security audit of a release or feature covering secrets, dependency supply chain, authentication/authorization, input handling, data protection, platform-specific risks, and security NFR coverage; reports severity-ranked findings and a Pass / Pass with findings / Block recommendation.
- `domain-modeling`: Create and maintain the semantic domain model of the project: Business Object Models, context maps, and business object documentation.
- `html-template-migration`: Migrate Markdown-driven HTML index generators to standalone templates rendered by the shared typed-placeholder script.
- `music-engraving-piano`: Use for creating and editing LilyPond piano scores. Handles multi-staff layout with right and left hands, master .ly file with includes, measure numbering synchronization, breaks in right hand only, and piano-specific score block with optional layout (staff size) and MIDI.
- `music-engraving-single-voice`: Use for creating and editing single-voice LilyPond scores (e.g. recorder, flute, violin, clarinet, trumpet, harp, etc.). Simple single-file layout (no split, no master, no .ily includes), one measure per line with measure number comments, and appropriate score/MIDI/layout setup.
- `music-engraving-musicxml-to-lilypond`: Convert MusicXML scores to LilyPond via musicxml2ly, then apply the matching piano or single-voice score layout.
- `markdown-move`: Move a markdown file to a new location and update all relative links within it to reflect the new path.
- `next-version`: Choose the next semantic version (final release or milestone RC) by analyzing Done tasks in the milestones of the current release; classifies changes as breaking (major), feature (minor), or chore/maintenance (patch); handles RC numbering with reset on major increment.
- `spring-boot-rest`: Design and implement REST APIs with Spring Boot.
- `postgresql-jpa`: Persistence with PostgreSQL and JPA/Hibernate.
- `spring-boot-testing`: Test Spring Boot applications (unit, slice, integration).
- `quality-verification`: Verify a completed backlog ticket against acceptance criteria, definition of done, and implementation plan; produce a read-only completion report with a ready-to-close/incomplete decision.
- `alpine-js`: Alpine.js patterns for lightweight reactive UIs.
- `frontend-testing`: Test vanilla JavaScript and Alpine.js UIs.
- `frontend-style-audit`: Audit frontend (JS, HTML, CSS, Alpine.js) for style, accessibility, and security.
- `cargo-build`: Build, test, and inspect Rust projects with Cargo, Clippy, and rustfmt.
- `rust-style-audit`: Audit Rust code against the official Rust Style Guide, rustfmt, and Clippy.
- `tdd`: Test-driven development workflow applicable to any language (red-green-refactor).
- `gradle-build`: Build, test, and inspect Android/Gradle projects.
- `kotlin-style-audit`: Audit Kotlin code for Google Kotlin style guide compliance.
- `non-functional-requirements-management`: Create, update, reorganize and check consistency of non-functional     requirements under docs/requirements/non-functional/.
- `project-initialization`: Used only by `planner` to gather specialist intake needs, scaffold approved documentation folders and indexes, and verify Backlog.md readiness.
- `requirements-index-visualization`: Regenerate index.html files — the tree + detail visualization of the requirements corpus — by running the skill's bundled scripts/generate_index.sh against the project docs/requirements. Use after any change to REQ/NFR files to keep the visual indexes in sync.
- `reverse-engineering`: Read existing source code and produce the business (REQ) and non-functional (NFR) requirements that correspond to the implemented behavior.
- `state-diagram-creation`: Create Mermaid state diagrams for domain objects with lifecycle states, statuses, and transitions. Each diagram is stored in docs/domain-model/ and shows valid states, transitions, and the conditions that trigger them.
- `ui-clickable-mockup`: Generate one self-contained HTML review package per ticket — review landing with the ticket and one button per platform, clickable per-platform prototypes of the current feature's happy path — saved as MOCK-XXXX-<slug>.html under docs/specifications/ui/mockups/.
- `task-refinement`: Audit, clarify, and complete a single backlog ticket by removing ambiguity, filling missing structure, and aligning it with specifications. Uses the backlog CLI for all ticket modifications.

## Instructions

- Always be concise. Use as few words as possible.
- Clear headings and concise bullets.
- No ambiguous status language (use explicit states like planned, in-progress, blocked, completed).
- If information is missing, list assumptions instead of inventing silent details.
- never use timestamps without timezone. 
- Backlog work: all backlog reads and writes MUST use the Backlog.md CLI via the `backlog-md` skill. Never edit files under `backlog/` directly, except the documented status-list update in the existing Backlog.md config required by `project-initialization`.
- Workflow ownership: `orchestrator` is the primary entry point and owns task, milestone, and release execution; it delegates planning to the `planner` subagent. `quality-gatekeeper` reviews tickets; `security-officer` audits every release.
- New feature/requirement intake: `orchestrator` sends the request to `requirements-engineer` first. After requirements are validated, it calls the `planner` subagent to check and correct the roadmap and task order. Never send a new feature directly to `planner`.
- Orchestration workflow: `workflows/orchestrator/README.md` is the canonical description of task, milestone, and release orchestration.
- Task execution selection (orchestrator): before new work, resume the highest-priority in-progress task. Otherwise take the first item returned by `backlog task list --ready --plain`.
- Backlog planning (`planner` subagent): plan and sequence only; do not execute tasks. Keep in-progress tasks stable unless the user asks to re-plan them.
- Release gate: every release ends with a `security-officer` audit. A `Block` decision (unresolved Critical or High finding) prevents release completion; `Pass with findings` requires documented owners for remaining findings.
- Release convention: a release is a milestone named `Release <version>` containing a release ticket labelled `release`. The orchestrator delegates creation to the `planner` subagent; `release-orchestration` verifies and closes only the release ticket.
- UI mockups: every created or materially revised mockup must be explicitly approved by the user (`approved` in the mockup registry) before implementation.
- Documentation indexes: run the `/regenerate-indexes` command (see `commands/regenerate-indexes.md`) to regenerate all documentation `index.html` files from their Markdown sources. Never hand-edit a generated index.
- Never write anything to `/tmp`, `/var/tmp`, or any system temporary directory. All temporary files and scratchpads MUST be created in the project-local `./.tmp/` directory (the `.tmp` folder in the folder where the agent is running), and MUST always be cleaned up/removed before the task ends, including on failure or when blocked.
- When working on a project's root `README.md`, ensure an AI notice is present at the end of the file (e.g., a short statement that the content was generated or assisted by AI).

## Temporary Files and Scratchpads

- Scope: any intermediate, scratch, or temporary file produced while working, including scratchpads, drafts, dumps, and working notes.
- Location: MUST be written only under `./.tmp/` relative to the folder where the agent is running. Never use `/tmp`, `/var/tmp`, or other system temp locations.
- Cleanup: remove all such files from `./.tmp/` before finishing, even if the task fails or is blocked; do not leave artifacts behind.
- Agents and skills must interpret `./.tmp/` as the sole permitted temporary/scratchpad location.
