---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-001
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-001
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-06-13

references:
  decisions:
    - DEC-009                       # edition 2021, stable, three-OS CI matrix + native-feature job
    - DEC-004                       # pure-Rust codecs by default → trivial multi-OS CI
    - DEC-007                       # thiserror in lib / anyhow at binary boundary (shapes error.rs + main split)
    - DEC-006                       # no async runtime (binary stays sync; nothing pulls tokio)
    - DEC-002                       # single canonical model + Operation trait (the lib this scaffold will eventually host)
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - every-public-fn-tested
    - no-new-top-level-deps-without-decision
    - no-async-runtime
    - one-spec-per-pr
    - no-secrets-in-code
  related_specs:
    - SPEC-002                      # next: canonical Image type + load/decode (adds the `image` dep)
    - SPEC-007                      # clap subcommand skeleton lands the full CLI surface

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-XXX's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling STAGE-001's runnable binary + green multi-OS CI gate that every later spec builds and tests against"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Null numeric fields are fine (e.g. claude.ai web sessions); reports
# skip them in sums but count them in session_count. Examples of
# interface: claude-code | claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 25
      recorded_at: 2026-06-13
      notes: "subagent; cost not separately reported"
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_input: null
      tokens_output: null
      estimated_usd: null
      duration_minutes: 15
      recorded_at: 2026-06-13
      notes: "subagent; cost not separately reported"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-001: cargo project and multi-OS CI

## Context

This is the very first spec of PROJ-001 (the crustyimg clean rebuild) and the
first item in STAGE-001's backlog (foundation and pipeline core). Nothing is
scaffolded yet — there is no `Cargo.toml`, no `src/`, no CI. Every other spec
in the stage (the `Image` type, the `Operation` trait, the pipeline, recipes,
source/sink, the clap surface) needs a compiling project and a green CI gate
to build and test against. This spec lays exactly that and nothing more.

The rebuild's defining discipline is "tests and multi-OS CI from spec one"
(project `brief.md`) — the prototype had zero tests, dead modules, and
warnings. So before any image logic exists, we stand up: a compiling Rust
project (library + `crustyimg` binary), a three-OS GitHub Actions matrix that
enforces build/test/clippy/fmt (DEC-009), and one smoke test proving the
binary runs and reports a version.

Deliberately NOT in this spec: any real subcommands, the pipeline, the
`Operation` trait, the `Image` type, or the full clap surface. Those are
SPEC-002..007. The binary here is a trivial entrypoint — just enough that
`--version` and `--help` work for a smoke test.

- Parent stage: `projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md` (backlog item #1)
- Project: `projects/PROJ-001-crustyimg-mvp/brief.md`
- Architecture: `docs/architecture.md` (module layout this scaffold will grow into)
- CLI contract: `docs/api-contract.md` (the eventual surface; only `--version`/`--help` matter here)

## Goal

Stand up a compiling Rust project — a `crustyimg` library crate plus a thin
binary that supports `--version`/`--help` — with a GitHub Actions CI matrix on
Linux/macOS/Windows that enforces `cargo build`, `cargo test`,
`cargo clippy -- -D warnings`, and `cargo fmt --check`, plus one integration
smoke test asserting the binary runs and prints a semver. No real commands or
pipeline logic.

## Inputs

- **Files to read:**
  - `AGENTS.md` — §5 stack (edition, no async), §6 exact commands, §11
    conventions (library-first, no dead code, lint clean), §12 testing
    (integration tests under `tests/`), §13 git/PR conventions.
  - `docs/architecture.md` — the planned `src/` module layout this scaffold
    seeds (lib + thin `main.rs`); do NOT create the submodules yet.
  - `docs/api-contract.md` — confirms binary name `crustyimg` and that
    `--version`/`--help` are standard clap; the full surface is SPEC-007.
  - `guidance/constraints.yaml` — the constraints listed below.
  - `decisions/DEC-009-ci-matrix-and-rust-edition.md` — the CI matrix + edition source of truth.
- **External APIs:** none.
- **Related code paths:** none yet — this spec creates the first code.

## Outputs

- **Files created (at build):**
  - `Cargo.toml` — package manifest. Package name `crustyimg`, edition `2021`,
    a `[lib]` and a `[[bin]] name = "crustyimg"`. Start with **no third-party
    runtime dependencies** (see dependency policy below). `version` is the
    semver the smoke test asserts (e.g. `0.1.0`).
  - `src/lib.rs` — the library crate root. Minimal: expose at least one tested
    public item so the lib is real and `every-public-fn-tested` is honored
    (e.g. a `pub fn version() -> &'static str` returning `env!("CARGO_PKG_VERSION")`,
    with a `#[cfg(test)]` unit test). Keep it tiny; submodules land in later specs.
  - `src/main.rs` — thin binary entrypoint. Parses argv minimally to support
    `--version`/`-V` (print the package version, exit 0) and `--help`/`-h`
    (print a one-line usage, exit 0); any other invocation prints usage to
    stderr and exits non-zero. May call `crustyimg::version()`. No clap
    required yet (clap is SPEC-007); a hand-rolled match on `std::env::args`
    is acceptable and keeps the dep set empty. Must contain no
    `unwrap()`/`expect()`/`panic!()` on recoverable paths.
  - `tests/smoke.rs` — integration smoke test (see Failing Tests).
  - `.github/workflows/ci.yml` — GitHub Actions workflow, three-OS matrix
    (`ubuntu-latest`, `macos-latest`, `windows-latest`) running build, test,
    clippy (`-D warnings`), and fmt (`--check`). Optionally one extra job
    builds with `--features mozjpeg` per DEC-009 — but only if it does not
    require a dependency this spec hasn't added; since no codec deps exist
    yet, the native-feature job is **out of scope here** and lands when the
    feature is introduced (note it as a follow-up, do not fabricate a feature).
  - `.gitignore` — ignore `/target`.
- **Files modified:** none (greenfield).
- **New exports:** `crustyimg::version() -> &'static str` (or equivalent
  minimal tested public item).
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] `cargo build` succeeds from a clean checkout with the default feature
      set (no system libraries required — pure Rust, DEC-004).
