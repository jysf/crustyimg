---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-040
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-007
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-039]
  constraints:
    - no-new-top-level-deps-without-decision
    - clippy-fmt-clean
    - every-public-fn-tested
    - test-before-implementation
    - no-agpl-default-deps
    - ergonomic-defaults
  related_specs: [SPEC-038, SPEC-039]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-007's <capability>". Optional; null is acceptable.
value_link: >
  Sixth STAGE-007 step (the last SAFE one): a user-facing README (install +
  usage) and clap-generated shell completions, delivering the stage's "a new
  user can install and run it" criterion — with no outward-facing release action.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work, not separately metered. Authored the spec +
        DEC-039 + the Sonnet build prompt for STAGE-007 #6 (README user-facing
        rewrite + clap_complete shell completions). Ran a design-time probe
        (probe-load-bearing-crates-at-design): added clap_complete =4.6.5 to a
        throwaway examples/ binary, confirmed it compiles+runs against pinned clap
        =4.6.1 (Shell ValueEnum: bash/elvish/fish/powershell/zsh; generate(...) API),
        and that `cargo deny check licenses` stays green (MIT OR Apache-2.0) — then
        reverted the probe (tree clean). Pinned: subcommand over build.rs, stdout
        only, lean build must work, README honesty for not-yet-live install channels,
        and the stale Apache-only License line corrected to MIT OR Apache-2.0. No
        tag/publish. Sixth STAGE-007 spec (last safe item).
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        code+docs: clap_complete =4.6.5 (DEC-039) + `completions <shell>` subcommand
        (stdout, 5 shells) with tests/completions.rs; README rewritten tool-first
        (install cargo/release/brew honestly labeled + works-today path, usage
        quickstart, completions, License corrected to MIT OR Apache-2.0, dev-process
        relocated). No tag/publish/tap. fmt/clippy/test/lean/deny green.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-040: readme rewrite and shell completions

## Context

**The sixth STAGE-007 step — and the last SAFE one (no outward-facing action).**
The crate is publish-ready (SPEC-038) and has a changelog + release policy
(SPEC-039). Two user-facing gaps remain before the outward-facing release items
(#3 cargo-dist, #4 Homebrew tap, #5 `cargo publish`, #7 dual artifacts):

1. **The README is still the *spec-workflow* README, not a *user* README.** It
   documents the spec-driven dev process (hierarchy, `just` commands,
   GETTING_STARTED) and quotes the project frame — but has **no install section**
   and **no usage examples**, and its License section is **stale**: it says
   "Licensed under the Apache License, Version 2.0. See `LICENSE`." while the crate
   is actually `MIT OR Apache-2.0` with dual `LICENSE-MIT` / `LICENSE-APACHE` files
   (SPEC-038). `Cargo.toml` sets `readme = "README.md"`, so this is exactly what
   crates.io and the GitHub landing page will show — it must lead with the **tool**.
2. **No shell completions.** A polished CLI ships completions; clap can generate
   them from the existing command surface for free (DEC-039).

This spec delivers STAGE-007's success criterion "README documents install (brew /
cargo / download) + a usage example" and adds completions. It is **docs + a small,
self-contained code addition** — no outward-facing action (no tag, no publish, no
release), and the not-yet-live install channels are documented honestly (see PINNED).

Parent: `STAGE-007` (backlog item #6). Related: `SPEC-038` (publish metadata, the
dual-license files), `SPEC-039` (CHANGELOG / RELEASING).

## Goal

Rewrite `README.md` into a user-facing landing page (what crustyimg is → install →
quickstart usage → completions → license, with the spec-driven dev process kept as a
clearly-marked secondary section), and add a `completions <SHELL>` subcommand
(`clap_complete`, DEC-039) that prints a shell-completion script to stdout for bash /
zsh / fish / powershell / elvish. Docs + a thin, tested code addition; nothing is
tagged, released, or published.

## Inputs

