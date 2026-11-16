---
id: ADR-0014
title: "Dependency-Free TTY Key Input for the Cursor-Driven Init Artifact List"
created_date: '2026-11-13T00:00:00Z'
updated_date: '2026-11-16T21:08:36Z'
labels: []
references: []
---
# Dependency-Free TTY Key Input for the Cursor-Driven Init Artifact List

## Status
superseded
## Date

2026-11-13 (status: Superseded, 2026-11-13)

## Context

REQ-01.006 requires the `papyrus init` walkthrough to present the artifact set as a single cursor-driven list: exactly one highlighted selectable row, Up/Down and `k`/`j` navigation, Space toggling the highlighted row, Enter confirming, a redraw after every keypress, clamped navigation, core rows (task, milestone, release) never highlighted, and end of input confirming the current state and continuing. The specification (`docs/specifications/interactive-init-walkthrough-for-artifact-activation.md`, TASK-104 revision) fixes the behavior but deliberately leaves one architecture question open: section 2 requires per-keypress input, and the validation section states that a raw/cbreak mode entry failure must "follow the architecture decision for TASK-104".

The delivered behavior in `src/init.rs` (`build_config_interactive`) renders a static list and then reads one line of space-separated tokens. That is the conformance defect: a line-oriented read cannot see individual keypresses, cannot move a highlight, and is the typed-token activation REQ-01.006 rejects.

Forces:

- **Dependency baseline.** ADR-0001 selects Rust and Cargo, ADR-0005 fixes the dependency-free, single-binary, offline posture, and ADR-0004 fixes the repository-root single crate. The crate depends only on `clap`, `serde`, `serde_json`, and `yaml_serde`; no terminal crate and no `libc` crate is present or permitted without a new ADR.
- **Platform.** ADR-0003 narrows support to Debian Forky on amd64, whose C library is glibc; no Windows/macOS terminal abstraction is required, and the glibc Linux `termios` layout is a single, stable target.
- **Toolchain.** Rust edition 2024 with `rust-version` 1.99.1: `unsafe extern` blocks are mandatory, and `IsTerminal`, `offset_of!`, and `OwnedFd` are in `std`.
- **No new user surface.** REQ-01.006 forbids a new CLI flag or environment variable, so key input follows the existing `std::io::IsTerminal` check on stdin.
- **Phase hand-off.** Interactive mode runs only when stdin is a terminal; the non-interactive/CI path must not change and must not read stdin. The prefix and folder prompts after the list stay line-based (`BufRead::read_line`) on the same stdin, so input buffered during the list must not be lost.
- **Terminal hygiene.** The terminal must never be left in a modified mode: not on Enter, not on end of input, not on Ctrl-C, not on an I/O error, not on panic.
- **Testability without a dev-dependency.** Selection, navigation, and toggling must be testable in-process; the real keyboard path is a thin adapter verified on a real TTY (TASK-104 DoD).

A parallel draft for this ticket proposed changing the terminal mode out-of-process with `stty`. That option is viable and is recorded under Alternatives Considered with the reasons it was not selected; this ADR merges both drafts into one decision.

## Decision

1. **The cursor-driven list is the only activation input.** `build_config_interactive` no longer reads a token line. The list state machine consumes decoded keys only; the prefix and folder prompts after it stay line-based.

2. **Terminal access uses a minimal in-crate FFI to the C library std already links.** Two symbols are declared in a `#[cfg(target_os = "linux")]` module and used only from `src/term.rs`:

   ```rust
   #[repr(C)]
   struct Termios {
       c_iflag: u32, c_oflag: u32, c_cflag: u32, c_lflag: u32,
       c_line: u8,
       c_cc: [u8; 32],
       c_ispeed: u32, c_ospeed: u32,
   }
   unsafe extern "C" {
       fn tcgetattr(fd: i32, t: *mut Termios) -> i32;
       fn tcsetattr(fd: i32, act: i32, t: *const Termios) -> i32;
   }
   ```

   The layout is the glibc Linux generic `struct termios` (`NCCS` 32, 60 bytes, `c_cc` at offset 17, in/out speed at offset 52) and is asserted by a unit test (`size_of` plus `offset_of!`). Neither a `libc` crate nor a terminal crate is added; nothing changes in `Cargo.toml`. The symbols resolve against the libc that std already links, which is not a Cargo dependency. Non-Linux targets return `io::ErrorKind::Unsupported`, which takes the fallback in decision 6.

