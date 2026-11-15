# Papyrus

Papyrus is a project-level backlog and documentation manager for Git repositories. It combines task, milestone, release, and decision tracking with long-term documentation (requirements, rules, glossary, ADRs) in one CLI, plus a local web UI and unified search over both stores.

## Inspiration

Papyrus is inspired by and builds on the ideas of **[Backlog.md](https://github.com/MrLesk/Backlog.md)** by **[MrLesk](https://github.com/MrLesk)** — a Markdown-native task manager and Kanban visualizer for Git repositories, MIT-licensed. Huge kudos to MrLesk and the Backlog.md contributors for pioneering git-native, Markdown-backed project collaboration for humans and AI agents. Papyrus's backlog workflow, CLI shape, and documentation model owe a great deal to their work.

---

## Upgrading to Release 1.0

Release 1.0 stops reading the Backlog.md on-disk layout. If your project's backlog records are still stored as `backlog/tasks/task-N - <slug>.md` or `backlog/milestones/m-N - <slug>.md`, run the one-time import before you rely on the new version: no import runs automatically, and the new version does not list, view, or search unimported records.

Run it explicitly: `papyrus import backlog --from <source-project-dir> --dry-run`. Review the printed plan, keep the source store until the import is verified, then run the same command without `--dry-run`. Imported records receive fresh Papyrus IDs and milestone numbers; `papyrus import backlog` is the only bridge from a Backlog.md store. Full instructions, options, caveats, and the rest of the Release 1.0 change summary: [Release 1.0 upgrade notes](docs/release-1.0-upgrade-notes.md).

Release 1.0 also makes document types configuration-defined. Static per-type document commands and the `nfreq` alias are gone, `deploy` is no longer seeded, and document `deprecate`/`restore` no longer exist. If you used any of them, or kept documents under `deprecated/`, follow [Document types are now configuration-defined](docs/release-1.0-upgrade-notes.md#document-types-are-now-configuration-defined).

---

## Man Page (`papyrus(1)`)

Scope: this section is the in-repo command reference. It documents the built-in commands and the document command family generically; the command name for each document type is an enabled `document_types` slug. Config-defined slugs have no man page: ADR-0021 is Proposed and the scope decision is deferred, and NFR-03.003 is not claimed for them.

<pre>
PAPYRUS(1)                      Papyrus Manual                      PAPYRUS(1)

<b style="color: light-dark(#0969da, #58a6ff);">NAME</b>

       papyrus - project-level backlog and documentation manager

<b style="color: light-dark(#0969da, #58a6ff);">SYNOPSIS</b>

       <b>papyrus &lt;command&gt; [options]</b>

       <b>papyrus &lt;command&gt; &lt;subcommand&gt; [options]</b>

<b style="color: light-dark(#0969da, #58a6ff);">DESCRIPTION</b>

       <b>papyrus</b> manages a project's backlog and long-term documentation from a single command-line interface. It combines:

       - **Backlog management** - tasks, milestones, releases, and decisions.
        - **Documentation management** - business and non-functional requirements, glossary terms, UI and technical specifications, audit records, business rules, use cases, test cases, and test reports.

       All project data lives in the project directory selected with <b>--dir</b> (default: the current working directory). IDs are generated automatically and remain stable across status changes and archive. Archived tasks can be restored; document records cannot.

<b style="color: light-dark(#0969da, #58a6ff);">GLOBAL OPTIONS</b>

       <b style="color: light-dark(#1a7f37, #3fb950);">--json</b>
       Emit stable, machine-readable JSON. Applies to every command that produces output, including errors.

       <b style="color: light-dark(#1a7f37, #3fb950);">--dir &lt;path&gt;</b>
       Project directory. Default: the current working directory.

       <b style="color: light-dark(#1a7f37, #3fb950);">--config &lt;file&gt;</b>
       Configuration file. Default: <b>papyrus.config.yml</b> in the project directory.

       <b style="color: light-dark(#1a7f37, #3fb950);">--no-color</b>
       Disable ANSI color output. The <b>NO_COLOR</b> environment variable has the same effect.

       <b style="color: light-dark(#1a7f37, #3fb950);">--version</b>
       Print the version and exit.

       <b style="color: light-dark(#1a7f37, #3fb950);">-h, --help</b>
       Print help and exit. Use <b>papyrus &lt;command&gt; --help</b> for command help.

<b style="color: light-dark(#0969da, #58a6ff);">COMMANDS</b>

<b style="color: light-dark(#0969da, #58a6ff);">init</b>

       Initialize a project in the directory where <b>papyrus init</b> is run. If <b>--dir &lt;path&gt;</b> is given, initialize in that directory instead; the directory is created if it does not exist. Initialization creates the backlog store, the documentation structure, and <b>papyrus.config.yml</b>.
       <code style="font-weight: bold;">
       papyrus init &lt;name&gt; [--no-git] [--force]
       </code>
       <b>&lt;name&gt;</b> is the project name stored in the configuration.

       In an interactive terminal, <b>papyrus init</b> walks through all available artifacts (task, milestone, release, decision types, and document types), lets you choose which to activate, and prompts for the ID prefix and storage folder of each activated artifact. Core artifacts (task, milestone, release) are always activated. Non-interactive or CI environments (no TTY) use default selections and defaults without prompting. Invalid prefixes or folders are rejected and re-prompted with hints naming the default equivalent value.

       <b style="color: light-dark(#1a7f37, #3fb950);">--no-git</b>
       Do not initialize a Git repository.

       <b style="color: light-dark(#1a7f37, #3fb950);">--force</b>
       Allow initialization in a non-empty directory and replace an existing <b>papyrus.config.yml</b> in full. No other existing file is overwritten.

<b style="color: light-dark(#0969da, #58a6ff);">config</b>

       Read and edit the configuration. Keys use dotted notation for nested values, for example <b>document_types.req.enabled</b>.
       <code style="font-weight: bold;">
       papyrus config list
       papyrus config get &lt;key&gt;
       papyrus config set &lt;key&gt; &lt;value&gt;
       papyrus config unset &lt;key&gt;
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       Print the effective keys as dotted <b>key: value</b> lines.

       <b style="color: light-dark(#8250df, #d2a8ff);">get &lt;key&gt;</b>
       Print one effective value.

       <b style="color: light-dark(#8250df, #d2a8ff);">set &lt;key&gt; &lt;value&gt;</b>
       Persist one value. For a scalar key the value is parsed as a YAML scalar, so a comma is ordinary data.

       The list-valued keys <b>statuses</b> and <b>document_types.&lt;slug&gt;.statuses</b> take a comma-separated value, for example <b>papyrus config set statuses "ToDo, Initiated, Done"</b>. Each element is trimmed and the result is stored in <b>papyrus.config.yml</b> as a YAML flow sequence (<b>statuses: [ToDo, Initiated, Done]</b>); <b>config get</b> and <b>config list</b> print the same flow form. An empty or whitespace-only value, an element that is empty after trimming, and a value containing a control character are rejected and nothing is written. Commas, quotes, and backslashes are not escaped: every comma separates elements, and a value that must contain a comma can only be written by editing <b>papyrus.config.yml</b> directly. The resulting configuration is validated before it is written; an invalid list is refused and the file is unchanged.

       <b style="color: light-dark(#8250df, #d2a8ff);">unset &lt;key&gt;</b>
       Remove one key and fall back to its default.

       Precedence is described in papyrus-config(5).

<b style="color: light-dark(#0969da, #58a6ff);">browser</b>

       Serve the local web UI over both the backlog and documentation stores.
       <code style="font-weight: bold;">
       papyrus browser [--host &lt;host&gt;] [--port &lt;port&gt;] [--no-open] [--debug]
       </code>
       <b style="color: light-dark(#1a7f37, #3fb950);">--host &lt;host&gt;</b>
       Bind address. Default: <b>127.0.0.1</b>. A non-loopback address is allowed but prints a warning.

       <b style="color: light-dark(#1a7f37, #3fb950);">--port &lt;port&gt;</b>
       TCP port. Default: <b>6420</b>.

       <b style="color: light-dark(#1a7f37, #3fb950);">--no-open</b>
       Do not open the default browser automatically.

       <b style="color: light-dark(#1a7f37, #3fb950);">--debug</b>
       Browser-only debugging flag. Logs one line per received request to standard error with the method, path, response status, and duration. Request headers and bodies are never logged. Errors are printed whether or not the flag is set.

<b style="color: light-dark(#0969da, #58a6ff);">import</b>

        One-way import of Backlog.md task and milestone records into the Papyrus-native layout. This is a one-time bridge: it reads a Backlog.md store read-only, allocates fresh Papyrus IDs, and writes Papyrus records. Re-running the command against the same source allocates new numbers and duplicates records. Backlog.md `docs`, `drafts`, and `decisions` are not imported; only `backlog/tasks`, `backlog/milestones`, and their `archive/` counterparts are in scope.
        <code style="font-weight: bold;">
        papyrus import backlog --from &lt;source-project-dir&gt; [--source-backlog-dir &lt;dir&gt;] [--dry-run] [--allow-existing] [--report &lt;path&gt;]
        </code>
        <b style="color: light-dark(#1a7f37, #3fb950);">--from &lt;source-project-dir&gt;</b>
        Backlog.md project root. Required.

        <b style="color: light-dark(#1a7f37, #3fb950);">--source-backlog-dir &lt;dir&gt;</b>
        Source backlog directory. Default: <b>&lt;source-project-dir&gt;/backlog</b>.

        <b style="color: light-dark(#1a7f37, #3fb950);">--dry-run</b>
        Perform planning and ID allocation and print the plan and report without writing anything.

        <b style="color: light-dark(#1a7f37, #3fb950);">--allow-existing</b>
        Required when the target already holds records of a family being imported. Without it the command refuses before writing.

        <b style="color: light-dark(#1a7f37, #3fb950);">--report &lt;path&gt;</b>
        Write the same report to a path inside the project directory only. A path outside the project directory is a validation error.

        <b>One-way caveat:</b> the source is opened read-only and left byte-identical. No mapping file, sidecar, marker, or extra front-matter field is written; the printed plan is the only record of the source-to-Papyrus mapping. Imported records receive fresh Papyrus IDs (task IDs and milestone numbers are reallocated; the source `m-` number is not preserved). Keep the source store until the import is verified; after import Papyrus owns the records. There is no reverse export and no re-import reconciliation.

        <b>Excluded scope:</b> Backlog.md `docs`, `drafts`, `decisions`, configuration, and locks are not read as records.

<b style="color: light-dark(#0969da, #58a6ff);">task</b>

        Manage tasks (<b>TASK-001</b>, ...). A task may be assigned to zero or one milestone, may depend on zero or more tasks, and may reference zero or more business requirements, non-functional requirements, UI specifications, technical specifications, audit records, and business rules.
       <code style="font-weight: bold;">
       papyrus task create &lt;title&gt; [options]
       papyrus task list [options]
       papyrus task view &lt;id&gt;
       papyrus task &lt;id&gt;
       papyrus task edit &lt;id&gt; [options]
       papyrus task archive &lt;id&gt;
       papyrus task restore &lt;id&gt;
       papyrus task delete &lt;id&gt; [--force]
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a task.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Task description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--ac &lt;criteria&gt;</b>
          Acceptance criterion. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--depends-on &lt;id&gt;</b>
           Task dependency. Repeatable; tasks may depend on zero or more tasks.

          <b style="color: light-dark(#1a7f37, #3fb950);">--milestone &lt;id&gt;</b>
          Parent milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--label &lt;label&gt;</b>
          Label. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--assignee &lt;name&gt;</b>
          Assignee.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List tasks.

          <b style="color: light-dark(#1a7f37, #3fb950);">-s, --status &lt;status&gt;</b>
          Filter by status. See STATUSES.

          <b style="color: light-dark(#1a7f37, #3fb950);">--milestone &lt;id&gt;</b>
          Filter by milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--release &lt;id&gt;</b>
          Filter by release. Collects tasks belonging to milestones that belong to the release.

          <b style="color: light-dark(#1a7f37, #3fb950);">--label &lt;label&gt;</b>
          Filter by label.

          <b style="color: light-dark(#1a7f37, #3fb950);">--assignee &lt;name&gt;</b>
          Filter by assignee.

          <b style="color: light-dark(#1a7f37, #3fb950);">--search &lt;text&gt;</b>
          Filter by text.

          <b style="color: light-dark(#1a7f37, #3fb950);">--limit &lt;n&gt;</b>
          Maximum number of results.

          <b style="color: light-dark(#1a7f37, #3fb950);">--sort &lt;field&gt;</b>
          Sort field, for example <b>id</b>, <b>created</b>, or <b>status</b>.

          <b style="color: light-dark(#1a7f37, #3fb950);">--watch</b>
          Keep the list open and refresh on changes.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a task.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a task.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--ac &lt;criteria&gt;</b>
          Add an acceptance criterion. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--comment &lt;text&gt;</b>
          Add a comment.

          <b style="color: light-dark(#1a7f37, #3fb950);">--comment-author &lt;name&gt;</b>
          Set the comment author.

          <b style="color: light-dark(#1a7f37, #3fb950);">--status &lt;status&gt;</b>
          Set the status.

          <b style="color: light-dark(#1a7f37, #3fb950);">--depends-on &lt;id&gt;</b>
          Add a dependency. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--milestone &lt;id&gt;</b>
          Set the parent milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--no-milestone [&lt;id&gt;]</b>
          Remove the specified milestone; without an ID, remove all milestone assignments.

          <b style="color: light-dark(#1a7f37, #3fb950);">--label &lt;label&gt;</b>
          Add a label. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--assignee &lt;name&gt;</b>
          Set the assignee.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the task body in $VISUAL or $EDITOR. All repeatable options add to the existing set; field flags update metadata without launching an editor.

       <b style="color: light-dark(#8250df, #d2a8ff);">archive &lt;id&gt;</b>
       Move a task off the board. The ID and every dependency are preserved.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Return an archived task to the board with its previous status.

       <b style="color: light-dark(#8250df, #d2a8ff);">delete &lt;id&gt; [--force]</b>
       Permanently delete a task. An archived task can be deleted; an active task is archived first.

          <b style="color: light-dark(#1a7f37, #3fb950);">--force</b>
          Skip confirmation prompt.

       <b>task view &lt;id&gt;</b> and <b>task &lt;id&gt;</b> are equivalent.

<b style="color: light-dark(#0969da, #58a6ff);">milestone</b>

       Manage milestones (<b>MS-001</b>, ...). A milestone may be assigned to zero or one release.
       The milestone lifecycle is fixed: <b>open</b> -&gt; <b>closed</b>, transitioned only by
       <b>papyrus milestone close &lt;id&gt;</b>.
       <code style="font-weight: bold;">
       papyrus milestone create &lt;title&gt; [-d &lt;description&gt;] [--release &lt;id&gt;]
       papyrus milestone list
       papyrus milestone view &lt;id&gt;
        papyrus milestone edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;description&gt;] [--release &lt;id&gt;] [--no-release [&lt;id&gt;]]
       papyrus milestone close &lt;id&gt;
       papyrus milestone delete &lt;id&gt; [--force]
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Milestone description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--release &lt;id&gt;</b>
          Assign the milestone to a release.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List milestones.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a milestone.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--release &lt;id&gt;</b>
          Assign the milestone to a release.

          <b style="color: light-dark(#1a7f37, #3fb950);">--no-release [&lt;id&gt;]</b>
          Remove the specified release; without an ID, remove all release assignments.

       <b style="color: light-dark(#8250df, #d2a8ff);">close &lt;id&gt;</b>
       Close a milestone. This is the only <b>open</b> -&gt; <b>closed</b> transition;
       milestones in any other state are rejected.

       <b style="color: light-dark(#8250df, #d2a8ff);">delete &lt;id&gt; [--force]</b>
       Permanently delete a milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--force</b>
          Skip confirmation prompt.

<b style="color: light-dark(#0969da, #58a6ff);">release</b>

       Manage releases (<b>REL-001</b>, ...). A release may contain multiple milestones; each milestone may be assigned to zero or one release.
       <code style="font-weight: bold;">
       papyrus release create &lt;title&gt;
       papyrus release list
       papyrus release view &lt;id&gt;
       papyrus release edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;description&gt;]
       papyrus release close &lt;id&gt;
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a release.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List releases.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a release.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a release.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

       <b style="color: light-dark(#8250df, #d2a8ff);">close &lt;id&gt;</b>
       Close a release. This is the only <b>planned</b> -&gt; <b>released</b>
       transition; releases in any other state are rejected.

