---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-012                        # stable, never reused
  type: decision                     # decision | analysis | recommendation | observation
  confidence: 0.9                    # 0.0 - 1.0, honest assessment
  audience:                          # who needs to know?
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001                       # the project during which this was decided
repo:
  id: crustyimg

created_at: 2026-06-14
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/cli/**
  - src/main.rs
  - Cargo.toml

tags:
  - cli
  - dependencies
  - ergonomics
---

# DEC-012: `clap` (derive) as the CLI framework

## Decision

`crustyimg` uses **`clap` 4 with the `derive` feature** as its sole
command-line parsing framework. The subcommand surface, global options, and
argument parsing live in `src/cli/` as `#[derive(Parser)]` / `#[derive(Subcommand)]`
types; `main.rs` is a thin shell that parses, dispatches into `cli`, and maps
typed library errors to the api-contract exit codes (DEC-007). No second arg
parser, and no hand-rolled argv matching (the SPEC-001 scaffold is replaced).

## Context

SPEC-007 turns the shipped library (Image/load, Operation/Pipeline, Source,
Sink, Recipe/registry) into a usable binary. It needs the full MVP subcommand
surface from `docs/api-contract.md` (view, info, resize, thumbnail, shrink,
convert, auto-orient, watermark, strip, clean, set, copy-metadata, edit, apply),
a table of global options (`-o/--output`, `--out-dir`, `--name-template`,
`-j/--jobs`, `--format`, `-q/--quality`, `-v/--verbose`, `-Q/--quiet`,
`-y/--yes`, `--keep-gps`, plus `--version`/`--help`), `-`/stdin-stdout handling,
and usage errors that exit with the contract's code 2.

`clap` is named as the intended CLI framework in three places already —
`AGENTS.md` §5 ("CLI framework: `clap` 4 (derive, subcommands)"),
`docs/architecture.md` (the crate-choices table lists `clap 4 (derive)`), and
`docs/api-contract.md` ("Built with `clap` derive, subcommand style"). But it is
a **new top-level dependency** not yet in `Cargo.toml`, and the constraint
`no-new-top-level-deps-without-decision` requires a DEC before adding any new
top-level crate. This file is that DEC.

The prototype's failure mode was ~1,000 lines of overlapping boolean flags in
one `main.rs`. The product principle (`ergonomic-defaults`) wants the common
single-image task to be one short command. `clap`'s derive macro gives a typed,
self-documenting subcommand model with generated `--help`/`--version`, standard
usage-error exit code 2, and per-subcommand help — directly serving both the
anti-flag-soup goal and ergonomic defaults.

## Alternatives Considered

- **Option A: hand-rolled argv parsing (the SPEC-001 scaffold, kept)**
  - What it is: keep matching `std::env::args()` by hand, growing it per command.
  - Why rejected: this is exactly the prototype's flag-soup path. No generated
    help, no typed subcommands, no consistent usage errors, no per-command help.
    Does not scale to 14 subcommands with global options.

- **Option B: a lighter arg parser (`pico-args`, `lexopt`, `argh`, `bpaf`)**
  - What it is: smaller-footprint parsers without clap's derive ergonomics.
  - Why rejected: smaller binary/compile cost, but no rich derive-based
    subcommand+global-option model, weaker generated help, and they are not the
    crate the architecture/contract/AGENTS already standardized on. Diverging
    would contradict three existing docs for marginal benefit.

- **Option C (chosen): `clap` 4 with `derive`**
  - What it is: the de-facto Rust CLI framework; derive macro maps structs/enums
    to args, generates `--help`/`--version`, emits exit code 2 on usage errors.
  - Why selected: already the named choice in AGENTS §5, architecture, and the
    CLI contract; typed subcommands kill the flag-soup; generated help and the
    standard usage exit code match the api-contract for free; derive keeps the
    arg surface declarative and ergonomic-defaults easy to express.

## Consequences

- **Positive:** Typed, declarative subcommand surface; generated, consistent
  `--help`/`--version` and per-command help; usage errors map to exit code 2
  with no custom code; the arg layer is the only part of the crate that knows
  about `clap` (the pixel core stays clap-free, per the architecture layering).
- **Negative:** `clap` + its derive macro add compile time and binary size
  versus a minimal parser; a proc-macro dependency in the build graph.
- **Neutral:** `clap`'s default usage/error exit code is `2`, which already
  matches the api-contract usage-error code — convenient, but it means the
  binary must let clap own code 2 and only map the *typed library* errors
  (codes 1, 3, 4, 5, 6) itself.

## Validation

Right if: the full subcommand surface parses per the api-contract, `--help`
lists every subcommand, `--version` prints the semver, an unknown subcommand /
bad args exit with code 2, and the pixel core never imports `clap`. Revisit if:
binary size / compile time become a shipping concern (then weigh a lighter
parser), or if clap's exit-code behavior diverges from the contract in a future
major version.

## References

- Related specs: SPEC-007 (introduces `clap`, the CLI skeleton + dispatch)
- Related decisions: DEC-007 (anyhow at the boundary + exit-code mapping),
  DEC-002 (the Operation/pipeline the CLI drives), DEC-005 (recipes the
  `apply` path runs), DEC-006 (`--jobs` is a parsed placeholder here)
- External docs: https://docs.rs/clap
- CLI contract: `docs/api-contract.md` (Global Options, Subcommand Surface,
  Exit Codes); Architecture: `docs/architecture.md` (the `cli/` module + layering)
- Constraint: `no-new-top-level-deps-without-decision` (this DEC satisfies it
  for `clap`); `ergonomic-defaults`
