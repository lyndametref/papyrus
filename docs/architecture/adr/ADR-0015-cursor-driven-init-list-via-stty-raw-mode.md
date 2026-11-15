---
id: ADR-0015
title: "Cursor-Driven Init Artifact List via `stty` Cbreak Mode and a KeySource Seam"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-15T15:47:30Z'
labels: []
references: []
---
# Cursor-Driven Init Artifact List via `stty` Cbreak Mode and a KeySource Seam

## Status

Accepted (2026-11-13). Acceptance follows the user's explicit instruction of 2026-11-13 to implement REQ-01.006 (TASK-104) along this route. This decision supersedes ADR-0014, the competing in-process `termios` FFI proposal, which is recorded as `Superseded` and must not be implemented.

## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Accepted, 2026-11-13)

## Context

REQ-01.006 and `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` section 2 require `papyrus init <project_name>` to present artifact activation as one cursor-driven vertical list: one highlighted selectable row, Up/Down and `k`/`j` navigation with clamping, Space toggling the highlighted row, Enter confirming, a full redraw on every keypress, and end-of-input confirming and continuing. The shipped `src/init.rs` (`build_config_interactive`) still prints a static `[x]` block and reads one line of space-separated artifact names, which is the presentation REQ-01.006 rejects. Supplying single keypresses requires leaving the terminal's canonical line discipline for the duration of the list.

Forces at play:

- **Dependency baseline.** ADR-0001 selects Rust, ADR-0004 fixes the single-crate repository layout, and ADR-0005 fixes the dependency-free, single-binary, offline posture with no new crate or runtime dependency. DoD/TASK-104 additionally forbids any new crate dependency. Rust `std` exposes `IsTerminal` but no termios/console-mode API, so any in-process mode change needs either a crate or `unsafe` FFI against the C library.
- **Packaging baseline.** ADR-0002 ships only `/usr/bin/papyrus` plus the copyright file; ADR-0003 targets Debian forky only. The `.deb` dependency set must not grow.
- **Testability.** The list logic must be driven from tests without a terminal (TASK-104 scope item 10, acceptance criterion 12), while the real keyboard path stays a thin adapter.
- **Captured output.** Acceptance criteria are checked on captured output, and the terminal may be a pipe or a captured log; redraw must be readable there without assuming terminal capabilities.
- **Ctrl-C and terminal hygiene.** Ctrl-C during the list must exit without writing `papyrus.config.yml` and without leaving the terminal in a modified input mode (acceptance criterion 14).
- **No new user surface.** No new CLI flag, environment variable, or configuration key may be introduced, and `--force`, `--no-git`, and the configuration schema are unchanged.

Terminology: the file name uses "raw mode" in the colloquial sense. The decision selects **cbreak** — non-canonical input with echo and signal generation disabled — not `stty raw`, which also disables output post-processing.

**Relationship to ADR-0014.** A second Proposed ADR covering the same question — `ADR-0014-dependency-free-tty-key-input-for-interactive-init.md` — records an in-process `tcgetattr`/`tcsetattr` FFI route. The two proposals are mutually exclusive for the same code path and both remain Proposed; this ADR records the `stty` alternative and its justification, and is numbered 0015 only because the 0014 number was already taken. The choice between them is a user decision, and no implementation may adopt either route until one of the two is Accepted and the other resolved (refused or superseded). **Resolution (2026-11-13):** the user selected this route for REQ-01.006 / TASK-104; this ADR is Accepted and ADR-0014 is Superseded without having been accepted.

## Decision

1. **Terminal mode is changed out-of-process with `stty` from Debian `coreutils`, never in-process.** At the start of the list phase only, and only when `std::io::stdin().is_terminal()` is true, Papyrus runs `stty` through `std::process::Command` with this exact argument vector:

   ```
   stty -icanon -echo -isig -ixon min 1 time 0
   ```

   - `-icanon` makes each byte available immediately instead of after a newline.
   - `-echo` stops the terminal echoing keystrokes; Papyrus renders every visible effect itself.
   - `-isig` makes Ctrl-C, Ctrl-\ and Ctrl-Z arrive as bytes instead of generating SIGINT/SIGQUIT/SIGTSTP, which is what makes decision 7 possible and keeps job control from suspending a process whose terminal is modified.
   - `-ixon` prevents Ctrl-S from silently freezing the session.
   - `min 1 time 0` makes a blocking read wait for exactly one byte rather than returning early.
   - `raw` is deliberately not used: it also clears output post-processing (`opost`), so `\n` would no longer produce a carriage return and the list would staircase.

   `stty` is invoked with `Stdio::inherit()` for stdin. `Command::output()` would otherwise attach `/dev/null` to stdin and `stty` would fail with "Inappropriate ioctl for device". The mode is applied to the terminal already attached to `stdin`; no `/dev/tty` handle is opened and no second stdin handle is created.

