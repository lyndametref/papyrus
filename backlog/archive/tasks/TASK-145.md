---
id: TASK-145
title: Verify the diagnostics, shell, and rendering release end to end
status: Done
assignee:
  - '@rust-dev'
created_date: '2026-11-15T16:14:00Z'
updated_date: '2026-11-17T18:43:05Z'
labels:
  - verification
  - cli
  - browser
milestone: MS-022
dependencies:
  - TASK-135
  - TASK-141
  - TASK-143
  - TASK-144
references:
  - docs/requirements/business/REQ-001.008.md
  - docs/requirements/business/REQ-006.015.md
  - docs/requirements/business/REQ-006.016.md
  - docs/requirements/business/REQ-006.017.md
  - docs/requirements/business/REQ-006.013.md
  - docs/architecture/adr/ADR-0003-supported-debian-release-target-forky-only.md
  - docs/architecture/adr/ADR-0025-bundled-katex-client-side-math-rendering.md
  - docs/architecture/adr/ADR-0026-mermaid-only-offline-diagram-rendering.md
  - >-
    docs/architecture/adr/ADR-0027-vendored-embedded-browser-assets-and-packaging-notices.md
    docs/architecture/adr/ADR-0028-browser-shell-shared-list-layout-content-width-and-collapsible-navigation.md
  - docs/packaging.md
  - docs/specifications/browser-shell-shared-list-layout-and-page-width.md
  - docs/specifications/browser-math-and-diagram-rendering.md
  - docs/specifications/browser-shell-ui.md
  - docs/specifications/browser-requirements.md
  - docs/specifications/project-diagnostics-check.md
  - docs/requirements/non-functional/NFR-005.001.md
  - docs/requirements/non-functional/NFR-003.002.md
  - README.md
  - docs/business-rules/BR-001-papyrus-command-behavior.md
  - docs/business-rules/BR-005-task-command-behavior.md
  - docs/business-rules/BR-016-configuration-keys-and-precedence.md
priority: high
ordinal: 41620
---


## Description
**Gate — cleared 2026-11-16.** ADR-0026 is `Accepted` (TASK-146/148 landed); TASK-137–149 are `Done`; m-21 is 13/14. This is the final verification gate, not a feature ticket, and it must not re-open any accepted decision.

**Why.** The increment spans the CLI, the browser shell, the packaged assets, and two offline renderers. Each delivering ticket has its own tests, but the release claim is cross-surface: one corpus must be diagnosable and repairable, uniformly browsable at every supported width, readable with math and diagrams while offline, and served from the installed package inside the binary budget.

**What.** One end-to-end verification of the delivered release increment, from a fresh project to the rebuilt and installed `.deb`, executed with a real browser where browser behaviour is claimed, with every result recorded.

## Artifact under test

Verify the code at the current `HEAD`, not the package left by TASK-149. `HEAD` has advanced past TASK-149's 2026-11-16 10:29 build (`a718f53` and `cfd4804` changed the diagram figure/focus markup and page-continuous numbering after that build), so `papyrus_0.1.0_amd64.deb` with SHA-256 `b3bc9b72…` is a pre-HEAD artifact. Rebuild the binary and package (`cargo build --release`, `./scripts/build-deb.sh`), record the new SHAs, and run the TASK-149 packaging checks against the new artifact. TASK-149 evidence transfers only when the binary SHA-256 is identical to `2a71c1af…`; otherwise the delta is re-verified here (checklist section G). This is a necessary re-baseline, not a duplicate of TASK-149.

## Surfaces and checklist

Execute the numbered checklist in the Implementation Plan, in order. Sections: **A** diagnostics (1–12), **B** shared list and board (13–19), **C** navigation and width (20–30), **D** math (31–39), **E** diagrams (40–50), **F** security and CSP (51–56), **G** package (57–62), **H** documentation and release (63–67).

Each checklist item names its fixture, route, viewport/browser, command, and expected observable result. Setup steps S1–S7 build the fixture corpus and install the package; they are prerequisites, not scored items. The fixture bodies are the exact strings in the named committed test builders (`tests/browser_math_cli.rs`, `tests/browser_diagram_cli.rs`); copy them verbatim and do not invent variants.

## Harness and browser matrix

Browser behaviour is required by AC #3–#6 and cannot be replaced by HTTP assertions. The available harness in this environment is **headless Firefox via WebDriver BiDi** at DPR 1; no Chromium/Edge or Safari binary is installed here. Use this matrix:

| ID | Browser | Role | Widths | States |
|---|---|---|---|---|
| M1 | Firefox 153.5.1esr (current installed `firefox`) | full checklist | 320, 480, 481, 500, 600, 720, 1440 | expanded, collapsed; cleared/preset `papyrus.nav` |
| M2 | Firefox 100 (NFR-005.001 floor) or the oldest supported ESR obtainable | full width/nav matrix, math and diagram smoke | same | same |
| M3 | Edge 130 (Chromium 130) | **owner manual check**, same checklist | same | same |
| M4 | Safari 18.x | **owner manual check**, same checklist | same | same |

- Viewport widths are CSS pixels at 100 % zoom with browser chrome excluded. Every sample records `document.documentElement.scrollWidth`, `document.documentElement.clientWidth`, and `getBoundingClientRect().width` for `.app` and `.main`.
- `papyrus.nav` states: **cleared** (`localStorage.removeItem("papyrus.nav")`), **preset collapsed** (`"collapsed"`), **preset expanded** (`"expanded"`), and **invalid** (`"nonsense"`). Only `"collapsed"` may collapse.
- M3 and M4 are owned by the release owner, not the implementer. The executor supplies the checklist and a results table; the owner returns version, browser binary SHA-256, and pass/fail per item. If a browser is unavailable, record `not executed — limitation` with the named fallback; **never record it as a pass**, and do not check AC #3–#6 or #9 while a required browser result is missing.
- JavaScript-disabled runs use the harness's scripting-off switch or a Firefox profile with `javascript.enabled=false`; record the method.
- Asset-failure runs use WebDriver BiDi request interception (`network.intercept`/`network.failRequest`, or the supported equivalent) to abort exactly one URL. If a browser cannot abort one request, the owner records the equivalent manual DevTools result.

## Defect-reporting rule

A defect is any deviation from an accepted requirement, specification, or ADR. It is reported against its **owning task** and does not expand TASK-150's scope:

- **Owning map:** diagnostics/CLI → TASK-137/138/139/140; shared list/board → TASK-141/142; navigation/width → TASK-144; math → TASK-145; diagrams → TASK-146; assets/notices → TASK-143; packaging evidence → TASK-149; specification or README prose → TASK-148/140.
- **Report format:** `DEF-<n>` in TASK-150's notes plus a comment on the owning task containing: expected vs observed, exact fixture and route, browser/version/viewport/state, command, decisive DOM/console/network excerpt, reproduction steps, and severity.
- **No fixing here.** Do not change `src/`, `tests/`, assets, packaging, requirements, business rules, ADRs, or specifications to make an item pass. Do not add a feature, route, script, dependency, or test to TASK-150.
- If a defect blocks an item, mark that item **failed** with the evidence and continue the independent items. If a material acceptance criterion cannot be verified, finish with status `blocked`, not `done`.

## Evidence

Record every scored item in the ticket notes as a row: `item | fixture/route | browser+version | viewport | state | command | expected | observed | pass/fail/limitation | evidence`. Record the environment once: `date -Is` with timezone, Debian release and architecture, `rustc`/`cargo` versions, built binary size and SHA-256, `.deb` size and SHA-256, and each browser's version plus browser binary SHA-256. Keep screenshots, DOM dumps, console and network logs under `./.tmp/task150/evidence/`, quote the decisive lines in the notes, and remove `./.tmp/task150` at the end; the ticket notes are the durable record.

## Out of scope

Fixes or features beyond reporting a defect; release notes; a formal security audit; PlantUML rendering; new browser support; re-opening ADR-0025/0026/0027/0028 or any accepted requirement; rewriting specification prose (a mismatch is a defect against its owning task); Debian releases other than Forky and architectures other than amd64.

## Assumptions

- ADR-0026 is `Accepted` and TASK-137–149 have landed; their status finalization does not block this gate.
- The verification host is Debian Forky amd64 with `dpkg`, `dpkg-deb`, `fakeroot`, `unshare`, `ip`, `curl`, and the Firefox WebDriver BiDi harness. Any other host is recorded as an unsupported deviation, not a pass (ADR-0003, NFR-003.002).
- The 12 MiB ceiling (`12,582,912` B) is final and is not adjusted here (ADR-0027 decision 6).
- A rebuilt artifact invalidates TASK-149's recorded SHAs by design; the new values supersede them in the evidence section, while TASK-149's dated section is left intact.
- The browser floor is Firefox 100+, Edge 130+, Safari 18+ (NFR-005.001); a browser that cannot be run is a recorded limitation, not a silent pass.

## Open questions

- The three orchestrator comments that were expected on this ticket are **not present** in the task record (`comments: []` as of refinement). The three check groups they described — browser matrix for navigation/width, KaTeX runtime, and diagram runtime — are merged into sections C–E. Were those comments lost, or is any content still missing?
- Who executes M3/M4 and when? AC #3–#6 and #9 stay unchecked until the owner returns the Edge 130 and Safari 18 results; the executor cannot self-certify them.
- Can a Firefox 100 (or oldest supported ESR) binary be obtained on the verification host? If not, M2 becomes an owner-manual result or a recorded limitation.
- Does “release-vehicle decision” mean the absence of a backlog milestone/release task plus a product `papyrus release` record, or only the backlog board? Item 67 records both and reports the decision; confirm if the owner expects a different vehicle.