- [ ] The project is edition `2021` and the binary target is named
      `crustyimg` (DEC-009).
- [ ] `cargo run -- --version` (and `-V`) exits 0 and prints a string matching
      a semver `MAJOR.MINOR.PATCH` (matches `Cargo.toml` `version`).
- [ ] `cargo run -- --help` (and `-h`) exits 0 and prints usage text naming
      the binary `crustyimg`.
- [ ] An unknown/no subcommand invocation exits non-zero and writes its
      message to stderr (stdout stays clean), consistent with the api-contract
      "diagnostics to stderr" rule.
- [ ] `cargo test` passes (the smoke test in `tests/smoke.rs` and the lib unit
      test) on the local platform.
- [ ] `cargo clippy -- -D warnings` produces zero warnings.
- [ ] `cargo fmt --check` reports no diffs.
- [ ] `.github/workflows/ci.yml` defines a matrix over `ubuntu-latest`,
      `macos-latest`, `windows-latest`, each running build, test,
      `clippy -- -D warnings`, and `fmt --check` (DEC-009; constraint
      `clippy-fmt-clean`).
- [ ] No async runtime appears in `Cargo.toml` or `src/` (constraint
      `no-async-runtime`).
- [ ] No new top-level dependency is added without a DEC; if the build adds
      any runtime dep, it emits a DEC justifying it (constraint
      `no-new-top-level-deps-without-decision`).

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in **build**
is to make these pass. Use `std::process::Command` against the compiled
binary; locate it via the `CARGO_BIN_EXE_crustyimg` env var Cargo sets for
integration tests (no extra dependency needed). Assert on exit status and the
captured stdout/stderr.

- **`tests/smoke.rs`**
  - `"version_flag_prints_semver"` — runs `crustyimg --version`; asserts the
    process exits 0, and stdout (trimmed) matches a semver regex
    `^\d+\.\d+\.\d+`. (Implement the match without a regex crate — e.g. split
    on `.` and check three numeric components — to keep the dep set empty.)
  - `"version_short_flag_matches_long"` — runs `crustyimg -V`; asserts exit 0
    and that its stdout equals the `--version` stdout.
  - `"version_matches_cargo_pkg_version"` — asserts the printed version equals
    `env!("CARGO_PKG_VERSION")` (the value compiled into the test crate),
    proving the binary reports its real package version.
  - `"help_flag_exits_zero_and_names_binary"` — runs `crustyimg --help`;
    asserts exit 0 and that stdout contains the string `crustyimg`.
  - `"unknown_invocation_exits_nonzero_on_stderr"` — runs
    `crustyimg bogus-subcommand`; asserts a non-zero exit, that stdout is
    empty, and that stderr is non-empty (diagnostics go to stderr per
    `docs/api-contract.md`).

