# SPEC-008 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-008 ("view command terminal display").
You are NOT the architect; the spec file is your source of truth. This spec
makes the `view` subcommand REAL — it replaces the NotImplemented("view")
stub with a viuer-backed display Sink, fit-to-terminal by default with
optional --width/--height, refusing on a non-tty. Use ABSOLUTE paths for
every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (the `display` feature gates viuer; NO new
   top-level dep needed — viuer is already in Cargo.toml), §6 the EXACT
   commands (the gates below), §11 coding conventions (library-first;
   `main.rs` is a THIN shell; typed errors; NO unwrap/expect/panic! on
   recoverable paths; DIAGNOSTICS TO STDERR NEVER STDOUT; the pixel core
   `image/` MUST NOT depend on clap; group imports std/external/local;
   comments explain WHY not WHAT; no dead code), §12 testing (integration
   under tests/, NATIVE in-memory fixtures via the `image` crate — NO
   ImageMagick, NO committed binary fixtures; trim stdout for Windows),
   §13 git/PR (branch naming, conventional commits + Co-Authored-By trailer,
   PR body template), §15 build-cycle rules (spec edits LIMITED to the
   `## Build Completion` section; append a build cost session entry; create
   DEC-* only for NON-trivial NEW decisions — NONE expected here).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-008-view-command-terminal-display.md
   — THE SPEC. Implement its "## Failing Tests" and "## Outputs" exactly.
   Read "## Implementation Context" and "## Notes for the Implementer" in
   FULL — they carry the EXACT `run_view` shape, the two locked design
   decisions (multi-input → first; non-tty → exit 5), and the construction
   sites you must touch.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   — the `view` contract: requires a tty, non-tty refuses with exit 5,
   optional sizing fits to terminal by default, resolves the first input on a
   directory/glob. (The architect already added the exit-5 note to this file;
   do NOT edit it again.)
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-011-viuer-terminal-display.md
   — viuer behind the off-by-default `display` feature; `Sink::Display` +
   `NotATty` refusal compile UNCONDITIONALLY, the viuer `print` call is
   `#[cfg(feature = "display")]`. This DEC names `view` as the consumer that
   needs `--features display` to actually render. Keep BOTH the default build
   AND the `--features display` build green. viuer is ALREADY a dependency
   (no new crate, no new DEC).
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-012-clap-cli-framework.md
   and .../DEC-007-error-handling-thiserror-anyhow.md
   — DEC-012: clap derive is the CLI framework; `view`'s arg surface is
   ALREADY declared (input + --width + --height) and must NOT change.
   DEC-007: typed errors → exit codes at the binary boundary;
   `SinkError::NotATty` ALREADY maps to exit 5 via `CliError::Sink(_) => 5`.
   NO new error variant, NO mapping change.
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — ergonomic-defaults, no-unwrap-on-recoverable-paths, every-public-fn-tested,
   clippy-fmt-clean, test-before-implementation, untrusted-input-hardening.
7. The SHIPPED code you wire together (read the real signatures):
   src/cli/mod.rs    — the clap `Commands::View` variant, `dispatch()` (the
                       `View { .. } => Err(NotImplemented("view"))` line you
                       replace), `run_apply` (the STRUCTURAL TEMPLATE for
                       run_view), the `CliError` enum + `code()` (do NOT edit),
                       the `exit_code_mapping_is_total` unit test (must stay
                       green unchanged).
   src/sink/mod.rs   — the `Sink::Display` variant (NO fields today, ~line 65)
                       and its `write()` arm (~line 349: non-tty check FIRST,
                       then `#[cfg(feature="display")]` viuer block, then
                       `#[cfg(not(feature="display"))]` block). `SinkError`,
                       `SinkInput`, `Overwrite`.
   src/source/mod.rs — `source::resolve`, `Input::{Path, Stdin}`,
                       `Input::stem()`, `Input::path()`, `SourceError::NotFound`.
   src/image/mod.rs  — `Image::load`, `Image::from_bytes`, `img.pixels()`.
   tests/cli.rs      — integration conventions + the `write_test_png` helper
                       you REUSE for view fixtures.
   tests/sink.rs     — the `display_sink_refuses_non_tty` test you UPDATE.

═══════════════════════════════════════════════════════════════════════════
STEP 0 — BRANCH FIRST (before editing ANY file)
═══════════════════════════════════════════════════════════════════════════

Do this BEFORE touching code so nothing ever lands on `main`:

  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout main
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg pull --ff-only
  git -C /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg checkout -b feat/spec-008-view-command-terminal-display

ALL code, test, and spec edits below happen ON THIS BRANCH. Never commit to
`main`. The "WHEN DONE" steps (commit, push, PR) all operate on this branch.

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact)
═══════════════════════════════════════════════════════════════════════════