- **Files to read:**
  - `README.md` — the current spec-workflow README being rewritten (preserve the
    dev-process content; relocate it below the user-facing content).
  - `docs/api-contract.md` — the authoritative command surface, global options,
    stdin/stdout (`-`) behavior, and exit codes → the usage section + examples.
  - `docs/moat.md` — the one-paragraph "what crustyimg is / why" capability framing.
  - `Cargo.toml` — `version = "0.1.0"`, `license = "MIT OR Apache-2.0"`, the
    `[features]` block (`display` default-on, DEC-027; `webp-lossy`, `avif` opt-in) →
    the install/feature notes and the correct License section.
  - `RELEASING.md` / `CHANGELOG.md` (SPEC-039) — cross-reference from the README;
    the install channels that are "available once v0.1.0 is cut".
  - `src/cli/mod.rs` — the clap `Cli`/`Commands`/`dispatch()` surface the
    `completions` subcommand plugs into; `tests/cli.rs` — the binary-invocation test
    harness to mirror.
  - `decisions/DEC-039-clap-complete-shell-completions.md` — the dep + design choice.
- **External APIs:** none networked. `clap_complete` `=4.6.5`
  (https://docs.rs/clap_complete) — the completions generator (DEC-039, probe-verified
  against the pinned `clap =4.6.1`).
- **Related code paths:** `src/cli/mod.rs` (subcommand + dispatch), `Cargo.toml`
  (dep), `tests/completions.rs` (new), `README.md`.

## Outputs

- **Files created:**
  - `tests/completions.rs` — integration tests driving the real binary (see Failing
    Tests).
- **Files modified:**
  - `Cargo.toml` — add `clap_complete = "=4.6.5"` to `[dependencies]` (default,
    non-optional; with a short justifying comment referencing DEC-039, matching the
    style of the other dep comments).
  - `src/cli/mod.rs` — add a `Completions { shell: clap_complete::Shell }` variant to
    `Commands`, a dispatch arm, and a `run_completions(shell)` handler that calls
    `clap_complete::generate(shell, &mut Cli::command(), "crustyimg", &mut
    io::stdout())` (needs `use clap::CommandFactory`). No input path, no file/image
    work, exit 0.
  - `README.md` — full rewrite (see PINNED structure).
  - `decisions/DEC-039-...md` — already authored by the architect; the build sets its
    `session_id` only if it records one (no other change needed).
- **New exports:** none beyond the new `Commands::Completions` variant +
  `run_completions` (private). The library API is unchanged.
- **Database changes:** none.

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] `crustyimg completions bash|zsh|fish|powershell|elvish` each exits `0` and
  writes a **non-empty** completion script to **stdout** (nothing image-related; no
  input path required).
- [ ] The generated script references the binary name `crustyimg` (e.g. the bash
  output contains `_crustyimg` / `crustyimg`).
- [ ] `crustyimg completions <bogus-shell>` is a **usage error, exit `2`** (clap
  `ValueEnum` rejection), and `crustyimg completions --help` lists the five shells.
- [ ] `completions` appears in `crustyimg --help` (the existing
  `help_lists_all_subcommands` test still passes — it uses `contains`).
- [ ] The completions path compiles and works in **both** the default build and the
  lean `cargo build --no-default-features` build (clap_complete is non-optional).
- [ ] `README.md` leads with the tool: a one-line description, an **Install** section
  documenting **cargo**, **prebuilt-binary download (GitHub Releases)**, and
  **Homebrew** — with the not-yet-live channels clearly marked "available once v0.1.0
  is published" and a **working-today** path (`cargo install --git
  https://github.com/jysf/crustyimg` / build-from-source) — a **Usage** quickstart
  with ≥3 real, correct command examples drawn from `docs/api-contract.md` (e.g.
  `view`, `shrink … -o`, `info`, a `-`/stdin pipe), a **Shell completions** subsection
  showing `crustyimg completions zsh > …` per shell, and a **License** section that
  correctly states `MIT OR Apache-2.0` (dual `LICENSE-MIT` / `LICENSE-APACHE`).
