# SPEC-001 — BUILD prompt

> Paste this into a **fresh Claude session**. You are NOT the architect who
> wrote the spec. The spec file is your only context. Do not rely on any
> prior conversation.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-001 ("cargo project and multi-OS CI"). You
are NOT the architect; the spec file is your source of truth.

Read these files in order before writing any code:

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (edition 2021, no async), §6 EXACT commands,
   §11 coding conventions (library-first, thin main.rs, no dead code,
   diagnostics to stderr), §12 testing (integration tests under tests/),
   §13 git/PR conventions.
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-001-cargo-project-and-multi-os-ci.md
   — the spec. Read the ENTIRE "## Implementation Context" section carefully:
   it lists the decisions, constraints, exact commands, out-of-scope items,
   and the dependency policy you must honor.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/stages/STAGE-001-foundation-and-pipeline-core.md
   — the parent stage (this is backlog item #1).
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/brief.md
   — the project.
5. The decisions referenced by the spec:
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-009-ci-matrix-and-rust-edition.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-004-codec-policy-pure-rust-default.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-006-no-async-runtime-rayon-for-batch.md
   - /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-002-single-image-model-and-operation-trait.md
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — full text of the constraints that apply to paths you'll touch.

Before coding, mark the build cycle `[~]` in:
  /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-001-cargo-project-and-multi-os-ci-timeline.md

If you hit something needing architect judgment or an external unblock
(constraint unclear, dependency genuinely required, scope drift), change the
marker to `[?]` with a one-line reason and STOP. `[?]` is not a "don't know
what to do" dumping ground — ask if unsure.

Create the branch:
  git checkout -b feat/spec-001-cargo-project-and-multi-os-ci

Implement to make the spec's "## Failing Tests" pass. Create exactly these
files (see the spec's ## Outputs for details):
- Cargo.toml          — package `crustyimg`, edition "2021", a [lib] and
                        [[bin]] name = "crustyimg", version e.g. 0.1.0.
                        EXPECTED [dependencies] is EMPTY. Do NOT pre-add the
                        architecture.md crate table — unused deps trip
                        `clippy -D warnings` and violate
                        no-new-top-level-deps-without-decision. clap, image,
                        thiserror, anyhow, etc. arrive with the specs that use
                        them (SPEC-002, SPEC-007, …).
- src/lib.rs          — minimal lib root exposing one tested public item
                        (e.g. `pub fn version() -> &'static str` returning
                        env!("CARGO_PKG_VERSION")) with a #[cfg(test)] unit
                        test. Do NOT create the image/operation/pipeline/…
                        submodules — those are later specs.
- src/main.rs         — thin, panic-free entrypoint. Hand-rolled argv match
                        (no clap yet): --version/-V → print version to stdout,
                        exit 0; --help/-h → print one-line usage naming
                        `crustyimg` to stdout, exit 0; unknown/none → message
                        to STDERR, non-zero exit (use ExitCode / process::exit,
                        NOT panic!). No unwrap/expect/panic! on recoverable
                        paths (DEC-007 principle).
- tests/smoke.rs      — integration test driving the binary via
                        std::process::Command and env!("CARGO_BIN_EXE_crustyimg")
                        (no assert_cmd/escargot dependency). Implement the
                        five tests named in the spec's ## Failing Tests.
                        Verify semver WITHOUT a regex crate (split on '.',
                        check three numeric parts, or compare to
                        env!("CARGO_PKG_VERSION")). Trim captured output so
                        Windows \r\n does not break assertions.
- .github/workflows/ci.yml — GitHub Actions, strategy.matrix.os over
                        ubuntu-latest, macos-latest, windows-latest;
                        runs-on ${{ matrix.os }}; stable toolchain with clippy
                        + rustfmt components; separate steps for cargo build,
                        cargo test, cargo clippy -- -D warnings,
                        cargo fmt --check. Trigger on push + pull_request.
                        NO secrets. The --features mozjpeg job (DEC-009) is
                        OUT OF SCOPE — no codec feature exists yet; note it as
                        a follow-up, do not invent a feature.
- .gitignore          — ignore /target.

Honor every constraint in the spec's Implementation Context. The expected
end state has an EMPTY [dependencies] table. If you believe a dependency is
truly required, STOP, write a /decisions/DEC-NNN-<slug>.md justifying it with
honest confidence, and only then add it. Otherwise stay std-only.

Run the gates locally until all green (from the repo root):
  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check
  cargo run -- --version    # sanity: prints semver
  cargo run -- --help       # sanity: prints usage

When done:
1. Fill in the spec's "## Build Completion" section INCLUDING the three
   build-phase reflection questions (not optional, answer honestly).
2. Append a build cost session entry to the spec's `cost.sessions`:
     - cycle: build
       agent: <your model>
       interface: claude-code
       tokens_input: <best available>
       tokens_output: <best available>
       estimated_usd: <best available>
       duration_minutes: <estimate>
       recorded_at: <YYYY-MM-DD>
       notes: <one line if rework/unusual, else null>
   In Claude Code, run /cost and use its numbers; if unavailable, use null
   numeric fields with a note.
3. Run from the repo root:
     just advance-cycle SPEC-001 verify
4. Commit with a Conventional Commit (one spec per PR — constraint
   one-spec-per-pr), e.g.:
     feat(SPEC-001): cargo project + multi-OS CI
   End the commit message with:
     Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>
5. Push the branch and open a GitHub PR on jysf/crustyimg (per AGENTS.md §13)
   using the gh CLI. PR title must carry the spec id, e.g.
   `feat(SPEC-001): cargo project + multi-OS CI`. PR body must follow the
   AGENTS.md §13 template:
     ## Summary
     - <bullets per structural change>
     ## Spec metadata
     - **Project:** PROJ-001
     - **Stage:** STAGE-001
     - **Spec:** SPEC-001
     ## Decisions referenced
     DEC-009 (CI matrix + edition 2021), DEC-004 (pure-Rust default), DEC-007
     (no-panic principle in main), DEC-006 (no async), DEC-002 (architecture
     this scaffold seeds)
     ## Constraints checked
     - `clippy-fmt-clean` ✅ — <evidence>
     - `test-before-implementation` ✅ — <evidence>
     - `every-public-fn-tested` ✅ — <evidence>
     - `no-new-top-level-deps-without-decision` ✅ — empty [dependencies]
     - `no-async-runtime` ✅ — no tokio/async-std
     - `one-spec-per-pr` ✅ — SPEC-001 only
     - `no-secrets-in-code` ✅ — CI uses no secrets
     ## New decisions
     - <DEC-NNN — title, or "No new DEC">
   End the PR body with the Claude Code generated-with footer.
6. Mark build `[x]` in the timeline with the PR number, cost, and date:
     - [x] **build** — prompt: `prompts/SPEC-001-build.md`
            PR #NNN, $X.XX, completed <YYYY-MM-DD>

Watch for:
- Resist pre-adding dependencies "for later specs" — empty [dependencies] is
  the correct outcome; unused deps fail clippy -D warnings.
- Windows line endings: trim() captured stdout in smoke tests.
- Keep main.rs panic-free (DEC-007); use ExitCode, not unwrap/expect.
- Do not stub the src/ submodule tree; lib.rs stays minimal.
```