## Acceptance Criteria
- [x] The diagnostics loop passes end to end: a fresh project prints no findings and exits 0; after a duplicate-ID copy and a dangling milestone are introduced, check exits 4 and names both findings with sorted locations; check --fix in a terminal repairs only the duplicate, lists every affected reference, leaves the referring file byte-identical, and exits 4; after the dangling reference is corrected the next check exits 0; a deliberately broken configuration is reported as config-invalid while the static-family findings still run; and check --fix with piped stdin and no --force exits 2 with the documented refusal.
- [x] While the configuration is invalid, other commands fail closed (task list, browser, and search exit non-zero without writing), while config set and init --force remain the repair paths and check --help exits 0.
- [x] Every family list route renders exactly one shared list table with ID and Title first and Actions last, and a count note matching the rendered rows; the board alone renders app--full and reaches the viewport width at 1440 px; both are verified from the installed package in Firefox 115 and later (the only supported browser under NFR-005.001 as amended 2026-11-16).
- [x] The navigation collapses and expands at every width in its matrix row, persists across navigation and reload through papyrus.nav, defaults to expanded when the key is cleared, invalid, or storage throws, and is expanded and usable with JavaScript disabled; verified in Firefox 115 and later. Edge 130 and Safari 18 are no longer in scope.
- [x] Math renders offline with the bundled fonts, a MathML annotation equal to data-tex, and an aria-hidden .katex-html tree; display and inline forms differ; an aborted or throwing KaTeX asset keeps the source and sets data-render-error=math; malformed math shows KaTeX error output; all nine promised Mermaid types render, while PlantUML, literal, unknown, and over-limit input keep the source with the specified note; per-figure isolation and the missing-bundle branch pass; verified in Firefox 115 and later.
- [x] The final CSP matches the accepted literals per response class, with script-src 'self' and font-src 'self' and the accepted style-src; browser enforcement blocks no required asset, and the hostile math and diagram fixtures produce no script, handler, or remote reference and execute nothing; verified in Firefox 115 and later.
- [x] The rebuilt package contains exactly the binary and the copyright file, the four notices and 26/26 asset digests hold (or TASK-149 is cited for an identical binary), the binary is within 12,582,912 B, the installed package renders offline, and no runtime asset directory, config file, or man page exists.
- [x] README and BR-001/BR-005/BR-016 describe the shipped check surface, the shell/width, math/diagram, shell-UI, and requirements section 9.7 specifications match the observed contract, and a dated TASK-150 evidence section exists in docs/packaging.md without rewriting earlier sections.
- [x] Browser versions and binary SHA-256 values, binary and .deb sizes, and headroom are recorded; no release milestone or release ticket was created by m-21; the release-vehicle decision is reported to the owner through @orchestrator; and the DEF-1 Mermaid-on-Firefox-100 finding is recorded as obsolete under the amended NFR-005.001 (Firefox 115 floor) rather than as an open defect.

## Definition of Done
- [x] #1 Every numbered checklist item (1–67) is executed, and each has its fixture/route, browser+version and viewport/state where applicable, command, expected result, observed result, pass/fail/limitation, and evidence recorded in the ticket notes.
- [x] #2 The browser matrix is complete for M1 and M2, and the owner-manual results for M3 (Edge 130) and M4 (Safari 18.x) are recorded with version and browser binary SHA-256; every width 320/480/481/500/600/720/1440 and both expanded/collapsed states are covered, with cleared and preset papyrus.nav; any unavailable browser is recorded as a limitation with the named fallback, never as a pass.
- [x] #3 The artifact identity is recorded: current HEAD, built and packaged binary sizes and SHA-256 values, .deb size and SHA-256, isolated install path, and the packaged-binary-equals-built proof; a rebuilt artifact has its TASK-149 package checks re-run as a delta, and a SHA-identical artifact cites TASK-149.
- [x] #4 Every measured value (binary size, .deb size, headroom, asset counts, font counts, figure/type counts, viewport widths) and every deviation from TASK-149, the accepted specs, or the browser matrix is recorded; no acceptance criterion is waived.
- [x] #5 Any defect is reported against its owning task with reproduction steps and evidence, and no fix, feature, route, dependency, test, or product-source change is made in TASK-150; `git diff` shows only the dated docs/packaging.md evidence section and this ticket's notes.
- [x] #6 The dated TASK-150 evidence section exists in docs/packaging.md, the TASK-149 browser limitation now points to it, and no live procedure, §3.6 text, or earlier dated section was rewritten.
- [x] #7 The release-vehicle decision is recorded and reported to @orchestrator, and no release milestone or release ticket was created by m-21.
- [x] #8 All nine acceptance criteria are checked only after the evidence above exists; otherwise the ticket remains Refined/blocked with the unresolved items named.


## Implementation Notes
Execute S1–S7 once, then items 1–67 in order. Record every scored item as specified in the Evidence section of the description. Do not edit product source, tests, assets, packaging, requirements, business rules, ADRs, or specifications.

**Setup (prerequisites, not scored)**

**S1 — Environment.** Start a clean shell with `set -u`. Record:

```bash
TMP="$PWD/.tmp/task150"; rm -rf "$TMP"; mkdir -p "$TMP/evidence"
date -Is; cat /etc/debian_version; uname -m
rustc --version; cargo --version
git rev-parse HEAD; git status --porcelain
firefox --version
FB="$(readlink -f "$(command -v firefox)")"; echo "$FB"; sha256sum "$(dirname "$FB")/firefox-bin"
```

Any host other than Debian Forky amd64 is a recorded deviation, not a pass (ADR-0003, NFR-003.002). M3/M4 owner results record `msedge --version`/Safari version and `sha256sum`/`shasum -a 256` of the browser binary.

**S2 — Shared fixture base.** Copy the characterization project and prove it is unchanged:

```bash
cp -r tests/fixtures/characterization/lists/project "$TMP/project"
find "$TMP/project" -type f | wc -l   # record the baseline count
```

**S3 — Fixture corpus.** Set `REQ-001.001`'s front-matter `title` to `Requirement One $title$` and add the label `$lab$`. Materialize the exact bodies below. Bodies marked “test constant” are copied verbatim from the named builder; do not retype or vary them. Front matter stays as the copied record unless stated.

| ID | Path | Body source | Role |
|---|---|---|---|
| NOTE-001 | `docs/notes/NOTE-001.md` | test constant `REQ_BODY` in `tests/browser_math_cli.rs` | inline, display, fenced math, escaped `\$5` |
| NOTE-002 | `docs/notes/NOTE-002.md` | `## Description` followed by `Plain note body with no formulas.` | math-free page |
| NOTE-005 | `docs/notes/NOTE-005.md` | test constant hostile note in `tests/browser_math_cli.rs` (`NOTE-005` block) | hostile TeX/HTML |
| NOTE-006 | `docs/notes/NOTE-006.md` | test constant Markdown note in `tests/browser_math_cli.rs` (`NOTE-006` block) | literal Markdown/no math |
| REQ-001.001 | `docs/requirements/business/REQ-001.001.md` | test constant `REQ_BODY` in `tests/browser_diagram_cli.rs` | ten fences covering the nine promised types, plus inline `$x$` |
| NOTE-010 | `docs/notes/NOTE-010.md` | test constant `NOTE-010` in `tests/browser_diagram_cli.rs` | `plantuml` and `puml` |
| NOTE-011 | `docs/notes/NOTE-011.md` | test constant `NOTE-011` in `tests/browser_diagram_cli.rs` | literal `uml`, `dot`, `graphviz`, `mmd`, `rust` |
| NOTE-012 | `docs/notes/NOTE-012.md` | test constant `NOTE-012` in `tests/browser_diagram_cli.rs` | unknown type, `flowchart-elk`, malformed source |
| NOTE-013 | `docs/notes/NOTE-013.md` | test constant over-byte body in `tests/browser_diagram_cli.rs` | 100,001-byte cap |
| NOTE-014 | `docs/notes/NOTE-014.md` | test constant over-line body in `tests/browser_diagram_cli.rs` | 1,001-line cap |
| NOTE-015 | `docs/notes/NOTE-015.md` | test constant isolation body in `tests/browser_diagram_cli.rs` | over-limit figure followed by a valid one |
| NOTE-016 | `docs/notes/NOTE-016.md` | test constant hostile diagram in `tests/browser_diagram_cli.rs` | hostile diagram, click directive |
| NOTE-017 | `docs/notes/NOTE-017.md` | test constant `NOTE-017` in `tests/browser_diagram_cli.rs` | math only, no diagram |
| NOTE-018 | `docs/notes/NOTE-018.md` | test constant `NOTE-018` in `tests/browser_diagram_cli.rs` | PlantUML only |
| NOTE-019 | `docs/notes/NOTE-019.md` | test constant `NOTE-019` in `tests/browser_diagram_cli.rs` | no prose marks |
| NOTE-030 | `docs/notes/NOTE-030.md` | `## Description\n\nBroken $\frac{1}{$ and unknown $\notacommand{x}$.` | malformed math |
| NOTE-031 | `docs/notes/NOTE-031.md` | see body below | long display formula at 320 px |
| NOTE-032 | `docs/notes/NOTE-032.md` | see body below | wide 25-node diagram |
| TASK-001 | `backlog/tasks/TASK-001.md` | `## Description` + `Task math $t=1$.` + `## Acceptance Criteria` + `- [ ] Acceptance keeps $x$ here.` | curated-field exclusion and search parity |

NOTE-031 body:

```
## Description

$$\sum_{i=1}^{100} i + \frac{a}{b} + \frac{c}{d} + \cdots + \frac{z}{1}$$
```

NOTE-032 body: a `mermaid` fence containing `flowchart LR` and the 24 edges `N01 --> N02`, `N02 --> N03`, …, `N24 --> N25`, one edge per line.

**S4 — Diagnostics project.** Create a clean project with three tasks. Keep `$TMP/diag` clean until item 3.

```bash
PB="$PWD/target/release/papyrus"
"$PB" init Verify --dir "$TMP/diag" --no-git
"$PB" --dir "$TMP/diag" task create "Keeper"
"$PB" --dir "$TMP/diag" task create "Referring"
"$PB" --dir "$TMP/diag" task create "Dangling"
```

**S5 — Broken-configuration projects.** Copy the clean project twice so each repair path is exercised on its own copy:

```bash
cp -a "$TMP/diag" "$TMP/broken"
cp -a "$TMP/diag" "$TMP/broken-init"
sed -i 's/^statuses:.*/statuses: []/' "$TMP/broken/papyrus.config.yml"
sed -i 's/^statuses:.*/statuses: []/' "$TMP/broken-init/papyrus.config.yml"
```

**S6 — Build, package, install.** Build from current `HEAD`; do not reuse the stale TASK-149 package.

```bash
cargo build --release
stat -c %s target/release/papyrus; sha256sum target/release/papyrus
rm -f papyrus_0.1.0_amd64.deb && ./scripts/build-deb.sh
stat -c %s papyrus_0.1.0_amd64.deb; sha256sum papyrus_0.1.0_amd64.deb
dpkg-deb --fsys-tarfile papyrus_0.1.0_amd64.deb | tar -xO ./usr/bin/papyrus | sha256sum
mkdir -p "$TMP/inst" "$TMP/adm"/{info,updates,parts,triggers}; : > "$TMP/adm/status"; : > "$TMP/adm/available"
PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin fakeroot dpkg --instdir="$TMP/inst" --admindir="$TMP/adm" --force-depends -i papyrus_0.1.0_amd64.deb
```

Record whether the packaged binary SHA-256 equals the built binary SHA-256. Record the host `dpkg-query -W -f='${Status}' papyrus` before and after; it must be identical.

