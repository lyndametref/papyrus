---
id: ADR-045
title: "Configuration-Driven Document Engine"
created_date: '2026-11-24T19:07:04Z'
updated_date: '2026-11-24T19:07:08Z'
labels: []
references:
  - REQ-007.002
  - REQ-007.003
  - REQ-007.004
  - REQ-005.016
superseded_by: null
---
# ADR-045 Configuration-Driven Document Engine

## Status
accepted
## Decision
This ADR replaces the six records that governed document-type configuration during the descriptor refactor: ADR-0016 (runtime dispatch), ADR-0019 (fail-closed validation), ADR-0024 (list-valued configuration keys), ADR-036 (config-declared field schema and one descriptor model), ADR-041 (seed list slots), and ADR-042 (CLI surface rulings). Their decisions are rescinded as of this record; where a requirement still cites one of them, it cites this ADR.

1. **Configuration is the only source of document types.** A document type exists if and only if `document_types` declares it. An absent slug is absent from commands, views, search, and validation. No command reads a built-in table at runtime.

2. **The Seed is an `init`-time template.** `papyrus init` offers the Seed entries, writes each selected entry into `papyrus.config.yml` in full, writes no unselected entry, and writes every entry when no interactive terminal is available. There is no `enabled` key: presence in the configuration is the only switch.

3. **`model` is a configuration key.** It accepts `SingleMarkdown` (the default when omitted), `GroupedMarkdown`, and `SingleHtmlMockup`. Any configured type may declare any of the three models; grouped and HTML models are not reserved to the Seed. An unknown value is rejected.

4. **Entry keys.** An entry declares `prefix` and `directory`, and accepts `model`, `label`, `plural`, `order`, `summary`, `type_alias`, `reference_visibility`, `statuses`, `default_status`, `slots`, `api_aliases`, `fields`, `group`, `member`, and `field_defaults`. `title`, `content`, and `id_shape` are not keys.

5. **Field items.** A field item declares `key`, at least one of `heading` and `frontmatter_key`, and accepts `option`, `label`, `kind` (`text`, `multiline`, `list`), `required`, `verbs` (a subset of `create` and `edit`), `help`, and `values`. Its position in the sequence is its order in the record body, the command line, and the form. Reserved keys may not be used as a field key or front-matter key, and field keys are unique within a role.

6. **`field_defaults` is a configuration key.** It maps a declared field key to the value written on create when that field is absent. On edit an absent field stays absent. It is not a field-item key, and a key that names no declared field is rejected.

7. **Omitted attributes take model-level defaults, never seed values.** The defaults are `[active, archived]` statuses, no `default_status`, `visible` reference visibility, a slug-derived label and plural, an `order` after the entries that declare one, and empty summary, aliases, slots, and field defaults. A `SingleMarkdown` entry that omits `fields` declares one optional multiline `description` field under `## Description`.

8. **Validation splits unknown from invalid.** An invalid value of a known attribute is rejected, fail-closed, with every diagnostic reported in one deterministic pass and addressed by the selector path `document_types.<slug>.<path>`. An unknown attribute is accepted, ignored, and reported as one warning, and `papyrus check` reports it as a finding. A configuration that still carries `enabled` therefore loads with a warning.

9. **Configuration commands.** `config set` changes a declared entry. Adding or removing a document type, and adding, removing, or reordering a field, is a configuration file edit. `config unset` removes a key so the model-level default applies. List-valued keys keep comma syntax and flow-sequence storage; `fields` and the role field sequences are never list-valued.

10. **Dispatch comes from the configuration.** The command tree is built from the validated configuration at startup. A type's verbs derive from its model; its options derive from its field sequence for the record's role; the slug is the command name, and a slug that collides with a built-in command is rejected when the configuration loads. `papyrus <slug> <ID>` with no verb means `view`.

11. **No per-type code.** No per-type module, static table, or code path defines a type's model, fields, options, or verbs; the descriptor is the only definition. Configuration is data and is never executable.

12. **Slots are template content.** `init` writes each type's slot set into its entry. At runtime, slots come from the entry and default to empty when omitted; no command reads a slot table.

13. **Seed content is specified by requirement.** This ADR fixes the mechanism. The Seed content is REQ-005.016, entry and field validation is REQ-007.002, the three models and their record shapes are REQ-007.004, and the command surface is REQ-007.003. A change to the Seed content is a requirement change, not an architectural one.

14. **No migration is provided.** A project that relied on implicit Seed entries must declare them, and a project that carries `enabled` receives a warning. Papyrus does not rewrite configuration files.
## Context
The descriptor refactor produced six decision records that each fixed one slice of the same mechanism. Read together they no longer describe the product: three of them assume the Seed is resolved at runtime, two reserve the grouped and HTML models to the Seed, four require unknown configuration keys to be rejected, and one treats `field_defaults` as data the configuration cannot carry. The product decisions of 2026-11-23 changed the mechanism: the configuration became the only source of document types, the Seed became an initialization template, the document model became configuration data, field defaults became configuration data, and unknown attributes became warnings rather than errors. Keeping the six in force would leave the requirements contradicting accepted records and would force every reader to reconcile them.

Options considered:

- **Amend the six in place.** Rejected: their bodies are frozen once accepted, and the changes overturn their premises rather than refine them.
- **Six replacements, one per record.** Rejected: the six overlap on every axis (seed resolution, validation, dispatch, slots), and the overlap is what produced contradictory statements in the first place.
- **One consolidated record, superseding all six.** Chosen: one statement of the mechanism, one place for the requirements to cite, and no residual overlap.

Forces: the requirements must be the behavioural source of truth while the ADR carries the mechanism; the configuration must be able to describe any document type without a code change; validation must stay fail-closed for values while tolerating unknown attributes; and the corpus must not accumulate one record per refactoring step.

The resulting split of responsibility is deliberate: requirements state what the product does (Seed content, entry and field validation rules, model shapes, command surface), and this ADR states how the engine realises it (resolution, defaults, validation order, dispatch, no per-type code).
## Consequences
Positive:

- One record states how document types are configured, validated, and dispatched.
- Any document type is expressible in configuration, including grouped and HTML types, with no code change.
- A configuration that carries an unknown attribute still works and reports the attribute, so a harmless typo or an obsolete key does not block every command.
- The Seed is reduced to an initialization convenience, so a project that removes a type really removes it.
- Requirements cite one ADR instead of reconciling six.

Negative and migration:

- Existing configurations that relied on implicit Seed entries must declare those entries; the Seed is no longer resolved at runtime.
- `enabled` becomes a warning with no effect: every declared entry is active, and removing a type means deleting its entry.
- `init` writes full entries, so a generated configuration is large; the user trims it.
- Slots must be written by `init`; a hand-written entry without `slots` has none.
- `field_defaults` moves from seed data into configuration, so a project can now set its own create-time defaults.
- Requirements that cite ADR-0016, ADR-0019, ADR-0024, ADR-036, ADR-041, or ADR-042 by decision number need their traceability re-pointed; the superseded records keep their bodies for history.
- ADR-039 and other accepted records that cite ADR-036 remain frozen and now point at a superseded record; their references resolve through the supersession link.
- The configuration metric is unchanged: no new dependency, no new runtime input, and validation stays fail-closed for values.

Validation:

- The effective configuration is validated once at load and once after every write of a command that changes it; the command tree, the registry, and the browser surface are built from that one snapshot.
- `papyrus check` reports unknown attributes as warnings and does not fail because of them.