- **`src/lib.rs`** (unit test in a `#[cfg(test)] mod tests` block)
  - `"version_returns_cargo_pkg_version"` — asserts
    `crustyimg::version() == env!("CARGO_PKG_VERSION")` (satisfies
    `every-public-fn-tested` for the one public fn).

## Implementation Context

*Read this section (and the files it points to) before starting the build
cycle. It is the equivalent of a handoff document, folded into the spec since
there is no separate receiving agent.*

### Decisions that apply

- `DEC-009` — **Edition 2021, stable toolchain, three-OS CI matrix.** This is
  the primary driver: set `edition = "2021"`, and the CI workflow must run
  build/test/clippy(`-D warnings`)/fmt(`--check`) on ubuntu, macos, and
  windows. The native-feature (`mozjpeg`) job described in DEC-009 is deferred
  here because no codec dependency/feature exists yet — do not invent one.
- `DEC-004` — **Pure-Rust codecs by default.** Keep the default build free of
  system libraries so the three-OS matrix is trivially green. This spec adds
  no codecs at all, which trivially satisfies it; honor it by NOT pre-adding
  native-dep crates.
- `DEC-007` — **`thiserror` in the library, `anyhow` at the binary boundary;
  no `unwrap`/`expect`/`panic!` on recoverable paths.** The full error
  scaffolding (`src/error.rs`, exit-code mapping) lands with SPEC-002+. For
  THIS spec, just respect the principle: the thin `main.rs` should not panic
  on recoverable input (unknown args → friendly stderr message + non-zero
  exit, not a panic). Do NOT add `thiserror`/`anyhow` deps yet — there is no
  typed error surface to justify them; they arrive with the spec that needs
  them (no-new-top-level-deps-without-decision).
- `DEC-006` — **No async runtime.** The binary is plain synchronous Rust.
  Nothing here should pull `tokio`/`async-std`.
- `DEC-002` — **Single canonical model + `Operation` trait.** Not implemented
  here, but it is the architecture this scaffold will grow into; the `src/`
  layout in `docs/architecture.md` (image/operation/pipeline/recipe/source/
  sink/metadata/cli) is the planned shape. Do NOT create those submodules in
  this spec — keep `lib.rs` minimal.

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `clippy-fmt-clean` — code must pass `cargo clippy -- -D warnings` and
  `cargo fmt --check`; no dead code, no commented-out code. The CI workflow
  enforces this on all three OSes.
- `test-before-implementation` — the failing tests above are the contract;
  make them pass, don't rewrite them to fit the code.
- `every-public-fn-tested` — every public fn gets at least one test; keep the
  public surface tiny (one `version()` fn) and test it.
- `no-new-top-level-deps-without-decision` — prefer **zero** runtime
  dependencies for this scaffold (std-only `main.rs`). If you add any, you
  MUST first write a `DEC-*` justifying it. (Pure-std is achievable here, so
  the expected outcome is an empty `[dependencies]`.)
- `no-async-runtime` — no `tokio`/`async-std`; the core is synchronous.
- `one-spec-per-pr` — exactly one SPEC reference (SPEC-001) in the PR body.
- `no-secrets-in-code` — the CI workflow uses no secrets; do not add any.

### Prior related work

- None — this is the first spec in the repo. No prior PRs.

### Out of scope (for this spec specifically)

If any of these feels necessary during build, create/await the owning spec
rather than expanding this one.

- Any real subcommand (`view`, `info`, `resize`, …) or the full clap
  subcommand surface — that is SPEC-007. The api-contract subcommand table is
  informational only here.
- The `clap` dependency itself — introduced by SPEC-007 (the spec that needs
  the subcommand parser). A hand-rolled argv match is the right call now.
- The canonical `Image` type, load/decode, and the `image` crate — SPEC-002.
- `error.rs`, typed `thiserror` enums, `anyhow`, exit-code mapping — they
  arrive with SPEC-002+ as real error surfaces appear.
- The full crate table from `docs/architecture.md` — **do NOT pre-add it.**
  Unused deps would trip `clippy -D warnings` and violate
  `no-new-top-level-deps-without-decision`. Each later spec adds the deps it
  uses, with its own DEC where required.
- The `src/` submodule tree (image/operation/pipeline/…) — seeded by the
  specs that fill them, not stubbed here.