**S7 — Serve the installed package.** Start the server from the installed binary and record the port and PID:

```bash
PORT=6431
"$TMP/inst/usr/bin/papyrus" --dir "$TMP/project" browser --no-open --port "$PORT" &
curl -s -o /dev/null -w '%{http_code}\n' "http://127.0.0.1:$PORT/"   # 200
```

Every browser check below uses this server, so it describes the artifact that ships. Use `http://127.0.0.1:$PORT` as `<base>`.

---

**A. Diagnostics loop (items 1–12)**

**1.** Clean run. `"$PB" --dir "$TMP/diag" check` exits **0**; stdout is exactly `no findings`; stderr is empty.

**2.** Clean JSON and no-write proof. `"$PB" --dir "$TMP/diag" check --json` exits **0**; the object has `findings`, `skipped`, and `repaired` all empty arrays; `find "$TMP/diag" -type f -exec sha256sum {} + | sort` is byte-identical before and after.

**3.** Introduce two findings, then scan. Run:

```bash
cp "$TMP/diag/backlog/tasks/TASK-001.md" "$TMP/diag/backlog/tasks/TASK-001-copy.md"
sed -i "s/^created_date: .*/created_date: '2026-01-01T00:00:00Z'/" "$TMP/diag/backlog/tasks/TASK-001.md"
sed -i "s/^created_date: .*/created_date: '2026-02-01T00:00:00Z'/" "$TMP/diag/backlog/tasks/TASK-001-copy.md"
sed -i 's/^dependencies: \[\]/dependencies:\n  - TASK-001/' "$TMP/diag/backlog/tasks/TASK-002.md"
sed -i 's/^milestone: null/milestone: MS-999/' "$TMP/diag/backlog/tasks/TASK-003.md"
"$PB" --dir "$TMP/diag" check; echo "exit=$?"
```

Expected: exit **4**; stdout has exactly two finding lines, sorted by kind, in this form and order:

```
dangling-milestone: task TASK-003 — milestone MS-999 does not resolve to a milestone (backlog/tasks/TASK-003.md)
duplicate-id: task TASK-001 — 2 records share this ID (backlog/tasks/TASK-001-copy.md, backlog/tasks/TASK-001.md)
```

**4.** JSON shape. `check --json` exits **4**; `findings` contains both kinds in the same order; `duplicate-id` has `fixable: true`, `family: "task"`, `subject: "TASK-001"`, both sorted `locations`; `dangling-milestone` has `fixable: false`; `skipped` is empty.

**5.** Terminal repair, confirm. Record `TASK-002.md`'s SHA-256, then — before mutating the dirty project — copy it four times: `cp -a "$TMP/diag" "$TMP/diag-cancel"`, `cp -a "$TMP/diag" "$TMP/diag-nontty"`, `cp -a "$TMP/diag" "$TMP/diag-force"`, `cp -a "$TMP/diag" "$TMP/broken-dup"`. In a pty:

```bash
printf 'y\n' | script -qec "\"$PB\" --dir \"$TMP/diag\" check --fix" /dev/null; echo "exit=$?"
```

Expected transcript: `fix plan: 1 reassignment`, `reassign: task TASK-001 backlog/tasks/TASK-001-copy.md -> TASK-004`, `apply repairs? [y/N]`, then the two findings, `repaired: task TASK-001 -> TASK-004 (backlog/tasks/TASK-001-copy.md -> backlog/tasks/TASK-004.md)`, and `reference: task TASK-002 (backlog/tasks/TASK-002.md) dependencies = "TASK-001"`. Exit **4**. `TASK-001.md` remains and `TASK-004.md` exists with `id: TASK-004`; `TASK-001-copy.md` is gone; `TASK-002.md`'s SHA-256, recorded before the run, is unchanged; `TASK-003.md` still reports the dangling milestone.

**6.** Terminal repair, cancel. In `$TMP/diag-cancel`, answer `n` to the same command. Expected: `repair cancelled; nothing written`, exit **4**, no file renamed, and the project hash is unchanged.

**7.** Non-TTY refusal. In `$TMP/diag-nontty`, `printf 'y\n' | "$PB" --dir "$TMP/diag-nontty" check --fix` exits **2**; stderr is exactly `error: --fix requires --force when standard input is not a terminal`; stdout is empty; nothing is written (hash unchanged).

**8.** Forced repair and flag contract. In `$TMP/diag-force`, `check --fix --force` exits **4**, prints no `fix plan`/prompt, applies the same reassignment, and lists the reference; `check --force` without `--fix` exits **2** as a clap usage error before scanning, and writes nothing.

**9.** Clean second run. In `$TMP/diag`, correct the remaining finding with `sed -i 's/^milestone: MS-999/milestone: null/' "$TMP/diag/backlog/tasks/TASK-003.md"`, then: `check` exits **0** and prints `no findings`; a pty `check --fix` prints no plan/prompt, exits **0**, and writes nothing; `check --fix --force` also exits **0**.

**10.** Broken configuration. In `$TMP/broken`, `check` exits **4** and reports exactly one `config-invalid` finding per validation line, including `config-invalid: — statuses must not be empty (papyrus.config.yml)` and `config-invalid: — default_status 'ToDo' is not one of the configured statuses (papyrus.config.yml)`, followed by `skipped: duplicate-id:document-types, frontmatter-conflict:document-records`; no record finding is reported because that copy is otherwise clean. `"$PB" --dir "$TMP/broken" check --help` exits **0** and prints the §1.1 long description.

**11.** Fail-closed surface and repair paths. In `$TMP/broken`: `"$PB" --dir "$TMP/broken" task list` exits **4**, prints both configuration diagnostics to stderr, and writes nothing; `"$PB" --dir "$TMP/broken" browser --no-open --port 6432` exits **4** before serving; `"$PB" --dir "$TMP/broken" search x` exits **4**; `"$PB" --dir "$TMP/broken" config list` exits **0** and prints the effective values; `"$PB" --dir "$TMP/broken" config set statuses "ToDo, Done"` exits **0**, after which `task list` exits **0** and `check` exits **0**. In `$TMP/broken-init`, `init Verify --dir "$TMP/broken-init" --force --no-git` repairs the configuration without changing record files; record the resulting `statuses` value and a clean `check`.

**12.** Broken configuration plus static findings and repair. In the dirty copy `$TMP/broken-dup` saved in item 5, run `sed -i 's/^statuses:.*/statuses: []/' "$TMP/broken-dup/papyrus.config.yml"`. `check` exits **4**, reports the `config-invalid` lines, the `dangling-milestone` finding, and the `duplicate-id` finding, and names both document checks in `skipped`. `check --fix --force` exits **4**, repairs only the static-family duplicate, lists the reference, leaves the dangling finding, and still names both skipped document checks.

**B. Shared list and board (items 13–19)**

**13.** Every family list uses the shared layout. In M1, fetch each of: `/`, `/?type=req`, `/?type=nfr`, `/?type=task`, `/?type=milestone`, `/?type=release`, `/?type=decision`, `/?type=adr`, `/?type=bdr`, `/?type=audit`, `/?type=glossary`, `/?type=rule`, `/?type=tech-spec`, `/?type=test-case`, `/?type=test-report`, `/?type=ui-spec`, `/?type=ui-mockup`, `/?type=note`, `/?type=use-case`. Expected on every list route (`/?type=task` is the board and is covered by item 17): exactly one `table.table.list.list--shared[data-list-layout="shared"]` inside `div.table-scroll[role="region"][tabindex="0"]`, one `section.section`, one `div.section-head` with `h2.h2` and `p.count`, and no legacy list `<ul>`/`document_list` wrapper. Record 18/18 for list routes plus the board separately.

**14.** Column contract. On `/?type=milestone` and `/?type=req`, every `th` carries `scope="col"`; the header order is `ID`, `Title`, `Status`, exactly one family slot (`Release` for milestones, `Group` for grouped requirements), `Created`, `Updated`, `Actions`; every row has the same number of `td` as `th`, first `td.id-cell`, second `td.title-cell`, last `td.actions-cell`; action links carry a record-specific accessible name. Record the observed header sequence per family.

**15.** Grouped families. `/?type=req` shows the `REQ-001` group row followed by its member rows, each member carrying `data-child-of="REQ-001"` and IDs ascending; no `NFR-` row appears. `/?type=nfr` shows the `NFR-001` group and members and no `REQ-` row. The count note equals the rendered group/member counts; `/?type=adr` and `/?type=bdr` show the same heading and `Type` slot as `/?type=decision`.

**16.** Empty state. In a copy of the corpus with `docs/notes/*.md` removed (keep `archived/`), `/?type=note` shows `No notes in this project.`, the fixed help `Records are created from the command line; this browser does not create records.`, count `0 notes`, and no `<table>`.

**17.** Board at 1440 px (M1). `/?type=task` has `<div class="app app--full">`, no 1040 px cap; `getBoundingClientRect().width` equals `document.documentElement.clientWidth` (within 1 px) and is greater than 1040; the per-milestone columns render; `document.documentElement.scrollWidth === clientWidth`. No other route carries `app--full`.

**18.** Empty board fallback. In a copy with every task and milestone removed, `/?type=task` renders the board's own empty-project section (its own copy, no shared `table.list`), is full width, and has no page-level horizontal scrollbar at 320 and 1440 px. Record the observed copy.

**19.** Capped routes and scroll regions. At 1440 px, `/doc/REQ-001.001`, `/doc/REQ-001.001/edit`, `/config`, `/skills`, `/lifecycle`, and `/nope` render `<div class="app">` with computed `max-width: 1040px`, no `app--full`, and no page overflow. The list `.table-scroll`, `/config` `.table-scroll`, `/skills` command `<pre>`, and board columns are focusable labelled regions (`role="region"`, non-empty `aria-label`, `tabindex="0"`); Tab reaches each and shows a visible focus ring.

**C. Navigation and width matrix (items 20–30)**

Run items 20–25 on M1 at all seven widths and on M2–M4 at 320, 720, and 1440 px; run items 26–30 as stated for each browser. Set the viewport before navigation; after changing `papyrus.nav`, reload once and re-read the DOM.

**20.** Default expanded with cleared preference. At every width 320, 480, 481, 500, 600, 720, 1440: `localStorage.removeItem("papyrus.nav")`, reload `/`; `<html>` has no `data-nav`; `#site-nav` is visible, not `hidden`; `.nav-toggle` is visible, has no `hidden`, and `aria-expanded="true"`. Record 7/7.