3. **Mode: cbreak-like with signal characters disabled.** From the snapshot taken by `tcgetattr` on fd 0, clear `ICANON`, `ECHO`, `ISIG`, and `IEXTEN` in `c_lflag`, and clear `IXON` and `ICRNL` in `c_iflag`; keep `OPOST` (so `\n` still becomes `\r\n`), set `VMIN = 1` and `VTIME = 0`, and apply with `TCSANOW`. Rationale:

   - `ICANON` off with `VMIN = 1` makes one keypress available per read.
   - `ECHO` off means Papyrus owns rendering, so the highlight is the only feedback.
   - `ISIG` off makes Ctrl-C (`0x03`), Ctrl-D (`0x04`), Ctrl-\ and Ctrl-Z arrive as bytes, so the reader can restore the terminal and exit deterministically instead of dying mid-mode; Ctrl-\ and Ctrl-Z are treated as no-ops for the short lifetime of the list.
   - `IXON` off prevents Ctrl-S from silently freezing output while the list is active.
   - `ICRNL` off makes Enter arrive as `\r`; the decoder maps both `\r` and `\n` to `Key::Enter`, so the choice is not observable.
   - `OPOST` stays enabled: `stty raw`-style full raw mode would staircase every `\n` and is deliberately not used.

4. **Restoration is guaranteed by RAII plus an explicit early release.** `TerminalKeys::enter()` snapshots termios and returns a guard; `Drop` restores it best-effort and is idempotent through an `active` flag. The `KeySource` trait exposes `fn finish(&mut self)` (default no-op) which the terminal source implements as the same restore, and the list phase calls `keys.finish()` immediately after confirmation, **before** the first line prompt, so the prefix/folder prompts run in canonical mode with echo. The release profile does not set `panic = "abort"`, so `Drop` also runs during unwinding: `?` returns and panics both restore. The modified-mode window is limited to the list itself.

5. **Key layer is artifact-agnostic, reads the same stream as the prompts, and is injectable.** `src/term.rs` owns:

   ```rust
   pub enum Key { Up, Down, Enter, Eof, Interrupt, Char(char), Unknown }

   pub trait KeySource {
       fn next_key(&mut self, input: &mut dyn BufRead) -> io::Result<Key>;
       fn finish(&mut self) {}
   }

   pub struct TerminalKeys { /* saved termios + active flag */ }  // TerminalKeys::enter() -> io::Result<Self>
   pub struct EofKeys;                                            // always yields Key::Eof
   ```

   Decoding reads one byte at a time through `BufRead::fill_buf`/`consume` on the caller's reader, retrying `ErrorKind::Interrupted`; an empty buffer is `Key::Eof`. This is why the input is passed to `next_key` instead of the source owning a handle: the list and the later line prompts share **one** buffered reader, so type-ahead typed after Enter is not lost at the hand-off, and the real path never opens a second stdin handle. The byte map:

   | Input | Key |
   |---|---|
   | `\r` (0x0D), `\n` (0x0A) | `Enter` |
   | `ESC [ A`, `ESC O A` | `Up` |
   | `ESC [ B`, `ESC O B` | `Down` |
   | `0x03` | `Interrupt` |
   | `0x04`, empty buffer | `Eof` |
   | printable ASCII | `Char(c)` |
   | anything else, including other CSI/SS3 finals and non-ASCII bytes | `Unknown` |

   SS3 (`ESC O A/B`) covers tmux/putty. A bare `ESC` consumes the following byte, so that one keypress is swallowed (documented limitation; no timer). `Key` carries no artifact knowledge: the list maps `Char(' ')` to toggle and `Char('k')`/`Char('j')` to movement, and `Unknown` only triggers a redraw.

6. **Failure to enter the mode confirms the current state and continues.** If `tcgetattr` or `tcsetattr` fails (including the non-Linux `Unsupported` case), `run_init` prints one warning line to stderr, renders the list once in plain mode, and runs the list loop against `EofKeys`. EOF already means "confirm the current state and continue", so no new fallback semantics are invented and no extra text-line prompt is added: the defaults stay selected, the walkthrough proceeds to the canonical prefix/folder prompts, and nothing blocks. Interactive detection stays `std::io::stdin().is_terminal()`; no flag or environment variable is introduced.

