# Dependency-Free TTY Key Input for the Cursor-Driven Init Artifact List

## Status

Superseded by [ADR-0015](ADR-0015-cursor-driven-init-list-via-stty-raw-mode.md) (2026-11-13).

Resolution: ADR-0014 and ADR-0015 answered the same question for the same code path and were mutually exclusive. On 2026-11-13 the user selected ADR-0015 (`stty` cbreak mode with a `KeySource`/`TerminalMode` seam) as the accepted route for REQ-01.006 (TASK-104). ADR-0014 is superseded without having been accepted: its decision, alternatives, and downstream constraints are retained for history only and MUST NOT be implemented. No `unsafe` FFI, `termios` struct, or `libc` symbol may be added to the crate for terminal handling; the rejection grounds recorded here (hand-maintained architecture-dependent layout, `unsafe` review surface) remain the architect's position.

## Date

2026-11-13 (status: Proposed, 2026-11-13; status: Superseded, 2026-11-13)

## Context

REQ-01.006 requires the `papyrus init` walkthrough to present the artifact set as a single cursor-driven list: exactly one highlighted selectable row, Up/Down and `k`/`j` navigation, Space toggling the highlighted row, Enter confirming, redraw after every keypress, clamped navigation, core rows (task, milestone, release) never highlighted, and end of input confirming the current state and continuing. The specification (`docs/specifications/interactive-init-walkthrough-for-artifact-activation.md`, TASK-104 revision) fixes the behavior but deliberately leaves one architecture question open: section 2 requires per-keypress input, and the validation section states that a raw/cbreak mode entry failure must "follow the architecture decision for TASK-104".

The delivered behavior in `src/init.rs` (`build_config_interactive`) renders a static list and then reads one line of space-separated tokens. That is a conformance defect: a line-oriented read cannot see individual keypresses, cannot move a highlight, and does not satisfy "no typed token input".

Forces:

- ADR-0001 (Rust + Cargo) and ADR-0005 (dependency-free baseline) fix the dependency posture; the crate currently depends only on `clap`, `serde`, `serde_json`, and `yaml_serde`. No terminal crate (`crossterm`, `termion`, `ratatui`, `console`) and no `libc` crate is available or permitted without a new ADR.
- ADR-0004 fixes the repository-root single crate, so the terminal layer is a module, not a crate.
- ADR-0003 narrows supported platforms to Debian Forky on amd64, whose C library is glibc; no Windows/macOS terminal abstraction is required.
- Rust edition 2024 with `rust-version` 1.99.1: `unsafe extern` blocks are mandatory, and `IsTerminal`, `offset_of!`, and `OwnedFd` are available in `std`.
- REQ-01.006 forbids any new CLI flag or environment variable, so key input cannot be switched on by configuration; it must follow the existing `std::io::IsTerminal` detection on stdin.
- Interactive mode runs only when stdin is a terminal; the non-interactive/CI path must not change and must not read stdin.
- Prefix and folder prompts that follow the list remain line-based (`BufRead::read_line`) and must keep working, including on the same stdin.
- The terminal must never be left in a modified mode, including on early return and on panic.

The decision needed is: how to obtain individual keypresses on a real TTY with no new dependency, what the fallback is when the terminal mode cannot be changed, how output is redrawn, where the code lives, and how it is tested without new dev-dependencies.

## Decision

1. **The cursor-driven list is the only activation input.** `build_config_interactive` no longer reads a token line. The list state machine consumes decoded keys only; the prefix/folder prompts after it stay line-based.

2. **Terminal access uses a minimal in-crate FFI to the C library that std already links.** Two symbols are declared in a `#[cfg(target_os = "linux")]` module and used only from `src/term.rs`:

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

   The layout is the glibc Linux generic `struct termios` (`NCCS` 32, 60 bytes, `c_cc` at offset 17, in/out speed at offset 52). No `libc` crate, no new dependency, and nothing is added to `Cargo.toml`. `libc` is **not** a dependency; the symbols resolve against the libc std already links, which is a different thing from a Cargo dependency.

