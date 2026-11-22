---
id: ADR-043
title: "Component build script as the packaging entry point"
created_date: '2026-11-22T15:23:49Z'
updated_date: '2026-11-22T15:24:16Z'
labels: []
references:
  - ADR-038
  - ADR-0027
  - TECH-008
  - TECH-013
  - TECH-024
  - TECH-027
  - TECH-069
  - MS-035
  - MS-036
  - MS-037
  - TASK-235
  - TASK-270
  - TASK-279
  - TASK-289
  - TASK-288
superseded_by: null
---
# ADR-043 Component build script as the packaging entry point

## Status
accepted
## Decision
## Supersession and scope

1. This ADR supersedes TASK-235 ruling 1, recorded 2026-11-21 (UTC): "use the existing build-deb script under scripts for verification and the release boundary; do not add a build-package script under scripts." The prohibition is retired. TASK-235 rulings 2 and 3 (no-new-crates gate including dev- and build-dependencies; production LOC as an informational measurement, never a gate) remain in force.
2. It supersedes, in ADR-038 decision 30, only the clause "No new file, build script, code generation, environment variable, or network access is introduced", and only insofar as that clause forbids a component build script and a version-passing mechanism. The rest of decision 30 survives: the enumeration of the vendor version directories, the read of the manifest version column, the failure conditions (missing version directory, more than one, manifest version mismatch, copyright version mismatch), the Cargo.toml derivation of the Papyrus package version as the default, and the rule that `cargo build` stays the whole build.
3. It supersedes the wording of TECH-069 section 6, and of TECH-008, TECH-013, TECH-024, and TECH-027, that names `scripts/build-deb.sh` as *the* packaging script and states that no separate build-package script exists. Their Debian inspection, isolated install, offline run, and removal procedures stand; only the entry-point name, the version source, and the output location change.
4. It supersedes the sentence recording that the single final release artifact remains the `scripts/build-deb.sh` package produced by the release task: the retained final package is produced by `scripts/build-package.sh`, which delegates the Debian assembly to `scripts/build-deb.sh`.
5. ADR-038 keeps its accepted status. This is an amendment by clause of a packaging-name question, not a change to any cache class, response header, asset URL, or asset rule.

## Entry point

6. `scripts/build-package.sh` at the repository root is the packaging entry point for the deployable component. The repository is single-app, so the generic `<component-dir>/scripts/build-package.sh` path resolves to `scripts/build-package.sh`; no `apps/<app>` directory and no per-component script is introduced.
7. The interface is fixed:
   - `scripts/build-package.sh --version <version> --out <dir>`
   - `scripts/build-package.sh --help`

   `--version` is the version exactly as packaging received it from the versioning step: a base version for an RC (`0.2.0-rc.1`) or the final version (`0.2.0`). It is not read from Git tags and not re-derived from the package manifest. `--out` is the directory that receives the deliverables; packaging supplies `builds/<version>/.build/<component>`. Mode 755, `#!/usr/bin/env bash`, `set -euo pipefail`. `--help` prints usage and exits 0; missing or unknown arguments exit non-zero with a clear message on stderr.
8. It is the only command packaging runs for the component, in both modes: the RC package at the end of a verified milestone and the final package at the release boundary. Ad-hoc `cargo build` plus `dpkg-deb` invocations in packaging are not permitted.

## Relationship to the Debian assembly script

9. `scripts/build-package.sh` invokes `scripts/build-deb.sh`. It does not assemble the `.deb` itself.
10. `scripts/build-deb.sh` remains the Debian assembly step and the single source of truth for package content: the staged tree (`DEBIAN/control`, `usr/bin/papyrus`, `usr/share/doc/papyrus/copyright`), the `amd64` architecture, and the ADR-038 decision 30 asset-version verification. The build script neither duplicates that layout nor re-implements that verification; a second copy of the assembly would let the two drift.
11. `build-deb.sh` keeps working as a developer-facing script. Run directly with no version supplied, it derives the version from the package manifest and writes the `.deb` to the project root exactly as today. That direct path produces no published package.
12. `build-package.sh` is responsible for what `build-deb.sh` does not do: version injection, output isolation, destination cleanup, artifact discovery, and the stdout/stderr contract.

## Version injection

13. The `--version` value is authoritative for the produced package. It must appear in the `Version:` field of `DEBIAN/control` and in the artifact file name (`papyrus_<version>_amd64.deb`), so an RC package is distinguishable from the final package of the same base version.
14. Injection happens without editing a tracked file: no `Cargo.toml`, `Cargo.lock`, `packaging/deb/DEBIAN/control`, or other committed file is rewritten. The mechanism that carries the value (a flag, an environment variable, or a generated staging file) is chosen by the implementing task; this ADR fixes the semantics only: an explicitly supplied version overrides the manifest derivation, and a packaging run must leave the working tree clean (`git status --porcelain` empty before and after).
15. The asset version is not the package version. The assembly step continues to derive the packaged asset version from the vendor directories and the manifest (ADR-038 decision 30), and an injected package version appears in no asset URL.