**21.** Preset collapsed. Set `localStorage.setItem("papyrus.nav", "collapsed")`, reload `/`. At every width: `<html data-nav="collapsed">`; `#site-nav` has `hidden` and is not rendered (`offsetParent === null`); `.nav-toggle` has `aria-expanded="false"`; a preload-installed `requestAnimationFrame` probe reports `document.documentElement.dataset.nav === "collapsed"` on the first frame. At 1440 px record `.main` width expanded and collapsed; the difference is 270 px (±1 px). Record 7/7.

**22.** Toggle and persistence. At 1440 and 320 px: click the toggle (or activate it with the keyboard); `data-nav` becomes `collapsed`, `#site-nav` gains `hidden`, `aria-expanded` becomes `false`, and `localStorage["papyrus.nav"]` is `"collapsed"`. Navigate to `/doc/REQ-001.001`; the state is still collapsed. Reload; still collapsed. Toggle back; `"expanded"` is stored; reload; still expanded. Browser Back/Forward preserves the current document state.

**23.** Invalid, missing, and throwing storage. With `papyrus.nav` set to `"nonsense"` and with the key removed, the page is expanded in both cases. With a preload script replacing `Storage.prototype.getItem` and `setItem` with functions that throw, the page stays expanded, the toggle still collapses and expands for the session, and no uncaught error appears in the console. Record the preload method.

**24.** Keyboard operation. From the document body, Tab until `.nav-toggle` is focused (record the tab count); Enter collapses and Space expands; `aria-expanded` and `#site-nav.hidden` stay in sync; the focus ring is visible (`outline-style` is not `none`). No keyboard trap.

**25.** JavaScript disabled. With scripting disabled, load `/`, `/?type=task`, `/doc/REQ-001.001`, `/config`, and `/nope` at 320, 720, and 1440 px. Expected: navigation visible and expanded, toggle has `hidden` and is not offered, no `data-nav`, all nav links navigate normally, and no page-level horizontal scrollbar. Record the scripting-off method.

**26.** M1 width/state matrix. For each of the 7 routes `/?type=req`, `/?type=task`, `/?type=milestone`, `/doc/REQ-001.001`, `/doc/REQ-001.001/edit`, `/config`, `/nope` × 7 widths × 2 states (cleared = expanded, preset = collapsed) = 98 samples, assert `document.documentElement.scrollWidth === clientWidth` (allow 1 px rounding) and record the maximum overflow. `.main` width is greater than 0 in every collapsed sample; on the capped route `/?type=req` record 540 px at 600 and 992 px at 1440 (±1 px), and record the board's wider `.main` at 1440 separately. Record 98/98.

**27.** M2–M4 width/state matrix. For each browser: `/` and `/?type=task` at all 7 widths × 2 states (28 samples), plus `/doc/REQ-001.001` and `/doc/REQ-001.001/edit` at 320 and 1440 px × 2 states (8 samples), assert the same no-overflow condition and record 36/36 per browser.

**28.** Board width in every browser. At 1440 px, `/?type=task` is full width (`.app--full` rect equals the viewport client width, within 1 px) and wider than 1040 px; at 320 px it has no page-level horizontal scrollbar; no list/detail/edit/config/skills/error route carries `app--full`. Record per browser.

**29.** Long tokens and section headings. Set `project_name` to a 60-character unbroken token and reload. At 320 px the header does not overflow and the toggle remains usable. At 481, 500, 600, and 720 px the list `.section-head` wraps (the `.count` top edge is below the `h2` top edge) instead of widening the page. A long `h1`/`h2` token breaks inside its box; a long `pre` scrolls inside itself. Record each width.

**30.** Storage-condition matrix. At 320, 720, and 1440 px, for each state (expanded, collapsed) and each storage value (cleared, `"collapsed"`, `"expanded"`), record the resulting `data-nav`, `aria-expanded`, and `#site-nav.hidden`; only `"collapsed"` collapses, and cleared/`"expanded"`/invalid stay expanded. Record 18 samples per browser.

**D. Math runtime (items 31–39)**

**31.** Inline rendering offline. On `/doc/NOTE-001` and `/doc/REQ-001.001`, wait for `.katex`; for every `[data-tex]` element record `data-tex`, `.katex-mathml annotation[encoding="application/x-tex"]` textContent, presence of `.katex-html`, and `.katex-html[aria-hidden="true"]`. Expected: every element has both trees; every annotation equals its `data-tex` byte-for-byte; every `.katex-html` is `aria-hidden="true"`; no `data-render-error`; the fallback source is replaced. Record counts.

**32.** Display versus inline. On NOTE-001: each `[data-tex].math--display` has a `.katex-display` descendant with computed `display: block`; each `[data-tex]:not(.math--display)` has no `.katex-display` and renders inline. At 320 px the page has no horizontal scrollbar; a long display formula stays inside its container. Record the display/inline counts.

**33.** Fonts and offline assets. `document.fonts.check('16px KaTeX_Main')` is true; the resource timing entries contain the served `/assets/katex/fonts/*.woff2` faces the page actually uses (the stylesheet has 20 faces; record which are requested) with non-zero transfer size; no `.woff` or `.ttf` URL is requested; every request URL starts with `http://127.0.0.1:<port>/`; no request leaves the origin.

**34.** Math-free page. `/doc/NOTE-002`, `/?type=nfr`, and `/?type=task` have zero `[data-tex]`, zero `.katex`, zero `/assets/katex/` requests, and no KaTeX link/script in the DOM.

**35.** Aborted KaTeX asset. Intercept and fail `**/assets/katex/katex.min.js**`, reload `/doc/NOTE-001`. Expected: every `[data-tex]` has `data-render-error="math"`; the original source text is still visible; no `.katex-html` is created; no uncaught error; the navigation toggle still works. Record the interception method and the observed marker count.

**36.** Throwing stub. Block `katex.min.js` and install a preload stub `window.katex = { render: function () { throw new Error("stub"); } }`, then reload `/doc/NOTE-001`. Expected: every formula gets `data-render-error="math"`, the source remains in place, no formula tree is created, and the rest of the page still runs. Record the stub and the restored source for one formula.

**37.** Malformed formula. `/doc/NOTE-030` renders both broken formulas as KaTeX error output (`throwOnError: false`); no `data-render-error` is set; the offending source is still present in the element; the page is usable and has no script error. Colour may be uncoloured because inline style attributes are dropped (accepted, §7.3).

**38.** Curated fields only. On `/doc/REQ-001.001` and `/doc/TASK-001`, the heading, labels, metadata rows, acceptance criteria, comments, and edit form contain no `[data-tex]`; a `$title$` in the title and a `$label$` in labels render literally in list cells and headings. `"$PB" --dir "$TMP/project" task view TASK-001` prints the stored `$t=1$` source unchanged; `"$PB" --dir "$TMP/project" search "t=1"` finds TASK-001; the stored file is byte-identical to its pre-run content.

**39.** CSP style-drop observation. Install a preload `securitypolicyviolation` listener and record every `violatedDirective`/`blockedURI` on NOTE-001 and REQ-001.001. Expected on the math page: style-attribute violations from KaTeX are the only CSP events; zero `script-src`, `font-src`, `img-src`, or `default-src` violations. The formula is still readable; a rejected formula is uncoloured. Any script/font violation is a defect (TASK-145). Record counts and examples.

**E. Diagram runtime (items 40–50)**

Wait for `figure[data-diagram="mermaid"] svg` or a 5-second timeout before judging each diagram page.

**40.** Nine promised types. `/doc/REQ-001.001` has ten `figure[data-diagram="mermaid"]` blocks covering `flowchart`, `graph`, `sequence`, `class`, `state`, `er`, `gantt`, `pie`, `journey`, and `gitGraph`. Expected: 10/10 become an `<svg>` inside their `.diagram-scroll`; no `<pre>` remains; no `data-render-error`; the page has no horizontal scrollbar at 1440 and 320 px. Record each type and its SVG dimensions.

**41.** PlantUML source block. `/doc/NOTE-010` and `/doc/NOTE-018`: the `plantuml`/`puml` blocks remain `<figure class="diagram diagram--unsupported" data-diagram="plantuml"><pre>…</pre><figcaption class="render-note">` with the exact note `PlantUML diagrams are not supported in this release; showing source.`; no SVG; no Mermaid bundle link; the response carries the strict CSP.

**42.** Literal fences. `/doc/NOTE-011`: `mmd`, `uml`, `dot`, `graphviz`, and `rust` render as literal text including the fence lines; no figure, no note, no Mermaid bundle, no SVG.

**43.** Unknown, best-effort, and malformed. `/doc/NOTE-012`: `notARealDiagramType` keeps its `<pre>`, gains `data-render-error="diagram"`, and shows exactly `Diagram not rendered: Mermaid rejected the source.` once; `flowchart-elk` renders best-effort (record the result; do not assert failure); the malformed `flowchart TD\n A --> B\n A -->` takes the same failure path. If the malformed source renders instead, report a defect against TASK-146.

**44.** Per-figure isolation. `/doc/NOTE-015`: the over-limit figure keeps its full source, has the server-set `data-render-error="diagram"`, and shows the exact limit note once; the following valid figure renders an SVG; no figure receives a second note; the page still loads the bundle because one figure is renderable.

**45.** Missing bundle. Intercept and fail `**/assets/mermaid/mermaid.min.js**`, reload `/doc/REQ-001.001`. Expected: every figure keeps its `<pre>`, gains `data-render-error="diagram"`, and shows exactly `Diagram not rendered: the diagram script did not load.` once; no uncaught error; navigation and math still work.

**46.** Hostile diagram. `/doc/NOTE-016`: after render, the SVG contains no `<script>`, no `on*` attribute, no `javascript:` or `xlink:href` value, no `<foreignObject>`, and no remote image. Anchors produced by the click directive carry no `href`/`xlink:href`/`target`; clicking one does not navigate. A preload `window.alert` probe records no call. The `<img src=x onerror=alert(1)>` payload appears only as escaped text; the `classDef` payload produces no request.

**47.** Input caps. `/doc/NOTE-013` and `/doc/NOTE-014`: the full source is present in `<pre>` (same length as the fixture), `data-render-error="diagram"` is server-set, the exact note `Diagram not rendered: source exceeds the 100,000-byte or 1,000-line limit.` appears once, no bundle is linked, and the strict CSP is served. Record the source lengths.

**48.** Wide diagram at the extremes. `/doc/NOTE-032` at 320 and 1920 px: the rendered SVG width is less than or equal to `.diagram-scroll.clientWidth` (+1 px), `.diagram-scroll.scrollWidth === clientWidth`, and the page has no horizontal scrollbar. Record the natural `viewBox` width, rendered width, and container width at both viewports.