<b style="color: light-dark(#0969da, #58a6ff);">decision</b>

       Manage decisions (<b>ADR-001</b>, <b>BDR-001</b>, ...). Decisions may reference zero or more business requirements and zero or more non-functional requirements, and may be superseded by zero or one decision of the same type.
       <code style="font-weight: bold;">
        papyrus decision create &lt;title&gt; [-t, --decision-type &lt;type&gt;] [-d &lt;decision&gt;] [--context &lt;text&gt;] [--consequences &lt;text&gt;]
        papyrus decision list [-t, --decision-type &lt;type&gt;]
       papyrus decision view &lt;id&gt;
       papyrus decision edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;decision&gt;]
                                     [--status &lt;status&gt;] [--context &lt;text&gt;] [--consequences &lt;text&gt;]
                                     [--superseded-by &lt;id&gt;]
       papyrus decision archive &lt;id&gt;
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a decision.

           <b style="color: light-dark(#1a7f37, #3fb950);">-t, --decision-type &lt;type&gt;</b>
          Decision type. Built-in types: <b>adr</b> (default) and <b>bdr</b>. Additional types are configured in papyrus-config(5).

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --decision &lt;text&gt;</b>
          The decision which was taken.

          <b style="color: light-dark(#1a7f37, #3fb950);">--context &lt;text&gt;</b>
          Context of the decision.

          <b style="color: light-dark(#1a7f37, #3fb950);">--consequences &lt;text&gt;</b>
          Consequences of the decision.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List decisions.

           <b style="color: light-dark(#1a7f37, #3fb950);">-t, --decision-type &lt;type&gt;</b>
           Filter by decision type.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a decision.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a decision.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --decision &lt;text&gt;</b>
          The decision which was taken.

          <b style="color: light-dark(#1a7f37, #3fb950);">--status &lt;status&gt;</b>
          Set the status.

          <b style="color: light-dark(#1a7f37, #3fb950);">--context &lt;text&gt;</b>
          Context of the decision.

          <b style="color: light-dark(#1a7f37, #3fb950);">--consequences &lt;text&gt;</b>
          Consequences of the decision.

          <b style="color: light-dark(#1a7f37, #3fb950);">--superseded-by &lt;id&gt;</b>
           Mark this decision as superseded by at most one other decision of the same type. Cannot be combined with --status.

       <b style="color: light-dark(#8250df, #d2a8ff);">archive &lt;id&gt;</b>
       Archive a decision.

       Decision records follow the Nygard ADR template: Title, Status, Decision, Context, and Consequences. Statuses are <b>proposed</b>, <b>accepted</b>, <b>rejected</b>, <b>deprecated</b>, and <b>superseded</b>.