3. **Mode: cbreak-like with signal characters disabled.** From the snapshot taken by `tcgetattr` on fd 0, clear `ICANON`, `ECHO`, `ISIG`, and `IEXTEN` in `c_lflag`, and clear `IXON` and `ICRNL` in `c_iflag`; keep `OPOST` (so `\n` still becomes `\r\n`), set `VMIN = 1` and `VTIME = 0`, and apply with `TCSANOW`. Rationale:

   - `ICANON` off + `VMIN = 1` yields one `read` per keypress.
   - `ECHO` off means the program owns rendering, so the highlight is the only feedback.
   - `ISIG` off makes Ctrl-C (`0x03`) and Ctrl-D (`0x04`) arrive as bytes; the reader can therefore restore the terminal and abort deterministically instead of dying mid-mode. Ctrl-Z and Ctrl-\ also become bytes and are treated as no-ops for the short lifetime of the list.
   - `IXON` off prevents Ctrl-S from freezing output while the list is active.
   - `ICRNL` off makes Enter arrive as `\r`; the decoder maps both `\r` and `\n` to `Key::Enter`, so the choice is not observable.

4. **Restoration is guaranteed by RAII plus an explicit early release.** `TerminalKeySource::enter()` snapshots termios and returns a guard; `Drop` restores it best-effort and is idempotent through an `active` flag. The trait exposes `fn finish(&mut self)` (default no-op) which `TerminalKeySource` implements as the same restore, and `build_config_interactive` calls `keys.finish()` immediately after the list is confirmed, **before** the first line prompt, so the prefix/folder prompts run in canonical mode with echo. Because the release profile does not set `panic = "abort"`, `Drop` also runs during unwinding, so `?` returns and panics both restore the terminal. The modified-mode window is limited to the list itself.

5. **Key layer is artifact-agnostic and injectable.** `src/term.rs` owns:

   ```rust
   pub enum Key { Up, Down, Enter, Eof, Interrupt, Char(char), Unknown }
   pub trait KeySource {
       fn next_key(&mut self) -> io::Result<Key>;
       fn finish(&mut self) {}
   }
   pub struct TerminalKeySource { /* saved termios + active flag */ }
   pub struct EofKeySource;   // always yields Eof; the fallback source
   ```

   `TerminalKeySource` decodes bytes: `ESC [ A` / `ESC O A` → `Up`, `ESC [ B` / `ESC O B` → `Down` (SS3 form covers tmux/putty), `\r` and `\n` → `Enter`, `0x03` → `Interrupt`, `0x04` and `read` returning `Ok(0)` → `Eof`, printable ASCII → `Char`, everything else (including other CSI/SS3 finals and non-ASCII bytes) → `Unknown`. An `ErrorKind::Interrupted` read is retried, not surfaced. A bare `ESC` consumes the following byte; the next keypress is therefore swallowed in that one case (documented limitation, no timer). `Key` carries no artifact knowledge; the list maps `Char(' ')` to toggle and `Char('k')`/`Char('j')` to movement.

   Key bytes are read through `std::io::stdin()`'s process-global buffered handle, the same buffer the later `stdin.lock()` line prompts use, so type-ahead typed after Enter is not lost at the hand-off; only type-ahead entered before Enter is consumed by the list as `Unknown`.

6. **Failure to enter the mode confirms the current state and continues.** If `tcgetattr` or `tcsetattr` fails (or the platform is not Linux, where the implementation returns `ErrorKind::Unsupported`), `run_init_with_io` prints one warning line to stderr, renders the list once in plain mode, and runs the list loop against `EofKeySource`. EOF already means "confirm the current state and continue", so no new fallback semantics are invented: the defaults stay selected, the walkthrough proceeds to the canonical prefix/folder prompts, and nothing blocks waiting on keys. Interactive detection stays `std::io::stdin().is_terminal()`; no flag or environment variable is introduced.

