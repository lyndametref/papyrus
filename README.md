# Papyrus

Papyrus is a project-level backlog and documentation manager for Git repositories. It combines task, milestone, release, and decision tracking with long-term documentation (requirements, rules, glossary, ADRs) in one CLI, plus a local web UI and unified search over both stores.

## Inspiration

Papyrus is inspired by and builds on the ideas of **[Backlog.md](https://github.com/MrLesk/Backlog.md)** by **[MrLesk](https://github.com/MrLesk)** — a Markdown-native task manager and Kanban visualizer for Git repositories, MIT-licensed. Huge kudos to MrLesk and the Backlog.md contributors for pioneering git-native, Markdown-backed project collaboration for humans and AI agents. Papyrus's backlog workflow, CLI shape, and documentation model owe a great deal to their work.

---

## Man Page (`papyrus(1)`)

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
        - **Documentation management** - business and non-functional requirements, glossary terms, UI and technical specifications, audit records, and business rules.

       All project data lives in the project directory selected with <b>--dir</b> (default: the current working directory). IDs are generated automatically and remain stable across archive/restore and deprecate/restore cycles.

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

       <b style="color: light-dark(#1a7f37, #3fb950);">--no-git</b>
       Do not initialize a Git repository.

       <b style="color: light-dark(#1a7f37, #3fb950);">--force</b>
       Allow initialization in a non-empty directory. Existing files are never overwritten.

<b style="color: light-dark(#0969da, #58a6ff);">config</b>

       Read and edit the configuration. Keys use dotted notation for nested values, for example <b>document_types.req.enabled</b>.
       <code style="font-weight: bold;">
       papyrus config list
       papyrus config get &lt;key&gt;
       papyrus config set &lt;key&gt; &lt;value&gt;
       papyrus config unset &lt;key&gt;
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       Print all effective keys, including defaults.

       <b style="color: light-dark(#8250df, #d2a8ff);">get &lt;key&gt;</b>
       Print one effective value.

       <b style="color: light-dark(#8250df, #d2a8ff);">set &lt;key&gt; &lt;value&gt;</b>
       Persist one value. The value is parsed as a YAML scalar.

       <b style="color: light-dark(#8250df, #d2a8ff);">unset &lt;key&gt;</b>
       Remove one key and fall back to its default.

       Precedence is described in papyrus-config(5).

<b style="color: light-dark(#0969da, #58a6ff);">browser</b>

       Serve the local web UI over both the backlog and documentation stores.
       <code style="font-weight: bold;">
       papyrus browser [--host &lt;host&gt;] [--port &lt;port&gt;] [--no-open]
       </code>
       <b style="color: light-dark(#1a7f37, #3fb950);">--host &lt;host&gt;</b>
       Bind address. Default: <b>127.0.0.1</b>. A non-loopback address is allowed but prints a warning.

       <b style="color: light-dark(#1a7f37, #3fb950);">--port &lt;port&gt;</b>
       TCP port. Default: <b>6420</b>.

       <b style="color: light-dark(#1a7f37, #3fb950);">--no-open</b>
       Do not open the default browser automatically.

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
       <code style="font-weight: bold;">
       papyrus milestone create &lt;title&gt; [-d &lt;description&gt;] [--release &lt;id&gt;]
       papyrus milestone list
       papyrus milestone view &lt;id&gt;
        papyrus milestone edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;description&gt;] [--release &lt;id&gt;] [--no-release [&lt;id&gt;]]
       papyrus milestone archive &lt;id&gt;
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

       <b style="color: light-dark(#8250df, #d2a8ff);">archive &lt;id&gt;</b>
       Archive a milestone.

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