<b style="color: light-dark(#0969da, #58a6ff);">&lt;document-slug&gt;</b>

        Manage documents of a configured type. The command name is the enabled
        <b>document_types</b> key. <b>papyrus --help</b> lists the enabled slugs
        and <b>papyrus &lt;document-slug&gt; --help</b> lists that type's verbs.
        <b>req</b> and <b>nfr</b> exist only because they are default entries.

       <code style="font-weight: bold;">
       papyrus &lt;document-slug&gt; create &lt;title&gt; [options]
       papyrus &lt;document-slug&gt; list
       papyrus &lt;document-slug&gt; view &lt;id&gt;
       papyrus &lt;document-slug&gt; edit &lt;id&gt; [options]
       papyrus &lt;document-slug&gt; archive &lt;id&gt;
       papyrus &lt;document-slug&gt; status &lt;id&gt; &lt;value&gt;
       papyrus &lt;document-slug&gt; delete &lt;id&gt; [--force]
       </code>

        A type with no built-in schema (a user-defined slug) uses the generic
        baseline: <b>create &lt;title&gt; [-d, --description &lt;text&gt;]
        [--label &lt;label&gt;...] [--reference &lt;value&gt;...]</b> and
        <b>edit &lt;id&gt; [--title &lt;title&gt;] [-d, --description &lt;text&gt;]
        [--label &lt;label&gt;...] [--reference &lt;value&gt;...] [--editor]</b>.
        Built-in types keep their typed fields; <b>papyrus &lt;document-slug&gt; --help</b>
        is authoritative for the selected type.

       <b style="color: light-dark(#8250df, #d2a8ff);">status &lt;id&gt; &lt;value&gt;</b>
        Set <b>active</b> or a declared custom status in place. Front-matter-backed
        types have this verb; <b>ui-mockup</b> does not. <b>archived</b> is rejected;
        use <b>archive</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">archive &lt;id&gt;</b>
        Move a document to the <b>archived/</b> subfolder of its type directory.
        The ID is preserved and the previous custom status is replaced by
        <b>archived</b>. Documents have no <b>restore</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">delete &lt;id&gt; [--force]</b>
        Delete a document. An active document is archived first.

          <b style="color: light-dark(#1a7f37, #3fb950);">--force</b>
          Skip confirmation prompt.

        Type-specific additions:

          <b style="color: light-dark(#1a7f37, #3fb950);">req</b> and <b>nfr</b>
          add <b>create-group --title &lt;title&gt; [--story &lt;text&gt;] [-d &lt;description&gt;]</b>;
          <b>create</b> requires <b>--group &lt;id&gt;</b>.

          <b style="color: light-dark(#1a7f37, #3fb950);">ui-mockup</b>
          adds <b>supersede</b> and has no <b>status</b>; its HTML metadata
          states <b>pending</b>, <b>approved</b>, or <b>archived</b>.

        The eleven defaults below are configuration written by <b>papyrus init</b>,
        not a fixed command list. <b>deploy</b> is not seeded; declare it in
        <b>document_types</b> to use <b>papyrus deploy</b>. A hand-written
        <b>document_types</b> map replaces the defaults wholesale, so keep every
        entry you still use.

       <code style="font-weight: bold;">
       req          REQ   requirements/business/
       nfr          NFR   requirements/non-functional/
       glossary     GLO   glossary/
       rule         RUL   business-rules/
       ui-spec      UI    specifications/ui/
       ui-mockup    MOCK  specifications/ui/mockups/
       audit        AUD   audit-records/
       tech-spec    TECH  technical/
       use-case     UC    use-cases/
       test-case    TC    test-cases/
       test-report  TR    test-reports/
       </code>

