---
id: TECH-103
title: "Interactive Init Terminal Input"
created_date: '2026-11-25T20:53:42Z'
updated_date: '2026-11-27T19:12:18Z'
labels: []
references:
  - REQ-001.006
  - NFR-001.003
  - NFR-003.001
status: verified
---
# TECH-103 Interactive Init Terminal Input

## Description
Covers the full-screen terminal selector used by the init record-type list: detection, screen lifecycle, layout, key handling, help overlay, resizing, test seams, degraded path, and interrupt exit. Offered rows and prefix/folder prompts are out of scope.

## Detection

- The full-screen selector runs only when stdin and stdout are both terminals.
- It also degrades when `TERM` is `dumb`; no other `TERM` inspection, terminfo lookup, or capability probing is performed.
- The line-based fallback runs when either stream is not a terminal, `TERM` is `dumb`, `stty` fails, or the terminal is smaller than the minimum size.
- No new CLI flag, environment variable, or configuration key is introduced; there is no `/dev/tty` probing.

## Screen lifecycle

- The selector enters the alternate screen with `\x1b[?1049h` and hides the cursor with `\x1b[?25l`.
- The guard restores the cursor with `\x1b[?25h`, leaves the alternate screen with `\x1b[?1049l`, and restores `stty` settings.
- Restoration runs on Enter, end of input, Ctrl-C, an I/O error, a degrade, and from `Drop`.
- A failed restore prints one line telling the user to run `reset` or `stty sane`.
- Colors are ANSI SGR sequences; `NO_COLOR` and `--no-color` suppress SGR but keep the layout.

## Terminal mode

- The mode is changed out of process with `stty` from Debian `coreutils`, through `std::process::Command`; never in process, and with no termios FFI.
- The argument vector is exactly `stty -icanon -echo -isig -ixon min 1 time 0`.
- `-icanon` makes each byte available immediately; `-echo` leaves all drawing to Papyrus; `-isig` delivers Ctrl-C, Ctrl-\ and Ctrl-Z as bytes; `-ixon` prevents Ctrl-S from freezing the session; `min 1 time 0` blocks for one byte.
- `raw` is never used, because it also clears output post-processing (`opost`).
- `stdin` is inherited with `Stdio::inherit()`; `Command::output()` is not used because it attaches `/dev/null` to stdin.
- `stty -g` is captured before the change and restored by the guard.

## Layout

- The terminal size is read with `stty size` before each redraw.
- Minimum size is 18 rows and 60 columns; below it the selector degrades to the line-based path.
- Line 1 is the title `Papyrus init · <NAME>`; line 2 is a full-width rule.
- Line 3 is the status line: the current key hint or the last error.
- The table header is `  Name                  Kind        Prefix    Folder`.
- Rows are grouped under `Backlog`, `Decisions`, and `Documents` headings.
- A selectable row is `[x]` or `[ ]`, the name, the kind, the prefix, and the folder.
- Backlog rows are always `[x]` and carry the marker `locked`; the cursor never enters them.
- The cursor row is prefixed with `>` and drawn in reverse video when colour is enabled.
- Text wider than the terminal is clipped with an ellipsis; nothing wraps.
- The footer carries `j/k move · space toggle · a all · n none · enter confirm · ? help · ctrl-c cancel`.
- The row area is a viewport: when entries exceed the available lines it scrolls to keep the cursor visible, and the status line shows the visible range.
- The whole frame is redrawn with `\x1b[H\x1b[2J` followed by positioned writes; no partial diffing is required.

## Keys

| Input | Meaning |
|---|---|
| `j`, down (`ESC [ B`, `ESC O B`) | next selectable row |
| `k`, up (`ESC [ A`, `ESC O A`) | previous selectable row |
| `0x20` (space) | toggle the highlighted row |
| `a` | select every selectable row |
| `n` | clear every selectable row |
| `0x0A`, `0x0D` | confirm and continue |
| `?` | open or close the help overlay |
| `0x04` (Ctrl-D) | end of input — confirm and continue |
| `0x03` (Ctrl-C) | interrupt |
| `read` returning `Ok(0)` | end of input — confirm and continue |
| `ErrorKind::Interrupted` | retry the read |
| anything else | ignore; redraw only |

- A lone `0x1B` or an unrecognised escape sequence is ignored; at most two further bytes are read and discarded.
- Navigation clamps at the first and last selectable row.
- Ctrl-\ (`0x1C`) and Ctrl-Z (`0x1A`) are ignored bytes.

## Help overlay

- `?` draws a centred panel over the frame listing the keys and their meanings.
- Any key closes the panel and redraws the selector without changing selection; Ctrl-C still interrupts.
- The overlay reads no record and writes no file.

## Resize

- No signal handler is installed. The size is re-read before each redraw and after each key.
- A size change re-lays out the frame; a size below the minimum degrades to the line-based path with the current selection preserved.
- A resize with no key press is picked up on the next key.

## Test seams

- `KeySource`: `fn next_key(&mut self) -> io::Result<Key>`; `ByteKeySource` is the real adapter and `ScriptedKeySource` models input, end of input, and interrupt.
- `TerminalMode`: `enter()` and `restore()`; `SttyTerminalMode` is real and a fake exercises the degraded path.
- `SizeSource`: returns rows and columns; a fake covers resize and too-small states.
- `render(state, size, color) -> String` is pure and snapshot-testable.
- The state machine takes `&mut dyn KeySource`, `&mut W: Write`, an initial selection map, and a size; it performs no other I/O and owns cursor and selection state.

## Degraded path

- The fallback renders the list once with the current selection, then a line-based confirmation prompt where Enter, empty input, or end of input confirms.
- It never accepts typed record-type tokens and never unselects backlog entries.
- It cannot change selection; the selector state carries over if the fallback is entered after a TUI session.
- It never fails the walkthrough because of the terminal.

## Interrupt

- Ctrl-C arrives as `0x03` and maps to `AppError::Interrupted`, code `130`, message `interrupted`; `main` exits with `err.code()`.
- The interrupt is raised before `Config::validate()`, any directory creation, and the configuration write.
- The guard restores cursor, alternate screen, and terminal mode before the error propagates.
- The prefix and folder prompts stay canonical; Ctrl-C there keeps the kernel SIGINT disposition and exits 130.

## Boundaries

- No crate, build, or dev dependency is added by this behaviour; the packaging layout is owned by the distribution specification (TECH-106).
- No `unsafe` block, FFI declaration, or signal handler is added.
- No shell interpolation and no user-controlled argument reaches `stty`; only the fixed mode vector, `-g`, and the saved-settings restore are used.
- External signals, a crash, or an abort can terminate the process without running `Drop`, leaving the terminal in the alternate screen or modified mode until `reset` or `stty sane`; the exposure is bounded to the list phase.
- The mechanism is POSIX-terminal specific; Windows and non-Debian platforms are out of scope.