7. **Ctrl-C aborts with exit code 130; Ctrl-D/EOF confirms and continues.**

   - Because `ISIG` is off, Ctrl-C arrives as `0x03` and is mapped to `Interrupt`. The list phase restores the terminal first, then returns a new `AppError::Interrupted` (`code()` 130, message `interrupted`); `main` already exits with `err.code()` and, in `--json`, prints the error object.
   - The interrupt is raised while the list phase runs, before `Config::validate()`, before directory creation, and before the `papyrus.config.yml` write, so no configuration file and no artifact directories are produced.
   - Ctrl-\ and Ctrl-Z are bytes under `ISIG` off and are ignored, never treated as confirmation or as signals.
   - The prefix/folder prompts remain canonical, where Ctrl-C keeps its default SIGINT disposition and the kernel already terminates with 130. Exit status 130 is therefore uniform across the walkthrough; only the mechanism differs.

8. **Rendering is a full redraw of the list, with ANSI only on a terminal.** Frames use a `>` cursor marker and `[x]`/`[ ]` markers, no colour, and core rows carry a fixed active marker plus an "always active" note but no cursor position. When stdout is a terminal, a frame is drawn by moving the cursor up over the previous frame and erasing each line (`\r\x1b[{n}A`, `\x1b[2K`); when it is not, a complete fresh copy of the list is appended per keypress. The mode is decided once by `std::io::stdout().is_terminal()` in `run_init` and passed down as a plain `ansi: bool`, so no `Write` implementation needs `IsTerminal`, no terminal database or `TERM` inspection is required, and captured output is free of escape sequences.

9. **Module boundaries inside the single crate.**

   - `src/term.rs` — `Key`, `KeySource`, `TerminalKeys` with the FFI and termios guard, `EofKeys`, and the byte decoder. It knows nothing about artifacts or configuration.
   - `src/init.rs` — `run_init`, `run_init_with_io` (signature unchanged), the new `run_init_with_io_and_keys`, `build_config_interactive`, the list loop (key → action, frame), and the prompt helpers.
   - `src/init/list.rs` — private submodule with `ArtifactList`: ordered rows (core, then decision types, then document types in schema order), the selectable-row index, `up`/`down` with clamping, `toggle`, and `render`. No IO, no knowledge of `term`.

   `run_init_with_io_and_keys<R: BufRead, W: Write>(dir, opts, reader: &mut R, writer: &mut W, keys: &mut dyn KeySource, ansi: bool)` takes the key source as a trait object so the terminal source owns no reader borrow and the test source can be a scripted sequence. `run_init_with_io` keeps its current signature and becomes a thin chooser: interactive → `TerminalKeys::enter()` or the `EofKeys` fallback; non-interactive → `EofKeys` with `ansi = false` and no stdin reads. Existing callers and tests of `run_init_with_io` are unaffected.

10. **Tests.** Deterministic coverage is in-process and needs no terminal: unit tests in `src/init/list.rs` for the state machine (initial selection, movement, clamping, toggling, core rows unreachable, rendering) and for the `Termios` layout assertion in `src/term.rs`; an integration test that calls `run_init_with_io_and_keys` with a scripted `KeySource` and a byte buffer writer, asserting the written `papyrus.config.yml` and the emitted frames (including a confirm followed by piped prompt answers, to prove the buffering hand-off). A `#[ignore]`d test drives the real binary under `script(1)` (util-linux) as an optional PTY gate; the real-TTY walkthrough is manual DoD evidence recorded in TASK-104. The FFI behavior itself has no deterministic automated test; the guarded window is small and the layout is asserted.

11. **No new dependency, flag, or environment variable**, and no change to activation semantics: defaults all active, core artifacts always active, inactive types written with `enabled: false`, rows ordered core → decision types → document types.

## Consequences

**Positive:**

- REQ-01.006 is implementable with no dependency change: the ADR-0005 baseline exists untouched and the `.deb` payload stays a single binary.
- Every interactive `init` runs in-process: no subprocess per run, no dependency on an external binary, and precise `errno` from the two calls.
- The terminal-state contract is structural: `finish()` before the line prompts plus idempotent `Drop` means no return path can leave the terminal modified.
- The list state machine is IO-free and the interactive flow is testable in-process without a PTY.
- Sharing one buffered reader between the list and the prompts removes a whole class of type-ahead loss and keeps the non-interactive path untouched.
- Ctrl-C, Ctrl-D, and a failed mode change all have one documented, non-blocking behavior; exit 130 is uniform across the walkthrough.