- [ ] The README's stale "Apache License, Version 2.0" License line is gone; the
  spec-driven **dev-process** content is retained but moved below the user-facing
  content (or condensed with a pointer to `AGENTS.md` / `GETTING_STARTED.md`).
- [ ] README cross-references `CHANGELOG.md` / `RELEASING.md` (keep/relocate the
  existing "Changelog & releases" pointer).
- [ ] No git tag, no `cargo publish`, no release is created. The full gate suite stays
  green: `cargo fmt --check`, `cargo clippy`, `cargo test`, `cargo build
  --no-default-features`, and `cargo deny check advisories bans sources licenses`
  (clap_complete is `MIT OR Apache-2.0` — no new exception).

## Failing Tests

Written during **design**, BEFORE build (`test-before-implementation`). The
implementer makes these pass. Mirror the `tests/cli.rs` harness: drive the real binary
via `Command::new(env!("CARGO_BIN_EXE_crustyimg"))` (no `assert_cmd` dep).

- **`tests/completions.rs`** (new)
  - `"completions_bash_emits_script"` — `completions bash` exits `0`, stdout is
    non-empty and contains `crustyimg` (and `_crustyimg`).
  - `"completions_all_shells_succeed"` — for each of `bash, zsh, fish, powershell,
    elvish`: exit `0`, non-empty stdout. (Parameterized loop is fine.)
  - `"completions_rejects_unknown_shell"` — `completions klingon` exits `2` (clap
    usage error) and writes nothing useful to stdout.
  - `"completions_needs_no_input_path"` — `completions zsh` succeeds with **no**
    positional path argument and produces no file output (pure stdout).
- **`tests/cli.rs`** (existing — must still pass)
  - `help_lists_all_subcommands` continues to pass; optionally extend its expected
    list with `"completions"` to lock the new subcommand into the help surface.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-039` — `clap_complete =4.6.5` (default dep) + the `completions <SHELL>`
  **subcommand** (runtime, over a `build.rs`) writing to stdout. Probe-verified
  against pinned `clap =4.6.1`; `Shell` is a `ValueEnum` (bash/elvish/fish/
  powershell/zsh); `deny` stays green. **Read this DEC — it contains the exact API
  call.**
- `DEC-012` — clap is isolated to `src/cli/` + `main.rs`; the pixel core must not
  depend on clap. The completions code lives in `src/cli/mod.rs`, consistent with this.
- `DEC-027` — `display` (viuer/`view`) is default-on; the lean headless build is
  `--no-default-features`. The README install/feature notes must reflect this, and the
  `completions` path must work in the lean build (clap_complete is non-optional).
- `DEC-038` / SPEC-038 — the dual `MIT OR Apache-2.0` license + `LICENSE-MIT` /
  `LICENSE-APACHE` files the corrected README License section must match.

### Constraints that apply

- `no-new-top-level-deps-without-decision` — satisfied by DEC-039 (clap_complete).
- `clippy-fmt-clean` — the new subcommand + handler must be warning-clean and
  formatted.
- `every-public-fn-tested` / `test-before-implementation` — the failing tests above
  exist before the handler; write the handler to make them pass.
- `no-agpl-default-deps` — clap_complete is `MIT OR Apache-2.0` (clap project), pure
  Rust; compliant.
- `ergonomic-defaults` — completions are a CLI ergonomics win; the README is the new
  user's first contact, so it must be accurate and tool-first.

### Prior related work

- `SPEC-038` (shipped, PR #42) — made the crate publish-ready and added the dual
  license files; this fixes the README License line to match.
- `SPEC-039` (shipped, PR #43) — CHANGELOG + RELEASING; the README cross-links them
  and inherits the "channels light up at release-cut" framing.

### Out of scope (for this spec specifically)

- **A man page (`clap_mangen`)** — deferred (DEC-039 alternatives); a small follow-up
  or part of cargo-dist packaging.
- **Installing completions into shell directories / committing static completion
  files** — we generate to stdout only; *installing* them is the user's / packager's
  step (documented in the README). A `build.rs` static-file approach was rejected
  (DEC-039).
- **The outward-facing release items**: #3 cargo-dist + MSRV, #4 Homebrew tap, #5
  `cargo publish`, #7 dual lean/full artifacts. The README may *describe* the
  brew/cargo/download install story (honestly marked "on release"), but creates no
  tag, release, tap, or publish here — those need explicit maintainer authorization.
- Any new image operation or library API change.

## Notes for the Implementer

- **Read `DEC-039` first** — it has the exact, probe-verified API:
  ```rust
  use clap::CommandFactory;            // brings Cli::command() into scope
  use clap_complete::{generate, Shell};
  fn run_completions(shell: Shell) -> Result<(), CliError> {
      let mut cmd = Cli::command();
      generate(shell, &mut cmd, "crustyimg", &mut std::io::stdout());
      Ok(())
  }
  ```
  Add `Completions { shell: clap_complete::Shell }` to `Commands` (a `///` doc line
  like the others), a `dispatch()` arm `Commands::Completions { shell } =>
  run_completions(*shell),`, and the handler. No `GlobalArgs` needed; no input path.