<b style="color: light-dark(#0969da, #58a6ff);">search</b>

       Search tasks, milestones, releases, decisions, and documents in one pass.
       <code style="font-weight: bold;">
        papyrus search &lt;query&gt; [--type &lt;type&gt;]... [--limit &lt;n&gt;]
        </code>
        <b style="color: light-dark(#1a7f37, #3fb950);">&lt;query&gt;</b>
        Free-text search term or phrase. Quote phrases that contain spaces.

        <b style="color: light-dark(#1a7f37, #3fb950);">--type &lt;type&gt;...</b>
        Repeatable. Restrict to record types: <b>task</b>, <b>milestone</b>, <b>release</b>, <b>decision</b>, or <b>doc</b>. Default: all types.

       <b style="color: light-dark(#1a7f37, #3fb950);">--limit &lt;n&gt;</b>
       Maximum number of results.

<b style="color: light-dark(#0969da, #58a6ff);">STATUSES</b>

<b style="color: light-dark(#0969da, #58a6ff);">Tasks</b>

        <b>ToDo</b> -&gt; <b>Initiated</b> -&gt; <b>Refined</b> -&gt; <b>Implemented</b> -&gt; <b>Validated</b> -&gt; <b>Done</b>

        <b style="color: light-dark(#8250df, #d2a8ff);">ToDo</b>
       Initial status.

       <b style="color: light-dark(#8250df, #d2a8ff);">Initiated</b>
       Scope confirmed.

       <b style="color: light-dark(#8250df, #d2a8ff);">Refined</b>
       Ready for implementation.

       <b style="color: light-dark(#8250df, #d2a8ff);">Implemented</b>
       Work complete.

       <b style="color: light-dark(#8250df, #d2a8ff);">Validated</b>
       Reviewed and accepted.

        <b style="color: light-dark(#8250df, #d2a8ff);">Done</b>
        Completion approved by the user. This is the only terminal status.

