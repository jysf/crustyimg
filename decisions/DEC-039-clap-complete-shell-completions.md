---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-039
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/mod.rs
  - Cargo.toml

tags:
  - dependencies
  - cli
  - completions
  - release
  - license
---

# DEC-039: `clap_complete` for shell completions via a `completions` subcommand

## Decision

Add **`clap_complete` `=4.6.5`** (MIT OR Apache-2.0, pure-Rust — part of the clap
project) as a **default (non-optional) dependency**, and expose shell completions
through a new **`completions <SHELL>` subcommand** that writes the generated script
to **stdout**. `SHELL` is `clap_complete::Shell`, a clap `ValueEnum` with the values
`bash`, `elvish`, `fish`, `powershell`, `zsh`. The handler is a thin call:

```rust
use clap::CommandFactory;
use clap_complete::generate;
let mut cmd = Cli::command();
generate(shell, &mut cmd, "crustyimg", &mut std::io::stdout());
```

It produces NO image work, takes no input path, touches no file system, and exits `0`.
A user (or a packager) runs e.g. `crustyimg completions zsh > _crustyimg`.

## Context

STAGE-007 backlog #6 calls for clap-generated shell completions alongside the README
install/usage rewrite. `clap_complete` is the canonical companion to the `clap` derive
surface the CLI already uses (DEC-012) — it reflects over the existing `Cli` command
tree, so completions stay in sync with the subcommands automatically; there is nothing
to hand-maintain.

A **design-time probe** (per the "probe load-bearing crates at design" practice)
confirmed the integration against the pinned `clap =4.6.1`:

- `clap_complete =4.6.5` resolves and compiles with `clap =4.6.1` (clap_complete tracks
  clap's minor version). A throwaway `examples/` binary built and ran:
  `crustyimg completions bash` emitted a valid `_crustyimg()` bash script; `--help`
  showed `[possible values: bash, elvish, fish, powershell, zsh]`.
- `cargo deny check licenses` stays **green** with the dep present — `clap_complete` is
  `MIT OR Apache-2.0`, pure-Rust, no new exception required.
- The probe was reverted; `Cargo.toml`/`Cargo.lock` returned to clean before design.

**Subcommand (runtime) over a `build.rs` (static files):**

- The subcommand drops straight into the existing `Commands` enum + `dispatch()` match
  with no new build plumbing, and — crucially — it works against the **installed**
  binary, which is what the downstream release packaging needs: Homebrew's
  `generate_completions_from_executable` and cargo-dist (backlog #3/#4) both run
  `crustyimg completions <shell>` at install time. Static `build.rs` output would
  require factoring `Cli::command()` into a path the build script can import and would
  ship files that drift from the binary.
- It must compile and work in **both** the default build and the lean
  `--no-default-features` (headless) build — `clap_complete` is a default, non-optional
  dep, so it is present in both; no feature gate.

`clap_complete` is a new top-level dependency, so the
`no-new-top-level-deps-without-decision` constraint requires this record.

## Alternatives Considered

- **`build.rs` + `clap_complete::generate_to` (static completion files in the repo /
  release tarball)** — rejected for v1: more build-time plumbing (shared `Command`
  factory), output drifts from the binary, and the release packagers prefer running the
  binary. Can still be added later if a distro wants committed files.
- **Hand-written completion scripts** — rejected: they rot the moment a subcommand or
  flag changes; the whole point of `clap_complete` is reflection over the real surface.
- **No completions** — rejected: the stage backlog explicitly wants them, and they are
  a meaningful CLI ergonomics win (`ergonomic-defaults`).
- **A separate `clap_mangen` man page in this spec** — deferred (out of scope here);
  `clap_mangen` is the analogous generator and can be a small follow-up or folded into
  the cargo-dist packaging.

## Consequences

- **Positive:** Completions for five shells, always in sync with the clap surface, with
  one tiny handler. Works on the installed binary so release packaging (brew/cargo-dist)
  can wire it up. Pure-Rust, permissive, `just deny` green; present in the lean build.
- **Negative:** +1 default dependency (small — it is a generator over clap's own types).
  The `completions` subcommand now appears in `--help` (the existing
  `help_lists_all_subcommands` test uses `contains`, so it is unaffected).
- **Neutral:** Completions are written to stdout only; *installing* them into a shell's
  completion directory is the user's / packager's step, documented in the README.