A. src/sink/mod.rs — thread width/height into the display Sink.

   A1. Change the `Sink::Display` variant from a bare unit variant to:

         /// Render in the terminal via viuer (behind the `display` cargo
         /// feature, DEC-011). Refuses with [`SinkError::NotATty`] on a
         /// non-tty regardless of whether the feature is enabled.
         /// `width`/`height` are optional sizing hints (`None`/`None` =
         /// fit to terminal, viuer's default).
         Display {
             width: Option<u32>,
             height: Option<u32>,
         },

   A2. In `Sink::write`, change the match arm head from `Sink::Display =>`
       to `Sink::Display { width, height } =>`. The non-tty check stays FIRST
       and unchanged:

         if !std::io::stdout().is_terminal() {
             return Err(SinkError::NotATty);
         }

       Then in the `#[cfg(feature = "display")]` block, thread the fields
       into the viuer config (the match binds `&Option<u32>`, so deref):

         let conf = viuer::Config {
             width: *width,
             height: *height,
             use_kitty: true,
             use_iterm: true,
             ..Default::default()
         };
         viuer::print(img.pixels(), &conf)
             .map_err(|e| SinkError::Display(e.to_string()))
             .map(|_| ())

       The `#[cfg(not(feature = "display"))]` block is UNCHANGED (still
       returns `SinkError::Display("built without the `display` feature")`).
       NOTE: when the feature is OFF, `width`/`height` are unused in that arm
       — clippy may warn about unused bindings. If so, bind them as
       `Sink::Display { width: _width, height: _height }` is NOT ideal because
       the feature-on build DOES use them. Prefer: keep `width, height` and,
       only inside the `#[cfg(not(feature = "display"))]` block, add
       `let _ = (width, height);` to silence the unused warning in the
       feature-off build. Verify BOTH `cargo clippy -- -D warnings` and
       `cargo clippy --features display -- -D warnings` are clean.

   `viuer::Config` (v0.11.0) has `width: Option<u32>` and
   `height: Option<u32>` fields, both defaulting to `None` — confirmed. NO
   other viuer change.

B. src/cli/mod.rs — replace the view stub with a real handler.

   B1. In `dispatch()`, replace:
         Commands::View { .. } => Err(CliError::NotImplemented("view")),
       with:
         Commands::View { input, width, height } => {
             run_view(input, *width, *height, &cli.global)
         }

   B2. Add a PRIVATE `run_view` (mirror `run_apply`'s structure, minus the
       recipe/pipeline steps). NO unwrap/expect/panic; use `?` throughout:

         /// The `view` path: resolve the single input, load the image, and
         /// render it via the display Sink. Resolves the FIRST input when a
         /// directory/glob yields many (single-image command). A non-tty
         /// stdout refuses with `SinkError::NotATty` → exit 5.
         fn run_view(
             input: &str,
             width: Option<u32>,
             height: Option<u32>,
             _global: &GlobalArgs,
         ) -> Result<(), CliError> {
             let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
             let first = resolved
                 .into_iter()
                 .next()
                 .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;
             let img = match &first {
                 crate::source::Input::Path(p) => Image::load(p)?,
                 crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
             };
             let sink = Sink::Display { width, height };
             let sink_input = SinkInput {
                 stem: first.stem(),
                 path: first.path(),
             };
             sink.write(
                 &img,
                 &sink_input,
                 Overwrite::Forbid,
                 &mut std::io::stdout().lock(),
             )?;
             Ok(())
         }

       `_global` is unused by view (no -o/--format for a display sink); the
       leading underscore keeps clippy quiet. Keep `Sink`, `SinkInput`,
       `Overwrite` imports (they are already `use`d for run_apply).

   B3. Do NOT touch `CliError`, `code()`, or `exit_code_mapping_is_total`.
       `NotATty` already maps to 5 via `CliError::Sink(_) => 5`.

C. tests/sink.rs — UPDATE the existing `display_sink_refuses_non_tty` test so
   it still compiles after the variant gains fields. Change the construction:
     Sink::Display
   to:
     Sink::Display { width: None, height: None }
   The `.write(...)` call and the `assert!(matches!(err, SinkError::NotATty))`
   assertion are UNCHANGED. Do NOT add a new test here.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass) — add to tests/cli.rs
═══════════════════════════════════════════════════════════════════════════