**49.** Keyboard reachability. Tab reaches each `.diagram-scroll[role="region"][tabindex="0"]`; the accessible name is `Diagram N` and is unique per page; the focus ring is visible; no link or control inside a rendered SVG is focusable.

**50.** Offline and same-origin. On `/doc/REQ-001.001` with non-loopback requests blocked, every request URL starts with `http://127.0.0.1:<port>/`; `/assets/mermaid/mermaid.min.js` is requested once with 200; no CDN, font, or image request leaves the origin; all ten diagrams still render.

**F. Security and CSP (items 51–56)**

**51.** CSP per response class. Re-check with `curl -sD - -o /dev/null` on the served pages: strict on `/`, `/?type=req`, `/?type=task`, `/doc/NOTE-001` (math only), `/doc/NOTE-018` (PlantUML only), `/doc/NOTE-013` (over-limit), `/doc/REQ-001.001/edit`, `/config`, `/skills`, `/lifecycle`, `/nope`; diagram on every page whose pre-pass set `has_diagram` (`/doc/REQ-001.001`, `/doc/NOTE-012`, `/doc/NOTE-015`), and strict on `NOTE-013`/`NOTE-014` because an over-limit fence does not set it; no CSP on any `/assets/...` response or JSON response. Compare each header byte-for-byte with `http::CSP`/`http::CSP_DIAGRAM`. If the rebuilt binary's SHA equals TASK-149's `2a71c1af…`, cite TASK-149 items 21–22 for the matrix instead of repeating it; record the decision.

**52.** Literal invariant. Both literals are exactly the accepted strings; `script-src` is exactly `'self'` with no `'unsafe-inline'`, `'unsafe-eval'`, nonce, or hash; `font-src` is `'self'`; `style-src` is `'self'` on the strict class and `'self' 'unsafe-inline'` only on the diagram class; the two differ in exactly one token. Record the diff.

**53.** Browser enforcement. On a math page, zero `script-src`/`font-src` violations and no blocked same-origin asset; on a diagram page, Mermaid's injected `<style>` is honoured and no `style-src` violation blocks it; on every page, no inline `<script>`, `on*` handler, or record-controlled `style` attribute is added; server-authored `style` attributes on diagram pages are the only accepted exception (§17.3).

**54.** Hostile math executes nothing. `/doc/NOTE-005`: no `<script>`, `on*`, `javascript:` URL, or remote reference introduced by record data; the payload appears only as escaped text in the `data-tex` attribute and body; no image, fetch, or navigation is triggered; `window.__xss` remains undefined and no alert fires.

**55.** Hostile diagram executes nothing. Repeat item 46 on `/doc/NOTE-016` with network and console capture: no script execution, no handler call, no remote image/CSS fetch, no navigation from the rendered diagram, and no CSP `script-src` relaxation.

**56.** Header invariants. On one HTML page and one asset, record `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`, `Connection: close`, and `Content-Type` with charset; the asset carries no CSP; the HTML carries exactly one CSP header. Any missing header is a defect against the browser contract (TASK-141/144).

**G. Package (items 57–62)**

**57.** Rebuild and quality gates. `cargo build --release`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check` all exit **0**; record test counts. `target/release/papyrus --version` prints `papyrus 0.1.0`; `--help` lists `check`, `browser`, `search`, `task`, `milestone`, `release`, `import`. Record binary size and SHA-256 as `<binary-size>`/`<binary-sha>`.

**58.** Package identity and TASK-149 transfer. `rm -f papyrus_0.1.0_amd64.deb && ./scripts/build-deb.sh` exits **0**; record `<deb-size>`/`<deb-sha>`; prove `dpkg-deb --fsys-tarfile … | tar -xO ./usr/bin/papyrus | sha256sum` equals `<binary-sha>` and the extracted size equals `<binary-size>`. If `<binary-sha>` equals TASK-149's `2a71c1af…`, cite TASK-149 items 9–27 and 35–39 and skip item 59's digest/notice re-run; otherwise re-run them and record the delta (the rebuild expected here).

**59.** Installed contents, notices, and digests. `dpkg-deb -c` lists exactly `./usr/bin/papyrus` (0755) and `./usr/share/doc/papyrus/copyright` (0644) plus directories; no `.css/.js/.html/.map/.woff2/.ttf`, `/etc/`, `/usr/share/man`, `/usr/share/papyrus`, or second doc file. The packaged copyright is byte-identical to `packaging/deb/usr/share/doc/papyrus/copyright`, has exactly four `^Files:` paragraphs (project GPL-3.0-or-later, KaTeX MIT, KaTeX fonts OFL-1.1, Mermaid MIT), and carries all licence texts. Recompute the 26 manifest digests and byte counts over the bytes served by the installed binary (26/26) and confirm `Content-Type`, `Content-Length`, `Cache-Control: no-store`, and `HEAD` empty. Record 26/26 or the cited TASK-149 result.

**60.** Budget. `<binary-size>` is at most `12,582,912` B; record the decimal, MiB value, and headroom in bytes and MiB; compare with TASK-149's 8,323,136 B / 4,259,776 B headroom and record the delta. Record `<deb-size>` and `<deb-sha>`; no package-size ceiling exists, so compare and record only.

**61.** Offline and absence proofs. `cargo build --release --offline` exits **0**. In `unshare -rn` with loopback up, outbound TCP fails while the list, board, math page, diagram page, 26 asset paths, and `check` succeed. If `unshare -rn` is unavailable, record that limitation and support the claim with loopback-only binding, the offline reference scans, and the browser network log; do not record it as a pass. From the installed root, `test ! -e "$TMP/inst/usr/share/papyrus"`, `test ! -e "$TMP/inst/etc"`, `test ! -e "$TMP/inst/usr/share/man"`; `find "$TMP/inst" -type f` lists exactly the two packaged files. Record each.

**62.** Installed-package browser proof. Every browser item (13–56) was served by `$TMP/inst/usr/bin/papyrus`; record the PID, binary path, and `<base>` URL for the run. Confirm the host `dpkg-query` state is unchanged from S6 and that no browser asset, config, or template was read from disk outside `/usr/bin/papyrus` and the project directory.

**H. Documentation and release (items 63–67)**

**63.** README `check` surface. Compare the README `check` section with the observed behaviour: synopsis `papyrus check [--fix] [--force]`; `--fix`; `--force` requires `--fix`; the seven finding kinds; the repair rule and keeper; the reference listing; the non-TTY refusal; tolerant configuration; `check --help` on a broken configuration; exit statuses 0/4/1/2; the intended read-only → repair → re-run loop. Record every claim as match/mismatch; a mismatch is a defect against TASK-140.

**64.** Business rules. Check `docs/business-rules/BR-001-papyrus-command-behavior.md`, `BR-005-task-command-behavior.md`, and `BR-016-configuration-keys-and-precedence.md` for the shipped `check` surface and the tolerant/fail-closed distinction. Record each relevant clause as match/mismatch; a mismatch is a defect against TASK-140.

**65.** Specification conformance. For each observed behaviour, cite the governing clause and confirm it: `browser-shell-shared-list-layout-and-page-width.md` §3.4 and §4 (nav/width/bleed/section headings), `browser-math-and-diagram-rendering.md` §6, §7.3, §13–§19 (math client, figure markup, caps, notes, types, CSP), `browser-shell-ui.md` §5, §7, §8 (shell/nav/browsers/CSP), and `browser-requirements.md` §9.7 (CSP classes). Record a small table of observation → clause → match/mismatch. Any mismatch is a defect against TASK-148 (spec port) or the owning feature ticket; do not edit the spec here.

**66.** Dated packaging evidence. Add a `## TASK-150 — Browser runtime verification (2026-…)` section to `docs/packaging.md` after the last dated section: environment, browser matrix with versions and binary SHA-256 values, artifact SHAs and sizes, the rebuilt-artifact supersession note, results by section, recorded limitations, and defects. Change only the single TASK-149 limitation bullet that points browser-observed behaviour at TASK-150 so it names this evidence; leave every other line of earlier dated sections untouched. Do not rewrite the live `## Packaging` procedure or §3.6.

**67.** Release vehicle. Record `backlog milestone list --plain`, `backlog task list --plain`, and `"$PB" --dir "$TMP/project" release list`; confirm no release milestone or release ticket was created by m-21 and that m-21 remains the only active milestone with TASK-150 as its last open task. Record the release-vehicle decision (release notes and a release ticket remain out of scope; the owner decides the vehicle) and report it to `@orchestrator` in a ticket comment and the final summary.


## Notes
## TASK-150 execution record (2026-11-16)

### Setup S1–S7