7. **Signal and end-of-input behavior.** `Key::Interrupt` (Ctrl-C) restores the terminal, aborts `init` with `AppError::Runtime("interrupted")` (exit code 1, the existing runtime-error code), and writes nothing, because the configuration is only written after the walkthrough returns. `Key::Eof` (Ctrl-D or closed stdin) confirms the list as shown and continues. No signal handler is installed, no signal is re-raised, and no new exit code is added. In canonical mode (the line prompts) Ctrl-C keeps its default behavior, which is safe because the terminal mode is already restored.

8. **Rendering is a full redraw of the list, with ANSI only on a terminal.** Frames are drawn with a `>` cursor marker and `[x]`/`[ ]` markers, no colour (per the specification assumption). When the writer is a terminal, a frame is drawn by moving the cursor up over the previous frame and erasing each line (`\r\x1b[{n}A`, `\x1b[2K`); when it is not, a complete fresh copy of the list is appended per keypress. The mode is decided once by `std::io::stdout().is_terminal()` in `run_init_with_io` and passed down as a plain `ansi: bool`, so no `Write` implementation needs `IsTerminal` and captured output stays free of escape sequences. Core rows are rendered with a fixed active marker and an "always active" note, with no cursor position of their own.

9. **Module boundaries inside the single crate.**

   - `src/term.rs` — key decoding, `KeySource`, `TerminalKeySource` with the FFI and termios guard, `EofKeySource`. It knows nothing about artifacts or configuration.
   - `src/init.rs` — `run_init`, `run_init_with_io` (signature unchanged), the new `run_init_with_io_and_keys`, `build_config_interactive`, and the prompt helpers. The list loop (key → action, frame) lives here.
   - `src/init/list.rs` — private submodule with `ArtifactList`: ordered rows (core, then decision types, then document types in `BTreeMap`/schema order), the selectable-row index, `up`/`down` with clamping, `toggle`, and `render`. It has no IO and no knowledge of `term`.

   `run_init_with_io_and_keys<R: BufRead, W: Write, K: KeySource>(dir, opts, interactive, reader, writer, keys: K, ansi: bool)` takes the key source **by value** so the guard cannot outlive the call and is dropped on every return path. `run_init_with_io` keeps its current signature and becomes a thin chooser: interactive → `TerminalKeySource::enter()` or the `EofKeySource` fallback; non-interactive → `EofKeySource` with `ansi = false` and no stdin reads. Existing callers and tests of `run_init_with_io` are unaffected.

10. **Tests.** Deterministic coverage is in-process and does not need a terminal: unit tests in `src/init/list.rs` for the state machine (initial selection, movement, clamping, toggling, core rows unreachable, rendering), and an integration test that calls `run_init_with_io_and_keys` with a scripted `KeySource` and a byte buffer writer, asserting the written `papyrus.config.yml` and the frames. A `#[ignore]`d test drives the real binary under `script(1)` (util-linux) as an optional PTY gate; the DoD verification on a real TTY is manual and recorded in the ticket. The termios FFI has one deterministic assertion (struct size and `offset_of!` of `c_lflag`, `c_cc`, `c_ispeed`), and its actual behavior is covered only by the PTY gate.

## Consequences

**Positive:**

- REQ-01.006 becomes implementable with zero dependency change: the dependency baseline of ADR-0005 stays intact and the `.deb` remains a single binary.
- The key layer is a small, reviewable surface (two FFI symbols, one enum, one trait) instead of a terminal crate with a large unsafe/OS matrix.
- The same seam serves the requirement and the tests: the list state machine is IO-free, and the interactive configuration flow is testable in-process without a PTY.
- `finish()` plus `Drop` make "never leave the terminal modified" structural rather than a discipline.
- The non-interactive path is untouched, so CI and existing tests keep their behavior.
- Failure to enter the mode degrades to a documented, non-blocking behavior (confirm defaults, continue) instead of hanging or inventing a second input mechanism.