Add these to the EXISTING /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/tests/cli.rs
(do NOT create a new test file). Binary-driven via env!("CARGO_BIN_EXE_crustyimg")
+ std::process::Command; REUSE the `write_test_png` helper already in the file;
`tempfile::tempdir()`; trim stdout; assert exit codes via `output.status.code()`.
Under `cargo test` the child's stdout is a pipe (non-tty), so `view` always
reaches the NotATty refusal — these run and pass in DEFAULT CI (no feature).

  - view_non_tty_refuses_exit_5
      Write a PNG via write_test_png; run `view <png>`. Assert:
        status.code() == Some(5);
        stderr (lowercased) contains "tty" OR "terminal";
        stdout is EMPTY (no image bytes leaked — assert output.stdout.is_empty()).
  - view_missing_input_exits_3
      Run `view <tempdir>/nope.png` (a path that does not exist). Assert:
        status.code() == Some(3).
  - view_directory_uses_first_input
      Make a tempdir, write ONE png into it via write_test_png, run `view <dir>`.
      Assert: status.code() == Some(5)  (resolved the first image, hit the
      non-tty refusal; did NOT panic, did NOT exit 2). This pins the
      "display the first resolved input" MVP decision.
  - view_width_flag_still_refuses_non_tty
      Write a PNG; run `view --width 80 <png>`. Assert:
        status.code() == Some(5) and stderr mentions a terminal/tty requirement.
      Proves --width parses and is wired without changing non-tty behavior.

The existing tests `each_subcommand_help_parses` and
`help_lists_all_subcommands` must STILL pass (view already parses; you do not
change its arg surface). Run the FULL `cargo test` and confirm the whole
prior suite plus your new SPEC-008 tests are green.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope)
═══════════════════════════════════════════════════════════════════════════

- The `info` command (separate STAGE-002 spec).
- Any assertion of real rendered output / pixel fidelity (CI has no tty; the
  render path is feature-gated and cannot be integration-tested — do NOT fake
  a tty).
- stdin-specific view logic (`view -` parses and would still hit the non-tty
  refusal; add NO special-casing).
- Multi-input fan-out / displaying more than one image / batch / --out-dir.
  view is single-image; resolve the FIRST input only.
- Any new error variant, exit-code change, new dependency, or DEC.
If you think a new crate or a new DEC is needed, STOP and add a question to
/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE GATES (run from the repo root; ALL must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check                              # `cargo fmt` to fix, then re-check
  cargo build --features display                 # keep the feature build from bit-rotting (DEC-011)
  cargo clippy --features display -- -D warnings  # feature path must be lint-clean too

The last two are REQUIRED for this spec (DEC-011): the viuer render path only
compiles under `--features display`, and it must stay green.

═══════════════════════════════════════════════════════════════════════════
WHEN DONE
═══════════════════════════════════════════════════════════════════════════

1. Fill in ONLY the spec's `## Build Completion` section (branch, PR, criteria
   met, deviations, follow-ups, and the 3-question build reflection). Do NOT
   edit any other part of the spec file.
2. Append a build cost session entry to the spec front-matter `cost.sessions`
   (cycle: build, agent: claude-sonnet-4-6, interface: claude-code,
   tokens_total: null, estimated_usd: null, duration_minutes: <est>,
   recorded_at: 2026-06-14, notes: "subagent; cost not separately reported").
   Do NOT recompute cost.totals (ship does that).
3. Advance the cycle to verify by HAND-EDITING the spec front-matter
   `task.cycle` from `build` to `verify`. DO NOT run `just advance-cycle` or
   `just archive-spec` — they MIS-GLOB in this repo; the orchestrator does all
   other bookkeeping by hand. Only edit the spec's Build Completion section +
   the cost session + task.cycle.
4. Commit ON THE BRANCH (created in Step 0) with Conventional Commits, e.g.
   `feat(cli): real view command via display sink (SPEC-008)`
   — a single commit covering the sink + cli + tests + spec is fine; end EACH
   commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
   (Confirm `git branch --show-current` prints
   `feat/spec-008-view-command-terminal-display`, NOT `main`, before committing.)
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-008-view-command-terminal-display-timeline.md).
   ACCURATE BOOKKEEPING: when you mark build `[x]`, write ONLY what is true at
   build time — say "PR #N opened" (with the real number). Do NOT write
   "merged", do NOT claim the PR is approved, and do NOT assert any post-merge
   fact. The verify and ship cycles record those later.
6. Push the branch and open a PR on the `jysf/crustyimg` remote per
   AGENTS.md §13 (one spec per branch / per PR):
   - PR title carries the spec id, e.g.
     `feat(cli): view command terminal display (SPEC-008)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-002/
     SPEC-008; Decisions referenced [DEC-011 (viuer display sink + feature
     gate), DEC-012 (clap surface), DEC-007 (NotATty → exit 5)]; Constraints
     checked with one-line evidence each (ergonomic-defaults,
     no-unwrap-on-recoverable-paths, every-public-fn-tested, clippy-fmt-clean
     [incl. --features display], test-before-implementation,
     untrusted-input-hardening); New decisions:
     "No new DEC during build — DEC-011/DEC-012/DEC-007 already govern this".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