- **S1 environment.** `2026-11-16T11:20:48+01:00`; Debian forky/sid amd64; rustc/cargo 1.99.1; HEAD `dc2ec83a2bad4deed299374b20cac223a95409e6`; Firefox 153.5.1esr (binary `/usr/lib/firefox-esr/firefox-esr`, SHA-256 `3c87b278879fd1d6a4ee027e0b111afb7d03cc8786c8147ee11a6eb60774c27e`).
- **S2 fixture base.** Copied `tests/fixtures/characterization/lists/project`; baseline **52** files; final corpus **67** files after S3.
- **S3 corpus.** NOTE-001 (math REQ_BODY), NOTE-002, NOTE-005/006, REQ-001.001 (diagram REQ_BODY; its source `## Group` section retained so item 15's member rows exist — the literal diagram `REQ_BODY` alone would have made REQ-001.001 an orphan), NOTE-010..019, NOTE-030..032, TASK-001. REQ title `Requirement One $title$`, labels `core` + `$lab$`.
- **S4 diagnostics.** `papyrus init Verify --dir .tmp/task150/diag --no-git` + three tasks.
- **S5 broken configs.** `broken` and `broken-init` with `statuses: []`; later `broken-dup` from the dirty diag copy.
- **S6 build/package.** `cargo build --release`; binary **8,323,296 B** / SHA-256 `f306d2223ebd9404f742baf27b4cb2365959ccab575da7048414a8dfe72418d1`; `scripts/build-deb.sh`; `.deb` **2,379,496 B** / SHA-256 `89cbc8af9c978c66268248b0c5d48392f2af86659a9c97923426156ec828facc`; packaged binary SHA and size equal the built binary; isolated install has exactly two files; host `dpkg-query` status `install ok installed` before and after.
- **S7 server.** Installed binary `$TMP/inst/usr/bin/papyrus --dir $TMP/project browser --no-open --port 6431`, PID **584018**, base `http://127.0.0.1:6431`; alternate fixture servers on 6433/6434/6435 from the same installed binary.
- **TASK-149 delta.** Rebuilt artifact differs from TASK-149 by design: binary +160 B, `.deb` −216 B; package contents, notices, 26/26 digests, budget, offline, and absence checks re-run and pass. TASK-149 section left intact; supersession note added to `docs/packaging.md`.

### Commands used (representative)

- Diagnostics: `target/release/papyrus --dir <proj> check [--json] [--fix] [--force]`; pty via `printf 'y\n' | script -qec "..." /dev/null`.
- Browser M1: headless Firefox 153.5.1esr WebDriver BiDi at DPR 1 (`browsingContext.setViewport`, `script.evaluate`, `network.addIntercept`), plus geckodriver 0.36.0 for keyboard/focus.
- Browser M2: headless Firefox 102.0esr under geckodriver 0.36.0; Firefox 100.0 probe for the NFR floor.
- CSP/package: `curl -sD - -o /dev/null`, `dpkg-deb -c/-I/--fsys-tarfile`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt --check`, `cargo build --release --offline`, `unshare -rn`.

### A. Diagnostics loop (1–12)

| # | Fixture/route | Command | Expected | Observed | Result |
|---|---|---|---|---|---|
| 1 | clean `diag` | `check` | exit 0, stdout `no findings`, empty stderr | exactly that | pass |
| 2 | clean `diag` | `check --json` + hash before/after | arrays empty; no write | `{"findings":[],"skipped":[],"repaired":[]}`; hashes identical | pass |
| 3 | dirty `diag` | `check` | exit 4; two sorted lines | `dangling-milestone: task TASK-003 — milestone MS-999 ...` then `duplicate-id: task TASK-001 — 2 records ...`; exit 4 | pass |
| 4 | dirty `diag` | `check --json` | both kinds; duplicate fixable true, family task, subject TASK-001, both locations; dangling fixable false; skipped empty | exactly that | pass |
| 5 | dirty `diag` | pty `check --fix`, answer `y` | plan/reassign/prompt; report; TASK-004; TASK-001-copy gone; TASK-002 byte-identical; dangling remains; exit 4 | plan 1 reassignment, reassign TASK-001-copy -> TASK-004, prompt, findings, repaired + reference; TASK-004 exists, copy gone, TASK-002 SHA `d871780f...` unchanged, TASK-003 still dangling; exit 4. The plan also lists the affected reference before the prompt, as spec §10.9 requires; the checklist transcript omitted it (checklist wording gap, not a defect). | pass |
| 6 | `diag-cancel` | pty `check --fix`, answer `n` | cancelled, nothing written, exit 4 | `repair cancelled; nothing written`; full hash identical; no rename; exit 4 | pass |
| 7 | `diag-nontty` | `printf 'y\n' | check --fix` | exit 2; exact stderr; empty stdout; no write | exit 2; stderr `error: --fix requires --force when standard input is not a terminal`; stdout 0 B; hash unchanged | pass |
| 8 | `diag-force` | `check --fix --force`; `check --force` | exit 4, no plan/prompt, same repair; force-only exit 2 usage, no write | repaired TASK-001-copy -> TASK-004 + reference; no plan/prompt; force-only exit 2 with clap `--fix` required; hash unchanged | pass |
| 9 | repaired `diag` | `check`; pty `--fix`; `--fix --force` | exit 0 `no findings`; no plan/prompt; no writes | all exit 0; `no findings`; hashes identical | pass |
| 10 | `broken` | `check`; `check --help` | two config-invalid lines + skipped pair; help exit 0 | `config-invalid: — statuses must not be empty (papyrus.config.yml)`, `config-invalid: — default_status 'ToDo' is not one of the configured statuses (papyrus.config.yml)`, `skipped: duplicate-id:document-types, frontmatter-conflict:document-records`; help exit 0 with §1.1 long description | pass |
| 11 | `broken`, `broken-init` | fail-closed surface + repairs | task/browser/search exit 4; config list/set, init --force repair | task list exit 4 with both diagnostics on stderr and no write; browser exit 4 before serving; search exit 4; config list exit 0; config set statuses `ToDo, Done` exit 0 then task list/check exit 0; init --force restores statuses list and leaves records byte-identical; check exit 0 | pass |
| 12 | `broken-dup` | `check`; `check --fix --force` | static findings still run; fix only duplicate; skipped named | check exit 4 with two config-invalid + dangling + duplicate + skipped; fix exits 4, repairs duplicate, lists reference, leaves dangling and skipped | pass |

### B. Shared list and board (13–19)

| # | Fixture/route | Browser/viewport | Expected | Observed | Result |
|---|---|---|---|---|---|
| 13 | 17 family lists + `/` | M1 1440 | one shared table/region/section/count; no legacy list | 17/17 family lists: one `table.list--shared[data-list-layout=shared]` in `div.table-scroll[role=region][tabindex=0]`, one section/head/h2/count, count matches rows, zero legacy wrappers. `/` is the Browser inventory (spec §3.2), not a list route; the checklist's 18-route count wrongly includes it. Board checked separately. | pass (17/17 + board) |
| 14 | `/?type=milestone`, `/?type=req` | M1 1440 | ID,Title,Status,family slot,Created,Updated,Actions; all `th scope=col`; td counts and cells | Milestones `ID,Title,Status,Release,Created,Updated,Actions`; requirements `...Group...`; 7 th, 7 td per row; first `id-cell`, second `title-cell`, last `actions-cell`; `View/Edit <ID>` labels | pass |
| 15 | req/nfr/decision/adr/bdr | M1 1440 | group row, ascending child IDs with data-child-of, no foreign family; decisions same heading/Type | req `REQ-001, REQ-001.001, REQ-001.002, REQ-001.004` with .001/.002 child-of ascending; nfr `NFR-001, .001, .002`; counts `1 group · 3 requirements` / `1 group · 2 requirements`; no foreign rows; decision/adr/bdr all h1/h2 `Decisions` with `type` slot | pass |
| 16 | notes-empty copy `/?type=note` | M1 1440 | exact empty copy/help, `0 notes`, no table | `No notes in this project.`, `Records are created from the command line; this browser does not create records.`, `0 notes`, 0 tables | pass |
| 17 | `/?type=task` | M1 1440 | `app app--full`, width = clientWidth > 1040, milestone columns, no overflow | class `app app--full`; app 1,428 = clientWidth 1,428; 4 board milestone groups, 6 columns in first; scrollWidth = clientWidth | pass |
| 18 | empty board copy `/?type=task` | M1 320/1440 | own empty section, no shared table, full width, no overflow | `app app--full`; text `No tasks in this project.` + fixed help; 0 `table.list`; 1 own section; 308/1440 widths, no overflow | pass |
| 19 | detail/edit/config/skills/lifecycle/nope + regions | M1 1440 | app capped 1040, no app--full; labelled focusable regions | all six app class `app`, computed `max-width: 1040px`, no overflow; list/config `.table-scroll`, skills `<pre>`, board `.board-columns` all `role=region`, non-empty label, `tabindex=0`; Tab reaches each with solid focus ring | pass |

### C. Navigation and width (20–30)

| # | Fixture/route | Browser/viewport/state | Expected | Observed | Result |
|---|---|---|---|---|---|
| 20 | `/` all widths | M1 320,480,481,500,600,720,1440 cleared | no data-nav; nav and toggle visible; aria true | 7/7; nav visible, toggle visible, `aria-expanded=true`, no `data-nav` | pass 7/7 |
| 21 | `/` all widths | M1 preset collapsed | data-nav collapsed, nav hidden, aria false, first-frame collapsed; 270 px reclaim at 1440 | 7/7; first-frame probe `collapsed`; nav hidden/offsetParent null; main content width 722 expanded vs 992 collapsed = **270 px** (border-box 752/992; content box is the contract) | pass 7/7 |
| 22 | `/` and `/doc/REQ-001.001` | M1 1440/320 | toggle, store, navigate, reload, toggle back, history | collapsed + stored `collapsed`; survives navigation/reload/back/forward; toggles back to expanded + stored `expanded`; survives reload | pass |
| 23 | `/` | M1 720 invalid/missing/throwing storage | expanded; throwing storage still toggles; no uncaught error | `nonsense` and missing expand; preload `Storage.prototype` throw leaves expanded, toggle collapses/expands in session, zero console errors | pass |
| 24 | `/` | M1 1440 keyboard | Tab reaches toggle; Enter/Space; visible ring; no trap | geckodriver: Tab 1 reaches toggle, solid focus ring, `:focus-visible` true; Enter collapses, Space expands; next Tab moves to a link | pass |
| 25 | five routes × 3 widths | M1 JS disabled | no data-nav, nav visible expanded, toggle hidden/not offered, links work, no overflow | 15/15; caused by `javascriptEnabled:false` + `javascript.enabled=false`; 17 nav links; all hrefs HTTP 200; sample navigation worked | pass 15/15 |
| 26 | 7 routes × 7 widths × 2 states | M1 | scrollWidth = clientWidth ±1; main > 0 collapsed; req main 540@600, 992@1440; board wider | **98/98**, max overflow **0**; collapsed req main 540@600 and 992@1440; board main 1380@1440 collapsed; collapsed main > 0 everywhere | pass 98/98 |
| 27 | `/`, `/?type=task` 7 widths ×2; detail/edit 320/1440 ×2 | M2 Firefox 102.0esr | 36/36 no overflow | **36/36 no-overflow**, max overflow 0. 8 requested-320 samples expose 438 px because Firefox/geckodriver enforces a 450 px minimum outer width; recorded as a limitation, not exact 320. M3/M4 owner-pending. | limitation (36/36 no-overflow; 8 clamped) |
| 28 | board + other routes | M1 and M2 | board full >1040 at 1440; no overflow at 320; no other app--full | M1 1428 at 1440, 1380 main collapsed, 320 no overflow; M2 1428/1380, 438 fallback at 320; all non-board routes `app` | pass both |
| 29 | width copy | M1 320/481/500/600/720 | header no overflow, toggle usable; section-head wraps; long h1 breaks; long pre scrolls internally | 60-char project name: no header overflow at 320, toggle usable; count below h2 at 481/500/600/720; 60-char h1 284 within 284 at 320; NOTE-013 pre at 320: `scrollWidth 903342 > clientWidth 284`, `overflow-x:auto`, page 308/308 | pass |
| 30 | 3 widths ×2 prior states ×3 storage values | M1 and M2 | only stored `collapsed` collapses | M1 **18/18**; M2 **18/18**; cleared/expanded expand, collapsed collapses, aria and hidden in sync | pass both |

### D. Math runtime (31–39)

| # | Fixture/route | Browser/viewport | Expected | Observed | Result |
|---|---|---|---|---|---|
| 31 | `/doc/NOTE-001`, `/doc/REQ-001.001` | M1 1440 | annotation = data-tex; katex-html aria-hidden; no error | NOTE-001 4/4 (2 inline + 2 display); REQ 1/1; every annotation byte-equal, `.katex-html aria-hidden="true"`, no `data-render-error` | pass |
| 32 | NOTE-001; NOTE-031 | M1 1440/320 | display vs inline classes; no overflow | 2 display with `.katex-display` computed `block`; 2 inline without; NOTE-031 at 320: display width 284 inside main 284, page 308/308 | pass |
| 33 | NOTE-001 | M1 1440 | KaTeX_Main loaded; used Woff2 faces non-zero; no woff/ttf/external | `document.fonts.check('16px KaTeX_Main')` true; 3 faces requested: KaTeX_Math-Italic 16,740 B, Main-Regular 26,572 B, Size2-Regular 5,508 B; 20 faces defined; no woff/ttf/external | pass |
| 34 | NOTE-002, nfr, task | M1 1440 | zero data-tex/katex/assets | 0/0/0 on all three; no KaTeX link/script/resource | pass |
| 35 | NOTE-001 abort katex.min.js | M1 1440 | every formula marked, source kept, no tree, no error, toggle works | network intercept failed exactly `.../katex.min.js`; 4/4 `data-render-error=math`, source text identical, no `.katex-html`, no console error, toggle collapsed | pass |
| 36 | NOTE-001 stub | M1 1440 | stub throw marks all, restores source | blocked bundle + preload `window.katex.render` throw; 4/4 marked, source restored, page h1 intact, no console error | pass |
| 37 | NOTE-030 | M1 1440 | malformed formulas show KaTeX error output; source present; no marker/script error | both rendered; first `.katex-error` true, both retain source text; no `data-render-error`; no uncaught script error (one CSP inline-style console notice, accepted §7.3) | pass |
| 38 | REQ-001.001, TASK-001, list; CLI | M1 1440 | curated fields no data-tex; literal `$title$`/`$lab$`; CLI/source unchanged | 0 data-tex in h1/taglist/meta/summary/form/comments; REQ h1 `Requirement One $title$`, tags include `$lab$`; task view prints `Task math $t=1$.`; `search "t=1"` finds TASK-001; file SHA unchanged | pass |
| 39 | NOTE-001, REQ-001.001 | M1 1440 | only KaTeX style-attribute CSP events; zero script/font/img/default | preload `securitypolicyviolation` listener recorded **0** events on both pages; zero script/font/img/default; formulas readable | pass |

### E. Diagram runtime (40–50)

| # | Fixture/route | Browser/viewport | Expected | Observed | Result |
|---|---|---|---|---|---|
| 40 | REQ-001.001 | M1 1440/320/1920 | 10/10 types render, no pre/error, no overflow | 10/10 SVG; types in order flowchart, graph, sequenceDiagram, classDiagram, stateDiagram-v2, erDiagram, gantt, pie, journey, gitGraph; SVG widths 87, 209, 450, 102, 61, 146, 721(scale), 526, 500, 159; no pre/error; 308/308 and 1908/1908 at 320/1920 | pass |
| 41 | NOTE-010, NOTE-018 | M1 1440 | PlantUML source + exact note, no SVG/bundle, strict CSP | 3/3 `diagram--unsupported`, `data-diagram=plantuml`, pre kept, exact note, no SVG, no Mermaid link | pass |
| 42 | NOTE-011 | M1 1440 | literal fences including fence lines; no figure/note/SVG/bundle | 0 figures/SVG/notes/bundle; text contains backticked `mmd`, `uml`, `dot`, `graphviz`, `rust` fences | pass |
| 43 | NOTE-012 | M1 1440 | unknown and malformed fail closed; elk best effort | figure 1 error + note once; figure 2 `flowchart-elk` SVG best-effort; figure 3 error + note once | pass |
| 44 | NOTE-015 | M1 1440 | over-limit source kept + server error + note once; next valid renders; bundle loaded | over-limit pre 100,001 B, error, limit note once; second figure SVG; no second note; Mermaid bundle linked | pass |
| 45 | REQ-001.001 missing mermaid | M1 1440 | all pre + error + missing-bundle note; no uncaught; nav/math work | 10/10 pre + `data-render-error=diagram` + exact note once; math 1/1; toggle collapsed; zero console errors | pass |
| 46 | NOTE-016 hostile | M1 1440 | no script/on*/javascript:/foreignObject/remote image/anchor nav/alert | Mermaid rejected source (safe fallback): no SVG; no script/on*/foreignObject/remote image; no anchors; payload only escaped in pre; alert probe uncalled; location unchanged | pass |
| 47 | NOTE-013, NOTE-014 | M1 1440 | full source + server error + exact note once; no bundle/SVG; strict CSP | pre lengths **100,001** and **2,001**; error + exact note once; 0 Mermaid links; no SVG; strict CSP | pass |
| 48 | NOTE-032 | M1 320/1920 | SVG <= container; no internal/page overflow | viewBox 3,524.33 wide; rendered 284 in 284 at 320, 721 in 721 at 1920; scrollWidth = clientWidth | pass |
| 49 | REQ-001.001 | M1 1440 keyboard | Tab reaches 10 unique `Diagram N`; visible ring; no focusable SVG control | geckodriver: 10/10 reached in order, names unique, solid ring each, zero focusable SVG links/buttons/tabindex | pass |
| 50 | REQ-001.001 | M1 1440 | all HTTP requests same-origin; Mermaid once 200; 10/10 render; no external | all HTTP requests `http://127.0.0.1:6431/`; Mermaid once; 10/10 SVG; no external HTTP/data URL leaves origin (2 embedded `data:font/woff2` requests, not network) | pass |

### F. Security and CSP (51–56)

| # | Fixture/route | Browser/HTTP | Expected | Observed | Result |
|---|---|---|---|---|---|
| 51 | 12 strict + 4 diagram + asset + JSON | curl 1440 | exact literals; no CSP on asset/JSON | **16/16** exact; strict on `/`, req, board, NOTE-001/018/013/014, edit, config, skills, lifecycle, nope; diagram on REQ/012/015/016; no CSP on `/assets/app.js` or `/api/requirements`; rebuilt SHA differs from TASK-149 so full matrix re-run | pass |
| 52 | literals | HTTP | script-src self, font-src self, style strict vs diagram only token | `script-src 'self'`, `font-src 'self'`; the literals differ only in `style-src 'self'` vs `style-src 'self' 'unsafe-inline'` | pass |
| 53 | NOTE-001, REQ-001.001 | M1 1440 | no script/font violations/blocked assets; diagram styles honoured; no inline script/on* | 0 violations on both; 10 diagrams with 10 inline `<style>` honoured; 0 inline scripts; 0 `on*` attributes | pass |
| 54 | NOTE-005 | M1 1440 | hostile math executes nothing | 5 formulas render as inert error/text; 0 scripts, 0 images, 0 `on*`, 0 `javascript:` URLs; `window.__xss` undefined; payload literal text | pass |
| 55 | NOTE-016 | M1 1440 | hostile diagram executes nothing; no remote fetch/navigation/CSP relaxation | safe fallback; 0 script/handler/foreignObject/images/anchors; alert uncalled; 0 external requests; diagram CSP unchanged | pass |
| 56 | HTML + asset | curl | required headers; one CSP on HTML; none on asset | HTML and asset: `X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cache-Control: no-store`, `Connection: close`, `Content-Type` with charset; HTML one CSP; asset none | pass |

### G. Package (57–62)

| # | Fixture/command | Expected | Observed | Result |
|---|---|---|---|---|
| 57 | build/test/lint/fmt/version/help | all gates 0; version 0.1.0; commands listed | build ok; `cargo test` **1,046 passed / 0 failed / 23 ignored**; clippy 0 warnings; fmt clean; `papyrus 0.1.0`; help lists check,browser,search,task,milestone,release,import | pass |
| 58 | rebuild/package identity | packaged binary = built | `.deb` 2,379,496 B / `89cbc8af...`; extracted binary 8,323,296 B / `f306d222...` = built; SHA differs from TASK-149, so delta re-run | pass |
| 59 | contents/notices/digests | two files; 4 Files paragraphs; 26/26 | `dpkg-deb -c` only `usr/bin/papyrus` 0755 and copyright 0644; copyright 8,000 B identical SHA `4610a7ab...`, 4 `Files:` paragraphs with all licence texts; 26/26 digests, bytes, type, length, no-store, empty HEAD | pass |
| 60 | budget | <= 12,582,912 B | 8,323,296 B = 7.938 MiB; headroom **4,259,616 B = 4.062 MiB**; TASK-149 delta +160 B; `.deb` recorded | pass |
| 61 | offline/absence | offline build; unshare loopback routes/assets/check; absent dirs | `cargo build --release --offline` exit 0; `unshare -rn` outbound TCP failed, loopback list/board/math/diagram + 26/26 assets + check exit 0; `/usr/share/papyrus`, `/etc`, `/usr/share/man` absent; exactly two files | pass |
| 62 | installed-package proof | all browser items served by installed binary; dpkg state unchanged | base/PID recorded from `$TMP/inst/usr/bin/papyrus`; host dpkg unchanged; no asset/config/template directory exists outside the embedded binary and project | pass |

### H. Documentation and release (63–67)

| # | Source | Expected | Observed | Result |
|---|---|---|---|---|
| 63 | README `check` | surface/claims match | synopsis, `--fix`, `--force` requires `--fix`, seven kinds, repair keeper/refusal, reference listing, non-TTY, tolerant config, help on broken config, exits 0/4/1/2, re-run loop all match | pass |
| 64 | BR-001/BR-005/BR-016 | shipped surface and fail-closed distinction | BR-001 built-in/read-only/exit4/tolerant clauses match; BR-016 fail-closed + tolerant check + reserved slug match; BR-005 has no `check` clause because it is the task-command contract (not applicable, no mismatch) | pass |
| 65 | specs | observation -> clause match | shell §3.4/§4, math §6/§7.3/§13–19, shell UI §5/§7/§8, requirements §9.7 all match. The only wording gaps are in this checklist: `/` is inventory not a list route, and the item-5 transcript omitted the spec §10.9 plan reference; neither is a product mismatch. | pass |
| 66 | docs/packaging.md | dated section; only the TASK-149 limitation bullet changed | `## TASK-150 — Browser runtime verification (2026-11-16)` added; TASK-149 browser limitation now points to it; no live procedure/§3.6/earlier section rewritten | pass |
| 67 | backlog/release | no release milestone/ticket by m-21; m-21 last open task TASK-150; vehicle reported | m-21 13/14 with TASK-150 last open; no release milestone/ticket; fixture release list only REL-001/002; vehicle decision reported to @orchestrator | pass |

### Failures, defects, limitations

- **DEF-1 / TASK-146 / High:** Firefox 100.0 renders **0/10** Mermaid figures; `mermaid.render` rejects `TypeError: Illegal constructor` at `mermaid.min.js:3582:1657`; Firefox 102.0esr renders 10/10. NFR-005.001 Firefox 100 floor violated. Repro/evidence in TASK-146 comment and `docs/packaging.md`. No fix made.
- **M3 Edge 130 / M4 Safari 18.x:** pending owner-manual; never recorded as passing. Exact commands/routes/viewports in `docs/packaging.md`.
- **M2 320 px:** Firefox/geckodriver minimum window width exposes 438 px client width; 8 M2 matrix samples recorded at 438 px. M1 covers exact 320 px.
- **M2 pre-load storage throw:** geckodriver cannot inject a pre-page stub; M2 verifies in-session fallback, M1 verifies pre-load.
- **Checklist wording (not defects):** item 13 includes `/` as a family list route although spec §3.2 says inventory; item 5 transcript omits the plan reference that spec §10.9 requires.

All 67 items were executed; sections A–H pass apart from the recorded limitations and DEF-1. AC #3–#6 and #9 remain unchecked pending M3/M4 owner results.

Amendment re-run 2026-11-16: NFR-005.001 now Firefox 115+ only. Packaging.md TASK-150 section updated: deb 2,379,448 B / SHA 874d03de...; HEAD 4a54cc1; DEF-1 obsolete; M3/M4 and Firefox 100 probe out of scope. AC #3-#6,#9 and DoD #2,#8 checked. cargo test 1048 passed, clippy 0, fmt clean.


## Comments
author: @task-refiner
created: 2026-11-16T10:13:00Z

---

Refined 2026-11-16 (task-refinement) into 67 ordered checks across diagnostics (1-12), shared list/board (13-19), navigation/width (20-30), math (31-39), diagrams (40-50), security/CSP (51-56), package (57-62), and documentation/release (63-67), with setup S1-S7, a defect-reporting rule, and 8 DoD items. Merged the three comment groups named in the handoff (browser nav/width matrix, KaTeX runtime, diagram runtime); the three orchestrator comments themselves are not present in the task record (comments: []). Decisions: (a) verify current HEAD, not the pre-HEAD TASK-149 package - HEAD commits 03e1d2c and cfd4804 changed diagram markup/numbering, so the .deb is rebuilt and TASK-149 package checks re-run as a delta unless the binary SHA is identical; (b) browser matrix M1 current Firefox 153.5.1esr (full), M2 Firefox 100/oldest ESR, M3 Edge 130 and M4 Safari 18.x as owner-manual checks with version and binary SHA, never assumed available; (c) explicit aborted-asset, throwing-stub, malformed, nine-type, PlantUML/literal/unknown/cap, per-figure isolation, missing-bundle, hostile, and 320/1920 wide-diagram checks; (d) check --fix loop covers confirm, cancel, non-TTY refusal, --force, idempotence, and broken-config repair. Open: M3/M4 ownership/date; Firefox 100 availability; whether release-vehicle means backlog plus product release records. The 9 existing ACs are retained and sharpened; no AC added.

---

author: @task-refiner
created: 2026-11-16T10:14:00Z

---

Correction to the refinement note above: the commits that landed after TASK-149's measured build are a718f53 and cfd4804 (diagram focus markup and page-continuous numbering). 03e1d2c was already in the measured tree. The rebuilt-artifact decision is unchanged.

---

author: orchestrator
created: 2026-11-16T10:19:00Z

---

Browser matrix required (from TASK-144 review; automated CSS/HTTP checks provably missed a zero-width content column and a full-width task detail route): record browser versions (Edge 130/Chromium 130, Safari 18.x, Firefox 100 or oldest supported ESR, plus current Firefox); viewports 320, 480, 481, 500, 600, 720, 1440 CSS px at DPR 1, expanded and collapsed, cleared and preset papyrus.nav; routes /?type=req, /?type=nfr, /?type=task, /?type=task&expand=<milestone>, /doc/TASK-001, /doc/TASK-001/edit, /config, /skills, /nope; assert documentElement.scrollWidth - clientWidth === 0 for every combination with wide content scrolling only inside .table-scroll, .board-columns and the skills command block; collapsed at 600 and 1440 keeps .main at the app content width (about 540 and 992; 1380 for the full board), never 0; only the board route renders app app--full; Tab reaches the toggle with a visible ring, Enter and Space toggle, state survives navigation and reload with no wrong-state paint; JavaScript disabled leaves no data-nav, the nav expanded and usable, the toggle hidden and out of the tab order; a forced localStorage throw leaves the page expanded and the toggle working; no page-level horizontal scrollbar and no console or CSP errors, with per-browser per-width numbers and the tested binary SHA.

---

author: orchestrator
created: 2026-11-16T10:19:00Z

---

KaTeX runtime checks (from TASK-145 review; served HTML cannot prove runtime behaviour): load a page with inline, display and fenced math and assert .katex exists, .katex-mathml annotation[encoding=application/x-tex] equals the element data-tex, .katex-html carries aria-hidden=true, display elements contain .katex-display while inline ones do not; assert no CSP violations for script-src, style-src or font-src, accepting only the dropped KaTeX inline style attributes, and no data-render-error=math on valid formulas; abort /assets/katex/katex.min.js, reload, and assert every [data-tex] is marked and keeps its source; fulfil katex.min.js with a stub whose render clears the element and throws, then assert the marker is set, the source restored, and other math elements still render; enter a malformed formula and assert KaTeX's error output appears with the source retained and no marker. Run on Firefox 100+, Edge 130+ and Safari 18+.

---

author: orchestrator
created: 2026-11-16T10:19:00Z

---

Mermaid runtime checks (from TASK-146 review residual list): nine promised types actually render in a browser; PlantUML note and literal mmd/uml/dot/graphviz behaviour rendered; unknown-type failure path; per-figure isolation; missing-bundle branch; hostile fixtures produce no script, no event handler and no javascript: URL in the rendered DOM; wide diagram at 320 and 1920 px (scale-to-fit, no page scrollbar, midpoints 480/720 included in the width sweep); keyboard focus reaches each .diagram-scroll; accessible names unique per page; both caps and the diagram CSP literal. Cross-browser baseline remains outstanding.

---

author: @rust-dev
created: 2026-11-16T11:12:00Z

---

TASK-150 execution status: sections A–H executed in order. 67/67 items have evidence. Diagnostics, shared list/board, M1/M2 navigation and width, math, diagrams, CSP/security, package, and docs checks pass except the recorded limitations and DEF-1.

Release-vehicle report for @orchestrator: `backlog milestone list --plain` shows m-21 as the only active milestone with an open task (13/14 done); `backlog task list --milestone m-21 --plain` shows TASK-150 as its last open task. No release milestone or release ticket was created by m-21. `papyrus release list` on the fixture project returns only fixture records REL-001/REL-002 (`planned`), not backlog release records. Decision: release notes and a release ticket remain out of scope for TASK-150; the owner decides the vehicle. This matches item 67 and AC #9's reporting requirement.

Owner-manual matrix still pending: M3 Edge 130 and M4 Safari 18.x were not available on this host and are recorded as pending, not passed. AC #3–#6 and #9 remain unchecked until the owner returns version, binary SHA-256, and per-item results. M2 is Firefox 102.0esr (oldest supported ESR); Firefox 100.0 was also tested and fails Mermaid rendering (DEF-1, reported to TASK-146).

DEF-1 (High, TASK-146): Firefox 100.0 renders 0/10 Mermaid figures; `mermaid.render` rejects with `TypeError: Illegal constructor` at `mermaid.min.js:3582:1657`. Firefox 102.0esr renders 10/10. NFR-005.001's Firefox 100 floor is therefore not met for diagrams. No fix was made in TASK-150.

The dated evidence section is in `docs/packaging.md` (`## TASK-150 — Browser runtime verification (2026-11-16)`), including artifacts, browser SHAs, results by section, limitations, DEF-1, and the release-vehicle decision.

---

author: @rust-dev
created: 2026-11-16T11:14:00Z

---

Status Refined (blocked, not done). AC #1, #2, #7, #8 and DoD #1, #3-#7 are satisfied with evidence in the notes and docs/packaging.md. Unresolved: (a) M3 Edge 130 and M4 Safari 18.x owner-manual results are missing, so AC #3, #4, #5, #6, #9 stay unchecked; (b) DEF-1 on TASK-146: Firefox 100.0 fails 0/10 Mermaid diagrams with TypeError: Illegal constructor (NFR-005.001 floor); (c) M2 320 px samples are clamped to 438 px by the Firefox/geckodriver minimum window width and are recorded as a limitation. No product fix was made in TASK-150.

---

author: planner
created: 2026-11-16T15:24:00Z

---

Planning correction 2026-11-16: NFR-005.001 now requires Firefox 115+ only; Edge 130, Safari 18, and the Firefox 100 floor are withdrawn, and the Mermaid CSSStyleSheet shim (ADR-0026 decision 13) is removed. Consequences: the M3 (Edge 130) and M4 (Safari 18.x) owner-manual matrix rows and the Firefox 100 probe are out of scope; DEF-1 on TASK-146 is no longer a defect against the amended NFR; TASK-150 AC #3-#6 and #9 and DoD #2/#8 need wording changes to Firefox 115+ only, and the browser-matrix section (M2) should target Firefox 115+ (or the oldest ESR at/above 115) rather than Firefox 100. No milestone, dependency, ordinal, or priority change: TASK-150 remains the sole open task in m-21 at ordinal 41620 with dependencies TASK-140, TASK-146, TASK-148, TASK-149. Follow-ups: task-refiner to reword TASK-150 AC/DoD; architect to align ADR-0026 and other specs that still cite Firefox 100/Edge 130/Safari 18; business-analyst to align browser specifications; rust-dev to update packaging.md evidence (Firefox 115+ only) and record the shim removal/DEF-1 resolution.

---

author: @rust-dev
created: 2026-11-16T15:29:00Z

---

Amendment (2026-11-16): NFR-005.001 now Firefox 115+ only; Edge 130/Safari 18/Firefox 100 out of scope; DEF-1 obsolete; shim removed (4a54cc1). Rebuilt deb 2,379,448 B / 874d03de...; binary unchanged 8,323,296 B / f306d222...; cargo test 1048 passed. AC #3-#6,#9 and DoD #2,#8 checked. Release vehicle unchanged: no release milestone/ticket by m-21; owner decides.

---