<b style="color: light-dark(#0969da, #58a6ff);">Milestones</b>

        <b>open</b> -&gt; <b>closed</b>

        <b>papyrus milestone close &lt;id&gt;</b> is the only
        <b>open</b> -&gt; <b>closed</b> transition. A milestone starts <b>open</b>.

<b style="color: light-dark(#0969da, #58a6ff);">Releases</b>

        <b>planned</b> -&gt; <b>released</b>

        Releases group milestones. <b>papyrus release close &lt;id&gt;</b> is the
        only <b>planned</b> -&gt; <b>released</b> transition.

<b style="color: light-dark(#0969da, #58a6ff);">Decisions</b>

       Nygard ADR statuses:

       <b>proposed</b> -&gt; <b>accepted</b> | <b>rejected</b>
       <b>accepted</b> -&gt; <b>deprecated</b> | <b>superseded</b>

<b style="color: light-dark(#0969da, #58a6ff);">Documents</b>

        Front-matter-backed document types start at <b>active</b> and move to
        <b>archived</b> only through <b>archive</b>:

        <b>active</b> -&gt; <b>archived</b>

        A type may declare additional custom statuses in
        <b>document_types.&lt;slug&gt;.statuses</b>. Custom statuses are ordinary
        front-matter values: <b>status &lt;id&gt; &lt;value&gt;</b> sets one, and
        <b>archive</b> replaces it with <b>archived</b>. <b>status</b> rejects
        <b>archived</b> (exit 4), and documents have no <b>restore</b> and no
        <b>deprecate</b>.

        UI mockups are HTML records whose state lives in file metadata:
        <b>pending</b>, <b>approved</b>, or <b>archived</b>. They have no
        <b>status</b> command.

