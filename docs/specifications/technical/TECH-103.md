---
id: TECH-103
title: "Interactive Init Terminal Input"
created_date: '2026-11-25T20:53:42Z'
updated_date: '2026-11-25T20:53:43Z'
labels: []
references:
  - REQ-001.006
  - NFR-001.003
  - NFR-003.001
  - NFR-004.002
status: draft
---
# TECH-103 Interactive Init Terminal Input

## Description
Covers the terminal-input mechanism of the init artifact list: interactive
detection, mode handling, key decoding, redraw, test seams, degraded path, and
the interrupt exit. Offered rows and prefix/folder prompts are out of scope.

## Interactive Detection

- Interactive detection is `std::io::stdin().is_terminal()`.
- The modified mode is entered only when stdin is a terminal and only for the
  duration of the list phase.
- The non-interactive path reads no stdin and prompts nothing; every artifact
  stays active and defaults are written. `stty` is never invoked there.
- No new CLI flag, environment variable, or configuration key is introduced;
  no `/dev/tty` probing or `TERM` inspection is performed.

## Terminal Mode

- The mode is changed out of process with `stty` from Debian `coreutils`,
  through `std::process::Command`; never in process, and with no termios FFI.
- The argument vector is exactly `stty -icanon -echo -isig -ixon min 1 time 0`:
  - `-icanon`: each byte becomes available immediately; `-echo`: keystrokes are
    not echoed, and Papyrus renders every visible effect itself.
  - `-isig`: Ctrl-C, Ctrl-\ and Ctrl-Z arrive as bytes instead of raising
    SIGINT, SIGQUIT and SIGTSTP.
  - `-ixon`: Ctrl-S cannot freeze the session; `min 1 time 0`: a blocking read
    waits for exactly one byte.
  - `raw` is never used, because it also clears output post-processing
    (`opost`), so `\n` would no longer return the carriage.
- `stdin` is inherited with `Stdio::inherit()`; `Command::output()` is not used,
  because it attaches `/dev/null` to stdin and `stty` fails with
  "Inappropriate ioctl for device". The mode applies to the terminal already
  attached to `stdin`; no `/dev/tty` handle and no second stdin handle is used.

## Mode Guard

- Before the change, `stty -g` is captured with the same inherited stdin and
  stored in a `TerminalModeGuard`.
- The guard restores with `stty <saved>` immediately when the list phase ends
  for any reason — Enter, end of input, Ctrl-C, or an I/O error — and from
  `Drop`, so unwinding panics also restore.
- A failed restore prints one line telling the user to run `stty sane` and
  continues; the prefix and folder prompts run under canonical line discipline.

## Key Reading

- The parser reads from the existing `R: BufRead` of
  `build_config_interactive`, one byte per `read_exact` call on a one-byte
  buffer. No second `BufReader` is constructed over stdin, so typeahead after
  the confirming Enter feeds the first prefix prompt.

## Key Map

| Input | Meaning |
|---|---|
| `0x0A` (LF), `0x0D` (CR) | Enter — confirm and continue |
| `0x20` (space) | toggle the highlighted selectable row |
| `k`, `j` | move the highlight to the previous/next selectable row |
| `0x1B` `[` `A` or `O` `A`, `0x1B` `[` `B` or `O` `B` | Up, Down |
| `0x04` (Ctrl-D) | end of input — confirm and continue |
| `0x03` (Ctrl-C) | interrupt |
| `read` returning `Ok(0)` | end of input — confirm and continue |
| `ErrorKind::Interrupted` (EINTR) | retry the read |
| anything else | ignore; redraw only |

- A lone `0x1B` or an unrecognised escape sequence is an ignored key, never a
  confirmation; at most two further bytes are read and discarded.
- In the modified mode Ctrl-D is an ordinary byte, so `0x04` is handled
  explicitly rather than as the canonical end-of-file condition.
- Navigation clamps at the first and last selectable row and never enters a
  non-selectable row.

## Redraw

- Each redraw appends a blank separator line and then the complete list, with
  the cursor marker on the highlighted row and `[x]`/`[ ]` on every selectable
  row.
- No ANSI cursor movement, clear-line, alternate-screen, or colour sequence is
  emitted; no terminal database, `TERM` inspection, or capability probing is
  needed. Captured output to a file, pipe, or test buffer stays readable.

## Seams

- `KeySource`: `fn next_key(&mut self) -> io::Result<Key>`. `ByteKeySource<'a, R: BufRead>`
  is the real adapter; `ScriptedKeySource` yields a fixed sequence and models
  end of input and the interrupt key without a terminal.
- `TerminalMode`: `enter() -> io::Result<TerminalModeGuard>` and `restore()`.
  `SttyTerminalMode` is the real adapter; a fake returns a failure to exercise
  the degraded path.
- The list state machine takes `&mut dyn KeySource`, `&mut W: Write`, and an
  initial activation map, and performs no other I/O: no `std::io::stdin`, no
  `std::process`, no filesystem. It alone owns cursor and activation state.
- `run_init_with_io` keeps its signature; the TTY branch constructs the real
  adapters around the caller-provided reader and writer, the non-TTY branch
  constructs neither.

## Degraded Path

- Failure covers a spawn failure, a non-zero status from `stty -g` or the mode
  change, unparsable saved settings, or an unrestorable mode; Papyrus restores
  what it set, then prints that key navigation is unavailable here.
- The list is rendered once with every row active, followed by a line-based
  confirmation prompt where only Enter, empty input, or end of input confirms
  and continues into the unchanged prefix and folder prompts.
- The degraded path never reinstates typed artifact tokens, cannot deactivate
  types, and never fails the walkthrough because of the terminal.

## Interrupt

- Ctrl-C arrives as `0x03` and maps to `AppError::Interrupted`, whose `code()`
  is `130` and whose message is `interrupted`; `main` exits with `err.code()`.
- The interrupt is raised inside `build_config_interactive`, before
  `Config::validate()`, before any directory creation, and before the
  `papyrus.config.yml` write, so no configuration file and no project
  directories are produced.
- The `TerminalModeGuard` restores the terminal before the error propagates.
- Ctrl-\ (`0x1C`) and Ctrl-Z (`0x1A`) are ignored bytes, never a confirmation.
  The prefix and folder prompts stay canonical; Ctrl-C there keeps the default
  SIGINT disposition and the kernel exits with 130, so the status is uniform.
- Interrupt is the only list-phase exit that leaves `papyrus.config.yml`
  unwritten. End of input and the degraded path confirm the state and continue.

## Boundaries

- No crate, build, or dev dependency is added; `Cargo.toml`, the `.deb` payload,
  and its `Depends` set are unchanged. Nothing is downloaded, so offline
  operation (NFR-001.003) is unaffected.
- No `unsafe` block, FFI declaration, or signal handler is added.
- No other command, prompt, or input path changes. No ANSI escape, alternative
  screen buffer, or terminal-capability query is introduced.
- No shell interpolation and no user-controlled argument reaches `stty`; only
  the fixed vector, the `-g` query, and the saved-settings restore are used.
- The mechanism is POSIX-terminal specific; Windows and non-Debian platforms are
  out of scope. External `SIGTERM`, `SIGHUP`, `SIGQUIT`, `SIGUSR1`, `SIGUSR2`,
  `SIGKILL`, `SIGSTOP`, a crash or abort, or `SIGPIPE` while writing redraws can
  terminate the process without running `Drop`, leaving the terminal modified
  until the user runs `stty sane`. The exposure window is bounded to the list
  phase, and every normal exit path restores.