**Negative:**

- The crate now contains `unsafe` FFI and hard-codes the glibc Linux `termios` layout; a target with a different layout would need review and a new ADR.
- Only Linux is implemented; other platforms silently take the `EofKeySource` fallback (acceptable: ADR-0003 supports Debian only).
- The key decoder is hand-written: escape-sequence coverage is limited to the arrow keys in CSI and SS3 form, and a bare `ESC` swallows the next keypress.
- `ISIG` is disabled while the list is active, so Ctrl-Z and Ctrl-\ are inert there and Ctrl-C is handled by us rather than by the shell; external `SIGTERM`/`SIGKILL` during the list can still leave the terminal non-canonical (the process cannot restore on `SIGKILL`).
- A no-TTY capture (`papyrus init > log`) under plain mode repeats the whole list per keypress, which is verbose but honest.

**Risks and mitigations:**

- *Termios layout drift or wrong constants*: one module, `#[repr(C)]` matching the documented glibc layout, a size/offset assertion, and the PTY/manual gate; any change to the layout requires a new ADR.
- *Terminal left in cbreak mode*: `finish()` before the line prompts, idempotent `Drop`, no `panic = "abort"`, and the modified window limited to the list. `SIGKILL`/`SIGTERM` during the list remain an accepted residual risk.
- *Buffered stdin swallows type-ahead*: the key reader reads one byte at a time through the process-global `Stdin` buffer, so bytes buffered during the list remain visible to the later `read_line` calls on the same handle; only type-ahead entered before Enter is consumed as list keys (as `Unknown`).
- *Escape-sequence ambiguity without a timer*: a lone `ESC` consumes the next byte; arrows in both common encodings are handled, everything else is a no-op redraw.
- *Accidental dependency creep*: the ADR forbids `libc` and any terminal crate; a review check on `Cargo.toml` and on `unsafe` outside `src/term.rs` keeps it honest.
- *`--json init` on a TTY*: the walkthrough still writes to stdout ahead of the JSON document (pre-existing behavior, out of scope here; tracked as a residual issue for the requirements owner).

## Downstream constraints

1. **No dependency change.** `Cargo.toml` must not gain `libc`, `crossterm`, `termion`, `ratatui`, `console`, or any other crate. FFI symbols may only be the two declared in decision 2, only in `src/term.rs`.
2. **`unsafe` is confined to `src/term.rs`**, each block carries a `// SAFETY:` comment, and `term` exposes safe functions (`enter`, `next_key`, `finish`) only.
3. **Restoration is mandatory on every path**: `finish()` after the list and before the first line prompt, idempotent `Drop`, and no early `return` between entering the mode and the list loop without dropping the guard.
4. **No new CLI flag or environment variable**; interactive detection remains `std::io::stdin().is_terminal()`. ANSI output is gated on `stdout().is_terminal()` and must not appear in captured output.
5. **`run_init_with_io` keeps its signature**; the non-interactive path performs no stdin reads and writes the same defaults as today. New behavior is reachable only through the new `run_init_with_io_and_keys` seam.
6. **List semantics are fixed by REQ-01.006**: initial highlight on the first selectable row (`adr`), navigation over selectable rows only, clamped at both ends, Space toggles only the highlighted row, core rows (task, milestone, release) never highlighted or toggled, rows ordered core → decision types → document types in schema order, defaults all active, Enter and EOF both confirm the displayed state.
7. **`term` stays artifact-agnostic**; no artifact name, configuration key, or prompt text may appear in `src/term.rs`.
8. **Tests**: the injected-`KeySource` unit and integration tests are required and must run in the default `cargo test`. The PTY test is `#[ignore]`d and environment-dependent (`script` from util-linux, a usable PTY); real-TTY verification is a manual DoD step recorded in TASK-104. Do not add a dev-dependency to get a PTY.
9. **Specification coordination**: `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` needs no behavioral change; its validation section's "follow the architecture decision for TASK-104" is resolved by this ADR and may cite ADR-0014. Spec edits remain with the business analyst.
10. **Recording the outcome**: the ticket must state which paths were verified (injected source, PTY, real TTY) and that Ctrl-C aborts with exit 1, Ctrl-D confirms and continues.