**Negative:**

- The crate gains `unsafe` FFI and a hand-declared, platform-specific `termios` layout; that is the maintenance price of avoiding a crate.
- Only Linux is implemented; other platforms always take the `EofKeys` fallback (acceptable under ADR-0003, but not portable).
- The decoder is hand-written: arrows are covered in CSI and SS3 form, a bare `ESC` swallows one keypress, and non-ASCII input is ignored.
- `ISIG` off makes Ctrl-Z/Ctrl-\ inert during the list, and an externally delivered `SIGTERM`/`SIGKILL` during the list can still leave the terminal non-canonical.
- Exit code 130 is a new user-visible code; it is conventional, but the requirement's error table must acknowledge it.
- Aborting on Ctrl-C is a deliberate behavior change from "Enter keeps defaults"; users who pressed Ctrl-C to escape a hung prompt now abort `init` (with nothing written).

**Risks and mitigations:**

- *Termios layout drift or wrong constants*: one module, `#[repr(C)]` matching the documented glibc layout, a size/offset assertion, and the PTY/manual gate; a layout change requires a new ADR.
- *Terminal left in cbreak mode*: `finish()` before the prompts, idempotent `Drop`, no `panic = "abort"`, and a window limited to the list. External `SIGTERM`, `SIGHUP`, `SIGQUIT`, `SIGPIPE`, `SIGSTOP`, or a crash during the list remain accepted residual risks; the recovery action is `stty sane`, stated in the warning only if a restore attempt fails.
- *Swallowed or reordered input at the hand-off*: the list and the prompts share one `BufRead`; the decoder never creates a second reader; the integration test covers confirm-then-piped-line.
- *Escape-sequence misparse*: the decoder consumes at most two bytes after `0x1B`, ignores unmatched sequences, and tests cover `ESC [ A`, `ESC O B`, and a truncated sequence.
- *Wrong confirmation/activation pair*: one state machine feeds both the rendered frame and the written configuration; the end-to-end assertion is on `papyrus.config.yml`.
- *Dependency creep*: the ADR forbids `libc` and terminal crates; review checks `Cargo.toml` and that `unsafe` exists only in `src/term.rs`.
- *`--json init` on a TTY*: the walkthrough still writes to stdout ahead of the JSON document (pre-existing, out of scope; tracked as a residual issue for the requirement owner).
- *Non-TTY capture under plain mode*: output grows with keypresses; bounded by the artifact set and accepted.

## Downstream constraints