<b style="color: light-dark(#0969da, #58a6ff);">FILES</b>

        <b style="color: light-dark(#8250df, #d2a8ff);">papyrus.config.yml</b>
        Project configuration. Its location can be overridden with <b>--config</b>
        or <b>PAPYRUS_CONFIG</b>. See papyrus-config(5).

        <b style="color: light-dark(#8250df, #d2a8ff);">backlog/tasks/</b>
        Task records. The <b>backlog_directory</b> setting controls the parent path.

        <b style="color: light-dark(#8250df, #d2a8ff);">backlog/milestones/</b>
        Milestone records. The <b>backlog_directory</b> setting controls the parent path.

        <b style="color: light-dark(#8250df, #d2a8ff);">backlog/releases/</b>
        Release records. The <b>backlog_directory</b> setting controls the parent path.

        <b style="color: light-dark(#8250df, #d2a8ff);">decisions/</b>
        Decision records. The <b>decision_directory</b> setting controls this path.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.&lt;slug&gt;.directory&gt;/</b>
        Active records of the document type whose <b>document_types</b> key is
        <b>&lt;slug&gt;</b>. The key is also the command name.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.&lt;slug&gt;.directory&gt;/archived/</b>
        Archived records of that type. <b>archive</b> is the only command that
        moves a record into this folder.

        A legacy <b>deprecated/</b> folder is inert: it is not read, migrated, or
        inventoried. Move records out by hand; see the Release 1.0 upgrade notes.

<b style="color: light-dark(#0969da, #58a6ff);">ENVIRONMENT</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">PAPYRUS_DIR</b>
       Default project directory. Overridden by <b>--dir</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">PAPYRUS_CONFIG</b>
       Default configuration file. Overridden by <b>--config</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">NO_COLOR</b>
       Disable color output when set to any value.

       <b style="color: light-dark(#8250df, #d2a8ff);">EDITOR</b>, <b>VISUAL</b>
        Editor used by document-type edit commands and other interactive edit commands.

<b style="color: light-dark(#0969da, #58a6ff);">EXIT STATUS</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">0</b>
       Success.

       <b style="color: light-dark(#8250df, #d2a8ff);">1</b>
       Runtime error.

       <b style="color: light-dark(#8250df, #d2a8ff);">2</b>
       Usage error.

       <b style="color: light-dark(#8250df, #d2a8ff);">3</b>
       Record not found.

       <b style="color: light-dark(#8250df, #d2a8ff);">4</b>
       Validation error or conflict.

<b style="color: light-dark(#0969da, #58a6ff);">EXAMPLES</b>

       Initialize a project:
       <code style="font-weight: bold;">
       papyrus init papyrus
       </code>

       Create and list tasks:
       <code style="font-weight: bold;">
       papyrus task create "Write the manpage" -d "Document every command" \
         --ac "Commands are listed" --ac "Examples are present"
        papyrus task list -s ToDo
       </code>

       Move a task through the workflow:
       <code style="font-weight: bold;">
       papyrus task edit TASK-001 --status Initiated
       papyrus task edit TASK-001 --status Done
       </code>

       Create a decision:
       <code style="font-weight: bold;">
        papyrus decision create "Use a single binary" --decision-type adr -d "One CLI, flat commands"
       </code>

       Archive a document and set a declared custom status:
       <code style="font-weight: bold;">
       papyrus req archive REQ-003
       papyrus req status REQ-004 review
       </code>

       Add a document type by editing papyrus.config.yml, then use its command.
       A hand-written document_types map replaces the defaults wholesale, so keep
       every entry you still use:
       <code style="font-weight: bold;">
       document_types:
         # ... your existing entries ...
         deploy: { prefix: DEP, directory: architecture/deployment/, enabled: true }
       </code>
       <code style="font-weight: bold;">
       papyrus deploy list
       </code>

       Search everything:
       <code style="font-weight: bold;">
       papyrus search "release" --type task --type doc
       </code>

<b style="color: light-dark(#0969da, #58a6ff);">SEE ALSO</b>

       papyrus-config(5)
</pre>

## Configuration Man Page (`papyrus-config(5)`)

<pre>
PAPYRUS-CONFIG(5)           Papyrus Manual           PAPYRUS-CONFIG(5)

<b style="color: light-dark(#0969da, #58a6ff);">NAME</b>

       papyrus.config.yml - Papyrus configuration file

<b style="color: light-dark(#0969da, #58a6ff);">SYNOPSIS</b>

       <b>papyrus.config.yml</b>

<b style="color: light-dark(#0969da, #58a6ff);">DESCRIPTION</b>

       <b>papyrus.config.yml</b> is a YAML file that controls project paths, ID formats, workflow statuses, document types, and the local web UI.

       The file is read from the project directory (<b>--dir</b>) unless <b>--config</b> or <b>PAPYRUS_CONFIG</b> names another file.