2. **The previous settings are saved and restored by a guard.** Before changing the mode Papyrus captures `stty -g` output (same inherited stdin) and stores it in a `TerminalModeGuard`. The guard restores with `stty <saved>`:
   - immediately when the list phase ends for any reason — Enter, end of input, Ctrl-C, or an I/O error — so the following prefix and folder prompts run under canonical line discipline;
   - from `Drop`, so unwinding panics also restore (the crate does not set `panic = "abort"`; an abort or a fatal signal remains a residual risk, see decision 8).

   A failed restore cannot be repaired in-process: Papyrus prints one line telling the user to run `stty sane`, ignores the restore status, and continues the walkthrough (the configuration state is already decided at that point).

3. **Keys are parsed byte-wise from the same reader the later prompts use.** The parser reads from the existing `R: BufRead` in `build_config_interactive`, one byte per read call (`read_exact` on a one-byte buffer). It never constructs a second `BufReader` over stdin: any bytes the reader has already buffered stay in that same reader, so typeahead after the confirming Enter feeds the first prefix prompt instead of being lost.

   The key map is fixed:

   | Input | Meaning |
   |---|---|
   | `0x0A` (LF), `0x0D` (CR) | Enter — confirm and continue |
   | `0x20` (space) | toggle the highlighted selectable row |
   | `k`, `j` | move the highlight to the previous/next selectable row |
   | `0x1B` followed by `[` `A` / `[` `B`, or `O` `A` / `O` `B` | Up / Down |
   | `0x04` (Ctrl-D) | end of input — confirm the current state and continue |
   | `0x03` (Ctrl-C) | interrupt (decision 7) |
   | `read` returning `Ok(0)` | end of input — confirm and continue |
   | `ErrorKind::Interrupted` (EINTR) | retry the read |
   | anything else | ignore; redraw only |

   A lone `0x1B` or an unrecognised escape sequence is an ignored key, never a confirmation: the parser reads at most two further bytes and discards what it cannot match. In cbreak mode Ctrl-D is an ordinary byte, not the canonical-mode end-of-file condition, so `0x04` is handled explicitly. Navigation clamps at the first and last selectable row and never enters the core rows; the list state machine is the only owner of cursor and activation state.

4. **Redraw is append-only text.** Each redraw writes a blank separator line and then the complete list — core rows with their fixed marker and "always active" indication, then decision types, then document types in schema order — with the cursor marker on the highlighted row and `[x]`/`[ ]` on every selectable row. No ANSI cursor movement, clear-line, alternate-screen, or colour sequences are emitted:

   - output stays readable when captured to a file or a test buffer, which is how the acceptance criteria are checked;
   - no terminal database, `TERM` inspection, or capability probing is needed;
   - no escape-sequence state can desynchronise the display.

   The cost is repeated output; the list is bounded by the artifact set, so this is accepted.

5. **Two seams keep the logic testable and the terminal handling thin.**
   - `KeySource`: `fn next_key(&mut self) -> io::Result<Key>`. `ByteKeySource<'a, R: BufRead>` is the real adapter implementing decision 3; `ScriptedKeySource` yields a fixed key sequence and is used by unit tests. It also models terminal end-of-input and the interrupt key without a terminal.
   - `TerminalMode`: `enter() -> io::Result<TerminalModeGuard>` / `restore()`. `SttyTerminalMode` is the real adapter implementing decisions 1–2; a fake implementation returns a failure in tests to exercise decision 6.

   The list state machine (`ArtifactList`) takes a `&mut dyn KeySource`, a `&mut W: Write`, and an initial activation map, and knows nothing about `stty`, files, or the OS. `run_init_with_io` keeps its current signature; the TTY branch constructs `SttyTerminalMode` and `ByteKeySource` around the caller-provided reader and writer, the non-TTY branch constructs neither.