- The handler writes to `io::stdout()` directly (the binary boundary already does
  stdout/stderr split). Don't route it through the image pipeline or the `Sink`.
- **README honesty:** the brew tap, crates.io publish, and Releases download do **not
  exist yet** (backlog #3/#4/#5). Document them as the intended install methods but
  clearly label the not-yet-live ones (e.g. "Once v0.1.0 is published:") and give a
  **works-today** path (`cargo install --git https://github.com/jysf/crustyimg` or
  clone + `cargo build --release`). Do not claim `brew install jysf/tap/crustyimg`
  works right now. Cross-reference `RELEASING.md` for when the channels light up.
- **Usage examples must be correct** — pull them from `docs/api-contract.md` (real
  subcommands, real flags: `-o`, `--max`, `-q`, the `-` stdin/stdout pipe). Prefer a
  small, copy-pasteable quickstart over an exhaustive command dump (link `crustyimg
  --help` for the full surface). Mention the default-on `view` (DEC-027) and the
  opt-in `webp-lossy` / `avif` features under an install/feature note.
- **Keep the dev-process content** (the spec-driven workflow, `just` commands,
  hierarchy diagram) — relocate it under a clearly-marked "Developing crustyimg" /
  "How this repo is built" section near the bottom, or condense it with a pointer to
  `AGENTS.md` / `GETTING_STARTED.md`. The goal is *tool-first*, not *delete the
  process docs*.
- **Run the lean build** (`cargo build --no-default-features`) and the full `cargo
  deny check advisories bans sources licenses` as part of build — both must be green
  (the lean job and the supply-chain gate run on main CI; verify includes them).
- **Do NOT** run `git tag`, `cargo publish`, `gh release`, or create the tap. This
  spec produces docs + a subcommand only.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-040-readme-completions`
- **PR (if applicable):** see PR opened after this commit
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none — DEC-039 was pre-authored by the architect
- **Deviations from spec:**
  - none
- **Follow-up work identified:**
  - none within scope; `clap_mangen` man page remains deferred (DEC-039 alternatives)

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing materially unclear. The DEC-039 probe-verified API was exact and
   worked first try. The build prompt's honesty requirements for the README install
   section were very precise, which reduced ambiguity rather than causing friction.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No gaps. DEC-012 (clap isolation to src/cli/), DEC-027 (display default-on),
   DEC-038/SPEC-038 (dual license) were all referenced correctly and the
   implementation followed them cleanly.

3. **If you did this task again, what would you do differently?**
   — Run `cargo fmt --all` immediately after writing new test files rather than
   after all code changes, to catch format issues before the first test run (minor).
   Otherwise the TDD flow (tests first, handler second) was frictionless.

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