<b style="color: light-dark(#0969da, #58a6ff);">PRECEDENCE</b>

       From highest to lowest:

       1. Command-line options.
       2. Environment variables (<b>PAPYRUS_DIR</b>, <b>PAPYRUS_CONFIG</b>).
       3. <b>papyrus.config.yml</b> in the project directory.
       4. Built-in defaults.

<b style="color: light-dark(#0969da, #58a6ff);">KEYS</b>

<b style="color: light-dark(#0969da, #58a6ff);">General</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">project_name</b>
       Project name. Default: the directory name.

       <b style="color: light-dark(#8250df, #d2a8ff);">task_prefix</b>
       Task ID prefix. Default: <b>TASK</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">milestone_prefix</b>
       Milestone ID prefix. Default: <b>MS</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">release_prefix</b>
       Release ID prefix. Default: <b>REL</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">id_padding</b>
        ID zero-padding width from <b>1</b> to <b>6</b>. Default: <b>3</b> (for example <b>TASK-001</b>). IDs wider than the padding are not truncated.

       <b style="color: light-dark(#8250df, #d2a8ff);">date_format</b>
       Date format. Default: <b>yyyy-mm-dd</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">editor</b>
       Editor command. Default: <b>$VISUAL</b>, then <b>$EDITOR</b>.

<b style="color: light-dark(#0969da, #58a6ff);">Paths</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">backlog_directory</b>
       Backlog store. Default: <b>backlog/</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">decision_directory</b>
       Decision records. Default: <b>decisions/</b>, outside the backlog store.

       <b style="color: light-dark(#8250df, #d2a8ff);">doc_directory</b>
       Root of the active documentation store. Default: <b>docs/</b>.

       Each document type stores its records under this path plus its
       <b>document_types.&lt;slug&gt;.directory</b>. Archived records are stored in
       an <b>archived/</b> subfolder of that type directory. A legacy
       <b>deprecated/</b> folder is inert.

<b style="color: light-dark(#0969da, #58a6ff);">Workflow</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">statuses</b>
       Ordered task statuses. Default: <b>["ToDo", "Initiated", "Refined", "Implemented", "Validated", "Done"]</b>.
       Use unique values; <b>default_status</b> must match one of them.
       This is a list-valued key: write it with <b>papyrus config set statuses "ToDo, Initiated, Done"</b>.
       The comma-separated form is split on commas, each element is trimmed, and the value is stored as a YAML flow sequence
       (<b>[ToDo, Initiated, Done]</b>). An empty value or an empty element is rejected. Commas, quotes, and backslashes are
       not escaped; a status that must contain a comma can only be written by editing the file directly.

       <b style="color: light-dark(#8250df, #d2a8ff);">default_status</b>
       Status assigned to new tasks. Default: <b>ToDo</b>.

<b style="color: light-dark(#0969da, #58a6ff);">Decisions</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">decision_types</b>
       Decision types and their ID prefixes. Default:

         <code style="font-weight: bold;">
         decision_types:
           adr: { prefix: ADR, enabled: true }
           bdr: { prefix: BDR, enabled: true }
         </code>

         Additional types may be added. A disabled type is rejected by <b>papyrus decision create</b> and hidden from lists and search.

