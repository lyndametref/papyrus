# BR-017: Papyrus tech-spec Command Behavior

## Description

The papyrus tech-spec command performs the behavior stated in its Papyrus manpage entry (README.md, papyrus(1) → COMMANDS → tech-spec); behavior not stated there is unspecified.

## Rationale

Based on REQ-05.009 — Manage Technical Specifications and REQ-05.007 — Manage Documentation Record Lifecycle (and REQ-01.002 — Read and Edit Project Configuration for the document_types.tech-spec key), this rule exists because the documented tech-spec behavior is the contract for papyrus tech-spec; README.md is the sole behavior source (tech-spec entry to be added).

## Applicability

Applies to all invocations of `papyrus tech-spec` and the project data they manage. The excerpt below reproduces the relevant command behavior; heading identifies the source location.

### papyrus(1): tech-spec — `README.md` lines (to be added)

```text
tech-spec

        Manage technical specifications (TECH-...).

        papyrus tech-spec create <title> --description <text>
        papyrus tech-spec list
        papyrus tech-spec view <id>
        papyrus tech-spec edit <id> [--title <title>] [--description <text>] [--editor]
        papyrus tech-spec deprecate <id>
        papyrus tech-spec restore <id>

        create
        Create a technical specification.

           <title>
           Technical specification title (required).

           --description <text>
           Description of the technical specification (required).

        list
        List technical specifications.

        view <id>
        View a technical specification.

        edit <id>
        Edit a technical specification.

           --title <title>
           Update the title.

           --description <text>
           Update the description.

           --editor
           Open the document body in $VISUAL or $EDITOR.

        deprecate <id>
        Move a technical specification to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

        restore <id>
        Return a deprecated technical specification to its original folder. The ID is preserved.
```

### IDs, directories, and configuration

- Technical specification IDs use the `TECH` prefix (for example `TECH-001`), padded to the configured `id_padding`, and remain stable across deprecate/restore cycles.
- Technical specifications are stored under `<doc_directory>/technical/` (`document_types.tech-spec.directory`); deprecated technical specifications are stored under `<doc_directory>/technical/deprecated/`.
- Configuration key: `document_types.tech-spec: { prefix: TECH, directory: technical/, enabled: true }`. A disabled type cannot be created and is hidden from lists and search.

### Sanitization and project-directory boundary

- Technical specifications follow the same input sanitization and project-directory path boundary as other document types.