- The `--features mozjpeg` CI job (DEC-009) — deferred until the codec feature
  exists; note it as a follow-up.
- Release/packaging workflows (brew, crates.io, GitHub Releases) — a later
  project concern, not STAGE-001.

### Exact commands (from AGENTS.md §6)

```bash
cargo build                         # debug build
cargo run -- --version              # smoke: prints version
cargo run -- --help                 # smoke: prints usage
cargo test                          # all tests (unit + integration)
cargo clippy -- -D warnings         # lint, warnings are errors
cargo fmt --check                   # formatting gate (CI); `cargo fmt` to fix
```

## Notes for the Implementer

- **Locate the binary in the smoke test the Cargo-native way:** integration
  tests get `env!("CARGO_BIN_EXE_crustyimg")` pointing at the built binary —
  use it with `std::process::Command`. No `assert_cmd`/`escargot` dependency
  needed (and adding one would trip the deps constraint).
- **Keep `main.rs` thin and panic-free.** A `match` over
  `std::env::args().nth(1).as_deref()` handling `Some("--version")|Some("-V")`,
  `Some("--help")|Some("-h")`, `None`/unknown is enough. Write version/help to
  stdout; write the unknown-arg/usage error to stderr and return a non-zero
  exit (e.g. `std::process::exit(2)` to align with the api-contract's usage
  exit code 2, or return a non-zero code via `main() -> ExitCode`). Prefer
  `std::process::ExitCode` over `panic!`.
- **Semver assertion without a regex crate:** trim stdout, split on `.`, take
  the first three segments, and confirm each parses as a `u32` (or check the
  prefix matches `env!("CARGO_PKG_VERSION")`). This keeps `[dependencies]`
  empty.
- **CI workflow shape:** a single workflow with a `strategy.matrix.os` of the
  three runners, `runs-on: ${{ matrix.os }}`, the `dtolnay/rust-toolchain@stable`
  (or `actions-rs`/`rustup` equivalent) action with `clippy` and `rustfmt`
  components, then steps for `cargo build`, `cargo test`,
  `cargo clippy -- -D warnings`, `cargo fmt --check`. Trigger on `push` and
  `pull_request`. No secrets. Keep `fmt --check` and `clippy` as their own
  steps so a failure points at the right gate.
- **Windows is the finicky leg** (DEC-009 consequences): keep stdout
  assertions tolerant of a trailing newline (`trim()` the captured output);
  don't assert on `\n` vs `\r\n`.
- **`.gitignore`:** ignore `/target` so the PR diff stays clean.
- The expected end state of `[dependencies]` is **empty**. If you find
  yourself reaching for a crate, stop and confirm it's truly required for the
  scaffold (it almost certainly isn't) — and if so, write a DEC first.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-001-cargo-project-and-multi-os-ci`
- **PR (if applicable):** #PR_NUMBER — PR_URL
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - None — the scaffold stayed std-only (empty `[dependencies]`), so no
    dependency-justifying DEC was required.
- **Deviations from spec:**
  - `Cargo.lock` is committed (not in spec's Outputs list). Standard practice
    for a binary crate (reproducible CI builds); `.gitignore` ignores `/target`
    only, as specified.
  - The repo already had a root `.gitignore`; rather than overwrite it I added
    a `# Rust` / `/target` entry, satisfying the "ignore `/target`" output.
- **Follow-up work identified:**
  - Add a `--features mozjpeg` CI job (DEC-009) once a codec feature exists
    (introduced with the spec that adds the native-codec dependency).
  - No new stage-backlog specs; SPEC-002..007 already cover the planned work.

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing material. `Cargo.lock` handling was the only judgment call the
   spec left implicit (its Outputs list neither includes nor excludes it); I
   committed it per binary-crate convention. The `## Failing Tests` were
   precise enough to implement directly.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The applicable constraints (`clippy-fmt-clean`,
   `no-new-top-level-deps-without-decision`, `no-async-runtime`, etc.) covered
   everything. Worth noting that `no-unwrap-on-recoverable-paths` is scoped to
   `src/**`, which is correct — the smoke test's `.expect()` calls are in
   `tests/` and are idiomatic for test setup, so no conflict arose.

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt` (not just `--check`) before the first gate pass; my
   initial `tests/smoke.rs` had a multi-line `assert!` that rustfmt collapsed,
   costing one extra format round-trip. Minor.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
</content>
</invoke>