<b style="color: light-dark(#0969da, #58a6ff);">board</b>

       Show a live Kanban board in the terminal.
       <code style="font-weight: bold;">
       papyrus board [--milestone &lt;id&gt;] [--status &lt;status&gt;]... [--watch]
       papyrus board export [--output &lt;file&gt;] [--milestone &lt;id&gt;]
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">board</b>
       Show the Kanban board.

          <b style="color: light-dark(#1a7f37, #3fb950);">--milestone &lt;id&gt;</b>
          Restrict the board to one milestone.

          <b style="color: light-dark(#1a7f37, #3fb950);">--status &lt;status&gt;</b>
          Restrict the board to one or more statuses. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--watch</b>
          Refresh on changes.

       <b style="color: light-dark(#8250df, #d2a8ff);">board export</b>
       Export a Markdown snapshot of the board.

          <b style="color: light-dark(#1a7f37, #3fb950);">--output &lt;file&gt;</b>
          Output file. Default: <b>board.md</b> in the project directory.

          <b style="color: light-dark(#1a7f37, #3fb950);">--milestone &lt;id&gt;</b>
          Restrict the board to one milestone.

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

<b style="color: light-dark(#0969da, #58a6ff);">req</b>

         Manage business requirements (REQ-...). Requirement-to-group cardinality is <b>1..1</b>; each group contains business requirements only.

       <code style="font-weight: bold;">
        papyrus req create &lt;title&gt; [--title &lt;title&gt;] [--story &lt;text&gt;] [-d &lt;description&gt;] --group &lt;id&gt;
       papyrus req create-group --title &lt;title&gt; [--story &lt;text&gt;] [-d &lt;description&gt;]
       papyrus req list
       papyrus req view &lt;id&gt;
       papyrus req edit &lt;id&gt; [--title &lt;title&gt;] [--story &lt;text&gt;] [-d &lt;description&gt;] [--editor]
       papyrus req deprecate &lt;id&gt;
       papyrus req restore &lt;id&gt;
       </code>

       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a business requirement.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Requirement title.

          <b style="color: light-dark(#1a7f37, #3fb950);">--story &lt;text&gt;</b>
          User story.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--group &lt;id&gt;</b>
           Add the requirement to its required business-requirement group.

       <b style="color: light-dark(#8250df, #d2a8ff);">create-group</b>
        Create a business-requirement group. It can contain business requirements only.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Requirement group title.

          <b style="color: light-dark(#1a7f37, #3fb950);">--story &lt;text&gt;</b>
          User story.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Description.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List business requirements.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a business requirement.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a business requirement.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">--story &lt;text&gt;</b>
          Update the user story.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the document body in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
       Move a business requirement to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Return a deprecated business requirement to its original folder. The ID is preserved.

<b style="color: light-dark(#0969da, #58a6ff);">nfreq</b>

          Manage non-functional requirements (NFR-...). Requirement-to-group cardinality is <b>1..1</b>; each group contains non-functional requirements only. Non-functional requirements may be referenced by tasks, decisions, and technical specifications.

       <code style="font-weight: bold;">
        papyrus nfreq create &lt;title&gt; [-d &lt;description&gt;] --group &lt;id&gt;
       papyrus nfreq create-group --title &lt;title&gt; [-d &lt;description&gt;]
       papyrus nfreq list
       papyrus nfreq view &lt;id&gt;
       papyrus nfreq edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;description&gt;] [--editor]
       papyrus nfreq deprecate &lt;id&gt;
       papyrus nfreq restore &lt;id&gt;
       </code>

       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a non-functional requirement.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Requirement description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--group &lt;id&gt;</b>
           Add the requirement to its required non-functional-requirement group.

       <b style="color: light-dark(#8250df, #d2a8ff);">create-group</b>
        Create a non-functional-requirement group. It can contain non-functional requirements only.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Requirement group title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Description.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List non-functional requirements.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a non-functional requirement.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a non-functional requirement.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the document body in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
       Move a non-functional requirement to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Return a deprecated non-functional requirement to its original folder. The ID is preserved.