6. **If `stty` is unavailable or fails, `init` degrades instead of aborting.** Failure covers: the process cannot be spawned, a non-zero exit status from either `stty -g` or the mode change, unparsable saved settings, or a successful mode change that cannot be restored later. In that case Papyrus:
   - restores whatever it managed to set (best effort), so the terminal is not left modified;
   - prints one line stating that key navigation is unavailable in this environment;
   - renders the list once with every row active and reads a line-based confirmation prompt where only Enter, empty input, or end of input confirms and continues into the unchanged prefix/folder prompts;
   - never reinstates typed artifact tokens and never fails the walkthrough because of the terminal.

   The degraded path therefore cannot deactivate types. A user who needs a different activation edits `papyrus.config.yml` or uses `config set` (REQ-01.002); this is a terminal-environment limitation, not a silent contract change, and REQ-01.006's rejection of typed-token activation still holds.

7. **Ctrl-C is a byte-mapped interrupt with exit code 130, not a signal-based abort.**
   - Because of `-isig`, Ctrl-C arrives as `0x03`. The list maps it to a new `AppError::Interrupted` variant whose `code()` is `130` and whose message is `interrupted`; `main` already exits with `err.code()` after printing the message (JSON-mode error object in `--json`).
   - The interrupt is raised while `build_config_interactive` is still running, before `Config::validate()`, before any directory creation, and before the `papyrus.config.yml` write in `run_init_with_io`, so no configuration file and no project directories are produced.
   - The `TerminalModeGuard` restores the terminal before the error propagates out of the list phase, so the shell prompt after exit is usable.
   - Ctrl-\ (`0x1C`) and Ctrl-Z (`0x1A`) are also bytes under `-isig` and are ignored, never treated as confirmation or as signals.
   - The prefix and folder prompts stay line-based and canonical, so Ctrl-C there keeps the default SIGINT disposition and the kernel already terminates with 130. Exit status 130 is therefore uniform across the walkthrough; only the mechanism differs.

8. **Residual signal and process-death risks are accepted and documented, not silently handled.** In-process signal handlers would require `libc`/`unsafe` FFI or a crate and are out of scope for this ADR (ADR-0005). Left uncovered:
   - `SIGTERM`, `SIGHUP`, `SIGQUIT`, `SIGUSR1`, `SIGUSR2`, `SIGKILL`, and `SIGSTOP` delivered externally, or any crash/abort (`SIGSEGV`, `SIGABRT`), terminate the process without running `Drop`; the terminal then stays in cbreak with echo off until the user runs `stty sane`. `SIGKILL` and `SIGSTOP` can never be covered.
   - An external `kill -TSTP` suspends the process with the terminal modified; there is no `SIGCONT` handler to re-apply and restore the mode.
   - `SIGPIPE` while writing redraws to a closed stdout terminates the process with the terminal modified.
   - The exposure window is bounded to the list phase — a keypress loop with no filesystem or network work — and every normal exit path restores, which is the mitigation this ADR relies on.
   - Installations that need stronger guarantees (handler-based restore, or a `-isig`-based interrupt plus re-raise) require a new ADR that supersedes this one, because they change the dependency baseline.

9. **`stty` does not violate ADR-0005: it is not a new dependency of Papyrus.** It is a binary provided by the Debian `coreutils` package, which is `Essential: yes` (Priority: required) and is present on every system that can run a login shell, including `debootstrap --variant=minbase`. Consequences:
   - no crate, build-time, or dev dependency is added; `Cargo.toml` is unchanged;
   - the `.deb` payload is unchanged (no bundled binary, no new `Depends` entry), consistent with ADR-0002 and ADR-0003;
   - nothing is downloaded, so the offline posture (NFR-01.003) and the "no runtime asset directory" property of ADR-0005 are unaffected;
   - the dependency is not fatal: decision 6 keeps `init` working when the binary is absent, unreadable, or fails, so `stty` is an optional enhancement rather than a hard requirement;
   - the crate already invokes an external program (`git init`, optional) through `std::process::Command`, so out-of-process system helpers are an established pattern in this codebase.