<b style="color: light-dark(#0969da, #58a6ff);">Documents</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">document_types</b>
       Document types keyed by slug. The slug is the command name:
       <b>papyrus &lt;slug&gt;</b>. When this key is present in
       <b>papyrus.config.yml</b>, exactly its entries exist: the default map is
       replaced wholesale and no default entry is re-added. Removing an entry
       stops managing its files without deleting them. <b>enabled: false</b>
       keeps the entry, its prefix reservation, and its files, but removes the
       command and the browser view.

       Each entry has these fields:

         <b>prefix</b>        ID prefix. Required. Unique across all entries,
                     including disabled entries (case-insensitive); no
                     whitespace, control characters, path separators, or "..".
         <b>directory</b>     Directory relative to <b>doc_directory</b>. Required.
                     Must stay inside the project and be unique among enabled
                     entries.
         <b>enabled</b>       Whether the type is active. Default: <b>true</b>.
         <b>statuses</b>      Optional ordered status list. Must contain
                     <b>active</b> and <b>archived</b> exactly once, with no
                     duplicates. Values use the slug character set. Not allowed
                     on <b>ui-mockup</b>.
         <b>default_status</b>
                     Status of a new record. Optional; defaults to <b>active</b>.
                     Must be a declared status and must not be <b>archived</b>.
                     Not allowed on <b>ui-mockup</b>.

       The slug must use ASCII letters, digits, <b>_</b>, and <b>-</b>, must not
       start with <b>-</b>, and must not match a built-in command name
       (case-insensitive). A repeated YAML key is rejected.

       The eleven defaults written by <b>papyrus init</b> (<b>deploy</b> is not
       one of them):

       <code style="font-weight: bold;">
       document_types:
         req:      { prefix: REQ, directory: requirements/business/,       enabled: true, statuses: [active, archived] }
         nfr:      { prefix: NFR, directory: requirements/non-functional/, enabled: true, statuses: [active, archived] }
         glossary: { prefix: GLO, directory: glossary/,                    enabled: true, statuses: [active, archived] }
         rule:     { prefix: RUL, directory: business-rules/,              enabled: true, statuses: [active, archived] }
         ui-spec:  { prefix: UI,  directory: specifications/ui/,           enabled: true, statuses: [active, archived] }
         ui-mockup: { prefix: MOCK, directory: specifications/ui/mockups/, enabled: true }
         audit:    { prefix: AUD, directory: audit-records/,               enabled: true, statuses: [active, archived] }
         tech-spec: { prefix: TECH, directory: technical/,                 enabled: true, statuses: [active, archived] }
         use-case: { prefix: UC,  directory: use-cases/,                   enabled: true, statuses: [active, archived] }
         test-case: { prefix: TC, directory: test-cases/,                 enabled: true, statuses: [active, archived] }
         test-report: { prefix: TR, directory: test-reports/,             enabled: true, statuses: [active, archived] }
       </code>

       Validation is fail-closed: an invalid configuration makes every command
       that validates it exit <b>4</b> with a message naming
       <b>document_types.&lt;slug&gt;.&lt;field&gt;</b>. Repair it with
       <b>papyrus config get|set|unset|list</b>; <b>set</b> and <b>unset</b>
       validate the merged configuration and write nothing when it is invalid.
       A malformed or new entry must be defined by editing
       <b>papyrus.config.yml</b>: a single <b>config set</b> cannot create a new
       type because the entry would still lack its <b>prefix</b> or
       <b>directory</b>. <b>papyrus init --force</b> replaces the configuration
       file in full (the eleven defaults when non-interactive).

       <b>statuses</b> and <b>document_types.&lt;slug&gt;.statuses</b> are
       list-valued keys. Write them with a comma-separated value, for example
       <b>papyrus config set document_types.audit.statuses "active, archived, review"</b>.
       Each element is trimmed and stored as a YAML flow sequence
       (<b>[active, archived, review]</b>); <b>config get</b> and
       <b>config list</b> print that form. An empty value, an empty element, or a
       control character is rejected and nothing is written. Commas, quotes, and
       backslashes are not escaped: every comma separates elements, so a status
       that must contain a comma can only be written by editing
       <b>papyrus.config.yml</b> directly.

       <b>req</b> and <b>nfr</b> are the default entries <b>req</b> and
       <b>nfr</b>, not special commands; there is no <b>nfreq</b>. Disabled types
       cannot be created and are hidden from lists and search.

<b style="color: light-dark(#0969da, #58a6ff);">Web UI</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">browser_host</b>
       Bind address. Default: <b>127.0.0.1</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">browser_port</b>
       TCP port from <b>1</b> to <b>65535</b>. Default: <b>6420</b>.

       <b style="color: light-dark(#8250df, #d2a8ff);">browser_open</b>
       Open the default browser automatically. Default: <b>true</b>.

<b style="color: light-dark(#0969da, #58a6ff);">EXAMPLE</b>
       <code style="font-weight: bold;">
       project_name: papyrus
       task_prefix: TASK
       milestone_prefix: MS
       release_prefix: REL
       id_padding: 3
       date_format: yyyy-mm-dd

       backlog_directory: backlog/
       decision_directory: decisions/
       doc_directory: docs/
       statuses: ["ToDo", "Initiated", "Refined", "Implemented", "Validated", "Done"]
       default_status: ToDo

       decision_types:
         adr: { prefix: ADR, enabled: true }
         bdr: { prefix: BDR, enabled: true }

       document_types:
         req:      { prefix: REQ, directory: requirements/business/,       enabled: true, statuses: [active, archived] }
         nfr:      { prefix: NFR, directory: requirements/non-functional/, enabled: true, statuses: [active, archived] }
         glossary: { prefix: GLO, directory: glossary/,                    enabled: true, statuses: [active, archived] }
         rule:     { prefix: RUL, directory: business-rules/,              enabled: true, statuses: [active, archived] }
         ui-spec:  { prefix: UI,  directory: specifications/ui/,           enabled: true, statuses: [active, archived] }
         ui-mockup: { prefix: MOCK, directory: specifications/ui/mockups/, enabled: true }
         audit:    { prefix: AUD, directory: audit-records/,               enabled: true, statuses: [active, archived, review], default_status: review }
         tech-spec: { prefix: TECH, directory: technical/,                 enabled: true, statuses: [active, archived] }
         use-case: { prefix: UC,  directory: use-cases/,                   enabled: true, statuses: [active, archived] }
         test-case: { prefix: TC, directory: test-cases/,                 enabled: true, statuses: [active, archived] }
         test-report: { prefix: TR, directory: test-reports/,             enabled: true, statuses: [active, archived] }
       # Optional user-defined type. A hand-written map replaces the defaults,
       # so keep the entries above when adding this one.
       # deploy: { prefix: DEP, directory: architecture/deployment/, enabled: true }
       # Archived records live under each type directory's archived/ subfolder.

       browser_host: 127.0.0.1
       browser_port: 6420
       browser_open: true
       </code>

<b style="color: light-dark(#0969da, #58a6ff);">FILES</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">papyrus.config.yml</b>
       Project configuration, normally at the project root.

<b style="color: light-dark(#0969da, #58a6ff);">SEE ALSO</b>

       papyrus(1)
</pre>

---

*This project was created with AI assistance.*