<b style="color: light-dark(#0969da, #58a6ff);">glossary</b>

       Manage glossary terms (GLO-...).

       <code style="font-weight: bold;">
       papyrus glossary create --term &lt;term&gt; --definition &lt;text&gt;
       papyrus glossary list
       papyrus glossary view &lt;id&gt;
       papyrus glossary edit &lt;id&gt; [--term &lt;term&gt;] [--definition &lt;text&gt;] [--editor]
       papyrus glossary deprecate &lt;id&gt;
       papyrus glossary restore &lt;id&gt;
       </code>

       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a glossary term.

          <b style="color: light-dark(#1a7f37, #3fb950);">--term &lt;term&gt;</b>
          Glossary term.

          <b style="color: light-dark(#1a7f37, #3fb950);">--definition &lt;text&gt;</b>
          Definition of the term.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List glossary terms.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a glossary term.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a glossary term.

          <b style="color: light-dark(#1a7f37, #3fb950);">--term &lt;term&gt;</b>
          Update the glossary term.

          <b style="color: light-dark(#1a7f37, #3fb950);">--definition &lt;text&gt;</b>
          Update the definition.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the document body in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
       Move a glossary term to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Return a deprecated glossary term to its original folder. The ID is preserved.

<b style="color: light-dark(#0969da, #58a6ff);">ui-spec</b>

        Manage UI specifications (UI-...).

       <code style="font-weight: bold;">
       papyrus ui-spec create &lt;title&gt; --view &lt;view&gt; --description &lt;text&gt;
       papyrus ui-spec list
       papyrus ui-spec view &lt;id&gt;
       papyrus ui-spec edit &lt;id&gt; [--view &lt;view&gt;] [--description &lt;text&gt;] [--editor]
       papyrus ui-spec deprecate &lt;id&gt;
       papyrus ui-spec restore &lt;id&gt;
       </code>

       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create a UI specification.

          <b style="color: light-dark(#1a7f37, #3fb950);">--view &lt;view&gt;</b>
          Associate the specification with a view.

          <b style="color: light-dark(#1a7f37, #3fb950);">--description &lt;text&gt;</b>
          Description of the UI specification.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List UI specifications.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View a UI specification.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit a UI specification.

          <b style="color: light-dark(#1a7f37, #3fb950);">--view &lt;view&gt;</b>
          Update the associated view.

          <b style="color: light-dark(#1a7f37, #3fb950);">--description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the document body in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
        Move a UI specification to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
        Return a deprecated UI specification to its original folder. The ID is preserved.

<b style="color: light-dark(#0969da, #58a6ff);">rule</b>

       Manage business rules (RUL-...).

       <code style="font-weight: bold;">
        papyrus rule create &lt;title&gt; --description &lt;text&gt; --rationale &lt;text&gt; --applicability &lt;text&gt; [--requirement &lt;id&gt;...]
       papyrus rule list
       papyrus rule view &lt;id&gt;
        papyrus rule edit &lt;id&gt; [--title &lt;title&gt;] [-d &lt;description&gt;] [--requirement &lt;id&gt;...] [--no-requirement [&lt;id&gt;]] [--editor]
       papyrus rule deprecate &lt;id&gt;
       papyrus rule restore &lt;id&gt;
        </code>

        <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
        Create a business rule.

          <b style="color: light-dark(#1a7f37, #3fb950);">--description &lt;text&gt;</b>
          State the business rule precisely.

          <b style="color: light-dark(#1a7f37, #3fb950);">--rationale &lt;text&gt;</b>
          Explain why the rule exists, referencing its requirement(s).

          <b style="color: light-dark(#1a7f37, #3fb950);">--applicability &lt;text&gt;</b>
          Describe when and where the rule applies.

          <b style="color: light-dark(#1a7f37, #3fb950);">--requirement &lt;id&gt;</b>
           Reference a business requirement. Repeatable; zero or more may be specified.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
        List business rules.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
        View a business rule.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
        Edit a business rule.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">-d, --description &lt;text&gt;</b>
          Update the description.

          <b style="color: light-dark(#1a7f37, #3fb950);">--requirement &lt;id&gt;</b>
           Add a business requirement to the existing list. Repeatable.

          <b style="color: light-dark(#1a7f37, #3fb950);">--no-requirement [&lt;id&gt;]</b>
           Remove the specified business requirement; if no ID is given, remove all business-requirement references.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the document body in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
        Move a business rule to the deprecated folder. Deprecated documents are hidden from lists, search, and the web UI.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Return a deprecated business rule to its original folder. The ID is preserved.