10. **Scope of the mechanism is exactly the init list.** No other command, prompt, or input path changes. No signal handler, ANSI escape, alternative screen buffer, or terminal-capability query is introduced. Windows and non-Debian platforms are out of scope (ADR-0003); the non-TTY path — no prompts, no `stdin` reads, all artifacts active, defaults written — stays byte-for-byte as today, and `stty` is never invoked there.

## Consequences

**Positive:**

- REQ-01.006's cursor contract is implementable without a new crate, without `unsafe`, and without changing the packaging or dependency baseline.
- `stty` is a single, auditable external call: the mode is a fixed argument vector, and the terminal state is captured and restored rather than assumed.
- `-isig` turns Ctrl-C into a byte, which gives one deterministic interrupt path with exit code 130, terminal restoration before exit, and no partially written configuration.
- The list logic is a pure state machine behind `KeySource`, so every acceptance criterion about cursor, toggling, clamping, and redraw is testable without a TTY, and the scripted tests cannot flake on terminal availability.
- Append-only redraw keeps captured output self-describing, so CI evidence and manual TTY evidence look the same.
- The fallback keeps `init` usable in stripped-down environments instead of turning a presentation requirement into a hard runtime dependency.

**Negative:**

- A subprocess is spawned and one process-boundary failure mode (spawn/exit status) must be handled.
- `init` acquires a de facto POSIX-terminal assumption; the interactive list is unavailable where `stty` is missing, and the degraded path cannot deactivate types.
- Append-only redraw grows output linearly with keypresses; acceptable for a bounded list, not a general TUI strategy.
- Signal-based termination can still leave the terminal modified, and the only remedy is a user-side `stty sane`.
- Ctrl-C handling now differs by phase (byte in the list, signal in the prompts), so the two paths must be tested separately even though both end at 130.
- Typeahead entered after the confirming Enter is consumed by the following prefix/folder prompts; escape bytes typed there are ordinary characters, subject to the existing prompt validation.

**Risks and mitigations:**

- *Terminal left in cbreak mode*: a guard restores on every return path including unwinding panics; restore is attempted before exit code 130 propagates; the degraded path never changes the mode at all. Residual signal cases are listed in decision 8 and are accepted with a documented recoverable action.
- *Swallowed or reordered input at the hand-off to line prompts*: the byte parser and the line prompts share one `BufRead`; the parser never creates a second reader or a second stdin handle; tests cover a confirm followed by a piped prefix line.
- *Escape-sequence misparse*: the parser consumes at most two bytes after `0x1B`, ignores unmatched sequences, and treats a lone `ESC` as an ignored key; tests cover `ESC [ A`, `ESC O B`, and a truncated sequence.
- *Missing or failing `stty`*: decision 6 detects spawn failure and non-zero status and falls back without writing anything unexpected; a fake `TerminalMode` test exercises it.
- *Locked-down environments where `stty` exists but the ioctl fails*: same fallback, since the failure is a non-zero status.
- *Confirmation/activation mismatch*: the state machine is the single source of both the rendered list and the written configuration; the end-to-end assertion is on `papyrus.config.yml` after Enter.
- *Regression of the non-TTY path*: `stty` is called only inside the `is_terminal()` branch; the existing non-interactive init tests gate the change.

## Downstream constraints