## Alternatives Considered

| Option | Outcome | Reason |
|---|---|---|
| Inline `tcgetattr`/`tcsetattr` FFI with an RAII termios guard (this ADR) | Selected | Meets per-keypress input with zero new dependencies; two symbols, one struct, Linux-only in line with ADR-0003. |
| Spawn `stty` (coreutils/util-linux) to enter and leave cbreak mode | Rejected | Adds a runtime executable dependency and a fork per run; error reporting is opaque; terminal state is carried by another process; the crate already restricts external commands to `git` and the browser opener. |
| Add the `libc` crate | Rejected | A Cargo dependency for two symbols; ADR-0005's baseline is dependency-free and `libc` is a new supply-chain entry. |
| Add `crossterm`/`termion`/`ratatui` | Rejected | New dependencies, large surface, and a cross-platform abstraction the project does not need for Debian-only support. |
| Keep the line-based token input and document it | Rejected | REQ-01.006 explicitly rejects typed tokens and requires a highlight; this is the conformance defect being fixed. |
| Per-line re-prompt ("artifact? [Y/n]") sequence | Rejected | Removed by the maintainer decision of 2026-11-11; the single cursor-driven list is the required presentation. |
| Read keys from `/dev/tty` in canonical mode | Rejected | Canonical mode delivers input only after Enter, so there is no per-keypress feedback. |
| Full `cfmakeraw` equivalent | Rejected | Clearing `OPOST`/`ICRNL` and every flag would require manual `\r\n` handling and is broader than needed; the minimal flag set in decision 3 is sufficient and easier to audit. |
| Map Ctrl-C to a new `AppError::Interrupted` with exit code 130 | Rejected | Mints a fifth exit code for a UI key; the requirement's error contract (REQ-01.006 validation section) lists only 1/2/4, and exit-code changes belong to the requirement owner. Exit 1 reuses the documented runtime-error code. |
| ANSI-only rendering with no plain fallback | Rejected | Escape sequences would pollute logs whenever stdin is a TTY and stdout is redirected. |
| A PTY-based automated test with `/dev/ptmx` FFI | Rejected for now | Adds more unsafe code in tests than the feature itself for a path already covered by a manual real-TTY gate; `script(1)` is used instead, `#[ignore]`d. |

## References

- `docs/requirements/business/REQ-01.006.md` — cursor-driven artifact list contract, TTY-only execution, no new flag or environment variable.
- `docs/specifications/interactive-init-walkthrough-for-artifact-activation.md` — TASK-104 presentation contract and validation deferral to this ADR.
- `docs/architecture/adr/ADR-0001-implementation-language-rust.md` — Rust and Cargo.
- `docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md` — Debian Forky/amd64 only.
- `docs/architecture/adr/ADR-0004-repository-root-single-crate-layout.md` — single-crate module layout.
- `docs/architecture/adr/ADR-0005-embedded-dependency-free-http-server-and-server-rendered-ui.md` — dependency-free baseline and no-new-dependency rule.
- Implementation: `src/term.rs`, `src/init.rs`, `src/init/list.rs`; existing defect at `src/init.rs` `build_config_interactive`.
- Tests: `src/init/list.rs` unit tests, `tests/init_interactive.rs`, `tests/init_tty.rs` (`#[ignore]`, `script(1)`).
- Platform reference: glibc `bits/termios-struct.h` (`NCCS` 32, 60-byte `struct termios`) and `bits/termios-c_lflag.h` (`ISIG` 0x1, `ICANON` 0x2, `ECHO` 0x8, `IEXTEN` 0x8000).