<b style="color: light-dark(#0969da, #58a6ff);">audit</b>

        Manage audit records (<b>AUD-001</b>, ...).
       <code style="font-weight: bold;">
       papyrus audit create --title &lt;title&gt; --content &lt;text&gt; [--audit-type &lt;type&gt;]
       papyrus audit list
       papyrus audit view &lt;id&gt;
       papyrus audit edit &lt;id&gt; [--title &lt;title&gt;] [--content &lt;text&gt;] [--audit-type &lt;type&gt;] [--editor]
       papyrus audit deprecate &lt;id&gt;
       papyrus audit restore &lt;id&gt;
       </code>
       <b style="color: light-dark(#8250df, #d2a8ff);">create</b>
       Create an audit record.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Audit record title.

          <b style="color: light-dark(#1a7f37, #3fb950);">--content &lt;text&gt;</b>
          Audit record content.

          <b style="color: light-dark(#1a7f37, #3fb950);">--audit-type &lt;type&gt;</b>
          Audit record type. Built-in types: <b>security</b> (default) and <b>licence</b>; additional types are configurable.

       <b style="color: light-dark(#8250df, #d2a8ff);">list</b>
       List audit records.

       <b style="color: light-dark(#8250df, #d2a8ff);">view &lt;id&gt;</b>
       View an audit record.

       <b style="color: light-dark(#8250df, #d2a8ff);">edit &lt;id&gt;</b>
       Edit an audit record.

          <b style="color: light-dark(#1a7f37, #3fb950);">--title &lt;title&gt;</b>
          Update the title.

          <b style="color: light-dark(#1a7f37, #3fb950);">--content &lt;text&gt;</b>
          Update the content.

          <b style="color: light-dark(#1a7f37, #3fb950);">--audit-type &lt;type&gt;</b>
          Set the audit record type.

          <b style="color: light-dark(#1a7f37, #3fb950);">--editor</b>
          Open the audit record content in $VISUAL or $EDITOR.

       <b style="color: light-dark(#8250df, #d2a8ff);">deprecate &lt;id&gt;</b>
       Deprecate an audit record.

       <b style="color: light-dark(#8250df, #d2a8ff);">restore &lt;id&gt;</b>
       Restore a deprecated audit record.

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

        <b>active</b> -&gt; <b>archived</b>

        Archiving a milestone is separate from task workflow status.

<b style="color: light-dark(#0969da, #58a6ff);">Releases</b>

        <b>active</b> -&gt; <b>archived</b>

        Releases group milestones.

<b style="color: light-dark(#0969da, #58a6ff);">Decisions</b>

       Nygard ADR statuses:

       <b>proposed</b> -&gt; <b>accepted</b> | <b>rejected</b>
       <b>accepted</b> -&gt; <b>deprecated</b> | <b>superseded</b>