1. `Cargo.toml` gains no dependency, and the `.deb` gains no payload file and no `Depends` entry. Any proposal to add `libc`, `crossterm`, `termion`, or a similar crate is rejected by this ADR and requires a new one.
2. The `stty` invocation is exactly decision 1's argument vector plus the `-g` query and the saved-settings restore. No other `stty` subcommand, no shell interpolation, and no user-controlled argument may reach it.
3. No `unsafe` block, FFI declaration, or signal handler is added for terminal handling.
4. Mode changes happen only after `stdin` is confirmed to be a terminal, only for the duration of the list phase, and are undone before the prefix/folder prompts.
5. `ArtifactList` must not perform I/O other than through its injected `KeySource` and `Write`; it must not call `std::io::stdin`, `std::process`, or the filesystem.
6. `AppError::Interrupted` maps to exit code 130 and is raised only for the interrupt key in the list phase; it must not be reused for other aborts, and the existing codes 1, 3, 4 are unchanged.
7. Interrupt (Ctrl-C, decision 7) is the only list-phase exit that leaves `papyrus.config.yml` unwritten and creates no artifact directories. End of input (decision 3) and `stty` failure (decision 6) both confirm the current state and continue through the prefix/folder prompts, and therefore do write the configuration.
8. The degraded path may not introduce any typed artifact-token activation; REQ-01.006 stays in force in every environment.
9. Interactive detection remains `std::io::stdin().is_terminal()` per the specification assumption; no `/dev/tty` probing, `TERM` inspection, or new flag is added.
10. TTY verification (acceptance criterion 14, DoD 6) is manual evidence recorded in the ticket; automated coverage of the real `stty` path is limited to the mode-abstraction seam.
11. The specification (`docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` §2, §3.1, "Validation and error behavior") already allows a mode-entry failure to follow this ADR; no wording change is required by this decision, and the business analyst owns any later specification text.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| `stty` cbreak via `std::process::Command`, byte-wise parsing, append-only redraw, `KeySource`/`TerminalMode` seams (this ADR) | Selected | Meets single-keypress input with no crate, no `unsafe`, and no packaging change; `coreutils` is Essential; every failure mode has a non-fatal path. |
| Direct `termios` FFI with `extern "C"` (`tcgetattr`/`tcsetattr`) | Rejected | No crate, but introduces `unsafe` and a hand-maintained, architecture-dependent `termios` layout; more review surface than one auditable subprocess for the same effect. |
| `crossterm` / `termion` / `console` / `ncurses` crate | Rejected | New dependency against ADR-0005 and TASK-104 DoD 5, for one bounded list in one command. |
| `stty raw` | Rejected | Also disables output post-processing, so `\n` no longer returns the carriage and every redraw staircase-edges; more terminal behaviour is changed than the list needs. |
| In-place redraw with ANSI cursor-up and clear-line sequences | Rejected | Requires capability assumptions, produces unreadable captured output, and adds escape-state handling that byte-wise redraw avoids. |
| `tput`, `read -n1`, or a `bash`/`sh` helper for key reading | Rejected | Adds a shell interpreter and a second language to the runtime surface, and `read -n1` still needs a mode change from somewhere. |
| Read from a second `/dev/tty` handle | Rejected | Two handles over one terminal complicate buffering and ordering, and `stdin` is already the terminal in interactive mode. |
| Fail `init` when the terminal mode cannot be changed | Rejected | Turns a presentation requirement into a hard runtime dependency and blocks initialization on minimal systems. |
| Keep line-based typed tokens or per-artifact `y`/`n` prompts | Rejected | Explicitly rejected by REQ-01.006 and the TASK-104 acceptance criteria. |
| Install signal handlers to restore the terminal on SIGINT/SIGTERM/SIGTSTP | Rejected | Needs `libc`/`unsafe` or a crate, changing the dependency baseline; residual risk recorded in decision 8 instead. |
| Full-screen/alternate-buffer TUI | Rejected | Escape-sequence and capability dependence, scrollback loss, and far more surface than one list needs. |

## References

- `docs/requirements/business/REQ-01.006.md` — interactive cursor-driven init list.
- `docs/requirements/business/REQ-01.001.md`, `REQ-01.002.md`, `REQ-01.003.md`, `REQ-01.007.md` — init behavior, configuration access, context selection, `id_padding`.
- `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` — §1.2, §2, §3, §4, "Validation and error behavior".
- `docs/business-rules/BR-016-configuration-keys-and-precedence.md` — activation via `enabled` flags and core artifacts.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust.
- `docs/architecture/adr/ADR-0002-distribution-format-amd64-deb-packages.md` — `.deb` payload.
- `docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md` — Debian forky only.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single crate.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — no new dependency, single binary, offline.
- `docs/architecture/adr/ADR-0014-dependency-free-tty-key-input-for-interactive-init.md` — competing Proposed route for the same question (in-process FFI); only one of the two may be selected.
- `docs/requirements/non-functional/NFR-01.003.md` — offline operation; `NFR-03.001.md` — Rust; `NFR-04.002.md` — project-directory boundary.
- Implementation: `src/init.rs` (`run_init`, `run_init_with_io`, `build_config_interactive`), `src/error.rs` (`AppError`), `src/main.rs` (exit code propagation), `src/sanitize.rs` (prompt validation).
- Backlog: TASK-104.
