# SPEC-007 — BUILD prompt

> Paste this into a **fresh Claude session** (this build runs on **Sonnet
> 4.6**). You are NOT the architect who wrote the spec. The spec file is your
> only context. Do not rely on any prior conversation. This prompt is
> deliberately prescriptive — follow it literally. Use ABSOLUTE paths.

```
Cycle: build. Repo root: /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg
You are the implementer for SPEC-007 ("clap CLI skeleton and dispatch").
You are NOT the architect; the spec file is your source of truth. This spec
COMPLETES STAGE-001. Use ABSOLUTE paths for every file you read or write.

═══════════════════════════════════════════════════════════════════════════
READ THESE FILES IN ORDER BEFORE WRITING ANY CODE
═══════════════════════════════════════════════════════════════════════════

1. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/AGENTS.md
   — conventions: §5 stack (`clap` 4 derive is the CLI framework; NO async, NO
   new top-level dep without a DEC — clap's DEC is DEC-012), §6 the EXACT
   commands (the four gates below), §11 coding conventions (library-first;
   `main.rs` is a THIN shell; typed errors; NO unwrap/expect/panic! on
   recoverable paths; DIAGNOSTICS TO STDERR NEVER STDOUT; the pixel core
   `image/` + `operation/` MUST NOT depend on clap/files/terminals; group
   imports std/external/local; comments explain WHY not WHAT; no dead code),
   §12 testing (integration under tests/, NATIVE in-memory fixtures via the
   `image` crate — NO ImageMagick, NO committed binary fixtures; trim stdout
   for Windows), §13 git/PR (branch naming, conventional commits +
   Co-Authored-By trailer, PR body template), §15 build-cycle rules (spec
   edits LIMITED to the `## Build Completion` section; append a build cost
   session entry; create DEC-* only for NON-trivial NEW decisions — none
   expected, DEC-012 already exists).
2. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-007-clap-cli-skeleton-and-dispatch.md
   — THE SPEC. Implement its "## Failing Tests" and "## Outputs" exactly. Read
   "## Implementation Context" and "## Notes for the Implementer" in FULL — they
   carry the EXACT shipped APIs to call for the real `apply` path, the global
   args + subcommand surface, and the AUTHORITATIVE exit-code mapping table.
3. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/docs/api-contract.md
   — THE contract: the Global Options table, the full Subcommand Surface (each
   command's documented args), the stdin/stdout (`-`) behavior, and the
   Exit Codes table (0–6). Your CLI must match this.
4. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-012-clap-cli-framework.md
   — clap (derive) is the CLI framework; src/cli/** + src/main.rs + Cargo.toml.
   clap is PRE-JUSTIFIED by this DEC — adding it needs NO new DEC. Do NOT add
   any other new crate.
5. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/decisions/DEC-007-error-handling-thiserror-anyhow.md
   — typed thiserror in the library; the BINARY boundary (cli/main) maps typed
   errors to exit codes (and MAY use anyhow for context — only if you actually
   use it; a clean mapping without anyhow is simpler and also fine). The pixel
   core does NOT depend on anyhow.
   Also skim: DEC-002 (Image/Operation/Pipeline the CLI drives), DEC-005
   (recipe TOML + registry the `apply` path runs), DEC-006 (`--jobs` is a
   PARSED PLACEHOLDER — NO rayon, NO parallelism here; STAGE-005 does that).
6. /Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/constraints.yaml
   — ergonomic-defaults, no-unwrap-on-recoverable-paths,
   no-new-top-level-deps-without-decision, clippy-fmt-clean,
   every-public-fn-tested, test-before-implementation.
7. The SHIPPED library code you wire together (read the real signatures):
   src/image/mod.rs  (Image::load, Image::from_bytes, ImageError)
   src/source/mod.rs (source::resolve, Input::{path,stem}, SourceError)
   src/recipe/mod.rs (Recipe::from_toml, Recipe::build_pipeline, RecipeError)
   src/operation/registry.rs (OperationRegistry::with_builtins)
   src/pipeline/mod.rs (Pipeline::run, OperationError)
   src/sink/mod.rs   (Sink, SinkInput, Overwrite, Sink::write, SinkError,
                      format_from_extension, extension_for_format)
   src/main.rs       (the SPEC-001 argv match you REPLACE)
   src/lib.rs        (add `pub mod cli;`)
   tests/smoke.rs    (the SPEC-001 smoke tests — see the GOTCHA below)

═══════════════════════════════════════════════════════════════════════════
WHAT TO BUILD (exact)
═══════════════════════════════════════════════════════════════════════════

A. Cargo.toml — add ONE new dependency, exact-pinned like the others:
     clap = { version = "=4.5.X", features = ["derive"] }
   (resolve the current 4.5.x patch and pin it exactly with `=`.) Do NOT add
   anyhow unless you genuinely use it at the boundary; do NOT add rayon,
   indicatif, or any other crate.

B. src/cli/mod.rs — the clap derive types + dispatch + exit-code mapping.

   B1. The parser types (clap derive). Shape:

     use clap::{Args, Parser, Subcommand};

     #[derive(Parser, Debug)]
     #[command(name = "crustyimg", version, about)]
     pub struct Cli {
         #[command(flatten)]
         pub global: GlobalArgs,
         #[command(subcommand)]
         pub command: Commands,
     }

     #[derive(Args, Debug)]
     pub struct GlobalArgs {
         #[arg(short = 'o', long)]            pub output: Option<String>,        // `-` = stdout
         #[arg(long)]                          pub out_dir: Option<String>,
         #[arg(long)]                          pub name_template: Option<String>,
         #[arg(short = 'j', long)]            pub jobs: Option<usize>,           // PLACEHOLDER (DEC-006)
         #[arg(long)]                          pub format: Option<String>,
         #[arg(short = 'q', long)]            pub quality: Option<u8>,           // 0-100
         #[arg(short = 'v', long, action = clap::ArgAction::Count)] pub verbose: u8,
         #[arg(short = 'Q', long)]            pub quiet: bool,                   // NOTE uppercase Q
         #[arg(short = 'y', long)]            pub yes: bool,
         #[arg(long)]                          pub keep_gps: bool,
     }

     #[derive(Subcommand, Debug)]
     pub enum Commands {
         View      { input: String, #[arg(long)] width: Option<u32>, #[arg(long)] height: Option<u32> },
         Info      { input: String, #[arg(long)] exif: bool, #[arg(long)] json: bool },
         Resize    { inputs: Vec<String>, #[arg(long)] max: Option<u32>, #[arg(long)] exact: Option<String>,
                     #[arg(long)] percent: Option<f32>, #[arg(long)] fit: Option<String>,
                     #[arg(long)] fill: Option<String>, #[arg(long)] cover: Option<String> },
         Thumbnail { inputs: Vec<String>, #[arg(long)] size: Option<u32>, #[arg(long)] square: bool },
         Shrink    { inputs: Vec<String>, #[arg(long)] max: Option<u32> },
         Convert   { inputs: Vec<String>, #[arg(long)] format: String },
         #[command(name = "auto-orient")]
         AutoOrient{ inputs: Vec<String> },
         Watermark { inputs: Vec<String>, #[arg(long)] image: String, #[arg(long)] gravity: Option<String>,
                     #[arg(long)] opacity: Option<f32>, #[arg(long)] scale: Option<f32>,
                     #[arg(long)] margin: Option<u32>, #[arg(long)] tile: bool },
         Strip     { inputs: Vec<String> },
         Clean     { inputs: Vec<String>, #[arg(long)] gps: bool },
         Set       { inputs: Vec<String>, #[arg(long)] artist: Option<String>,
                     #[arg(long)] copyright: Option<String>, #[arg(long)] description: Option<String> },
         #[command(name = "copy-metadata")]
         CopyMetadata { #[arg(long)] from: String, #[arg(long)] to: String },
         Edit      { input: String, #[arg(long)] save_recipe: Option<String> },
         Apply     { #[arg(long)] recipe: String, inputs: Vec<String> },
     }

   NOTES:
   - `--version`/`--help` are automatic from `#[command(version, about)]`; do
     NOT declare them. clap provides `-V`/`-h`.
   - `-q` quality is NOT the same as `-Q` quiet — keep the casing exactly.
   - The op flags on resize/watermark/etc. are PARSED ONLY (the commands are
     stubs); they exist so `<cmd> --help` lists the documented args. You do not
     need clap `group`/mutually-exclusive wiring for the stubs — declaring the
     args is enough for this stage. Keep it minimal.
   - `convert` documents `--format` per-command; the global `--format` also
     exists. That is fine — convert's own `--format` is required; the global one
     is optional. (convert is a stub here regardless.)

   B2. The typed CLI error + exit-code mapping (ONE place; unit-tested):

     #[derive(Debug, thiserror::Error)]
     pub enum CliError {
         #[error(transparent)] Source(#[from] crate::source::SourceError),
         #[error(transparent)] Image(#[from] crate::image::ImageError) // adjust the path to where ImageError lives,
         #[error(transparent)] Recipe(#[from] crate::recipe::RecipeError),
         #[error(transparent)] Operation(#[from] crate::operation::OperationError),
         #[error(transparent)] Sink(#[from] crate::sink::SinkError),
         #[error("{0} is not yet implemented")] NotImplemented(&'static str),
     }

     impl CliError {
         pub fn code(&self) -> u8 {
             match self {
                 CliError::Source(crate::source::SourceError::NotFound(_)) => 3,
                 CliError::Source(crate::source::SourceError::Stdin(_)) => 3,
                 CliError::Source(crate::source::SourceError::InvalidPattern { .. }) => 2,
                 CliError::Image(crate::image::ImageError::Io(_)) => 3,
                 CliError::Image(crate::image::ImageError::Decode(_)) => 1,
                 CliError::Image(crate::image::ImageError::UnsupportedFormat) => 4,
                 CliError::Recipe(_) => 1,
                 CliError::Operation(_) => 1,
                 CliError::Sink(crate::sink::SinkError::UnsupportedExtension(_)) => 4,
                 CliError::Sink(crate::sink::SinkError::UnknownFormat) => 4,
                 CliError::Sink(_) => 5,
                 CliError::NotImplemented(_) => 1,
             }
         }
     }

     (Verify the exact module paths of each error type against the real source —
     e.g. ImageError may be re-exported from crate::error or crate::image. Use
     whatever the shipped code exports. The MAPPING TABLE in the spec's
     Implementation Context is authoritative for which code each variant gets.)

   B3. `pub fn run() -> std::process::ExitCode` — the binary entry:
     - `let cli = Cli::parse();`  // clap prints usage + exits 2 on parse error
       (unknown subcommand, bad args) ALL ON ITS OWN — that satisfies exit 2.
     - `match dispatch(&cli) { Ok(()) => ExitCode::SUCCESS,
         Err(e) => { eprintln!("error: {e}"); ExitCode::from(e.code()) } }`
     - `dispatch` matches `cli.command`: `Commands::Apply { recipe, inputs }` →
       the REAL path (below); every other arm → `Err(CliError::NotImplemented("<cmd>"))`.

   B4. The REAL `apply` path (single input — see spec Notes step-by-step):
       read recipe text (io err → CliError mapped to 3) → Recipe::from_toml →
       OperationRegistry::with_builtins() → recipe.build_pipeline(&reg) →
       source::resolve(first input arg, &mut std::io::stdin().lock()) → take the
       first Input → Image::load(path) or Image::from_bytes(&bytes) →
       pipeline.run(img) → build the Sink from global -o/--out-dir/--format,
       SinkInput { stem: input.stem(), path: input.path() }, Overwrite from
       --yes → sink.write(&out, &sink_input, overwrite, &mut io::stdout().lock())
       → Ok(()).
       Sink selection: `-o -` → Sink::Stdout { format } (format from --format via
       image::ImageFormat::from_extension; if None and output is "-", let the
       write surface UnknownFormat → 4); `-o <path>` → Sink::File { path, format:
       --format-as-ImageFormat }; `--out-dir <dir>` → Sink::Dir { dir, template:
       --name-template.unwrap_or("{stem}.{ext}"), format }. The spec only
       exercises `-o <file>` and `-o - --format png`.
       Use `?` with `CliError`'s `#[from]` impls so the typed errors convert
       automatically; reading the recipe FILE returns io::Error — map it to
       SourceError::NotFound or wrap so it lands on exit 3 (a small helper or a
       map_err is fine).

   ERGONOMICS (ergonomic-defaults): inputs are POSITIONAL, `--jobs` defaults to
   CPU count conceptually (Option → treat None as "all cores" — but it is
   ignored this stage), no required boilerplate flags beyond what each command
   documents.

C. src/main.rs — REPLACE the entire SPEC-001 argv body with a thin shell:

     use std::process::ExitCode;
     fn main() -> ExitCode { crustyimg::cli::run() }

   Delete the SPEC-001 `EXIT_USAGE`/`USAGE` constants and the argv match.
   `--version`/`--help` still work — now via clap.

D. src/lib.rs — add `pub mod cli;` (and one doc line: "SPEC-007 adds the
   [`cli`] module: the clap subcommand surface + dispatch + exit-code mapping
   (DEC-012, DEC-007)."). Keep the existing `version()` fn and its unit test.

═══════════════════════════════════════════════════════════════════════════
GOTCHA — the SPEC-001 smoke test (tests/smoke.rs) WILL BREAK; FIX IT
═══════════════════════════════════════════════════════════════════════════

clap's default `--version` output is `crustyimg 0.1.0` (binary NAME + space +
version), NOT bare `0.1.0`. So these existing tests break and MUST be updated
(do NOT delete the coverage — adjust the assertions to the new, correct
behavior):

  - `version_matches_cargo_pkg_version` — currently asserts stdout.trim() ==
    CARGO_PKG_VERSION exactly. Update to assert stdout CONTAINS
    env!("CARGO_PKG_VERSION") (clap prints "crustyimg <version>").
  - `version_flag_prints_semver` / `starts_with_semver` — currently assumes the
    first token is the semver. Update so it checks the semver appears in the
    output (e.g. strip a leading "crustyimg " prefix, or assert the trimmed
    stdout ends-with / contains the semver). Keep the intent: --version exits 0
    and prints the package version.
  - `version_short_flag_matches_long` — still holds (`-V` == `--version`); no
    change needed.
  - `help_flag_exits_zero_and_names_binary` — still holds (clap's --help names
    "crustyimg"); no change needed.
  - `unknown_invocation_exits_nonzero_on_stderr` — still holds (clap exits 2 to
    stderr, stdout empty); no change needed. (Optionally tighten to assert exit
    code == 2.)

Keep ALL other shipped tests green: tests/image_load.rs, tests/pipeline.rs,
tests/recipe_round_trip.rs, tests/sink.rs, tests/source.rs, and the unit tests
inside each src module. Run the FULL `cargo test` and confirm the whole
SPEC-001..006 suite plus your new SPEC-007 tests are green.

═══════════════════════════════════════════════════════════════════════════
TESTS YOU WRITE (make them pass)
═══════════════════════════════════════════════════════════════════════════

tests/cli.rs — binary-driven (env!("CARGO_BIN_EXE_crustyimg") +
std::process::Command), tempfile::tempdir() fixtures, native PNG fixtures via
the `image` crate, trim stdout for Windows. Implement EXACTLY the test names in
the spec's "## Failing Tests":
  help_lists_all_subcommands, version_prints_semver,
  unknown_subcommand_is_usage_error (exit==2), each_subcommand_help_parses,
  apply_recipe_runs_end_to_end, apply_to_stdout_keeps_stdout_clean
  (use `-o - --format png`), stub_command_returns_not_implemented (exit==1,
  no output file, "not yet implemented" on stderr), apply_missing_input_exits_3,
  apply_bad_recipe_version_exits_1.

src/cli/mod.rs `#[cfg(test)] mod tests` — unit (no process spawn):
  cli_parses_global_and_apply (Cli::try_parse_from is Ok, apply variant +
  recipe + global output), cli_unknown_subcommand_is_err
  (Cli::try_parse_from is Err; clap error exit_code()==2),
  exit_code_mapping_is_total (each CliError variant maps to its documented
  code; NotImplemented→1).

Add `tempfile` to [dev-dependencies] is ALREADY present (=3.27.0). The `image`
crate is a normal dependency available to integration tests via the crate.
Generate a tiny RGB image and encode it to PNG in the tempdir for fixtures.

═══════════════════════════════════════════════════════════════════════════
DO NOT BUILD (out of scope — stubs only)
═══════════════════════════════════════════════════════════════════════════

- Any real operation/command logic (resize/view/info/thumbnail/shrink/convert/
  auto-orient/watermark/strip/clean/set/copy-metadata/edit) — these are
  STAGE-002/003/004; they are NotImplemented stubs (exit 1) here.
- Real parallel batch / rayon / indicatif (`--jobs` is parsed + ignored,
  DEC-006; STAGE-005).
- Multi-input batch fan-out in apply + exit-6 partial-batch (STAGE-005); only
  the SINGLE-input apply path is wired real here.
- The metadata lane (STAGE-004); terminal display correctness (viuer is behind
  the `display` feature; `view` is a stub).
- edit --save-recipe (STAGE-005).
Only `identity` + `invert` ops exist (from with_builtins) for the apply recipe.
If you think a new crate or a new DEC is needed, STOP and add a question to
/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/guidance/questions.yaml
instead of inventing it.

═══════════════════════════════════════════════════════════════════════════
THE FOUR GATES (run from the repo root; all must pass before the PR)
═══════════════════════════════════════════════════════════════════════════

  cargo build
  cargo test
  cargo clippy -- -D warnings
  cargo fmt --check        # `cargo fmt` to fix, then re-check

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
3. Advance the cycle to verify. NOTE: `just advance-cycle` MIS-GLOBS in this
   repo — instead HAND-EDIT the spec front-matter `task.cycle` from `build`
   to `verify`, and verify the change is correct before committing.
4. Commit with Conventional Commits, e.g.
   `feat(cli): clap subcommand skeleton + dispatch + exit codes (SPEC-007)`,
   ending EACH commit message with:
       Co-Authored-By: Claude Sonnet 4.6 <noreply@anthropic.com>
5. Mark build `[x]` in the timeline
   (/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg/projects/PROJ-001-crustyimg-mvp/specs/SPEC-007-clap-cli-skeleton-and-dispatch-timeline.md).
   ACCURATE BOOKKEEPING (lesson from SPEC-006): when you mark build `[x]`,
   write ONLY what is true at build time — say "PR #N opened" (with the real
   number). Do NOT write "merged", do NOT claim the PR is approved, and do NOT
   assert any post-merge fact. The verify and ship cycles record those later.
6. Branch `feat/spec-007-clap-cli-skeleton-and-dispatch` off SYNCED `main`
   (one spec per branch / per PR). Push and open a PR on the `jysf/crustyimg`
   remote per AGENTS.md §13:
   - PR title carries the spec id, e.g.
     `feat(cli): clap CLI skeleton + dispatch (SPEC-007)`.
   - PR body uses the §13 template — Summary; Spec metadata PROJ-001/STAGE-001/
     SPEC-007; Decisions referenced [DEC-012 governs (clap), DEC-007
     (exit-code mapping), DEC-002, DEC-005, DEC-006]; Constraints checked with
     one-line evidence each (ergonomic-defaults, no-unwrap-on-recoverable-paths,
     no-new-top-level-deps-without-decision [→ DEC-012], clippy-fmt-clean,
     every-public-fn-tested, test-before-implementation); New decisions:
     "No new DEC during build — DEC-012 written during design justifies clap".
   - End the PR body with the Claude Code generated-with footer.

Remember: build edits to the spec are LIMITED to `## Build Completion` (plus the
front-matter cost session + task.cycle). Verify/ship bookkeeping lands on main
later, not on this branch.
```