## Output contract

16. `--out` receives deliverables only: the `.deb` and, where the implementing task needs it, its `.sha256`. The script cleans its own destination subtree before writing, so a rerun into the same directory is idempotent and leaves nothing from a previous run.
17. The script prints one produced file path per line on stdout and sends every diagnostic to stderr. It exits non-zero when the build fails, when the toolchain is missing, or when a promised artifact is absent or empty; it never exits 0 with an incomplete output.
18. After a `build-package.sh` run, no package file is left in the project root. The Debian staging directory under `target/` and the normal tool cache remain the only writes outside `--out`.
19. The script is non-interactive. It commits, tags, pushes, uploads, and deploys nothing.

## When the gate applies

20. MS-034 remains a waived internal checkpoint: no RC version is determined, no package is published, and `scripts/build-package.sh` need not exist for that boundary. The recorded waiver stands.
21. From MS-035 onward, every milestone whose verification passes publishes an RC package for the RC version, from the committed verified tree, through this script, under the git-ignored `builds/<version>/`. MS-035, MS-036, and MS-037 verification tasks carry that gate in their definition of done and acceptance criteria.
22. The final release publishes the final package through the same script with the final version. One script, two versions; the final package is not a separately built artifact type and is not a different format from the RC package.
23. A packaging waiver remains user-only and explicit, with scope, reason, and timestamp with timezone recorded in the verification task or the release report. Without one, an absent, non-executable, or failing `scripts/build-package.sh` blocks the milestone.

## Downstream constraints

1. `scripts/build-package.sh` is the only packaging entry point; `scripts/build-deb.sh` is its Debian assembly step.
2. `--version` is authoritative and injected without editing tracked files; the manifest derivation is the direct-run default only.
3. `--out` receives deliverables only and is cleaned first; stdout lists produced paths, diagnostics go to stderr.
4. The assembly step keeps the package layout and the ADR-038 asset-version check; the build script duplicates neither.
5. No tracked-file edit, no network access at build time, and no toolchain beyond cargo and `dpkg-deb`.
6. MS-034 stays waived; the gate starts at MS-035, repeats at every verified milestone, and the final release uses the same script.
## Context
The packaging workflow calls every deployable component's `scripts/build-package.sh` with a version and an output directory: `--version <X.Y.Z>` or `--version <X.Y.Z-rc.N>`, and `--out builds/<version>/.build/<component>`. In a multi-app repository the script sits under `apps/<app>/scripts/`; in a single-app repository it sits at `scripts/build-package.sh` from the repository root. The packaging and build-package-script skills were updated to state that single-app exception, and the milestone acceptance criteria of MS-035 to MS-037, the definitions of done of their verification tasks, and `.gitignore` (`builds/`) already assume it.

Two recorded rulings predate that contract and now block it:

- The architect ruling in TASK-235 ruling 1, 2026-11-21 (UTC), directed that the existing `scripts/build-deb.sh` be used "for verification and the release boundary" and that no build-package script be added under `scripts`. It also called the build-package name in MS-037 milestone text an inconsistency for the milestone owner to correct.
- ADR-038 decision 30 and TECH-069 section 6 name `scripts/build-deb.sh` as the packaging script, and TECH-008, TECH-013, TECH-024, and TECH-027 state that no separate build-package script exists and that the version is read from the package manifest. ADR-038 decision 30 additionally states that no new build script is introduced.

The user decided the packaging contract authoritatively: packaging starts at MS-035, MS-034 stays a waived internal checkpoint with no RC and no package, every verified milestone from MS-035 publishes an RC package, the final release publishes the final package, and the entry point is `scripts/build-package.sh` at the repository root.

Forces:

- One entry point that packaging can call with a version and an output directory, in both the RC and the final mode, so that no one reconstructs `cargo build` plus `dpkg-deb` by hand.
- The Debian layout, the control file, the copyright file, and the ADR-038 asset-version verification already live in `scripts/build-deb.sh`; duplicating them in a second script would create two assembly implementations that can drift.
- An RC version (`0.2.0-rc.1`) is not in the package manifest, so the version must arrive as an argument; it must not be injected by editing tracked files, or the packaged tree would no longer be the committed, verified tree.
- Packages must land in the git-ignored `builds/<version>/`, not in the project root, and a rerun must not pick up an earlier artifact.
- The verifier of a milestone must be able to reproduce exactly what packaging did from the recorded command.

Alternatives considered:

| Option | Outcome | Reason |
|---|---|---|
| `scripts/build-package.sh` invokes `scripts/build-deb.sh`; the version is injected without tracked-file edits; `--out` receives the deliverable (this ADR) | Selected | Satisfies the packaging contract, keeps one Debian assembly implementation, and keeps the packaged tree equal to the committed tree. |
| Keep `scripts/build-deb.sh` as the entry point and read the version from the manifest | Rejected | An RC version does not exist in the manifest, so an RC package cannot be produced without editing a tracked file; the version would also have to be re-derived by whoever runs packaging. |
| Make `build-package.sh` a second, self-contained `dpkg-deb` assembly | Rejected | Two assembly implementations of the same package layout and the same asset-version check; they would diverge, and the ADR-038 check would have to be maintained twice. |
| Rename `build-deb.sh` to `build-package.sh` and teach it the new interface | Rejected | It breaks the developer-facing direct invocation that derives the version from the manifest and writes to the project root, for no gain over a thin delegating wrapper. |
| Symlink `scripts/build-package.sh` to `scripts/build-deb.sh` | Rejected | A symlink cannot add the `--version` and `--out` interface, the destination cleanup, or the stdout contract without changing the assembly script's behaviour anyway. |
| Inject the RC version by editing `Cargo.toml` or the control template and reverting afterwards | Rejected | A packaging run must not modify tracked files; a crash mid-run would leave a dirty, mis-versioned tree, and the manifest-derived version would disagree with the artifact name. |
## Consequences
**Positive:**

- Packaging has one entry point with one fixed interface in both modes, so the RC and the final package are built by the same code path and differ only in the version argument.
- The Debian layout, the control and copyright files, and the ADR-038 asset-version verification keep a single implementation in `scripts/build-deb.sh`; the wrapper adds the interface, not a second assembly.
- The packaged tree stays identical to the committed, verified tree: the version arrives as an argument and no tracked file is edited, so the recorded commit SHA remains a faithful description of the artifact.
- The RC version appears in the control `Version:` field and in the artifact file name, so an RC package is distinguishable from the final package of the same base version without opening it.
- Packages land only in the git-ignored `builds/<version>/`, and the destination is cleaned first, so a rerun is idempotent and no package is left in the project root.
- The developer-facing direct run of `scripts/build-deb.sh` keeps working unchanged, including its manifest-derived version and project-root output.

**Negative:**

- There are now two scripts with a defined division of responsibility: the wrapper and the assembly step. A reader must know that `--version` overrides the manifest derivation and that the wrapper, not the assembly, owns the output directory.
- `scripts/build-deb.sh` must accept an externally supplied version without rewriting a tracked file, which is a small interface extension to a script that previously read the manifest only.
- The recorded packaging rulings of TASK-235, ADR-038 decision 30, TECH-069 section 6, and TECH-008, TECH-013, TECH-024, and TECH-027 now describe the previous entry point until they are revised, so the specification is temporarily inconsistent with the contract packaging runs.
- A packaging run now depends on `dpkg-deb` as well as the Rust toolchain; a missing `dpkg-deb` fails the RC gate at the end of a milestone rather than earlier.

**Risks and mitigations:**

- *Two assembly implementations drift*: the wrapper must only invoke the assembly step and must not stage, render, or verify the package itself; review checks that the wrapper contains no `dpkg-deb --build`, no control rendering, and no asset-version enumeration.
- *A tracked file is edited to carry the version*: the run is only valid when `git status --porcelain` is empty before and after, and the verification evidence records the commit SHA; a dirty tree after packaging fails the gate.
- *The final package is mistaken for a different artifact type*: the same script with the final version produces it, and the recorded evidence names the script, the version, and the output path for both modes.
- *The wrapper silently succeeds without an artifact*: it must exit non-zero when a promised artifact is missing or empty, and the verification records the file path, size, and SHA-256 from `--out`.
- *An old package from a previous run is published*: the destination is cleaned before writing, and a version directory is never rebuilt in place; changed content needs a new RC version.
- *The milestone gate is applied before the script exists*: MS-034 stays waived, and the first milestone that needs the script is MS-035, so the script is created before that gate, not at the release boundary.
- *The package version leaks into an asset URL*: the asset version stays manifest- and directory-derived (ADR-038 decisions 9, 29, and 30), and the injected package version appears in no asset path.

**Migration:**

- A task for the Rust owner creates `scripts/build-package.sh` with the interface of this decision and extends the assembly step to accept an externally supplied version without editing tracked files, before the MS-035 packaging gate runs.
- `tech-writer` revises TECH-008, TECH-013, TECH-024, TECH-027, and TECH-069 section 6: the packaging entry point, the version source, the output location, and the statement that no build-package script exists.
- The milestone owner corrects the release-boundary milestone text that names `scripts/build-deb.sh` as the single final artifact and the matching verification-task sentence, without changing the requirement that exactly one final release artifact is kept.
- No application code, test, requirement, business rule, cache class, header, or asset URL changes.