<b style="color: light-dark(#0969da, #58a6ff);">Documents</b>

        Business requirements, non-functional requirements, glossary terms,
        UI specifications, and business rules share this lifecycle:

        <b>active</b> -&gt; <b>deprecated</b>

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

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/requirements/business/</b>
        Business requirements. Path is <b>doc_directory</b> plus
        <b>document_types.req.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.req.directory&gt;/deprecated/</b>
        Deprecated business requirements, under the configured requirement folder.

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/requirements/non-functional/</b>
        Non-functional requirements. Path is <b>doc_directory</b> plus
        <b>document_types.nfr.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.nfr.directory&gt;/deprecated/</b>
        Deprecated non-functional requirements, under the configured requirement folder.

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/glossary/</b>
        Glossary terms. Path is <b>doc_directory</b> plus
        <b>document_types.glossary.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.glossary.directory&gt;/deprecated/</b>
        Deprecated glossary terms, under the configured glossary folder.

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/specifications/ui/</b>
        UI specifications. Path is controlled by <b>doc_directory</b> and
        <b>document_types.ui-spec.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.ui-spec.directory&gt;/deprecated/</b>
        Deprecated UI specifications, under the configured UI specifications folder.

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/business-rules/</b>
        Business rules. Path is <b>doc_directory</b> plus
        <b>document_types.rule.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.rule.directory&gt;/deprecated/</b>
        Deprecated business rules, under the configured business rules folder.

        <b style="color: light-dark(#8250df, #d2a8ff);">docs/audit-records/</b>
        Audit records. Path is <b>doc_directory</b> plus
        <b>document_types.audit.directory</b>.

        <b style="color: light-dark(#8250df, #d2a8ff);">&lt;doc_directory&gt;/&lt;document_types.audit.directory&gt;/deprecated/</b>
        Deprecated audit records, under the configured audit records folder.

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

       Deprecate a document:
       <code style="font-weight: bold;">
       papyrus req deprecate REQ-003
       papyrus req restore REQ-003
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
        Positive integer for ID zero-padding width. Default: <b>3</b> (for example <b>TASK-001</b>). IDs wider than the padding are not truncated.

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
       Active documentation store. Default: <b>docs/</b>.

       Each document type has its own configured directory relative to this path.
       Deprecated documents are stored in a <b>deprecated/</b> subfolder within
       their type directory.

<b style="color: light-dark(#0969da, #58a6ff);">Workflow</b>

       <b style="color: light-dark(#8250df, #d2a8ff);">statuses</b>
       Ordered task statuses. Default: <b>["ToDo", "Initiated", "Refined", "Implemented", "Validated", "Done"]</b>.
       Use unique values; <b>default_status</b> must match one of them.

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
       Document type keys, ID prefixes, directories relative to <b>doc_directory</b>,
       and enabled flags. Deprecated documents are stored in a <b>deprecated/</b>
       subfolder of each type directory. Default:

       <code style="font-weight: bold;">
       document_types:
         req:      { prefix: REQ, directory: requirements/business/,       enabled: true }
         nfr:      { prefix: NFR, directory: requirements/non-functional/, enabled: true }
         glossary: { prefix: GLO, directory: glossary/,                    enabled: true }
         rule:     { prefix: RUL, directory: business-rules/,              enabled: true }
         ui-spec:  { prefix: UI,  directory: specifications/ui/,           enabled: true }
         audit:    { prefix: AUD, directory: audit-records/,                enabled: true }
         deploy:   { prefix: DEP, directory: architecture/deployment/,     enabled: true }
       </code>

       The <b>nfreq</b> command uses the <b>nfr</b> type key. Additional types may
       be added. Disabled types cannot be created and are hidden from lists and search.

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
         req:      { prefix: REQ, directory: requirements/business/,       enabled: true }
         nfr:      { prefix: NFR, directory: requirements/non-functional/, enabled: true }
         glossary: { prefix: GLO, directory: glossary/,                    enabled: true }
         rule:     { prefix: RUL, directory: business-rules/,              enabled: true }
         ui-spec:  { prefix: UI,  directory: specifications/ui/,           enabled: true }
         audit:    { prefix: AUD, directory: audit-records/,                enabled: true }
         deploy:   { prefix: DEP, directory: architecture/deployment/,     enabled: true }
       # Deprecated documents live under each type directory's deprecated/ subfolder.

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