1. **No dependency change.** `Cargo.toml` must not gain `libc`, `crossterm`, `termion`, `ratatui`, `console`, or any other crate. The only foreign symbols are the two in decision 2.
2. **`unsafe` is confined to `src/term.rs`**, each block carries a `// SAFETY:` comment, and `term` exposes safe functions only (`enter`, `next_key`, `finish`).
3. **Restoration on every path**: `finish()` after the list and before the first line prompt, idempotent `Drop`, and no `return` between entering the mode and the list loop that bypasses the guard.
4. **No new CLI flag or environment variable**; interactive detection remains `std::io::stdin().is_terminal()`; ANSI output is gated on `stdout().is_terminal()` and must not appear in captured output.
5. **`run_init_with_io` keeps its signature**; the non-interactive path performs no stdin reads and writes the same defaults as today. New behavior is reachable only through `run_init_with_io_and_keys`.
6. **List semantics fixed by REQ-01.006**: initial highlight on the first selectable row (`adr`), navigation over selectable rows only, clamped at both ends, Space toggles only the highlighted row, core rows never highlighted or toggled, rows ordered core → decision types → document types, defaults all active, Enter and EOF both confirm the displayed state.
7. **`term` stays artifact-agnostic**; no artifact name, configuration key, or prompt text may appear in `src/term.rs`.
8. **Exit code 130**: `AppError::Interrupted` is raised only for the interrupt key in the list phase, maps to `code()` 130, and is not reused for other aborts; existing codes 1, 3, 4 are unchanged.
9. **Tests**: injected-`KeySource` unit and integration tests are required in the default `cargo test`. The PTY test is `#[ignore]`d and environment-dependent (`script` from util-linux, a usable PTY); real-TTY verification is manual DoD evidence in TASK-104. Do not add a dev-dependency to obtain a PTY.
10. **Requirement and specification coordination** (owned by `requirements-engineer`/`business-analyst`, not by this ADR): REQ-01.006's error behavior should acknowledge exit code 130 for Ctrl-C, and `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` "Validation and error behavior" may cite ADR-0014 as the resolved mode-failure decision. No behavioral change to the specification is required.
11. **Ticket evidence**: TASK-104 must state which paths were verified (injected source, PTY gate, real TTY) and that Ctrl-C aborts with 130, Ctrl-D confirms and continues.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Inline `tcgetattr`/`tcsetattr` FFI with an RAII termios guard (this ADR) | Selected | Per-keypress input with zero new dependencies, no subprocess, exact `errno` and exact saved state; Linux-only in line with ADR-0003. |
| `stty` cbreak via `std::process::Command` (`stty -icanon -echo -isig -ixon min 1 time 0`, saved with `stty -g`) | Rejected | Viable and avoids `unsafe`, but adds a subprocess per interactive run and a second executable dependency, transports the saved state as an opaque string, and adds spawn/status/parse failure modes plus a third fallback prompt for the same effect; the crate's only other external command is an optional `git init`. |
| `libc` crate | Rejected | A Cargo dependency for two symbols, against ADR-0005's baseline. |
| `crossterm`/`termion`/`ratatui`/`console`/`ncurses` | Rejected | New dependency and a large cross-platform surface for one bounded list on one platform. |
| Append-only redraw only (no ANSI, ever) | Rejected | Readable in logs, but repeats the whole list per keypress on a real terminal, which is exactly where the highlight must be stable; the TTY-gated ANSI path gives both readability and a usable cursor. |
| ANSI redraw without a non-TTY fallback | Rejected | Escape sequences would pollute logs whenever stdin is a TTY and stdout is redirected. |
| Full raw mode / `cfmakeraw` | Rejected | Also clears `OPOST`, so every `\n` staircases; more mode change than the list needs. |
| Change the terminal mode and also keep typed tokens as a key source | Rejected | REQ-01.006 rejects typed-token activation outright. |
| Per-artifact `Activate? [Y/n]` prompts | Rejected | Replaced by the maintainer decision of 2026-11-11. |
| Read keys from a second `/dev/tty` handle | Rejected | Two handles over one terminal complicate ordering and lose the shared-buffer hand-off; stdin is already the terminal in interactive mode. |
| Map Ctrl-C to `AppError::Runtime` / exit 1 | Rejected | Makes a user abort look like a runtime failure and diverges from the 130 the kernel produces for Ctrl-C during the line prompts. |
| Install signal handlers to restore the terminal on SIGTERM/SIGHUP/SIGTSTP | Rejected | Broader `unsafe` surface than the rest of the feature; residual risk is documented and the recovery action is `stty sane`. |
| PTY-based automated test built on `/dev/ptmx` FFI | Rejected | More unsafe test code than the feature itself for a path already covered by a manual real-TTY gate; `script(1)` is used behind `#[ignore]` instead. |
| Fail `init` when the terminal mode cannot be changed | Rejected | Turns a presentation requirement into a hard runtime dependency and blocks initialization on minimal systems. |

## References

- `docs/requirements/business/REQ-01.006.md` — cursor-driven artifact list, TTY-only execution, no new flag or environment variable.
- `docs/requirements/business/REQ-01.001.md`, `REQ-01.002.md`, `REQ-01.007.md` — init behavior, configuration access, `id_padding`.
- `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` — §2 presentation contract, §3 prompts, "Validation and error behavior".
- `docs/business-rules/BR-016-configuration-keys-and-precedence.md` — activation via `enabled` flags and core artifacts.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust and Cargo.
- `docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md` — Debian Forky/amd64 only.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single-crate module layout.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — dependency-free baseline, no new dependency.
- Implementation: `src/term.rs`, `src/init.rs`, `src/init/list.rs`, `src/error.rs` (`AppError::Interrupted`), `src/main.rs` (exit-code propagation); defect at `src/init.rs` `build_config_interactive`.
- Tests: `src/term.rs` and `src/init/list.rs` unit tests, `tests/init_interactive.rs`, `tests/init_tty.rs` (`#[ignore]`, `script(1)`).
- Platform reference: glibc `bits/termios-struct.h` (`NCCS` 32, 60-byte `struct termios`) and `bits/termios-c_lflag.h` (`ISIG` 0x1, `ICANON` 0x2, `ECHO` 0x8, `IEXTEN` 0x8000).
